use crate::errors::ApiError;
use crate::extractors::TypedJson;
use crate::{entities::users, state::AppState};
use axum::http::StatusCode;
use axum::{Json, extract::State};
use bcrypt::{DEFAULT_COST, hash};
use sea_orm::{ActiveModelTrait, Set};
use serde::Deserialize;

pub mod login;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

pub async fn register(
    State(state): State<AppState>,
    TypedJson(payload): TypedJson<RegisterRequest>,
) -> Result<Json<users::Model>, ApiError> {
    // Hash password
    let hashed = hash(&payload.password, DEFAULT_COST)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Create ActiveModel
    let new_user = users::ActiveModel {
        email: Set(payload.email.clone()),
        password_hash: Set(hashed),
        ..Default::default()
    };

    // Insert into DB
    let user = new_user
        .insert(&state.db)
        .await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(user))
}
