use std::sync::Arc;
use std::time::Duration;

use sysinfo::System;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, MissedTickBehavior};
use tracing::error;

use crate::config::MonitoringConfig;

use super::collect::{collect_cpu, collect_memory, collect_processes, collect_snapshot, collect_storage};
use super::types::{SystemSnapshot, TelemetryFrame};

pub struct SystemMonitor {
    state: Arc<RwLock<SystemSnapshot>>,
    telemetry_tx: broadcast::Sender<TelemetryFrame>,
    config: MonitoringConfig,
}

impl SystemMonitor {
    pub fn new(
        state: Arc<RwLock<SystemSnapshot>>,
        telemetry_tx: broadcast::Sender<TelemetryFrame>,
        config: MonitoringConfig,
    ) -> Self {
        Self {
            state,
            telemetry_tx,
            config,
        }
    }

    pub async fn run(self) {
        let mut sys = System::new_all();
        sys.refresh_all();
        tokio::time::sleep(Duration::from_millis(250)).await;

        let snap = collect_snapshot(&mut sys, self.config.process_limit);
        *self.state.write().await = snap;

        let mut memory_tick = interval(Duration::from_millis(self.config.memory_interval_ms));
        let mut cpu_tick = interval(Duration::from_millis(self.config.cpu_interval_ms));
        let mut storage_tick = interval(Duration::from_millis(self.config.storage_interval_ms));
        let mut processes_tick = interval(Duration::from_millis(self.config.processes_interval_ms));

        for tick in [
            &mut memory_tick,
            &mut cpu_tick,
            &mut storage_tick,
            &mut processes_tick,
        ] {
            tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
        }

        loop {
            tokio::select! {
                _ = memory_tick.tick() => {
                    sys.refresh_memory();
                    let mem = collect_memory(&sys);
                    let frame = TelemetryFrame::Memory {
                        timestamp: chrono::Utc::now(),
                        used_percent: mem.usage_percent,
                        used_bytes: mem.used_bytes,
                        available_bytes: mem.available_bytes,
                    };
                    {
                        let mut guard = self.state.write().await;
                        guard.timestamp = chrono::Utc::now();
                        guard.memory = mem;
                    }
                    let _ = self.telemetry_tx.send(frame);
                }
                _ = cpu_tick.tick() => {
                    sys.refresh_cpu_all();
                    let cpu = collect_cpu(&sys);
                    let frame = TelemetryFrame::Cpu {
                        timestamp: chrono::Utc::now(),
                        usage_percent: cpu.usage_percent,
                        cores: cpu.cores,
                        per_core: cpu.per_core.clone(),
                    };
                    {
                        let mut guard = self.state.write().await;
                        guard.timestamp = chrono::Utc::now();
                        guard.cpu = cpu;
                    }
                    let _ = self.telemetry_tx.send(frame);
                }
                _ = storage_tick.tick() => {
                    let mounts = collect_storage();
                    let frame = TelemetryFrame::Storage {
                        timestamp: chrono::Utc::now(),
                        mounts: mounts.clone(),
                    };
                    {
                        let mut guard = self.state.write().await;
                        guard.timestamp = chrono::Utc::now();
                        guard.storage = mounts;
                    }
                    let _ = self.telemetry_tx.send(frame);
                }
                _ = processes_tick.tick() => {
                    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                    let processes = collect_processes(&sys, self.config.process_limit);
                    let frame = TelemetryFrame::Processes {
                        timestamp: chrono::Utc::now(),
                        processes: processes.clone(),
                    };
                    {
                        let mut guard = self.state.write().await;
                        guard.timestamp = chrono::Utc::now();
                        guard.processes = processes;
                    }
                    let _ = self.telemetry_tx.send(frame);
                }
            }
        }
    }
}
