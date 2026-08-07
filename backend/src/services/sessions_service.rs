use crate::{entities::sessions, errors::ApiError, routes::sessions::handlers::AddSessionRequest};
use sea_orm::*;

#[derive(Clone)]
pub struct SessionsService {
    pub db: DatabaseConnection,
}

impl SessionsService {
    pub async fn add_session(&self, payload: &AddSessionRequest) -> Result<sessions::Model, DbErr> {
        let new = sessions::ActiveModel {
            title: Set(payload.title.clone()),
            user_id: Set(payload.user_id),
            start_time: Set(payload.start_time.naive_utc()),
            end_time: Set(payload.end_time.map(|t| t.naive_utc())),
            ..Default::default()
        };

        new.insert(&self.db).await
    }

    pub async fn get_all_sessions(&self) -> Result<Vec<sessions::Model>, ApiError> {
        let sessions = sessions::Entity::find()
            .order_by_desc(sessions::Column::StartTime)
            .all(&self.db)
            .await?;

        Ok(sessions)
    }

    pub async fn get_session(&self, id: i32) -> Result<Option<sessions::Model>, ApiError> {
        Ok(sessions::Entity::find_by_id(id).one(&self.db).await?)
    }

    pub async fn delete_session(&self, id: i32) -> Result<(), ApiError> {
        sessions::Entity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }
}
