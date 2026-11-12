use std::env;
use std::error::Error;
use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;

const DEFAULT_BIND_ADDR: &str = "127.0.0.1:5000";
const DEFAULT_DATA_DIR: &str = "./data/stats";
const DEFAULT_BODY_LIMIT_BYTES: usize = 65_536;
const DEFAULT_ROTATE_SECONDS: u64 = 900;
const DEFAULT_ROTATE_BYTES: u64 = 67_108_864;
const DEFAULT_RAW_RETENTION_DAYS: u64 = 180;
const DEFAULT_RATE_LIMIT_TTL_SECONDS: u64 = 60;
const DEFAULT_RATE_LIMIT_MAX: u64 = 120;
const DEFAULT_API_KEY_RATE_LIMIT_MAX: u64 = 100_000;
const DEFAULT_PAYLOAD_RATE_LIMIT_MAX: u64 = 10_000;
const DEFAULT_PAYLOAD_RATE_LIMIT_TTL_SECONDS: u64 = 900;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IngestConfig {
    pub bind_addr: SocketAddr,
    pub data_dir: PathBuf,
    pub hmac_secret: Vec<u8>,
    pub api_key: String,
    pub body_limit_bytes: usize,
    pub rotate_seconds: u64,
    pub rotate_bytes: u64,
    pub raw_retention_days: u64,
    pub rate_limit_ttl_seconds: u64,
    pub rate_limit_max: u64,
    pub api_key_rate_limit_max: u64,
    pub payload_rate_limit_ttl_seconds: u64,
    pub payload_rate_limit_max: u64,
}

impl IngestConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|key| env::var(key).ok())
    }

    pub fn from_lookup<F>(lookup: F) -> Result<Self, ConfigError>
    where
        F: Fn(&str) -> Option<String>,
    {
        let hmac_secret = match lookup("ORIHA_STATS_HMAC_SECRET") {
            Some(secret) if !secret.is_empty() => secret.into_bytes(),
            _ => match lookup("ORIHA_STATS_HMAC_SECRET_FILE") {
                Some(path) if !path.is_empty() => {
                    let bytes = std::fs::read(&path)
                        .map_err(|source| ConfigError::ReadSecretFile { path, source })?;
                    trim_ascii_whitespace(bytes)
                }
                _ => return Err(ConfigError::MissingHmacSecret),
            },
        };

        if hmac_secret.is_empty() {
            return Err(ConfigError::MissingHmacSecret);
        }
        let api_key = secret_from_lookup(
            &lookup,
            "ORIHA_STATS_API_KEY",
            "ORIHA_STATS_API_KEY_FILE",
            || ConfigError::MissingApiKey,
        )?;

        Ok(Self {
            bind_addr: parse_or_default(
                &lookup,
                "ORIHA_STATS_BIND_ADDR",
                DEFAULT_BIND_ADDR
                    .parse()
                    .expect("default bind address must be valid"),
            )?,
            data_dir: PathBuf::from(
                lookup("ORIHA_STATS_DATA_DIR").unwrap_or_else(|| DEFAULT_DATA_DIR.to_string()),
            ),
            hmac_secret,
            api_key,
            body_limit_bytes: parse_or_default(
                &lookup,
                "ORIHA_STATS_BODY_LIMIT_BYTES",
                DEFAULT_BODY_LIMIT_BYTES,
            )?,
            rotate_seconds: parse_or_default(
                &lookup,
                "ORIHA_STATS_ROTATE_SECONDS",
                DEFAULT_ROTATE_SECONDS,
            )?,
            rotate_bytes: parse_or_default(
                &lookup,
                "ORIHA_STATS_ROTATE_BYTES",
                DEFAULT_ROTATE_BYTES,
            )?,
            raw_retention_days: parse_or_default(
                &lookup,
                "ORIHA_STATS_RAW_RETENTION_DAYS",
                DEFAULT_RAW_RETENTION_DAYS,
            )?,
            rate_limit_ttl_seconds: parse_or_default(
                &lookup,
                "ORIHA_STATS_RATE_LIMIT_TTL_SECONDS",
                DEFAULT_RATE_LIMIT_TTL_SECONDS,
            )?,
            rate_limit_max: parse_or_default(
                &lookup,
                "ORIHA_STATS_RATE_LIMIT_MAX",
                DEFAULT_RATE_LIMIT_MAX,
            )?,
            api_key_rate_limit_max: parse_or_default(
                &lookup,
                "ORIHA_STATS_API_KEY_RATE_LIMIT_MAX",
                DEFAULT_API_KEY_RATE_LIMIT_MAX,
            )?,
            payload_rate_limit_ttl_seconds: parse_or_default(
                &lookup,
                "ORIHA_STATS_PAYLOAD_RATE_LIMIT_TTL_SECONDS",
                parse_or_default(
                    &lookup,
                    "ORIHA_STATS_DEDUPE_TTL_SECONDS",
                    DEFAULT_PAYLOAD_RATE_LIMIT_TTL_SECONDS,
                )?,
            )?,
            payload_rate_limit_max: parse_or_default(
                &lookup,
                "ORIHA_STATS_PAYLOAD_RATE_LIMIT_MAX",
                DEFAULT_PAYLOAD_RATE_LIMIT_MAX,
            )?,
        })
    }
}

