mod recorder;
mod states;
mod utils;

use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};

use crate::{states::recorder_state::RecorderState, utils::upload::upload_session};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = RecorderState::new();
    let (rx, _conn) = recorder::start_midi_listener()?;

    println!("Idle... press any key on the piano to start recording.");

    let idle_timeout = Duration::from_secs(5);
    let mut session_events = Vec::new();
    let mut recording = false;
    let mut session_start = Instant::now();

    let mut session_start_time: Option<DateTime<Utc>> = None;

    loop {
        if let Ok((stamp, msg)) = rx.recv_timeout(Duration::from_millis(200)) {
            // delegate MIDI logic to state
            state.handle_midi(&msg);

            if !recording {
                println!("Started recording session...");
                recording = true;
                session_events.clear();
                session_start = stamp;

                session_start_time = Some(Utc::now());
            }

            let t = (stamp - session_start).as_secs_f64();
            session_events.push((t, msg));
        }

        if recording && state.is_idle(idle_timeout) {
            println!(
                "Session idle for {} seconds. Finalizing...",
                idle_timeout.as_secs()
            );

            if !session_events.is_empty() {
                let smf = recorder::events_to_smf(session_events.clone())?;

                let session_end_time = Utc::now();
                upload_session(
                    &std::env::var("API_ENDPOINT")
                        .unwrap_or("http://localhost:3000/sessions/add".to_string()),
                    &smf,
                    session_start_time.expect("start_time missing"),
                    session_end_time,
                )
                .await?;

                println!("Session uploaded successfully");
            }

            // 🔑 reset musical state
            state.reset();
            recording = false;

            println!("Idle... waiting for next session.");
        }
    }
}
