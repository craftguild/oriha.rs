use crate::config::IngestConfig;
use crate::state_store::{InMemoryShortStateStore, ShortStateStore, ValueReservation};
use crate::writer::{
    cleanup_old_raw_partitions, StatsWriterHandle, StoredUsageAnalyticsRecord, WriterError,
};
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::Router;
use hmac::{Hmac, Mac};
use oriha_stats_schema::validate_json_bytes;
use sha2::Sha256;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

type HmacSha256 = Hmac<Sha256>;
const API_KEY_HEADER: &str = "x-oriha-stats-key";
const CF_CONNECTING_IP_HEADER: &str = "cf-connecting-ip";

#[derive(Clone)]
pub struct AppState {
    pub writer: StatsWriterHandle,
    pub short_state: Arc<dyn ShortStateStore>,
    pub hmac_secret: Arc<Vec<u8>>,
    pub api_key: Arc<str>,
    pub rate_limit_ttl: Duration,
    pub rate_limit_max: u64,
    pub api_key_rate_limit_max: u64,
    pub payload_rate_limit_ttl: Duration,
    pub payload_rate_limit_max: u64,
}

impl AppState {
    pub fn from_config(config: &IngestConfig) -> Self {
        let writer = StatsWriterHandle::start(
            config.data_dir.clone(),
            config.rotate_seconds,
            config.rotate_bytes,
        );
        spawn_retention_cleanup(config.data_dir.clone(), config.raw_retention_days);

        Self {
            writer,
            short_state: Arc::new(InMemoryShortStateStore::new()),
            hmac_secret: Arc::new(config.hmac_secret.clone()),
            api_key: Arc::<str>::from(config.api_key.clone()),
            rate_limit_ttl: Duration::from_secs(config.rate_limit_ttl_seconds),
            rate_limit_max: config.rate_limit_max,
            api_key_rate_limit_max: config.api_key_rate_limit_max,
            payload_rate_limit_ttl: Duration::from_secs(config.payload_rate_limit_ttl_seconds),
            payload_rate_limit_max: config.payload_rate_limit_max,
        }
    }
}

pub fn router(state: AppState, body_limit_bytes: usize) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/ingest", post(post_render_stats))
        .layer(DefaultBodyLimit::max(body_limit_bytes))
        .with_state(state)
}

