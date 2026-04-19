use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DEFAULT_PORT: u16 = 9123;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActiveLight {
    pub ip: String,
    pub port: u16,
}

pub fn active_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("holikeyz").join("active.json")
}

pub fn load_active() -> Option<ActiveLight> {
    let text = std::fs::read_to_string(active_config_path()).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn save_active(active: &ActiveLight) -> Result<()> {
    let path = active_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(active)?)?;
    Ok(())
}

/// Resolve the target light from (in order): an explicit override, the
/// persisted `active.json`, or `RING_LIGHT_IP`/`RING_LIGHT_PORT` env vars.
///
/// Returns `None` when nothing is configured — callers should surface a
/// clear message rather than falling back to an arbitrary hardcoded IP,
/// which never matches the user's actual network.
pub fn resolve_active(ip_override: Option<&str>, port_override: Option<u16>) -> Option<ActiveLight> {
    if let Some(ip) = ip_override {
        return Some(ActiveLight {
            ip: ip.to_string(),
            port: port_override.unwrap_or(DEFAULT_PORT),
        });
    }
    if let Some(a) = load_active() {
        return Some(ActiveLight {
            ip: a.ip,
            port: port_override.unwrap_or(a.port),
        });
    }
    if let Ok(ip) = std::env::var("RING_LIGHT_IP") {
        let port = std::env::var("RING_LIGHT_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .or(port_override)
            .unwrap_or(DEFAULT_PORT);
        return Some(ActiveLight { ip, port });
    }
    None
}
