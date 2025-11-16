use crate::{models::user::User, routes::auth::handlers::login::LoginRequest};
use axum::http::StatusCode;
use bcrypt::verify;
use sqlx::PgPool;

#[derive(Clone)]
pub struct UserService {
    pub db: PgPool,
}

impl UserService {
    pub async fn authenticate(&self, payload: &LoginRequest) -> Result<User, (StatusCode, String)> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(&payload.email)
            .fetch_one(&self.db)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "User not found".into()))?;

        let valid = verify(&payload.password, &user.password_hash)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Verify error".into()))?;

        if !valid {
            return Err((StatusCode::UNAUTHORIZED, "Wrong password".into()));
        }

        Ok(user)
    }
}
