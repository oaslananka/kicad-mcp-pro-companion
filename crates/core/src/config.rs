//! Layered configuration: CLI flags > environment variables > defaults.
//!
//! `load_from` is the pure, testable core (takes an explicit env map so
//! tests never mutate real process environment variables, which would be
//! unsound under parallel test execution). `load` is the thin production
//! entry point that reads real environment variables.

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use url::Url;

use crate::error::CompanionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode {
    Disabled,
    Mock,
}

impl FromStr for TransportMode {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "disabled" => Ok(TransportMode::Disabled),
            "mock" => Ok(TransportMode::Mock),
            other => Err(ConfigError::InvalidTransportMode(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CliOverrides {
    pub data_dir: Option<PathBuf>,
    pub log_level: Option<String>,
    pub core_bridge_endpoint: Option<String>,
    pub transport_mode: Option<TransportMode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompanionConfig {
    pub data_dir: PathBuf,
    pub log_level: String,
    pub core_bridge_endpoint: Url,
    pub transport_mode: TransportMode,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("invalid core bridge endpoint url: {0}")]
    InvalidEndpoint(String),
    #[error("invalid transport mode: {0}")]
    InvalidTransportMode(String),
}

impl CompanionError for ConfigError {
    fn code(&self) -> &'static str {
        match self {
            ConfigError::InvalidEndpoint(_) => "CONFIG_INVALID_ENDPOINT",
            ConfigError::InvalidTransportMode(_) => "CONFIG_INVALID_TRANSPORT_MODE",
        }
    }

    fn retryable(&self) -> bool {
        false
    }
}

const DEFAULT_LOG_LEVEL: &str = "info";
const DEFAULT_CORE_BRIDGE_ENDPOINT: &str = "http://127.0.0.1:3334/mcp";
const DEFAULT_TRANSPORT_MODE: TransportMode = TransportMode::Disabled;
const ENV_DATA_DIR: &str = "COMPANION_DATA_DIR";
const ENV_LOG_LEVEL: &str = "COMPANION_LOG_LEVEL";
const ENV_CORE_BRIDGE_ENDPOINT: &str = "COMPANION_CORE_BRIDGE_ENDPOINT";
const ENV_TRANSPORT_MODE: &str = "COMPANION_TRANSPORT_MODE";

/// Production entry point: layers real process environment variables under
/// `overrides`.
pub fn load(overrides: CliOverrides) -> Result<CompanionConfig, ConfigError> {
    let env: HashMap<String, String> = std::env::vars().collect();
    load_from(overrides, &env)
}

/// Pure precedence resolution: `overrides` > `env` > built-in defaults.
pub fn load_from(
    overrides: CliOverrides,
    env: &HashMap<String, String>,
) -> Result<CompanionConfig, ConfigError> {
    let data_dir = overrides
        .data_dir
        .or_else(|| env.get(ENV_DATA_DIR).map(PathBuf::from))
        .unwrap_or_else(default_data_dir);

    let log_level = overrides
        .log_level
        .or_else(|| env.get(ENV_LOG_LEVEL).cloned())
        .unwrap_or_else(|| DEFAULT_LOG_LEVEL.to_string());

    let endpoint_str = overrides
        .core_bridge_endpoint
        .or_else(|| env.get(ENV_CORE_BRIDGE_ENDPOINT).cloned())
        .unwrap_or_else(|| DEFAULT_CORE_BRIDGE_ENDPOINT.to_string());
    let core_bridge_endpoint = Url::parse(&endpoint_str)
        .map_err(|_| ConfigError::InvalidEndpoint(endpoint_str.clone()))?;

    let transport_mode = match overrides.transport_mode {
        Some(mode) => mode,
        None => match env.get(ENV_TRANSPORT_MODE) {
            Some(raw) => raw.parse()?,
            None => DEFAULT_TRANSPORT_MODE,
        },
    };

    Ok(CompanionConfig {
        data_dir,
        log_level,
        core_bridge_endpoint,
        transport_mode,
    })
}

fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("kicad-mcp-companion")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_apply_when_nothing_is_set() {
        let config = load_from(CliOverrides::default(), &HashMap::new()).unwrap();
        assert_eq!(config.log_level, "info");
        assert_eq!(
            config.core_bridge_endpoint.as_str(),
            "http://127.0.0.1:3334/mcp"
        );
        assert_eq!(config.transport_mode, TransportMode::Disabled);
    }

    #[test]
    fn mock_transport_requires_explicit_configuration() {
        let mut env = HashMap::new();
        env.insert(ENV_TRANSPORT_MODE.to_string(), "mock".to_string());
        let config = load_from(CliOverrides::default(), &env).unwrap();
        assert_eq!(config.transport_mode, TransportMode::Mock);
    }

    #[test]
    fn env_var_overrides_default() {
        let mut env = HashMap::new();
        env.insert(ENV_LOG_LEVEL.to_string(), "debug".to_string());
        let config = load_from(CliOverrides::default(), &env).unwrap();
        assert_eq!(config.log_level, "debug");
    }

    #[test]
    fn cli_override_beats_env() {
        let mut env = HashMap::new();
        env.insert(ENV_LOG_LEVEL.to_string(), "debug".to_string());
        let overrides = CliOverrides {
            log_level: Some("trace".to_string()),
            ..Default::default()
        };
        let config = load_from(overrides, &env).unwrap();
        assert_eq!(config.log_level, "trace");
    }

    #[test]
    fn malformed_endpoint_produces_typed_error_not_panic() {
        let overrides = CliOverrides {
            core_bridge_endpoint: Some("not a url".to_string()),
            ..Default::default()
        };
        let result = load_from(overrides, &HashMap::new());
        assert_eq!(
            result,
            Err(ConfigError::InvalidEndpoint("not a url".to_string()))
        );
    }

    #[test]
    fn non_loopback_endpoint_is_accepted_at_parse_time() {
        // Loopback-only enforcement is core-bridge's responsibility, not config's.
        let overrides = CliOverrides {
            core_bridge_endpoint: Some("http://example.com/mcp".to_string()),
            ..Default::default()
        };
        let config = load_from(overrides, &HashMap::new()).unwrap();
        assert_eq!(config.core_bridge_endpoint.host_str(), Some("example.com"));
    }

    #[test]
    fn invalid_transport_mode_is_a_typed_error() {
        let mut env = HashMap::new();
        env.insert(ENV_TRANSPORT_MODE.to_string(), "quantum-relay".to_string());
        let result = load_from(CliOverrides::default(), &env);
        assert_eq!(
            result,
            Err(ConfigError::InvalidTransportMode(
                "quantum-relay".to_string()
            ))
        );
    }

    #[test]
    fn error_codes_are_stable_and_not_retryable() {
        let err = ConfigError::InvalidEndpoint("x".into());
        assert_eq!(err.code(), "CONFIG_INVALID_ENDPOINT");
        assert!(!err.retryable());
    }
}
