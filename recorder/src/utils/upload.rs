use reqwest::multipart;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn upload_session(
    api_url: &str,
    api_key: &str,
    user_id: &str,
    title: Option<&str>,
    smf: &midly::Smf<'_>,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Serialize MIDI into memory
    let mut midi_buf = Vec::new();
    smf.write_std(&mut midi_buf)?;

    // Timestamp (previously used for filename)
    let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let title = title
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("fp30x-{ts}"));

    // let file_name = format!("fp30x-{}", ts);

    let metadata = json!({
        "title": title,
        "user_id": user_id,
        "start_time": start_time.to_rfc3339(),
        "end_time": end_time.to_rfc3339()
    });

    // Multipart form
    let form = multipart::Form::new()
        .text("metadata", metadata.to_string())
        .part(
            "midi_file",
            multipart::Part::bytes(midi_buf)
                .file_name(format!("{title}.mid"))
                .mime_str("audio/midi")?,
        );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let resp = client
        .post(api_url)
        .header("x-api-key", api_key)
        .multipart(form)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp
            .text()
            .await
            .unwrap_or_else(|err| format!("could not read response body: {err}"));
        return Err(format!("backend rejected session upload ({status}): {body}").into());
    }

    println!("Upload OK: {status}");
    Ok(())
}
