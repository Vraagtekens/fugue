mod recorder;
mod utils;
use std::time::{Duration, Instant};

use crate::utils::upload::upload_session;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (rx, _conn) = recorder::start_midi_listener()?;

    println!("Idle... press any key on the piano to start recording.");

    let idle_timeout = Duration::from_secs(15);
    let mut session_events = Vec::new();
    let mut last_event_time = Instant::now();
    let mut recording = false;
    let mut session_start = Instant::now();

    loop {
        if let Ok((stamp, msg)) = rx.recv_timeout(Duration::from_millis(200)) {
            if !recording {
                println!("Started recording session...");
                recording = true;
                session_events.clear();
                session_start = stamp;
            }

            let t = (stamp - session_start).as_secs_f64();
            session_events.push((t, msg));
            last_event_time = Instant::now();
        }

        if recording && last_event_time.elapsed() > idle_timeout {
            println!(
                "Session idle for {} seconds. Finalizing...",
                idle_timeout.as_secs()
            );

            if !session_events.is_empty() {
                // Convert events → MIDI
                let smf = recorder::events_to_smf(session_events.clone())?;

                // Async HTTP upload
                upload_session(
                    &std::env::var("API_ENDPOINT")
                        .unwrap_or("http://localhost:3000/sessions/add".to_string()),
                    &smf,
                )
                .await?;

                println!("Session uploaded successfully");
            }

            recording = false;
            println!("Idle... waiting for next session.");
        }
    }
}
