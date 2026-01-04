use crate::entities::sessions;
use crate::errors::ApiError;
use crate::state::AppState;

use aws_sdk_s3::primitives::ByteStream;
use axum::response::Response;
use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use sea_orm::prelude::{DateTimeWithTimeZone, Uuid};
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tracing::{error, warn};

#[derive(Deserialize)]
pub struct AddSessionRequest {
    pub title: String,
    pub user_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

pub async fn add(
    State(state): State<AppState>,
    // Extension(claims): Extension<Claims>,
    // TypedJson(payload): TypedJson<AddSessionRequest>,
    mut multipart: Multipart,
) -> Result<Json<sessions::Model>, ApiError> {
    let mut payload: Option<AddSessionRequest> = None;
    let mut midi_bytes: Option<Vec<u8>> = None;
    let mut extension: Option<String> = None;

    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some("metadata") => {
                let json = field.text().await?;
                payload = Some(serde_json::from_str(&json)?);
            }
            Some("midi_file") => {
                // Extract extension from uploaded file
                extension = field
                    .file_name()
                    .and_then(|f| Path::new(f).extension())
                    .map(|ext| ext.to_string_lossy().to_string());

                // Read bytes
                let bytes = field.bytes().await?.to_vec();
                midi_bytes = Some(bytes);
            }
            _ => {}
        }
    }

    let payload =
        payload.ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "missing metadata"))?;
    let file =
        midi_bytes.ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "missing midi file"))?;

    // Default to ".mid" if no extension is found
    let ext = extension.unwrap_or_else(|| "mid".to_string());

    // Append extension to session title for S3 key
    let key = format!("{}.{}", &payload.title, ext);

    let session = state.services.sessions.add_session(&payload).await?;
    let x = state.s3.add_file(&key, file, None).await?;
    println!("{:?}", x);

    Ok(Json(session))
}

pub async fn get_sessions(
    State(state): State<AppState>,
) -> Result<Json<Vec<sessions::Model>>, ApiError> {
    let sessions = state.services.sessions.get_all_sessions().await?;
    Ok(Json(sessions))
}

// pub async fn get_session_midi(
//     State(state): State<AppState>,
// ) -> Result<Json<Vec<sessions::Model>>, ApiError> {
//     let sessions = state.services.sessions.get_all_sessions().await?;
//     Ok(Json(sessions))
// }

pub async fn get_session_midi_pdf(
    axum::extract::Path(key): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    // Fetch MIDI from S3
    let stream: ByteStream = state.s3.get_file(&key).await?;
    let midi_bytes = stream
        .collect()
        .await
        .map_err(|e| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read S3 stream: {}", e),
            )
        })?
        .into_bytes();

    // MSCORE
    let mscore = state.mscore;
    let (midi_path, pdf_path) = mscore.make_temp_paths("pdf");

    mscore.write_midi_file(&midi_path, &midi_bytes).await?;

    let output = mscore.generate(&midi_path, &pdf_path).await.map_err(|e| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("MuseScore spawn failed: {}", e),
        )
    })?;

    let pdf_bytes = mscore
        .read_generated_file(&pdf_path, &output.stderr, "PDF")
        .await?;

    mscore
        .cleanup_files(&[midi_path.clone(), pdf_path.clone()])
        .await;

    // Return response
    let mut resp = Response::new(pdf_bytes.into());
    let headers = resp.headers_mut();
    headers.insert("Content-Type", "application/pdf".parse().unwrap());
    headers.insert(
        "Content-Disposition",
        format!("attachment; filename=\"{}.pdf\"", key)
            .parse()
            .unwrap(),
    );

    Ok(resp)
}

pub async fn get_session_midi_mp3(
    axum::extract::Path(key): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    // Fetch MIDI from S3
    let stream: ByteStream = state.s3.get_file(&key).await?;
    let midi_bytes = stream
        .collect()
        .await
        .map_err(|e| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read S3 stream: {}", e),
            )
        })?
        .into_bytes();

    // MSCORE
    let mscore = state.mscore;
    let (midi_path, mp3_path) = mscore.make_temp_paths("mp3");

    mscore.write_midi_file(&midi_path, &midi_bytes).await?;

    let output = mscore.generate(&midi_path, &mp3_path).await.map_err(|e| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("MuseScore spawn failed: {}", e),
        )
    })?;

    let mp3_bytes = mscore
        .read_generated_file(&mp3_path, &output.stderr, "MP3")
        .await?;

    mscore
        .cleanup_files(&[midi_path.clone(), mp3_path.clone()])
        .await;

    // Return response
    let mut resp = Response::new(mp3_bytes.into());
    let headers = resp.headers_mut();
    headers.insert("Content-Type", "audio/mpeg".parse().unwrap());
    headers.insert(
        "Content-Disposition",
        format!("attachment; filename=\"{}.mp3\"", key)
            .parse()
            .unwrap(),
    );

    Ok(resp)
}
