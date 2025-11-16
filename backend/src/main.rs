use crate::{
    config::Config,
    db::create_pool,
    routes::create_routes,
    services::{Services, sessions_service::SessionsService, user_service::UserService},
    state::AppState,
    utils::jwt::JwtManager,
};

mod config;
mod db;
mod errors;
mod middleware;
mod models;
mod routes;
mod services;
mod state;
mod utils;

#[tokio::main]
async fn main() {
    let config = Config::from_env();

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

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();

    println!("Running on http://localhost:{}", config.port);

    axum::serve(listener, app).await.unwrap();
}
