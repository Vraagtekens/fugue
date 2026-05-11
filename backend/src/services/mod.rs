use crate::services::{sessions_service::SessionsService, user_service::UserService};

pub mod live_service;
pub mod sessions_service;
pub mod user_service;

#[derive(Clone)]
pub struct Services {
    pub user: UserService,
    pub sessions: SessionsService,
}
