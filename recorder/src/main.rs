mod recorder;

use std::time::{Duration, Instant};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {


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
                session_start = stamp; // mark first event
            }
            // Compute seconds relative to session start
            let t = (stamp - session_start).as_secs_f64();
            session_events.push((t, msg));
            last_event_time = Instant::now();
        }

        if recording && last_event_time.elapsed() > idle_timeout {
            println!("Session idle for {} seconds. Finalizing...", idle_timeout.as_secs());
            if !session_events.is_empty() {
                let filename = recorder::write_midi_file(session_events.clone())?;
                println!("Saved session as {}", filename);
            }
            recording = false;
            println!("Idle... waiting for next session.");
        }
    }

}
