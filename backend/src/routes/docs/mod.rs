use utoipa::OpenApi;

use crate::routes::auth::handlers::{RegisterRequest, login::UserResponse};

/// Top-level OpenAPI definition
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::auth::handlers::login::login,
        crate::routes::auth::handlers::register,
        // crate::routes::sessions::handlers::get_sessions,
    ),
    components(
        schemas(
            crate::routes::auth::handlers::login::LoginRequest,
            UserResponse,
            RegisterRequest,
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "sessions", description = "Session management endpoints")
    )
)]
pub struct ApiDoc;
