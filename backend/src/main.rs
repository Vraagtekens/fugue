use axum::error_handling::HandleErrorLayer;
use std::time::Duration;
use tower::ServiceBuilder;

use crate::{
    config::Config,
    db::create_pool,
    errors::{global_error_handler, handle_timeout_error},
    routes::create_routes,
    services::{Services, sessions_service::SessionsService, user_service::UserService},
    state::AppState,
    utils::jwt::JwtManager,
};

mod config;
mod db;
mod entities;
mod errors;
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

    let app = create_routes(state.clone()).with_state(state);

    // let svc = ServiceBuilder::new()
    //     .layer(HandleErrorLayer::new(handle_timeout_error))
    //     .service(app)
    //     .await;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();

    println!("Running on http://localhost:{}", config.port);

    axum::serve(listener, app).await.unwrap();
}
