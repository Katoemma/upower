use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tracing::debug;

use crate::api::AppState;
use crate::auth::OptionalAuth;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    OptionalAuth(_user): OptionalAuth,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.events.subscribe();

    {
        let snap = state.power.read().await;
        let pct = snap.primary_battery().and_then(|b| b.percentage);
        let msg = json!({
            "type": "snapshot",
            "ac_connected": snap.ac_connected,
            "battery_percentage": pct,
            "state": snap.primary_battery().map(|b| b.state),
        });
        if sender
            .send(Message::Text(msg.to_string().into()))
            .await
            .is_err()
        {
            return;
        }
    }

    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let msg = json!({
                "type": "power_event",
                "event": event.event.as_str(),
                "timestamp": event.timestamp.to_rfc3339(),
                "battery_percentage": event.battery_percentage,
                "ac_connected": event.ac_connected,
            });
            if sender
                .send(Message::Text(msg.to_string().into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if matches!(msg, Message::Close(_)) {
                break;
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => { recv_task.abort(); }
        _ = &mut recv_task => { send_task.abort(); }
    }

    debug!("websocket client disconnected");
}
