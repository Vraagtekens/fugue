// use std::time::Duration;

// use crate::{
//     errors::global_error_handler,
//     middleware::{auth_middleware::require_auth, logging_middleware::logging_middleware},
//     state::AppState,
// };
// use axum::{Router, error_handling::HandleErrorLayer, middleware, routing::get};
// use tower::ServiceBuilder;

// pub mod auth;
// pub mod sessions;

// pub fn create_routes(app_state: AppState) -> Router<AppState> {
//     let auth_routes = auth::auth_routes();

//     let protected_sessions = sessions::sessions_routes().layer(middleware::from_fn_with_state(
//         app_state.clone(),
//         require_auth,
//     ));

//     Router::new()
//         .nest("/auth", auth_routes)
//         .nest("/sessions", protected_sessions)
//         .layer(middleware::from_fn_with_state(
//             app_state.clone(),
//             logging_middleware,
//         ))
// }

use crate::{
    middleware::{auth_middleware::require_auth, logging_middleware::logging_middleware},
    state::AppState,
};
use axum::{Router, middleware};

pub mod auth;
pub mod sessions;

pub fn create_routes(state: AppState) -> Router<AppState> {
    let auth_routes = auth::auth_routes();

    // let protected_sessions = sessions::sessions_routes();
    let protected_sessions = sessions::sessions_routes()
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    Router::new()
        .nest("/auth", auth_routes)
        .nest("/sessions", protected_sessions)
        .layer(middleware::from_fn(logging_middleware))
}
