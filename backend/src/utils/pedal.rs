use midly::{MidiMessage, Smf, Timing, TrackEventKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PedalKind {
    Start,
    Stop,
}

#[derive(Clone, Copy, Debug)]
struct PedalEvent {
    tick: u64,
    kind: PedalKind,
    value: u8,
}

#[derive(Debug)]
struct Measure {
    insertion_at: usize,
    divisions: u32,
    duration: u32,
    staff: u8,
}

pub fn add_pedal_markings(midi: &[u8], musicxml: &str) -> Result<String, String> {
    let (ticks_per_quarter, events) = pedal_events(midi)?;
    if events.is_empty() {
        return Ok(musicxml.to_string());
    }

    let measures = measures(musicxml)?;
    if measures.is_empty() {
        return Err("MuseScore MusicXML contains no measures".to_string());
    }

    let mut insertions = vec![Vec::<(u32, PedalEvent)>::new(); measures.len()];
    for event in events {
        let quarter = event.tick as f64 / ticks_per_quarter as f64;
        let mut elapsed_quarters = 0.0;
        let mut target = measures.len() - 1;
        let mut offset = measures[target].duration;

        for (index, measure) in measures.iter().enumerate() {
            let length = measure.duration as f64 / measure.divisions as f64;
            if quarter < elapsed_quarters + length || index == measures.len() - 1 {
                target = index;
                offset = ((quarter - elapsed_quarters).max(0.0) * measure.divisions as f64)
                    .round()
                    .clamp(0.0, measure.duration as f64) as u32;
                break;
            }
            elapsed_quarters += length;
        }

        insertions[target].push((offset, event));
    }

    let mut result = musicxml.to_string();
    for (measure, events) in measures.iter().zip(insertions).rev() {
        if events.is_empty() {
            continue;
        }
        let directions = events
            .into_iter()
            .map(|(offset, event)| pedal_direction(offset, measure.staff, event))
            .collect::<String>();
        result.insert_str(measure.insertion_at, &directions);
    }

    Ok(result)
}

fn pedal_events(midi: &[u8]) -> Result<(u16, Vec<PedalEvent>), String> {
    let smf = Smf::parse(midi).map_err(|error| format!("Invalid MIDI: {error}"))?;
    let Timing::Metrical(timing) = smf.header.timing else {
        return Err("SMPTE-timed MIDI is not supported for pedal notation".to_string());
    };

    let mut events = Vec::new();
    for track in &smf.tracks {
        let mut tick = 0u64;
        for event in track {
            tick += event.delta.as_int() as u64;
            if let TrackEventKind::Midi {
                message:
                    MidiMessage::Controller {
                        controller, value, ..
                    },
                ..
            } = event.kind
                && controller.as_int() == 64
            {
                events.push((tick, value.as_int()));
            }
        }
    }
    events.sort_by_key(|(tick, _)| *tick);

    let mut pedal_down = false;
    let mut transitions = Vec::new();
    for (tick, value) in events {
        let down = value >= 64;
        if down == pedal_down {
            continue;
        }
        pedal_down = down;
        transitions.push(PedalEvent {
            tick,
            kind: if down {
                PedalKind::Start
            } else {
                PedalKind::Stop
            },
            value,
        });
    }

    Ok((timing.as_int(), transitions))
}

fn measures(xml: &str) -> Result<Vec<Measure>, String> {
    let mut result = Vec::new();
    let mut search_from = 0usize;
    let mut divisions = 1u32;
    let mut beats = 4u32;
    let mut beat_type = 4u32;
    let mut staff = 1u8;

    while let Some(relative_start) = xml[search_from..].find("<measure ") {
        let measure_start = search_from + relative_start;
        let content_start = xml[measure_start..]
            .find('>')
            .map(|offset| measure_start + offset + 1)
            .ok_or_else(|| "Malformed MusicXML measure".to_string())?;
        let content_end = xml[content_start..]
            .find("</measure>")
            .map(|offset| content_start + offset)
            .ok_or_else(|| "Unclosed MusicXML measure".to_string())?;
        let content = &xml[content_start..content_end];
        let insertion_at = ["</print>", "</attributes>"]
            .into_iter()
            .filter_map(|tag| {
                content
                    .find(tag)
                    .map(|offset| content_start + offset + tag.len())
            })
            .max()
            .unwrap_or(content_start);

        divisions = tag_value(content, "divisions").unwrap_or(divisions).max(1);
        beats = tag_value(content, "beats").unwrap_or(beats).max(1);
        beat_type = tag_value(content, "beat-type").unwrap_or(beat_type).max(1);
        staff = tag_value::<u8>(content, "staves")
            .map(|count| if count >= 2 { 2 } else { 1 })
            .unwrap_or(staff);
        let nominal_duration = beats * 4 * divisions / beat_type;
        let duration = performed_measure_duration(content)
            .unwrap_or(nominal_duration)
            .max(1);

        result.push(Measure {
            insertion_at,
            divisions,
            duration,
            staff,
        });
        search_from = content_end + "</measure>".len();
    }

    Ok(result)
}

fn performed_measure_duration(content: &str) -> Option<u32> {
    let mut cursor = 0i64;
    let mut furthest = 0i64;
    let mut search_from = 0usize;

    while search_from < content.len() {
        let candidates = ["<note", "<backup", "<forward"]
            .into_iter()
            .filter_map(|tag| content[search_from..].find(tag).map(|offset| (offset, tag)))
            .min_by_key(|(offset, _)| *offset);
        let Some((offset, tag)) = candidates else {
            break;
        };
        let start = search_from + offset;
        let close = match tag {
            "<note" => "</note>",
            "<backup" => "</backup>",
            _ => "</forward>",
        };
        let Some(end_offset) = content[start..].find(close) else {
            break;
        };
        let end = start + end_offset + close.len();
        let block = &content[start..end];
        let duration = tag_value::<i64>(block, "duration").unwrap_or(0);

        match tag {
            "<note" if !block.contains("<chord") && !block.contains("<grace") => {
                cursor += duration;
                furthest = furthest.max(cursor);
            }
            "<backup" => cursor = (cursor - duration).max(0),
            "<forward" => {
                cursor += duration;
                furthest = furthest.max(cursor);
            }
            _ => {}
        }
        search_from = end;
    }

    (furthest > 0).then_some(furthest as u32)
}

fn tag_value<T: std::str::FromStr>(xml: &str, tag: &str) -> Option<T> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");
    let start = xml.find(&start_tag)? + start_tag.len();
    let end = xml[start..].find(&end_tag)? + start;
    xml[start..end].trim().parse().ok()
}

