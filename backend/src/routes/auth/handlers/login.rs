use crate::models::user::User;
use crate::state::AppState;

use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::cookie::CookieJar;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<User>), (StatusCode, String)> {
    let user = state.services.user.authenticate(&payload).await?;

    let token = state.jwt.generate(user.id);
    let cookie = state.jwt.build_cookie(token);
    let jar = jar.add(cookie);

    Ok((jar, Json(user)))
}
