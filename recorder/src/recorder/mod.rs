use midir::MidiInput;
use midly::{
    Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind, num::u24,
    num::u28,
};
use std::sync::mpsc::{self, Receiver};
use std::time::Instant;

pub type MidiEvent = (f64, Vec<u8>);

/// Starts listening to a MIDI port and returns a receiver channel for events
pub fn start_midi_listener(
    port_name_match: &str,
) -> Result<
    (Receiver<(Instant, Vec<u8>)>, midir::MidiInputConnection<()>),
    Box<dyn std::error::Error>,
> {
    println!("Waiting for MIDI device...");

    let (tx, rx) = mpsc::channel();

    loop {
        let mut midi_in = MidiInput::new("midi-rec")?;
        midi_in.ignore(midir::Ignore::None);

        let in_ports = midi_in.ports();
        let port_names = in_ports
            .iter()
            .filter_map(|p| midi_in.port_name(p).ok())
            .collect::<Vec<_>>();

        if let Some(p) = in_ports.iter().find(|p| {
            midi_in
                .port_name(p)
                .map(|name| name.contains(port_name_match))
                .unwrap_or(false)
        }) {
            let p = p.clone(); // clone to own it
            let name = midi_in.port_name(&p)?;
            println!("MIDI device found: {name}");

            let tx = tx.clone();
            match midi_in.connect(
                &p,
                "midir-read",
                move |_, msg, _| {
                    let _ = tx.send((Instant::now(), msg.to_vec()));
                },
                (),
            ) {
                Ok(conn) => return Ok((rx, conn)),
                Err(err) => {
                    eprintln!("MIDI device disappeared before connect ({err}); retrying...");
                }
            }
        } else if port_names.is_empty() {
            println!("No MIDI input devices found.");
        } else {
            println!("Available MIDI input devices:");
            for name in &port_names {
                println!("  - {name}");
            }
        }

        println!("Waiting for MIDI device matching \"{port_name_match}\"...");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

/// Write MIDI events to a timestamped .mid file
pub fn events_to_smf(events: &[MidiEvent]) -> Result<Smf<'static>, Box<dyn std::error::Error>> {
    if events.is_empty() {
        return Err("No events to write".into());
    }

    let ppqn = 480u16;
    let bpm = 120.0;
    let microsec_per_quarter = 60_000_000f64 / bpm;

    let sec_to_ticks =
        |s: f64| -> u32 { ((s * 1_000_000.0) / microsec_per_quarter * ppqn as f64).round() as u32 };

    let mut track_events = Vec::<(u32, usize, TrackEventKind)>::new();
    let mut order = 0usize;

    // Tempo (must be first)
    track_events.push((
        0,
        order,
        TrackEventKind::Meta(MetaMessage::Tempo(u24::new(microsec_per_quarter as u32))),
    ));
    order += 1;

    for (t, bytes) in events {
        let tick = sec_to_ticks(*t);
        let Some((&status_byte, data)) = bytes.split_first() else {
            continue;
        };
        if data.len() < 2 {
            continue;
        }

        let key = data[0];
        let val = data[1];
        let status = status_byte & 0xF0;
        let channel = status_byte & 0x0F;

        let kind = match status {
            0x90 if val > 0 => TrackEventKind::Midi {
                channel: channel.into(),
                message: MidiMessage::NoteOn {
                    key: key.into(),
                    vel: val.into(),
                },
            },

            0x80 | 0x90 => TrackEventKind::Midi {
                channel: channel.into(),
                message: MidiMessage::NoteOff {
                    key: key.into(),
                    vel: val.into(),
                },
            },

            0xB0 => TrackEventKind::Midi {
                channel: channel.into(),
                message: MidiMessage::Controller {
                    controller: key.into(),
                    value: val.into(),
                },
            },

            _ => continue,
        };

        track_events.push((tick, order, kind));
        order += 1;
    }

    // CRITICAL: stable ordering
    track_events.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    let mut last_tick = 0u32;
    let mut midly_events = Vec::new();

    for (abs_tick, _, kind) in track_events {
        let delta = abs_tick.saturating_sub(last_tick);
        midly_events.push(TrackEvent {
            delta: u28::new(delta),
            kind,
        });
        last_tick = abs_tick;
    }

    midly_events.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    Ok(Smf {
        header: Header {
            format: Format::SingleTrack,
            timing: Timing::Metrical(ppqn.into()),
        },
        tracks: vec![midly_events],
    })
}
