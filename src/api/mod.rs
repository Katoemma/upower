use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::{broadcast, RwLock};
use tower_http::trace::TraceLayer;

use crate::database::EventStore;
use crate::power::{PowerEvent, PowerState};
use crate::push;
use crate::websocket;

#[derive(Clone)]
pub struct AppState {
    pub power: Arc<RwLock<PowerState>>,
    pub store: Arc<EventStore>,
    pub events: broadcast::Sender<PowerEvent>,
    pub data_dir: PathBuf,
}

pub fn router(
    power: Arc<RwLock<PowerState>>,
    store: Arc<EventStore>,
    events: broadcast::Sender<PowerEvent>,
    data_dir: PathBuf,
) -> Router {
    let state = AppState {
        power,
        store,
        events,
        data_dir,
    };

    Router::new()
        .route("/api/v1/power", get(get_power))
        .route("/api/v1/battery", get(get_battery))
        .route("/api/v1/power/status", get(get_power_status))
        .route("/api/v1/events", get(get_events))
        .route("/api/v1/push/tokens", get(list_push_tokens).post(register_push_token))
        .route("/ws", get(websocket::ws_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn get_power(State(state): State<AppState>) -> Json<Value> {
    let snap = state.power.read().await;
    let bat = snap.primary_battery();
    let mut body = json!({
        "ac_connected": snap.ac_connected,
    });

    if let Some(b) = bat {
        body["state"] = json!(b.state);
        if let Some(p) = b.percentage {
            body["battery_percentage"] = json!(p);
        }
        if let Some(h) = b.health {
            body["battery_health"] = json!(h);
        }
        if let Some(t) = b.time_to_full_seconds {
            body["time_to_full_seconds"] = json!(t);
        } else {
            body["time_to_full_seconds"] = Value::Null;
        }
        if let Some(t) = b.time_remaining_seconds {
            body["time_remaining_seconds"] = json!(t);
        } else {
            body["time_remaining_seconds"] = Value::Null;
        }
    } else {
        body["battery"] = Value::Null;
    }

    Json(body)
}

async fn get_battery(State(state): State<AppState>) -> Json<Value> {
    let snap = state.power.read().await;
    match snap.primary_battery() {
        None => Json(json!({ "battery": null })),
        Some(b) => {
            let mut body = json!({
                "id": b.id,
                "state": b.state,
            });
            if let Some(p) = b.percentage {
                body["percentage"] = json!(p);
            }
            if let Some(h) = b.health {
                body["health"] = json!(h);
            }
            if let Some(v) = b.energy_now_wh {
                body["energy_now_wh"] = json!(v);
            }
            if let Some(v) = b.energy_full_wh {
                body["energy_full_wh"] = json!(v);
            }
            if let Some(v) = b.energy_full_design_wh {
                body["energy_full_design_wh"] = json!(v);
            }
            if let Some(v) = b.temperature_celsius {
                body["temperature_celsius"] = json!(v);
            }

            // Multi-battery list (excluding DisplayDevice duplicate when physical listed).
            let batteries: Vec<Value> = snap
                .batteries
                .iter()
                .filter(|x| x.id != "DisplayDevice")
                .map(|x| {
                    let mut m = json!({ "id": x.id, "state": x.state });
                    if let Some(p) = x.percentage {
                        m["percentage"] = json!(p);
                    }
                    m
                })
                .collect();
            if !batteries.is_empty() {
                body["batteries"] = json!(batteries);
            }

            Json(body)
        }
    }
}

async fn get_power_status(State(state): State<AppState>) -> Json<Value> {
    let snap = state.power.read().await;
    Json(json!({
        "connected": snap.ac_connected,
        "source": if snap.ac_connected { "AC" } else { "Battery" },
    }))
}

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

async fn get_events(
    State(state): State<AppState>,
    Query(q): Query<EventsQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, String)> {
    let page = q.page.unwrap_or(1);
    let limit = q.limit.unwrap_or(50);
    let events = state
        .store
        .query(
            page,
            limit,
            q.event_type.as_deref(),
            q.from.as_deref(),
            q.to.as_deref(),
        )
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
            )
        })?;

    let items: Vec<Value> = events
        .into_iter()
        .map(|e| {
            json!({
                "event": e.event.as_str(),
                "timestamp": e.timestamp.to_rfc3339(),
                "battery_percentage": e.battery_percentage,
                "battery_state": e.battery_state.map(|s| s.as_str()),
                "ac_connected": e.ac_connected,
            })
        })
        .collect();

    Ok(Json(json!({
        "page": page,
        "limit": limit,
        "events": items,
    })))
}

#[derive(Debug, Deserialize)]
pub struct PushTokenBody {
    pub token: String,
}

async fn list_push_tokens(State(state): State<AppState>) -> Json<Value> {
    let path = state.data_dir.join("fcm_tokens.txt");
    let tokens = push::load_tokens_file(&path).unwrap_or_default();
    Json(json!({
        "count": tokens.len(),
        "tokens": tokens,
    }))
}

async fn register_push_token(
    State(state): State<AppState>,
    Json(body): Json<PushTokenBody>,
) -> Result<Json<Value>, (axum::http::StatusCode, String)> {
    let path = state.data_dir.join("fcm_tokens.txt");
    let mut tokens = push::load_tokens_file(&path).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            e.to_string(),
        )
    })?;
    let token = body.token.trim().to_string();
    if token.is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "token is required".into(),
        ));
    }
    if !tokens.iter().any(|t| t == &token) {
        tokens.push(token.clone());
        push::save_tokens_file(&path, &tokens).map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
            )
        })?;
    }
    Ok(Json(json!({ "ok": true, "count": tokens.len() })))
}
