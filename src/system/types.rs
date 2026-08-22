use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SwapSnapshot {
    pub total_bytes: u64,
    pub used_bytes: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
    pub swap: SwapSnapshot,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuSnapshot {
    pub usage_percent: f64,
    pub cores: u32,
    pub per_core: Vec<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StorageMount {
    pub mount: String,
    pub device: String,
    pub filesystem: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessSnapshot {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f64,
    pub memory_bytes: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub timestamp: DateTime<Utc>,
    pub memory: MemorySnapshot,
    pub cpu: CpuSnapshot,
    pub storage: Vec<StorageMount>,
    pub processes: Vec<ProcessSnapshot>,
}

/// Live telemetry frame pushed over WebSocket `/api/v1/stream`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TelemetryFrame {
    Memory {
        timestamp: DateTime<Utc>,
        used_percent: f64,
        used_bytes: u64,
        available_bytes: u64,
    },
    Cpu {
        timestamp: DateTime<Utc>,
        usage_percent: f64,
        cores: u32,
        per_core: Vec<f64>,
    },
    Storage {
        timestamp: DateTime<Utc>,
        mounts: Vec<StorageMount>,
    },
    Processes {
        timestamp: DateTime<Utc>,
        processes: Vec<ProcessSnapshot>,
    },
}
