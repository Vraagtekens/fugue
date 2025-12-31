use chrono::{Duration, Utc};
use cookie::Cookie;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};
use time::{Duration as TimeDuration, OffsetDateTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
}

#[derive(Clone)]
pub struct JwtManager {
    secret: String,
    expiration_hours: u64,
}

impl JwtManager {
    pub fn new(secret: String, expiration_hours: u64) -> Self {
        Self {
            secret,
            expiration_hours,
        }
    }

    /// Generate a JWT for a user
    pub fn generate(&self, user_id: Uuid) -> String {
        let expiration = Utc::now() + Duration::hours(self.expiration_hours as i64);

        let claims = Claims {
            sub: user_id,
            exp: expiration.timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .expect("JWT encoding failed")
    }

    /// Verify a JWT, return Claims if valid
    pub fn verify(&self, token: &str) -> Option<Claims> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .ok()
        .map(|data| data.claims)
    }

    /// Builds a secure session cookie
    pub fn build_cookie(&self, token: String) -> Cookie<'static> {
        let expires = OffsetDateTime::now_utc() + TimeDuration::hours(self.expiration_hours as i64);

        Cookie::build(("session", token))
            .http_only(true)
            .secure(true) // set false for local dev
            .expires(expires)
            .path("/")
            .build()
    }
}
