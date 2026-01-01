use crate::errors::ApiError;
use crate::extractors::TypedJson;
use crate::state::AppState;
use crate::utils::jwt::Claims;
use crate::{entities::sessions, middleware::logging_middleware};
use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use sea_orm::prelude::{DateTimeWithTimeZone, Uuid};
use serde::Deserialize;

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

    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some("metadata") => {
                let json = field.text().await?;
                payload = Some(serde_json::from_str(&json)?);
            }
            Some("midi_file") => {
                midi_bytes = Some(field.bytes().await?.to_vec());
            }
            _ => {}
        }
    }

    let payload =
        payload.ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "missing metadata"))?;
    let file =
        midi_bytes.ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "missing midi file"))?;

    // let user_id = claims.sub;
    let session = state.services.sessions.add_session(&payload).await?;
    let x = state.s3.add_file(&session.title, file, None).await?;

    Ok(Json(session))
}

pub async fn get_sessions(
    State(state): State<AppState>,
) -> Result<Json<Vec<sessions::Model>>, ApiError> {
    let sessions = state.services.sessions.get_all_sessions().await?;
    Ok(Json(sessions))
}
