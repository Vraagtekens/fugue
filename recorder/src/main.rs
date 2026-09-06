mod config;
mod recorder;
mod states;
mod utils;

use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};

use crate::{
    config::Config,
    states::recorder_state::RecorderState,
    utils::{live::LiveClient, upload::upload_session},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let mut state = RecorderState::new();
    let rx = recorder::start_midi_listener(&config.midi_port_name)?;
    let live = config
        .live_ws_endpoint
        .clone()
        .map(|endpoint| LiveClient::start(endpoint, config.api_key.clone()));

    println!("Idle... press any key on the piano to start recording.");

    let mut session_events = Vec::new();
    let mut recording = false;
    let mut session_start = Instant::now();

    let mut session_start_time: Option<DateTime<Utc>> = None;
    let mut session_id: Option<String> = None;
    let mut session_title: Option<String> = None;

    loop {
        if let Ok(event) = rx.recv_timeout(Duration::from_millis(200)) {
            let recorder::MidiListenerEvent::Message(stamp, msg) = event else {
                state.reset();
                eprintln!("MIDI connection lost; cleared held notes and pedal state.");
                continue;
            };

            // delegate MIDI logic to state
            state.handle_midi(&msg);

            if !recording {
                println!("Started recording session...");
                recording = true;
                session_events.clear();
                session_start = stamp;

                let started_at = Utc::now();
                let id = format!("fp30x-{}", started_at.timestamp());
                session_start_time = Some(started_at);
                session_id = Some(id.clone());
                session_title = Some(id.clone());

                if let Some(live) = &live {
                    live.session_started(&id, &id, started_at);
                }
            }

            let t = (stamp - session_start).as_secs_f64();
            if let (Some(live), Some(id)) = (&live, &session_id) {
                live.midi_event(id, (t * 1000.0).round() as u64, msg.clone());
            }
            session_events.push((t, msg));
        }

        if recording && state.is_idle(config.idle_timeout) {
            println!(
                "Session idle for {} seconds. Finalizing...",
                config.idle_timeout.as_secs()
            );

            if !session_events.is_empty() {
                let smf = recorder::events_to_smf(&session_events)?;

                let session_end_time = Utc::now();
                if let (Some(live), Some(id)) = (&live, &session_id) {
                    live.session_finished(id, session_end_time);
                }

                upload_session(
                    &config.api_endpoint,
                    &config.api_key,
                    &config.user_id,
                    session_title.as_deref(),
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
            session_id = None;
            session_title = None;

            println!("Idle... waiting for next session.");
        }
    }
}
