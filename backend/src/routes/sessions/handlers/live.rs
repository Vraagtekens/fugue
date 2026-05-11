use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use tracing::{debug, info, warn};

use crate::{errors::ApiError, services::live_service::LiveSessionEvent, state::AppState};

pub async fn record_ws(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    Ok(ws.on_upgrade(move |socket| handle_recorder_socket(state, socket)))
}

pub async fn subscribe_ws(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    Ok(ws.on_upgrade(move |socket| handle_subscriber_socket(state, session_id, socket)))
}

async fn handle_recorder_socket(state: AppState, mut socket: WebSocket) {
    info!("live recorder connected");

    while let Some(message) = socket.next().await {
        match message {
            Ok(Message::Text(text)) => match serde_json::from_str::<LiveSessionEvent>(&text) {
                Ok(event) => {
                    debug!(session_id = %event.session_id(), "live event received");
                    state.live.publish(event);
                }
                Err(err) => warn!(error = %err, "invalid live event payload"),
            },
            Ok(Message::Binary(bytes)) => {
                match serde_json::from_slice::<LiveSessionEvent>(&bytes) {
                    Ok(event) => state.live.publish(event),
                    Err(err) => warn!(error = %err, "invalid binary live event payload"),
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(Message::Ping(_) | Message::Pong(_)) => {}
            Err(err) => {
                warn!(error = %err, "live recorder socket error");
                break;
            }
        }
    }

    info!("live recorder disconnected");
}

async fn handle_subscriber_socket(state: AppState, session_id: String, socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();
    let mut events = state.live.subscribe();

    info!(session_id = %session_id, "live subscriber connected");

    loop {
        tokio::select! {
            event = events.recv() => {
                let Ok(event) = event else {
                    continue;
                };

                if event.session_id() != session_id {
                    continue;
                }

                match serde_json::to_string(&event) {
                    Ok(payload) => {
                        if sender.send(Message::Text(payload.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(err) => warn!(error = %err, "failed to serialize live event"),
                }
            }
            message = receiver.next() => {
                match message {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(err)) => {
                        warn!(error = %err, "live subscriber socket error");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    info!(session_id = %session_id, "live subscriber disconnected");
}
