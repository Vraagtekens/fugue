use crate::entities::sessions;
use crate::errors::ApiError;
use crate::state::AppState;

use aws_sdk_s3::primitives::ByteStream;
use axum::response::Response;
use axum::{
    Json,
    extract::{Multipart, Path as AxumPath, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use sea_orm::prelude::Uuid;
use serde::Deserialize;
use std::path::Path;
use tracing::info;

#[derive(Deserialize)]
pub struct AddSessionRequest {
    pub title: String,
    pub user_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
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

    validate_session_upload(&payload, &file)?;

    // Default to ".mid" if no extension is found
    let ext = normalize_midi_extension(extension.as_deref())?;

    // Append extension to session title for S3 key
    let key = format!("{}.{}", sanitize_s3_key_part(&payload.title), ext);

    let session = state.services.sessions.add_session(&payload).await?;
    let s3_url = state.s3.add_file(&key, file, Some("audio/midi")).await?;
    info!(session_id = session.id, s3_key = %key, s3_url = %s3_url, "session uploaded");

    Ok(Json(session))
}

fn validate_session_upload(payload: &AddSessionRequest, file: &[u8]) -> Result<(), ApiError> {
    if payload.title.trim().is_empty() {
        return Err(ApiError::new(StatusCode::BAD_REQUEST, "title is required"));
    }

    if let Some(end_time) = payload.end_time
        && end_time < payload.start_time
    {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "end_time cannot be earlier than start_time",
        ));
    }

    if file.is_empty() {
        return Err(ApiError::new(StatusCode::BAD_REQUEST, "midi file is empty"));
    }

    if !file.starts_with(b"MThd") {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "uploaded file is not a valid MIDI file",
        ));
    }

    Ok(())
}

fn normalize_midi_extension(ext: Option<&str>) -> Result<&'static str, ApiError> {
    match ext.unwrap_or("mid").to_ascii_lowercase().as_str() {
        "mid" => Ok("mid"),
        "midi" => Ok("midi"),
        _ => Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "midi_file must have a .mid or .midi extension",
        )),
    }
}

fn sanitize_s3_key_part(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if sanitized.is_empty() {
        "session".to_string()
    } else {
        sanitized
    }
}

pub async fn get_sessions(
    State(state): State<AppState>,
) -> Result<Json<Vec<sessions::Model>>, ApiError> {
    let sessions = state.services.sessions.get_all_sessions().await?;
    Ok(Json(sessions))
}

pub async fn delete_session(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<i32>,
) -> Result<StatusCode, ApiError> {
    let session = state
        .services
        .sessions
        .get_session(id)
        .await?
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "session not found"))?;
    let key = format!("{}.mid", sanitize_s3_key_part(&session.title));

    state.s3.delete_file(&key).await?;
    state.services.sessions.delete_session(id).await?;

    info!(session_id = id, s3_key = %key, "session deleted");
    Ok(StatusCode::NO_CONTENT)
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
    render_session_file(state, key, RenderedSessionFile::Pdf).await
}

pub async fn get_session_midi_mp3(
    axum::extract::Path(key): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    render_session_file(state, key, RenderedSessionFile::Mp3).await
}

pub async fn get_session_audio(
    axum::extract::Path(key): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    render_session_file(state, key, RenderedSessionFile::Audio).await
}

enum RenderedSessionFile {
    Pdf,
    Mp3,
    Audio,
}

impl RenderedSessionFile {
    fn extension(&self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Mp3 => "mp3",
            Self::Audio => "flac",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Pdf => "PDF",
            Self::Mp3 => "MP3",
            Self::Audio => "FLAC audio",
        }
    }

    fn content_type(&self) -> &'static str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Mp3 => "audio/mpeg",
            Self::Audio => "audio/flac",
        }
    }

    fn disposition(&self) -> &'static str {
        match self {
            Self::Audio => "inline",
            Self::Pdf | Self::Mp3 => "attachment",
        }
    }
}

async fn render_session_file(
    state: AppState,
    key: String,
    file_kind: RenderedSessionFile,
) -> Result<Response, ApiError> {
    let midi_bytes = read_s3_bytes(state.s3.get_file(&key).await?).await?;
    let renderer = state.mscore;
    let (midi_path, output_path) = renderer.make_temp_paths(file_kind.extension());

    renderer.write_midi_file(&midi_path, &midi_bytes).await?;

    let output = match file_kind {
        RenderedSessionFile::Audio => renderer
            .generate_audio(&midi_path, &output_path)
            .await
            .map_err(|e| render_spawn_error("FluidSynth", e))?,
        RenderedSessionFile::Pdf | RenderedSessionFile::Mp3 => renderer
            .generate(&midi_path, &output_path)
            .await
            .map_err(|e| render_spawn_error("MuseScore", e))?,
    };

    let file_bytes = renderer
        .read_generated_file(&output_path, &output.stderr, file_kind.label())
        .await?;

    renderer.cleanup_files(&[midi_path, output_path]).await;

    let mut response = Response::new(file_bytes.into());
    let headers = response.headers_mut();
    headers.insert("Content-Type", file_kind.content_type().parse().unwrap());
    headers.insert(
        "Content-Disposition",
        format!(
            "{}; filename=\"{}.{}\"",
            file_kind.disposition(),
            key,
            file_kind.extension()
        )
        .parse()
        .unwrap(),
    );

    Ok(response)
}

async fn read_s3_bytes(stream: ByteStream) -> Result<Vec<u8>, ApiError> {
    Ok(stream
        .collect()
        .await
        .map_err(|e| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read S3 stream: {}", e),
            )
        })?
        .into_bytes()
        .to_vec())
}

fn render_spawn_error(renderer: &str, error: std::io::Error) -> ApiError {
    ApiError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("{renderer} spawn failed: {error}"),
    )
}

pub mod live;
