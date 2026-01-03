use std::path::Path;
use std::process::Command;

use crate::errors::ApiError;
use crate::extractors::TypedJson;
use crate::state::AppState;
use crate::utils::jwt::Claims;
use crate::{entities::sessions, middleware::logging_middleware};
use aws_sdk_s3::primitives::ByteStream;
use axum::http::HeaderMap;
use axum::response::Response;
use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use sea_orm::prelude::{DateTimeWithTimeZone, Uuid};
use serde::Deserialize;
use std::io::Write;
use tempfile::{NamedTempFile, Builder};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tracing::{error, warn};
use std::path::PathBuf;

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
//     Path(key): Path<String>,
//     State(state): State<AppState>,
// ) -> impl IntoResponse {
//     match state.s3.get_file(&key).await {
//         Ok(stream) => {
//             let mut headers = HeaderMap::new();
//             headers.insert(
//                 axum::http::header::CONTENT_TYPE,
//                 "audio/midi".parse().unwrap(),
//             );

//             (headers, stream)
//         }
//         Err(err) => {
//             eprintln!("S3 error: {err}");
//             StatusCode::NOT_FOUND.into_response()
//         }
//     }
// }

pub async fn get_session_midi(
    axum::extract::Path(key): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> Result<ByteStream, ApiError> {
    let file = state.s3.get_file(&key).await?;

    println!("{:?}", file);

    Ok(file)
}

pub async fn get_session_midi_pdf(
    axum::extract::Path(key): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    // 1️⃣ Get the MIDI file from S3
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

    // 2️⃣ Write MIDI bytes to a temp file
    let mut midi_file = NamedTempFile::new().map_err(|e| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Temp file error: {}", e),
        )
    })?;

    midi_file.write_all(&midi_bytes).map_err(|e| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Write error: {}", e),
        )
    })?;
    let midi_path = midi_file.path();

    // 3️⃣ Prepare temp file for PDF output
    let pdf_file = Builder::new()
    .suffix(".pdf")
    .tempfile()
    .map_err(|e| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Temp PDF error: {}", e),
        )
    })?;

    let pdf_path = pdf_file.path();

    // 4️⃣ Call MuseScore CLI to generate PDF
let output = Command::new("mscore")
    .env("QT_LOGGING_RULES", "qt.qml.typeregistration=false")
    .arg("/home/dylan/Repositories/fugue/recorder/sessions/2025-12-23/piano-1766512496.mid")
    .arg("-o")
    .arg("/home/dylan/Repositories/fugue/recorder/pdf/bruh.pdf")
    .output()
    .map_err(|e| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("MuseScore spawn failed: {}", e),
        )
    })?;

    println!("{:?}", output);

    if !output.status.success() {
        return Err(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "MuseScore failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }

    println!("{:?}" ,pdf_file);


    // 5️⃣ Read PDF bytes
    let mut pdf_bytes = Vec::new();
    let mut f = File::open(pdf_path).await.map_err(|e| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Open PDF failed: {}", e),
        )
    })?;

    println!("{:?}", pdf_bytes);


    f.read_to_end(&mut pdf_bytes).await.map_err(|e| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Read PDF failed: {}", e),
        )
    })?;

    // 6️⃣ Return PDF as HTTP response
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
