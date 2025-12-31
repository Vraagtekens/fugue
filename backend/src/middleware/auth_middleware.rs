use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;

use crate::{errors::ApiError, state::AppState};

pub async fn require_jwt(
    jar: CookieJar,
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Get the "session" cookie
    let cookie = jar.get("session").ok_or(StatusCode::UNAUTHORIZED)?;
    let token = cookie.value();

    // Verify JWT
    let claims = state.jwt.verify(token).ok_or(StatusCode::UNAUTHORIZED)?;

    // Attach claims to request extensions for handlers to read
    req.extensions_mut().insert(claims);

    // Continue to the next middleware / handler
    Ok(next.run(req).await)
}

pub async fn require_api_key(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let api_key = req
        .headers()
        .get("x-api-key")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if api_key != state.config.api_key {
        return Err(ApiError::new(StatusCode::UNAUTHORIZED, ""));
    }

    // Continue to the next middleware / handler
    Ok(next.run(req).await)
}
