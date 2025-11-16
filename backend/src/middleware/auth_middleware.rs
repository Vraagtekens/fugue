use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;

use crate::state::AppState;

pub async fn require_auth(
    jar: CookieJar,
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
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
