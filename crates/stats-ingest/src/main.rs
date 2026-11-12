use oriha_stats_ingest::app::{router, AppState};
use oriha_stats_ingest::config::IngestConfig;
use std::error::Error;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(env_filter)
        .with_level(true)
        .with_target(false)
        .init();

    let config = IngestConfig::from_env()?;
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    let state = AppState::from_config(&config);
    let writer = state.writer.clone();
    let app = router(state, config.body_limit_bytes);

    let serve_result = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await;
    let shutdown_result = writer.shutdown().await;

    serve_result?;
    shutdown_result?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            signal.recv().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
