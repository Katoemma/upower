use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use crate::api::AppState;
use crate::auth::OptionalAuth;

pub async fn get_memory(
    State(state): State<AppState>,
    OptionalAuth(_user): OptionalAuth,
) -> Json<Value> {
    let snap = state.system.read().await;
    Json(json!({
        "timestamp": snap.timestamp,
        "total_bytes": snap.memory.total_bytes,
        "used_bytes": snap.memory.used_bytes,
        "available_bytes": snap.memory.available_bytes,
        "usage_percent": snap.memory.usage_percent,
        "swap": snap.memory.swap,
    }))
}

pub async fn get_cpu(
    State(state): State<AppState>,
    OptionalAuth(_user): OptionalAuth,
) -> Json<Value> {
    let snap = state.system.read().await;
    Json(json!({
        "timestamp": snap.timestamp,
        "usage_percent": snap.cpu.usage_percent,
        "cores": snap.cpu.cores,
        "per_core": snap.cpu.per_core,
    }))
}

pub async fn get_storage(
    State(state): State<AppState>,
    OptionalAuth(_user): OptionalAuth,
) -> Json<Value> {
    let snap = state.system.read().await;
    Json(json!({
        "timestamp": snap.timestamp,
        "mounts": snap.storage,
    }))
}

pub async fn get_processes(
    State(state): State<AppState>,
    OptionalAuth(_user): OptionalAuth,
) -> Json<Value> {
    let snap = state.system.read().await;
    Json(json!({
        "timestamp": snap.timestamp,
        "processes": snap.processes,
    }))
}

pub async fn get_system(
    State(state): State<AppState>,
    OptionalAuth(_user): OptionalAuth,
) -> Json<Value> {
    let sys = state.system.read().await.clone();
    let power = state.power.read().await;
    let bat = power.primary_battery();

    let mut power_json = json!({ "ac_connected": power.ac_connected });
    if let Some(b) = bat {
        power_json["state"] = json!(b.state);
        if let Some(p) = b.percentage {
            power_json["battery_percentage"] = json!(p);
        }
    }

    Json(json!({
        "timestamp": sys.timestamp,
        "cpu": sys.cpu,
        "memory": sys.memory,
        "storage": sys.storage,
        "processes": sys.processes,
        "power": power_json,
    }))
}
