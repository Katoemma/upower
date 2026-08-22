use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub notifications: NotificationConfig,
    #[serde(default)]
    pub email: EmailConfig,
    #[serde(default)]
    pub push: PushConfig,
    #[serde(default)]
    pub battery: BatteryConfig,
    #[serde(default)]
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub ac_connected: bool,
    #[serde(default = "default_true")]
    pub ac_disconnected: bool,
    #[serde(default = "default_true")]
    pub fully_charged: bool,
    #[serde(default = "default_true")]
    pub low_battery: bool,
    #[serde(default = "default_true")]
    pub critical_battery: bool,
}

/// Email alerts (SMTP credentials come from `.env`, not this file).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// AC connect emails are noisy; off by default.
    #[serde(default = "default_false")]
    pub ac_connected: bool,
    #[serde(default = "default_true")]
    pub ac_disconnected: bool,
    #[serde(default = "default_false")]
    pub fully_charged: bool,
    #[serde(default = "default_true")]
    pub low_battery: bool,
    #[serde(default = "default_true")]
    pub critical_battery: bool,
}

/// Firebase push (credentials via `.env`; device tokens in fcm_tokens.txt).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_false")]
    pub ac_connected: bool,
    #[serde(default = "default_true")]
    pub ac_disconnected: bool,
    #[serde(default = "default_false")]
    pub fully_charged: bool,
    #[serde(default = "default_true")]
    pub low_battery: bool,
    #[serde(default = "default_true")]
    pub critical_battery: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryConfig {
    #[serde(default = "default_low")]
    pub low_threshold: u8,
    #[serde(default = "default_critical")]
    pub critical_threshold: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    #[serde(default = "default_memory_interval")]
    pub memory_interval_ms: u64,
    #[serde(default = "default_cpu_interval")]
    pub cpu_interval_ms: u64,
    #[serde(default = "default_storage_interval")]
    pub storage_interval_ms: u64,
    #[serde(default = "default_processes_interval")]
    pub processes_interval_ms: u64,
    #[serde(default = "default_process_limit")]
    pub process_limit: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            notifications: NotificationConfig::default(),
            email: EmailConfig::default(),
            push: PushConfig::default(),
            battery: BatteryConfig::default(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ac_connected: true,
            ac_disconnected: true,
            fully_charged: true,
            low_battery: true,
            critical_battery: true,
        }
    }
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ac_connected: false,
            ac_disconnected: true,
            fully_charged: false,
            low_battery: true,
            critical_battery: true,
        }
    }
}

impl Default for PushConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ac_connected: false,
            ac_disconnected: true,
            fully_charged: false,
            low_battery: true,
            critical_battery: true,
        }
    }
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            low_threshold: default_low(),
            critical_threshold: default_critical(),
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            memory_interval_ms: default_memory_interval(),
            cpu_interval_ms: default_cpu_interval(),
            storage_interval_ms: default_storage_interval(),
            processes_interval_ms: default_processes_interval(),
            process_limit: default_process_limit(),
        }
    }
}

fn default_host() -> String {
    "127.0.0.1".into()
}

fn default_port() -> u16 {
    8765
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_low() -> u8 {
    20
}

fn default_critical() -> u8 {
    10
}

fn default_memory_interval() -> u64 {
    1000
}

fn default_cpu_interval() -> u64 {
    1000
}

fn default_storage_interval() -> u64 {
    5000
}

fn default_processes_interval() -> u64 {
    2000
}

fn default_process_limit() -> usize {
    20
}

impl Config {
    pub fn load(explicit: Option<&Path>) -> Result<Self> {
        let path = match explicit {
            Some(p) => p.to_path_buf(),
            None => Self::default_config_path(),
        };

        if path.exists() {
            let text = fs::read_to_string(&path)
                .with_context(|| format!("reading config {}", path.display()))?;
            let cfg: Config = toml::from_str(&text)
                .with_context(|| format!("parsing config {}", path.display()))?;
            return Ok(cfg);
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let default = Config::default();
        if let Ok(text) = toml::to_string_pretty(&default) {
            let _ = fs::write(&path, text);
        }
        Ok(default)
    }

    pub fn default_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("power-monitor")
            .join("config.toml")
    }

    pub fn data_dir(&self) -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("power-monitor")
    }

    pub fn api_base(&self) -> String {
        format!("http://{}:{}", self.server.host, self.server.port)
    }
}
