use crate::{entities::users, routes::auth::handlers::login::LoginRequest};
use axum::http::StatusCode;
use bcrypt::verify;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

#[derive(Clone)]
pub struct UserService {
    pub db: sea_orm::DatabaseConnection,
}

impl UserService {
    pub async fn authenticate(
        &self,
        payload: &LoginRequest,
    ) -> Result<users::Model, (StatusCode, String)> {
        // Fetch user by email
        let user = users::Entity::find()
            .filter(users::Column::Email.eq(payload.email.as_str()))
            .one(&self.db)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error".into()))?
            .ok_or((StatusCode::UNAUTHORIZED, "User not found".into()))?;

        // Check password
        let valid = verify(&payload.password, &user.password_hash)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Verify error".into()))?;

        if !valid {
            return Err((StatusCode::UNAUTHORIZED, "Wrong password".into()));
        }

        Ok(user)
    }
}
