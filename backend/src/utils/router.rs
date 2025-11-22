use axum::Router;

use crate::{
    config::Config,
    db::create_pool,
    middleware::logging_middleware::logging_middleware,
    routes::create_routes,
    services::{Services, sessions_service::SessionsService, user_service::UserService},
    state::AppState,
    utils::jwt::JwtManager,
};

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter("fugue_backend=info") // reads RUST_LOG
        .with_file(false)
        .with_target(false) // optional: hide module path
        .with_thread_ids(false) // optional: show thread ids
        .init();
}

pub async fn build_router(config: Config) -> Router {
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

    create_routes(state.clone())
        // .layer(
        //     ServiceBuilder::new()
        //         // `timeout` will produce an error if the handler takes
        //         // too long so we must handle those
        //         .layer(HandleErrorLayer::new(global_error_handler))
        //         .timeout(Duration::from_secs(30)),
        // )
        .layer(axum::middleware::from_fn(logging_middleware))
        .with_state(state)
}
