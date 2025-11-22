use axum::{
    Json,
    extract::{FromRequest, Request},
    http::StatusCode,
};

use serde::de::DeserializeOwned;

use crate::errors::ApiError;

/// A JSON extractor that returns `ApiError` instead of Axum rejections.
pub struct TypedJson<T>(pub T);

impl<S, T> FromRequest<S> for TypedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Send,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Use Axum's Json extractor, but map its rejection into ApiError
        let json = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| ApiError::new(StatusCode::BAD_REQUEST, rejection.body_text()))?;

        Ok(Self(json.0))
    }
}
