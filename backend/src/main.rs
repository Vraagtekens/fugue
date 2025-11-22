use crate::{config::Config, utils::router::build_router};

mod config;
mod db;
mod entities;
mod errors;
mod extractors;
mod middleware;
mod routes;
mod services;
mod state;
mod utils;

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    let router = build_router(config.clone()).await;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();

    println!("Running on http://localhost:{}", config.port);

    axum::serve(listener, router).await.unwrap();
}
