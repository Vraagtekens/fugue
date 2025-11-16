use axum::http::StatusCode;
use sqlx::PgPool;

use crate::{
    errors::ApiError, models::session::Session, routes::sessions::handlers::AddSessionRequest,
};

#[derive(Clone)]
pub struct SessionsService {
    pub db: PgPool,
}

impl SessionsService {
    pub async fn add_session(
        &self,
        user_id: i32,
        payload: &AddSessionRequest,
    ) -> Result<Session, ApiError> {
        let session = sqlx::query_as::<_, Session>(
            r#"
        INSERT INTO sessions (
            user_id,
            category_id,
            start_time,
            end_time,
            kind,
            completed
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
        )
        .bind(user_id)
        .bind(payload.category_id)
        .bind(payload.start_time)
        .bind(payload.end_time)
        .bind(&payload.kind)
        .bind(payload.completed)
        .fetch_one(&self.db)
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;

        Ok(session)
    }

    pub async fn get_all_sessions(&self) -> Result<Vec<Session>, ApiError> {
        let sessions = sqlx::query_as::<_, Session>(
            r#"
            SELECT * FROM sessions
            ORDER BY start_time DESC
            "#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(sessions)
    }
}
