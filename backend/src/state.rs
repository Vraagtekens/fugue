use crate::{
    config::Config,
    services::{Services, live_service::LiveSessionHub},
    utils::{jwt::JwtManager, mscore::MscoreManager, s3::S3Manager},
};
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub s3: S3Manager,
    pub jwt: JwtManager,
    pub mscore: MscoreManager,
    pub config: Config,
    pub services: Services,
    pub live: LiveSessionHub,
}
