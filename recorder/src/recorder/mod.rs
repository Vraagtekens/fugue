use chrono::Local;
use midir::MidiInput;
use midly::{Smf, Header, Format, Timing, TrackEvent, TrackEventKind, MetaMessage, MidiMessage, num::u28, num::u24};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Instant, SystemTime, UNIX_EPOCH, Duration};
use std::{fs, thread};

pub type MidiEvent = (f64, Vec<u8>);

/// Starts listening to a MIDI port and returns a receiver channel for events
pub fn start_midi_listener() -> Result<(Receiver<(Instant, Vec<u8>)>, midir::MidiInputConnection<()>), Box<dyn std::error::Error>> {
    let mut midi_in = MidiInput::new("midi-rec")?;
    midi_in.ignore(midir::Ignore::None);

    println!("Waiting for MIDI device...");

    // <--- REPLACE the original in_ports[0] selection with this loop
   let port = loop {
    let in_ports = midi_in.ports();

    // try to find a port whose name contains "Roland"
    if let Some(p) = in_ports.iter().find(|p| midi_in.port_name(p).unwrap().contains("Roland")) {
        let p = p.clone(); // clone to own it
        println!("Roland MIDI device found: {}", midi_in.port_name(&p)?);
        break p;
    }

    println!("Waiting for Roland MIDI device...");
    std::thread::sleep(std::time::Duration::from_secs(1));
};

    let (tx, rx) = mpsc::channel();

    // connect MIDI input
    let conn = midi_in.connect(&port, "midir-read", move |_, msg, _| {
        let _ = tx.send((Instant::now(), msg.to_vec()));
    }, ())?;

    Ok((rx, conn))
}


/// Write MIDI events to a timestamped .mid file
pub fn write_midi_file(events: Vec<MidiEvent>) -> Result<String, Box<dyn std::error::Error>> {
    if events.is_empty() {
        return Err("No events to write".into());
    }

    let ppqn = 480u16;
    let bpm = 120.0;
    let microsec_per_quarter = 60_000_000f64 / bpm;
    let sec_to_ticks = |s: f64| -> u32 {
        ((s * 1_000_000.0) / microsec_per_quarter * (ppqn as f64)).round() as u32
    };

    let mut track_events = Vec::<(u32, TrackEventKind)>::new();
    track_events.push((0, TrackEventKind::Meta(MetaMessage::Tempo(u24::new(microsec_per_quarter as u32)))));

    let start_time = events.first().unwrap().0;
    for (t, bytes) in &events {
        let tick = sec_to_ticks(*t - start_time);
        if bytes.len() == 3 {
            let status = bytes[0] & 0xF0;
            let channel = bytes[0] & 0x0F;
            let key = bytes[1];
            let vel = bytes[2];

            let kind = match status {
                0x90 if vel > 0 => TrackEventKind::Midi {
                    channel: channel.into(),
                    message: MidiMessage::NoteOn { key: key.into(), vel: vel.into() },
                },
                0x80 | 0x90 => TrackEventKind::Midi {
                    channel: channel.into(),
                    message: MidiMessage::NoteOff { key: key.into(), vel: vel.into() },
                },
                _ => continue,
            };
            track_events.push((tick, kind));
        }
    }

    track_events.sort_by_key(|(t, _)| *t);
    let mut last_tick = 0u32;
    let mut midly_events = Vec::new();
    for (abs_tick, kind) in track_events {
        let delta = abs_tick.saturating_sub(last_tick);
        midly_events.push(TrackEvent { delta: u28::new(delta), kind });
        last_tick = abs_tick;
    }
    midly_events.push(TrackEvent { delta: u28::new(0), kind: TrackEventKind::Meta(MetaMessage::EndOfTrack) });

    let smf = Smf {
        header: Header { format: Format::SingleTrack, timing: Timing::Metrical(ppqn.into()) },
        tracks: vec![midly_events],
    };

    

    // Get current day string, e.g., "2025-12-23"
    let day = Local::now().format("%Y-%m-%d").to_string();

    // Create folder if it doesn't exist
    let dir_path = format!("sessions/{}", day);
    fs::create_dir_all(&dir_path)?; // creates parent directories if needed

    // Timestamp for filename
    let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let filename = format!("{}/piano-{}.mid", dir_path, ts);

    // Write the file
    let mut out = fs::File::create(&filename)?;
    smf.write_std(&mut out)?;
    Ok(filename)
}