fn secret_from_lookup<F>(
    lookup: &F,
    value_key: &'static str,
    file_key: &'static str,
    missing_error: impl Fn() -> ConfigError,
) -> Result<String, ConfigError>
where
    F: Fn(&str) -> Option<String>,
{
    let secret = match lookup(value_key) {
        Some(secret) if !secret.is_empty() => secret,
        _ => match lookup(file_key) {
            Some(path) if !path.is_empty() => {
                let bytes = std::fs::read(&path)
                    .map_err(|source| ConfigError::ReadSecretFile { path, source })?;
                String::from_utf8(trim_ascii_whitespace(bytes)).map_err(|source| {
                    ConfigError::InvalidSecretUtf8 {
                        key: file_key,
                        source,
                    }
                })?
            }
            _ => return Err(missing_error()),
        },
    };

    if secret.is_empty() {
        Err(missing_error())
    } else {
        Ok(secret)
    }
}

fn parse_or_default<F, T>(lookup: &F, key: &'static str, default: T) -> Result<T, ConfigError>
where
    F: Fn(&str) -> Option<String>,
    T: FromStr,
    T::Err: fmt::Display,
{
    match lookup(key) {
        Some(value) if !value.is_empty() => {
            value
                .parse::<T>()
                .map_err(|source| ConfigError::InvalidValue {
                    key,
                    value,
                    source: source.to_string(),
                })
        }
        _ => Ok(default),
    }
}

fn trim_ascii_whitespace(mut bytes: Vec<u8>) -> Vec<u8> {
    while bytes.last().is_some_and(u8::is_ascii_whitespace) {
        bytes.pop();
    }
    let first_non_ws = bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    bytes.drain(..first_non_ws);
    bytes
}

#[derive(Debug)]
pub enum ConfigError {
    MissingHmacSecret,
    MissingApiKey,
    ReadSecretFile {
        path: String,
        source: std::io::Error,
    },
    InvalidSecretUtf8 {
        key: &'static str,
        source: std::string::FromUtf8Error,
    },
    InvalidValue {
        key: &'static str,
        value: String,
        source: String,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingHmacSecret => write!(
                f,
                "ORIHA_STATS_HMAC_SECRET or ORIHA_STATS_HMAC_SECRET_FILE must be set"
            ),
            Self::MissingApiKey => {
                write!(
                    f,
                    "ORIHA_STATS_API_KEY or ORIHA_STATS_API_KEY_FILE must be set"
                )
            }
            Self::ReadSecretFile { path, source } => {
                write!(f, "failed to read HMAC secret file {path:?}: {source}")
            }
            Self::InvalidSecretUtf8 { key, source } => {
                write!(f, "{key} must contain UTF-8 text: {source}")
            }
            Self::InvalidValue { key, value, source } => {
                write!(f, "{key} has invalid value {value:?}: {source}")
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadSecretFile { source, .. } => Some(source),
            Self::InvalidSecretUtf8 { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn config_uses_defaults_with_required_secret() {
        let config = IngestConfig::from_lookup(|key| match key {
            "ORIHA_STATS_HMAC_SECRET" => Some("secret".to_string()),
            "ORIHA_STATS_API_KEY" => Some("api-key".to_string()),
            _ => None,
        })
        .expect("config should load");

        assert_eq!(config.bind_addr.to_string(), DEFAULT_BIND_ADDR);
        assert_eq!(config.data_dir, PathBuf::from(DEFAULT_DATA_DIR));
        assert_eq!(config.body_limit_bytes, DEFAULT_BODY_LIMIT_BYTES);
        assert_eq!(config.rotate_seconds, 900);
        assert_eq!(config.raw_retention_days, 180);
        assert_eq!(config.rate_limit_max, 120);
        assert_eq!(config.api_key_rate_limit_max, 100_000);
        assert_eq!(config.payload_rate_limit_ttl_seconds, 900);
        assert_eq!(config.payload_rate_limit_max, 10_000);
    }

    #[test]
    fn config_reads_env_overrides() {
        let values = HashMap::from([
            ("ORIHA_STATS_HMAC_SECRET", "secret"),
            ("ORIHA_STATS_API_KEY", "api-key"),
            ("ORIHA_STATS_BIND_ADDR", "127.0.0.1:7000"),
            ("ORIHA_STATS_DATA_DIR", "/tmp/stats"),
            ("ORIHA_STATS_ROTATE_SECONDS", "120"),
            ("ORIHA_STATS_RAW_RETENTION_DAYS", "10"),
            ("ORIHA_STATS_PAYLOAD_RATE_LIMIT_TTL_SECONDS", "30"),
            ("ORIHA_STATS_PAYLOAD_RATE_LIMIT_MAX", "40"),
            ("ORIHA_STATS_API_KEY_RATE_LIMIT_MAX", "50"),
        ]);
        let config =
            IngestConfig::from_lookup(|key| values.get(key).map(|value| value.to_string()))
                .expect("config should load");

        assert_eq!(config.bind_addr.to_string(), "127.0.0.1:7000");
        assert_eq!(config.data_dir, PathBuf::from("/tmp/stats"));
        assert_eq!(config.rotate_seconds, 120);
        assert_eq!(config.raw_retention_days, 10);
        assert_eq!(config.payload_rate_limit_ttl_seconds, 30);
        assert_eq!(config.payload_rate_limit_max, 40);
        assert_eq!(config.api_key_rate_limit_max, 50);
    }

    #[test]
    fn config_requires_hmac_secret() {
        let err = IngestConfig::from_lookup(|_| None).unwrap_err();
        assert!(matches!(err, ConfigError::MissingHmacSecret));
    }

    #[test]
    fn config_requires_api_key() {
        let err = IngestConfig::from_lookup(|key| {
            (key == "ORIHA_STATS_HMAC_SECRET").then(|| "secret".to_string())
        })
        .unwrap_err();
        assert!(matches!(err, ConfigError::MissingApiKey));
    }
}
