// use crate::routes::auth::auth_routes;
// use axum::Router;

// pub mod auth;
// // pub mod user;
// // pub mod sessions;

// pub fn create_routes() -> Router {
//     Router::new().nest("/auth", auth_routes())
//     // .nest("/user", user_routes())
//     // .nest("/sessions", session_routes())
// }

use crate::state::AppState;
use axum::Router;

pub mod auth;

pub fn create_routes() -> Router<AppState> {
    let router: Router<AppState> = Router::new().nest("/auth", auth::auth_routes());

    router
}
