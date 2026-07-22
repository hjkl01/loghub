use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use chrono::{DateTime, Local};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::{broadcast, Mutex};

use crate::AppState;

#[derive(Clone, Debug)]
pub struct LogEvent {
    pub id: i64,
    pub log_time: DateTime<Local>,
    pub level: String,
    pub message: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub file_name: Option<String>,
    pub function_name: Option<String>,
    pub line_number: Option<i32>,
    pub extra: Value,
}

#[derive(Clone)]
pub struct WsState {
    pub tx: broadcast::Sender<LogEvent>,
}

impl WsState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self { tx }
    }
}

#[derive(Deserialize, Clone)]
struct WsFilter {
    service: Option<String>,
    level: Option<Vec<String>>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let ws_state = state.ws_state.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, ws_state))
}

async fn handle_socket(socket: WebSocket, ws_state: Arc<WsState>) {
    let (mut sender, mut receiver) = socket.split();

    let mut rx = ws_state.tx.subscribe();
    let filter = Arc::new(Mutex::new(None::<WsFilter>));

    let send_filter = filter.clone();
    let send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let f = send_filter.lock().await;
            if let Some(ref f) = *f {
                if let Some(ref service) = f.service {
                    if &event.service != service {
                        continue;
                    }
                }
                if let Some(ref levels) = f.level {
                    if !levels.contains(&event.level) {
                        continue;
                    }
                }
            }
            drop(f);

            let msg = serde_json::json!({
                "type": "log",
                "data": {
                    "id": event.id,
                    "time": event.log_time,
                    "level": event.level,
                    "message": event.message,
                    "service": event.service,
                    "trace_id": event.trace_id,
                    "file_name": event.file_name,
                    "function_name": event.function_name,
                    "line_number": event.line_number,
                }
            });

            if sender.send(Message::Text(msg.to_string())).await.is_err() {
                break;
            }
        }
    });

    let recv_filter = filter.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(f) = serde_json::from_str::<WsFilter>(&text) {
                        let mut guard = recv_filter.lock().await;
                        *guard = Some(f);
                    }
                }
                Ok(Message::Close(_)) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}
