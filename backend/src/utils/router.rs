use axum::Router;

use crate::{
    config::Config,
    db::create_pool,
    middleware::logging_middleware::logging_middleware,
    routes::create_routes,
    services::{
        Services, live_service::LiveSessionHub, sessions_service::SessionsService,
        user_service::UserService,
    },
    state::AppState,
    utils::{jwt::JwtManager, mscore::MscoreManager, s3::S3Manager},
};

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter("fugue_backend=info") // reads RUST_LOG
        .with_file(false)
        .with_target(false) // optional: hide module path
        .with_thread_ids(false) // optional: show thread ids
        .init();
}

pub async fn build_router(config: Config) -> Result<Router, sea_orm::DbErr> {
    init_tracing();

    let db = create_pool(&config.database_url).await?;

    let s3 = S3Manager::new(&config).await;

    let jwt = JwtManager::new(config.jwt_secret.clone(), config.jwt_expiration_hours);

    let services = Services {
        user: UserService { db: db.clone() },
        sessions: SessionsService { db: db.clone() },
    };

    let mscore = MscoreManager::new();
    let live = LiveSessionHub::new(1024);

    let state = AppState {
        db,
        s3,
        jwt,
        mscore,
        config: config.clone(),
        services,
        live,
    };

    Ok(create_routes(state.clone())
        // .layer(
        //     ServiceBuilder::new()
        //         // `timeout` will produce an error if the handler takes
        //         // too long so we must handle those
        //         .layer(HandleErrorLayer::new(global_error_handler))
        //         .timeout(Duration::from_secs(30)),
        // )
        .layer(axum::middleware::from_fn(logging_middleware))
        .with_state(state))
}
