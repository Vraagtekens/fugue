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
use std::path::PathBuf;
use tempfile::{Builder, NamedTempFile};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
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
    // 1️⃣ Fetch MIDI from S3
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

    // 2️⃣ Create stable, unique temp paths
    let tmp_dir = std::env::temp_dir();
    let id = "51b07456-f561-409f-9c8d-1d012758931d";

    let midi_path: PathBuf = tmp_dir.join(format!("{}.mid", id));
    let pdf_path: PathBuf = tmp_dir.join(format!("{}.pdf", id));

    // 3️⃣ Write MIDI to disk
    tokio::fs::write(&midi_path, &midi_bytes)
        .await
        .map_err(|e| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Write MIDI failed: {}", e),
            )
        })?;

    // 4️⃣ Run MuseScore
    let output = Command::new("mscore")
        .env("QT_LOGGING_RULES", "qt.qml.typeregistration=false")
        .arg(&midi_path)
        .arg("-o")
        .arg(&pdf_path)
        .output()
        .map_err(|e| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("MuseScore spawn failed: {}", e),
            )
        })?;

    // 5️⃣ Validate PDF output (ignore exit code!)
    let meta = tokio::fs::metadata(&pdf_path).await.map_err(|_| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "MuseScore failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        )
    })?;

    if meta.len() == 0 {
        return Err(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "MuseScore produced empty PDF",
        ));
    }

    // 6️⃣ Read PDF
    let pdf_bytes = tokio::fs::read(&pdf_path).await?;

    // 7️⃣ Best-effort cleanup
    let _ = tokio::fs::remove_file(&midi_path).await;
    let _ = tokio::fs::remove_file(&pdf_path).await;

    // 8️⃣ Return response
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
