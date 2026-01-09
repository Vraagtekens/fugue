use crate::{entities::users, extractors::TypedJson, state::AppState};

use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::cookie::CookieJar;

use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String, // <-- Uuid serialized as string
    pub email: String,
    // pub created_at: Option<String>, // optional, see below
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = UserResponse),
        (status = 401, description = "Invalid credentials")
    )
)]
#[axum::debug_handler]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    TypedJson(payload): TypedJson<LoginRequest>,
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    let user_model = state.services.user.authenticate(&payload).await?;

    let token = state.jwt.generate(user_model.id);
    let cookie = state.jwt.build_cookie(token);
    let jar = jar.add(cookie);

    Ok(Json(UserResponse {
        id: user_model.id.to_string(),
        email: user_model.email,
        // created_at: user_model.created_at,
    }))
}
