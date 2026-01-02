use chrono::{Duration, Local, Utc};
use reqwest::multipart;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn upload_session(
    api_url: &str,
    smf: &midly::Smf<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Serialize MIDI into memory
    let mut midi_buf = Vec::new();
    smf.write_std(&mut midi_buf)?;

    // Timestamp (previously used for filename)
    let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    // Build metadata from runtime info
    let start_time = Utc::now();
    let end_time = start_time + Duration::minutes(25);

    let metadata = json!({
        "title": format!("piano-{}", ts),
        "user_id": "9e20b80e-5143-4f77-ad9d-3637ca4c3eba",
        "start_time": start_time.to_rfc3339(),
        "end_time": end_time.to_rfc3339()
    });

    // Multipart form
    let form = multipart::Form::new()
        .text("metadata", metadata.to_string())
        .part(
            "midi_file",
            multipart::Part::bytes(midi_buf)
                .file_name(format!("piano-{}.mid", ts))
                .mime_str("audio/midi")?,
        );

    let client = reqwest::Client::new();

    let resp = client
        .post(api_url)
        .multipart(form)
        .send()
        .await?
        .error_for_status()?;

    println!("Upload OK: {}", resp.status());
    Ok(())
}
