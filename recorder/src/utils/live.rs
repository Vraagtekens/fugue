use chrono::{DateTime, Utc};
use futures_util::SinkExt;
use serde::Serialize;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, client::IntoClientRequest, http::HeaderValue},
};

#[derive(Clone)]
pub struct LiveClient {
    sender: mpsc::Sender<LiveSessionEvent>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum LiveSessionEvent {
    SessionStarted {
        session_id: String,
        title: String,
        started_at: String,
    },
    MidiEvent {
        session_id: String,
        timestamp_ms: u64,
        bytes: Vec<u8>,
    },
    SessionFinished {
        session_id: String,
        ended_at: String,
    },
}

impl LiveClient {
    pub fn start(endpoint: String, api_key: String) -> Self {
        let (sender, receiver) = mpsc::channel(2048);

        tokio::spawn(async move {
            if let Err(err) = run_live_stream(endpoint, api_key, receiver).await {
                eprintln!("Live stream disabled: {err}");
            }
        });

        Self { sender }
    }

    pub fn session_started(&self, session_id: &str, title: &str, started_at: DateTime<Utc>) {
        self.publish(LiveSessionEvent::SessionStarted {
            session_id: session_id.to_string(),
            title: title.to_string(),
            started_at: started_at.to_rfc3339(),
        });
    }

    pub fn midi_event(&self, session_id: &str, timestamp_ms: u64, bytes: Vec<u8>) {
        self.publish(LiveSessionEvent::MidiEvent {
            session_id: session_id.to_string(),
            timestamp_ms,
            bytes,
        });
    }

    pub fn session_finished(&self, session_id: &str, ended_at: DateTime<Utc>) {
        self.publish(LiveSessionEvent::SessionFinished {
            session_id: session_id.to_string(),
            ended_at: ended_at.to_rfc3339(),
        });
    }

    fn publish(&self, event: LiveSessionEvent) {
        if let Err(err) = self.sender.try_send(event) {
            eprintln!("Dropped live MIDI event: {err}");
        }
    }
}

async fn run_live_stream(
    endpoint: String,
    api_key: String,
    mut receiver: mpsc::Receiver<LiveSessionEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let api_key_header: HeaderValue = api_key.parse()?;
    let mut pending: Option<LiveSessionEvent> = None;

    loop {
        let event = match pending.take() {
            Some(event) => event,
            None => match receiver.recv().await {
                Some(event) => event,
                None => return Ok(()),
            },
        };

        let mut request = endpoint.clone().into_client_request()?;
        request
            .headers_mut()
            .insert("x-api-key", api_key_header.clone());

        let Ok((mut socket, _)) = connect_async(request).await else {
            pending = Some(event);
            tokio::time::sleep(Duration::from_secs(2)).await;
            continue;
        };

        println!("Live MIDI stream connected.");
        pending = Some(event);

        loop {
            let event = match pending.take() {
                Some(event) => event,
                None => match receiver.recv().await {
                    Some(event) => event,
                    None => return Ok(()),
                },
            };

            let payload = serde_json::to_string(&event)?;

            if socket.send(Message::Text(payload.into())).await.is_err() {
                pending = Some(event);
                eprintln!("Live MIDI stream disconnected; reconnecting...");
                break;
            }
        }
    }
}
