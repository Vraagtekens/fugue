use crate::{db::create_pool, routes::create_routes, state::AppState};
use dotenvy::dotenv;

mod config;
mod db;
mod errors;
mod models;
mod routes;
mod state;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let pool = create_pool().await.expect("Failed to init DB");
    let state = AppState { db: pool };

    // let routes = create_routes();
    // let app = routes.layer(Extension(state));
    let app = create_routes().with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
