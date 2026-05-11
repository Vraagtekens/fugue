use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LiveSessionEvent {
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

impl LiveSessionEvent {
    pub fn session_id(&self) -> &str {
        match self {
            Self::SessionStarted { session_id, .. }
            | Self::MidiEvent { session_id, .. }
            | Self::SessionFinished { session_id, .. } => session_id,
        }
    }
}

#[derive(Clone)]
pub struct LiveSessionHub {
    sender: broadcast::Sender<LiveSessionEvent>,
}

impl LiveSessionHub {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn publish(&self, event: LiveSessionEvent) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LiveSessionEvent> {
        self.sender.subscribe()
    }
}
