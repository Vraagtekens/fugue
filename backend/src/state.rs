use crate::{config::Config, services::Services, utils::jwt::JwtManager};
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt: JwtManager,
    pub config: Config,
    pub services: Services,
}
