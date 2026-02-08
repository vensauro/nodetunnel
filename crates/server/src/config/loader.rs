use std::env;
use tracing::{info, warn};
use crate::config::error::ConfigError;

#[derive(Debug, Clone)]
pub struct Config {
    pub udp_bind_address: String,
    pub allowed_versions: Vec<String>,
    pub whitelist: Vec<String>,
    pub remote_whitelist_endpoint: String,
    pub remote_whitelist_token: String,
    pub relay_id: String,
}

pub fn load_config() -> Result<Config, ConfigError> {
    match dotenvy::dotenv() {
        Ok(path) => info!("loaded .env from {}", path.display()),
        Err(_) => info!("no .env file found; using system environment variables (if any)"),
    }

    let cfg = Config {
        udp_bind_address: env_string("UDP_BIND_ADDRESS").unwrap_or_else(defaults::udp_bind_address),
        allowed_versions: env_csv("ALLOWED_VERSIONS").unwrap_or_else(defaults::allowed_versions),
        whitelist: env_csv("WHITELIST").unwrap_or_else(defaults::whitelist),
        remote_whitelist_endpoint: env_string("REMOTE_WHITELIST_ENDPOINT")
            .unwrap_or_else(defaults::empty_string),
        remote_whitelist_token: env_string("REMOTE_WHITELIST_TOKEN")
            .unwrap_or_else(defaults::empty_string),
        relay_id: env_string("RELAY_ID").unwrap_or_else(defaults::empty_string),
    };

    if cfg.allowed_versions.is_empty() {
        warn!("ALLOWED_VERSIONS is empty; all client versions will be treated as allowed unless you enforce otherwise");
    }
    if cfg.remote_whitelist_endpoint.is_empty() && cfg.whitelist.is_empty() {
        warn!("both REMOTE_WHITELIST_ENDPOINT and WHITELIST are empty; no apps will be allowed unless your code treats empty as allow-all");
    }

    Ok(cfg)
}

/// Reads an env var, trims whitespace, treats "" as None
fn env_string(key: &str) -> Option<String> {
    let v = env::var(key).ok()?;
    let v = v.trim().to_string();
    if v.is_empty() { None } else { Some(v) }
}

/// Reads comma-separated values from an env var
/// - "a,b,c" -> ["a","b","c"]
/// - "a" -> ["a"]
/// - "" / not set -> None
fn env_csv(key: &str) -> Option<Vec<String>> {
    let raw = env_string(key)?;
    let items: Vec<String> = raw
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    if items.is_empty() { None } else { Some(items) }
}

mod defaults {
    pub fn udp_bind_address() -> String { "0.0.0.0:8080".to_string() }
    pub fn allowed_versions() -> Vec<String> { vec![] }
    pub fn whitelist() -> Vec<String> { vec![] }
    pub fn empty_string() -> String { "".to_string() }
}
