use crate::models::user::User;
use crate::state::AppState;
use axum::{Json, extract::State};
use bcrypt::{DEFAULT_COST, hash};
use serde::Deserialize;

pub mod login;

pub async fn test() -> &'static str {
    "Test"
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    email: String,
    password: String,
}
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<User>, (axum::http::StatusCode, String)> {
    // Hash password
    let hashed = hash(&payload.password, DEFAULT_COST)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Insert into database
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, password_hash)
        VALUES ($1, $2)
        RETURNING id, email, password_hash, created_at, updated_at
        "#,
    )
    .bind(&payload.email)
    .bind(&hashed)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Return user JSON (password_hash skipped automatically)
    Ok(Json(user))
}
