use crate::models::user::User;
use crate::state::AppState;

use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::cookie::{Cookie, CookieJar};

use bcrypt::verify;
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,   // user id
    pub exp: usize, // expiration timestamp
}

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<User>), (StatusCode, String)> {
    // Fetch user by email
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users WHERE email = $1
        "#,
    )
    .bind(&payload.email)
    .fetch_one(&state.db)
    .await
    .map_err(|_| (StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

    // Verify password
    let valid = verify(&payload.password, &user.password_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to verify".into()))?;

    if !valid {
        return Err((StatusCode::UNAUTHORIZED, "Wrong password".to_string()));
    }

    // Create JWT
    let exp = (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;

    let claims = Claims { sub: user.id, exp };

    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "JWT encode failed".into(),
        )
    })?;

    // Create secure cookie
    let cookie = Cookie::build(("session", token))
        .http_only(true)
        .secure(true) // set false if testing on http
        .expires(OffsetDateTime::now_utc() + Duration::days(1))
        .path("/");

    let jar = jar.add(cookie);

    // Return (cookie jar, user JSON)
    Ok((jar, Json(user)))
}
