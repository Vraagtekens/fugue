use crate::{
    config::Config,
    db::create_pool,
    routes::{auth::handlers::register, create_routes},
    services::{Services, sessions_service::SessionsService, user_service::UserService},
    state::AppState,
    utils::jwt::JwtManager,
};

use axum::{
    BoxError, Router,
    error_handling::HandleErrorLayer,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use std::time::Duration;
use tower::ServiceBuilder;

use crate::errors::ApiError;

mod config;
mod db;
mod entities;
mod errors;
mod extractors;
mod hell;
mod middleware;
mod models;
mod routes;
mod services;
mod state;
mod utils;

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter("fugue_backend=info") // reads RUST_LOG
        .with_file(false)
        .with_target(false) // optional: hide module path
        .with_thread_ids(false) // optional: show thread ids
        .init();
}

#[tokio::main]
async fn main() {
    // GLOBAL ERROR HANDLER (the glue!)
    async fn global_error_handler(err: BoxError) -> impl IntoResponse {
        println!("test");
        println!("test");
        println!("test");
        // Timeout? → ApiError
        if err.is::<tower::timeout::error::Elapsed>() {
            return ApiError::new(StatusCode::REQUEST_TIMEOUT, "Request took too long")
                .into_response();
        }

        // Panic? → ApiError
        if err.is::<std::convert::Infallible>() {
            return ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal panic")
                .into_response();
        }

        // Any other tower error → ApiError
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {err}"),
        )
        .into_response()
    }

    let config = Config::from_env();
    init_tracing();

    let db = create_pool(&config.database_url)
        .await
        .expect("Failed to init DB");

    let jwt = JwtManager::new(config.jwt_secret.clone(), config.jwt_expiration_hours);

    let services = Services {
        user: UserService { db: db.clone() },
        sessions: SessionsService { db: db.clone() },
    };

    let state = AppState {
        db,
        jwt,
        config: config.clone(),
        services,
    };

    let app = create_routes(state.clone())
        .layer(
            ServiceBuilder::new()
                // `timeout` will produce an error if the handler takes
                // too long so we must handle those
                .layer(HandleErrorLayer::new(global_error_handler))
                .timeout(Duration::from_secs(30)),
        )
        .fallback(handler_404)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", 3000))
        .await
        .unwrap();

    println!("Running on http://localhost:{}", 3000);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn handler_404() -> ApiError {
    ApiError::new(StatusCode::NOT_FOUND, "Route not found")
}

// use crate::{
//     config::Config,
//     db::create_pool,
//     routes::create_routes,
//     services::{Services, sessions_service::SessionsService, user_service::UserService},
//     state::AppState,
//     utils::jwt::JwtManager,
// };

// use axum::{BoxError, error_handling::HandleErrorLayer, http::StatusCode, response::IntoResponse};

// use crate::errors::ApiError;

// mod config;
// mod db;
// mod entities;
// mod errors;
// mod hell;
// mod middleware;
// mod models;
// mod routes;
// mod services;
// mod state;
// mod utils;

// fn init_tracing() {
//     tracing_subscriber::fmt()
//         .with_env_filter("fugue_backend=info") // reads RUST_LOG
//         .with_file(false)
//         .with_target(false) // optional: hide module path
//         .with_thread_ids(false) // optional: show thread ids
//         .init();
// }

// #[tokio::main]
// async fn main() {
//     let config = Config::from_env();
//     init_tracing();

//     let db = create_pool(&config.database_url)
//         .await
//         .expect("Failed to init DB");

//     let jwt = JwtManager::new(config.jwt_secret.clone(), config.jwt_expiration_hours);

//     let services = Services {
//         user: UserService { db: db.clone() },
//         sessions: SessionsService { db: db.clone() },
//     };

//     let state = AppState {
//         db,
//         jwt,
//         config: config.clone(),
//         services,
//     };

//     let app = create_routes(state.clone())
//         .layer(HandleErrorLayer::new(global_error_handler))
//         .with_state(state);

//     let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
//         .await
//         .unwrap();

//     println!("Running on http://localhost:{}", config.port);

//     // axum::serve(listener, app).await.unwrap();
//     axum::serve(listener, app).await.unwrap();
// }

// async fn global_error_handler(err: BoxError) -> impl IntoResponse {
//     println!("test");
//     // Timeout? → ApiError
//     if err.is::<tower::timeout::error::Elapsed>() {
//         return ApiError::new(StatusCode::REQUEST_TIMEOUT, "Request took too long").into_response();
//     }

//     // Panic? → ApiError
//     if err.is::<std::convert::Infallible>() {
//         return ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal panic").into_response();
//     }

//     // Any other tower error → ApiError
//     ApiError::new(
//         StatusCode::INTERNAL_SERVER_ERROR,
//         format!("Unhandled internal error: {err}"),
//     )
//     .into_response()
// }
