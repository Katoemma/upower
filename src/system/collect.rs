use chrono::Utc;
use sysinfo::{Disks, ProcessesToUpdate, System};

use super::types::{
    CpuSnapshot, MemorySnapshot, ProcessSnapshot, StorageMount, SwapSnapshot, SystemSnapshot,
};

fn pct(used: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (used as f64 / total as f64) * 100.0
    }
}

pub fn collect_memory(sys: &System) -> MemorySnapshot {
    let total = sys.total_memory();
    let used = sys.used_memory();
    let available = sys.available_memory();
    MemorySnapshot {
        total_bytes: total,
        used_bytes: used,
        available_bytes: available,
        usage_percent: pct(used, total),
        swap: SwapSnapshot {
            total_bytes: sys.total_swap(),
            used_bytes: sys.used_swap(),
        },
    }
}

pub fn collect_cpu(sys: &System) -> CpuSnapshot {
    let per_core: Vec<f64> = sys.cpus().iter().map(|c| f64::from(c.cpu_usage())).collect();
    CpuSnapshot {
        usage_percent: f64::from(sys.global_cpu_usage()),
        cores: per_core.len() as u32,
        per_core,
    }
}

pub fn collect_storage() -> Vec<StorageMount> {
    let disks = Disks::new_with_refreshed_list();
    disks
        .iter()
        .map(|d| {
            let total = d.total_space();
            let available = d.available_space();
            let used = total.saturating_sub(available);
            StorageMount {
                mount: d.mount_point().to_string_lossy().into_owned(),
                device: d.name().to_string_lossy().into_owned(),
                filesystem: d.file_system().to_string_lossy().into_owned(),
                total_bytes: total,
                used_bytes: used,
                available_bytes: available,
                usage_percent: pct(used, total),
            }
        })
        .collect()
}

pub fn collect_processes(sys: &System, limit: usize) -> Vec<ProcessSnapshot> {
    let mut rows: Vec<ProcessSnapshot> = sys
        .processes()
        .iter()
        .map(|(pid, proc_)| ProcessSnapshot {
            pid: pid.as_u32(),
            name: proc_.name().to_string_lossy().into_owned(),
            cpu_percent: f64::from(proc_.cpu_usage()),
            memory_bytes: proc_.memory(),
        })
        .collect();
    rows.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
    rows.truncate(limit);
    rows
}

pub fn collect_snapshot(sys: &mut System, process_limit: usize) -> SystemSnapshot {
    sys.refresh_memory();
    sys.refresh_cpu_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    SystemSnapshot {
        timestamp: Utc::now(),
        memory: collect_memory(sys),
        cpu: collect_cpu(sys),
        storage: collect_storage(),
        processes: collect_processes(sys, process_limit),
    }
}