async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn post_render_stats(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    let Some(api_key) = authenticated_api_key(&state, &headers) else {
        return StatusCode::UNAUTHORIZED;
    };
    let Some(client_ip) = cloudflare_client_ip(&headers) else {
        return StatusCode::BAD_REQUEST;
    };

    if is_ip_rate_limited(&state, client_ip) || is_api_key_rate_limited(&state, api_key) {
        return StatusCode::TOO_MANY_REQUESTS;
    }

    let payload = match validate_json_bytes(&body) {
        Ok(payload) => payload,
        Err(_) => return StatusCode::BAD_REQUEST,
    };
    let digest_input = match payload.hmac_digest_input() {
        Ok(bytes) => bytes,
        Err(_) => return StatusCode::BAD_REQUEST,
    };
    let digest = hmac_hex(&state.hmac_secret, &digest_input);
    let event_keys = payload.event_id.as_ref().map(|event_id| {
        let hash = hmac_hex(&state.hmac_secret, event_id.as_bytes());
        (
            format!("event-id:pending:{hash}"),
            format!("event-id:committed:{hash}"),
        )
    });
    if let Some((pending_key, committed_key)) = event_keys.as_ref() {
        match state.short_state.reserve_value(
            pending_key.clone(),
            committed_key,
            digest.clone(),
            state.payload_rate_limit_ttl,
        ) {
            ValueReservation::Reserved => {}
            ValueReservation::Duplicate => {
                tracing::info!(
                    digest_prefix = digest_prefix(&digest),
                    "stats ingest duplicate ignored"
                );
                return StatusCode::ACCEPTED;
            }
            ValueReservation::Pending => return StatusCode::TOO_MANY_REQUESTS,
            ValueReservation::Conflict => return StatusCode::CONFLICT,
        }
    }
    if is_payload_rate_limited(&state, &digest) {
        if let Some((pending_key, _)) = event_keys.as_ref() {
            state.short_state.remove(pending_key);
        }
        return StatusCode::TOO_MANY_REQUESTS;
    }

    let record = StoredUsageAnalyticsRecord::new(digest.clone(), payload);
    let storage_id = record.storage_id;
    let platform = record.payload.platform.clone();
    let app_version = record.payload.app_version.clone();
    let build_version = record.payload.build_version.clone();
    let subscription_plan = record.payload.subscription_plan.clone();
    let total_blocks = record.payload.stats.total_blocks;
    let has_event_id = record.payload.event_id.is_some();
    match state.writer.write(record).await {
        Ok(()) => {
            if let Some((pending_key, committed_key)) = event_keys.as_ref() {
                state.short_state.commit_value(
                    pending_key,
                    committed_key.clone(),
                    digest.clone(),
                    state.payload_rate_limit_ttl,
                );
            }
            tracing::info!(
                storage_id = %storage_id,
                digest_prefix = digest_prefix(&digest),
                platform = %platform,
                app_version = %app_version,
                build_version = %build_version,
                subscription_plan = %subscription_plan,
                total_blocks,
                has_event_id,
                "stats ingest accepted"
            );
            StatusCode::ACCEPTED
        }
        Err(WriterError::Closed | WriterError::Io(_) | WriterError::Serialize(_)) => {
            if let Some((pending_key, _)) = event_keys.as_ref() {
                state.short_state.remove(pending_key);
            }
            StatusCode::SERVICE_UNAVAILABLE
        }
        Err(WriterError::InvalidRetention(_)) => {
            if let Some((pending_key, _)) = event_keys.as_ref() {
                state.short_state.remove(pending_key);
            }
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

fn authenticated_api_key<'a>(state: &AppState, headers: &'a HeaderMap) -> Option<&'a str> {
    let provided = headers.get(API_KEY_HEADER)?.to_str().ok()?;
    (provided == state.api_key.as_ref()).then_some(provided)
}

fn cloudflare_client_ip(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get(CF_CONNECTING_IP_HEADER)?
        .to_str()
        .ok()?
        .parse()
        .ok()
}

fn is_ip_rate_limited(state: &AppState, client_ip: IpAddr) -> bool {
    is_rate_limited(
        state,
        "ip",
        format!("cf-ip:{client_ip}").as_bytes(),
        state.rate_limit_ttl,
        state.rate_limit_max,
    )
}

fn is_api_key_rate_limited(state: &AppState, api_key: &str) -> bool {
    is_rate_limited(
        state,
        "api-key",
        api_key.as_bytes(),
        state.rate_limit_ttl,
        state.api_key_rate_limit_max,
    )
}

fn is_payload_rate_limited(state: &AppState, digest: &str) -> bool {
    is_rate_limited(
        state,
        "payload",
        digest.as_bytes(),
        state.payload_rate_limit_ttl,
        state.payload_rate_limit_max,
    )
}

fn is_rate_limited(
    state: &AppState,
    namespace: &str,
    input: &[u8],
    ttl: Duration,
    max: u64,
) -> bool {
    let key = hmac_hex(&state.hmac_secret, input);
    !state
        .short_state
        .increment_with_limit(format!("rate:{namespace}:{key}"), ttl, max)
}

pub fn hmac_hex(secret: &[u8], input: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(input);
    hex::encode(mac.finalize().into_bytes())
}

fn digest_prefix(digest: &str) -> &str {
    digest.get(..12).unwrap_or(digest)
}

fn spawn_retention_cleanup(data_dir: std::path::PathBuf, retention_days: u64) {
    tokio::spawn(async move {
        let _ = cleanup_old_raw_partitions(&data_dir, retention_days).await;
        let mut interval = time::interval(Duration::from_secs(60 * 60));
        loop {
            interval.tick().await;
            let _ = cleanup_old_raw_partitions(&data_dir, retention_days).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use serde_json::json;
    use tower::ServiceExt;

    fn sample_payload() -> Vec<u8> {
        serde_json::to_vec(&sample_payload_value()).expect("sample payload should serialize")
    }

    fn sample_payload_value() -> serde_json::Value {
        json!({
            "event": "markdown_render_complete",
            "schemaVersion": 1,
            "platform": "macOS",
            "appVersion": "1.0",
            "buildVersion": "202605181631",
            "locale": "ja",
            "subscriptionPlan": "free",
            "stats": {
                "schemaVersion": 1,
                "totalBlocks": 1,
                "blocksByKind": [{ "kind": "paragraph", "count": 1 }],
                "headingsByLevel": [],
                "codeBlocksByLanguageKind": [],
                "calloutsByKind": [],
                "lockedFeaturesByKind": [{ "kind": "syntaxTheme", "count": 1 }],
                "unsupportedByKind": [],
                "issuesByKind": []
            }
        })
    }

    fn test_config(data_dir: std::path::PathBuf) -> IngestConfig {
        IngestConfig {
            bind_addr: "127.0.0.1:0".parse().expect("addr"),
            data_dir,
            hmac_secret: b"test-secret".to_vec(),
            api_key: "test-api-key".to_string(),
            body_limit_bytes: 65_536,
            rotate_seconds: 900,
            rotate_bytes: 67_108_864,
            raw_retention_days: 180,
            rate_limit_ttl_seconds: 60,
            rate_limit_max: 120,
            api_key_rate_limit_max: 100_000,
            payload_rate_limit_ttl_seconds: 900,
            payload_rate_limit_max: 10_000,
        }
    }

    async fn post(app: Router, body: Vec<u8>) -> StatusCode {
        post_with_headers(app, body, Some("test-api-key"), Some("203.0.113.10")).await
    }

    async fn post_with_headers(
        app: Router,
        body: Vec<u8>,
        api_key: Option<&str>,
        cf_ip: Option<&str>,
    ) -> StatusCode {
        let mut builder = Request::builder()
            .method("POST")
            .uri("/api/ingest")
            .header("content-type", "application/json");
        if let Some(api_key) = api_key {
            builder = builder.header(API_KEY_HEADER, api_key);
        }
        if let Some(cf_ip) = cf_ip {
            builder = builder.header(CF_CONNECTING_IP_HEADER, cf_ip);
        }
        let response = app
            .oneshot(builder.body(Body::from(body)).expect("request"))
            .await
            .expect("response");
        response.status()
    }

    #[tokio::test]
    async fn valid_post_writes_one_jsonl_line() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config = test_config(temp.path().to_path_buf());
        let app = router(AppState::from_config(&config), config.body_limit_bytes);

        assert_eq!(post(app, sample_payload()).await, StatusCode::ACCEPTED);

        let lines = read_all_stats_lines(temp.path());
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("\"receivedAt\""));
        assert!(lines[0].contains("\"storageId\""));
        assert!(lines[0].contains("\"digest\""));
        assert!(!lines[0].contains("\"eventId\""));
        assert!(!lines[0].contains("127.0.0.1"));
    }

    #[tokio::test]
    async fn duplicate_payload_is_saved_as_another_event() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config = test_config(temp.path().to_path_buf());
        let app = router(AppState::from_config(&config), config.body_limit_bytes);

        assert_eq!(
            post(app.clone(), sample_payload()).await,
            StatusCode::ACCEPTED
        );
        assert_eq!(post(app, sample_payload()).await, StatusCode::ACCEPTED);

        assert_eq!(read_all_stats_lines(temp.path()).len(), 2);
    }

    #[tokio::test]
    async fn invalid_payload_returns_bad_request() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config = test_config(temp.path().to_path_buf());
        let app = router(AppState::from_config(&config), config.body_limit_bytes);

        assert_eq!(
            post(app, br#"{"event":"bad"}"#.to_vec()).await,
            StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn body_limit_returns_payload_too_large() {
        let temp = tempfile::tempdir().expect("tempdir");
        let mut config = test_config(temp.path().to_path_buf());
        config.body_limit_bytes = 8;
        let app = router(AppState::from_config(&config), config.body_limit_bytes);

        assert_eq!(
            post(app, sample_payload()).await,
            StatusCode::PAYLOAD_TOO_LARGE
        );
    }

    #[tokio::test]
    async fn rate_limit_returns_too_many_requests() {
        let temp = tempfile::tempdir().expect("tempdir");
        let mut config = test_config(temp.path().to_path_buf());
        config.rate_limit_max = 1;
        let app = router(AppState::from_config(&config), config.body_limit_bytes);

        assert_eq!(
            post(app.clone(), sample_payload()).await,
            StatusCode::ACCEPTED
        );
        assert_eq!(
            post(app, sample_payload()).await,
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[tokio::test]
    async fn missing_or_invalid_api_key_is_unauthorized() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config = test_config(temp.path().to_path_buf());
        let app = router(AppState::from_config(&config), config.body_limit_bytes);

        assert_eq!(
            post_with_headers(app.clone(), sample_payload(), None, Some("203.0.113.10")).await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            post_with_headers(
                app,
                sample_payload(),
                Some("wrong-key"),
                Some("203.0.113.10")
            )
            .await,
            StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn missing_or_invalid_cloudflare_ip_is_bad_request() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config = test_config(temp.path().to_path_buf());
        let app = router(AppState::from_config(&config), config.body_limit_bytes);

        assert_eq!(
            post_with_headers(app.clone(), sample_payload(), Some("test-api-key"), None).await,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            post_with_headers(
                app,
                sample_payload(),
                Some("test-api-key"),
                Some("not-an-ip")
            )
            .await,
            StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn payload_digest_limit_is_throttle_not_dedupe() {
        let temp = tempfile::tempdir().expect("tempdir");
        let mut config = test_config(temp.path().to_path_buf());
        config.payload_rate_limit_max = 1;
        let app = router(AppState::from_config(&config), config.body_limit_bytes);

        assert_eq!(
            post_with_headers(
                app.clone(),
                sample_payload(),
                Some("test-api-key"),
                Some("203.0.113.10")
            )
            .await,
            StatusCode::ACCEPTED
        );
        assert_eq!(
            post_with_headers(
                app,
                sample_payload(),
                Some("test-api-key"),
                Some("203.0.113.11")
            )
            .await,
            StatusCode::TOO_MANY_REQUESTS
        );

        assert_eq!(read_all_stats_lines(temp.path()).len(), 1);
    }

    #[tokio::test]
    async fn payload_digest_limit_ignores_event_id() {
        let temp = tempfile::tempdir().expect("tempdir");
        let mut config = test_config(temp.path().to_path_buf());
        config.payload_rate_limit_max = 1;
        let app = router(AppState::from_config(&config), config.body_limit_bytes);
        let mut first_payload = sample_payload_value();
        first_payload["eventId"] = serde_json::json!("550e8400-e29b-41d4-a716-446655440000");
        let mut second_payload = sample_payload_value();
        second_payload["eventId"] = serde_json::json!("550e8400-e29b-41d4-a716-446655440001");

        assert_eq!(
            post_with_headers(
                app.clone(),
                serde_json::to_vec(&first_payload).expect("payload"),
                Some("test-api-key"),
                Some("203.0.113.10")
            )
            .await,
            StatusCode::ACCEPTED
        );
        assert_eq!(
            post_with_headers(
                app,
                serde_json::to_vec(&second_payload).expect("payload"),
                Some("test-api-key"),
                Some("203.0.113.11")
            )
            .await,
            StatusCode::TOO_MANY_REQUESTS
        );

        assert_eq!(read_all_stats_lines(temp.path()).len(), 1);
    }

    #[tokio::test]
    async fn event_id_duplicate_is_retry_deduped() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config = test_config(temp.path().to_path_buf());
        let app = router(AppState::from_config(&config), config.body_limit_bytes);
        let mut payload = sample_payload_value();
        payload["eventId"] = serde_json::json!("550e8400-e29b-41d4-a716-446655440000");
        let body = serde_json::to_vec(&payload).expect("payload");

        assert_eq!(post(app.clone(), body.clone()).await, StatusCode::ACCEPTED);
        assert_eq!(post(app, body).await, StatusCode::ACCEPTED);

        let lines = read_all_stats_lines(temp.path());
        assert_eq!(lines.len(), 1);
        assert!(!lines[0].contains("\"eventId\""));
        assert!(!lines[0].contains("550e8400"));
    }

    #[tokio::test]
    async fn event_id_reuse_with_different_payload_conflicts() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config = test_config(temp.path().to_path_buf());
        let app = router(AppState::from_config(&config), config.body_limit_bytes);
        let mut first_payload = sample_payload_value();
        first_payload["eventId"] = serde_json::json!("550e8400-e29b-41d4-a716-446655440000");
        let mut second_payload = first_payload.clone();
        second_payload["locale"] = serde_json::json!("en-US");

        assert_eq!(
            post_with_headers(
                app.clone(),
                serde_json::to_vec(&first_payload).expect("payload"),
                Some("test-api-key"),
                Some("203.0.113.10")
            )
            .await,
            StatusCode::ACCEPTED
        );
        assert_eq!(
            post_with_headers(
                app,
                serde_json::to_vec(&second_payload).expect("payload"),
                Some("test-api-key"),
                Some("203.0.113.11")
            )
            .await,
            StatusCode::CONFLICT
        );

        assert_eq!(read_all_stats_lines(temp.path()).len(), 1);
    }

    fn read_all_stats_lines(path: &std::path::Path) -> Vec<String> {
        let mut lines = Vec::new();
        collect_lines(path, &mut lines);
        lines
    }

    fn collect_lines(path: &std::path::Path, output: &mut Vec<String>) {
        for entry in std::fs::read_dir(path).expect("read dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                collect_lines(&path, output);
            } else if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".jsonl.open") || name.ends_with(".jsonl"))
            {
                let content = std::fs::read_to_string(path).expect("read stats file");
                output.extend(content.lines().map(str::to_string));
            }
        }
    }
}
