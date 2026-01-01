use crate::{
    config::Config,
    services::Services,
    utils::{jwt::JwtManager, s3::S3Manager},
};
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub s3: S3Manager,
    pub jwt: JwtManager,
    pub config: Config,
    pub services: Services,
}