fn pedal_direction(offset: u32, staff: u8, event: PedalEvent) -> String {
    let (kind, damper) = match event.kind {
        PedalKind::Start => ("start", pedal_percentage(event.value)),
        PedalKind::Stop => ("stop", "no".to_string()),
    };
    format!(
        "\n      <direction placement=\"below\">\n        <direction-type>\n          <pedal type=\"{kind}\" line=\"yes\"/>\n          </direction-type>\n        <offset sound=\"yes\">{offset}</offset>\n        <staff>{staff}</staff>\n        <sound damper-pedal=\"{damper}\"/>\n        </direction>"
    )
}

fn pedal_percentage(value: u8) -> String {
    ((value as u16 * 100) / 127).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures_include_pickups_and_multiple_voices() {
        let xml = r#"<part><measure number="1"><attributes><divisions>4</divisions><time><beats>4</beats><beat-type>4</beat-type></time><staves>2</staves></attributes><note><rest/><duration>4</duration></note><backup><duration>4</duration></backup><note><rest/><duration>4</duration><staff>2</staff></note></measure></part>"#;
        let parsed = measures(xml).unwrap();
        assert_eq!(parsed[0].duration, 4);
        assert_eq!(parsed[0].staff, 2);
    }

    #[test]
    fn injects_standard_musicxml_pedal_directions() {
        let midi = [
            b'M', b'T', b'h', b'd', 0, 0, 0, 6, 0, 0, 0, 1, 1, 0xe0, b'M', b'T', b'r', b'k', 0, 0,
            0, 13, 0, 0xb0, 64, 127, 0x83, 0x60, 0xb0, 64, 0, 0, 0xff, 0x2f, 0,
        ];
        let xml = r#"<score-partwise><part><measure number="1"><attributes><divisions>4</divisions></attributes><note><rest/><duration>16</duration></note></measure></part></score-partwise>"#;
        let result = add_pedal_markings(&midi, xml).unwrap();
        assert!(result.contains("<pedal type=\"start\" line=\"yes\"/>"));
        assert!(result.contains("<pedal type=\"stop\" line=\"yes\"/>"));
        assert!(result.contains("<offset sound=\"yes\">4</offset>"));
    }
}
