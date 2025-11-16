use sqlx::PgPool;

use crate::{config::Config, services::Services, utils::jwt::JwtManager};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt: JwtManager,
    pub config: Config,
    pub services: Services,
}
