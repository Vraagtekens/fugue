use crate::entities::sessions;
use crate::errors::ApiError;
use crate::state::AppState;
use crate::utils::jwt::Claims;
use axum::Extension;
use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddSessionRequest {
    pub category_id: Option<i32>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub kind: String, // "pomodoro", "break", etc.
    pub completed: bool,
}

pub async fn add(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<AddSessionRequest>,
) -> Result<Json<sessions::Model>, ApiError> {
    let user_id = claims.sub;
    let session = state
        .services
        .sessions
        .add_session(user_id, &payload)
        .await?;

    Ok(Json(session))
}

pub async fn get_pomodoro_sessions(
    State(state): State<AppState>,
) -> Result<Json<Vec<sessions::Model>>, ApiError> {
    let sessions = state.services.sessions.get_all_sessions().await?;
    Ok(Json(sessions))
}

// #[debug_handler]
// pub async fn test(State(state): State<AppState>) -> Result<Json<Vec<session::Model>>, ApiError> {
//     // let sessions = state.services.sessions.get_all_sessions().await?;

//     // Ok("dd".to_string())
//     Ok(Json(vec![]))
// }

// pub async fn test(State(state): State<AppState>) -> Result<Json<Vec<session::Model>>, ApiError> {
//     let sessions = state.services.sessions.get_all_sessions().await?;
//     Ok(Json(sessions))
// }
