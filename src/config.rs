use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub connection: ConnectionConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub host: Option<String>,
    #[serde(default = "default_discovery_timeout")]
    pub discovery_timeout: u64,
    #[serde(default = "default_reconnect_delay")]
    pub reconnect_delay: u64,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            host: None,
            discovery_timeout: default_discovery_timeout(),
            reconnect_delay: default_reconnect_delay(),
        }
    }
}

fn default_discovery_timeout() -> u64 {
    5
}

fn default_reconnect_delay() -> u64 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_volume_step")]
    pub volume_step: u8,
    #[serde(default = "default_refresh_rate")]
    pub refresh_rate: u64,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            volume_step: default_volume_step(),
            refresh_rate: default_refresh_rate(),
        }
    }
}

fn default_volume_step() -> u8 {
    5
}

fn default_refresh_rate() -> u64 {
    250
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }

    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("heos-tui")
            .join("config.toml")
    }
}
