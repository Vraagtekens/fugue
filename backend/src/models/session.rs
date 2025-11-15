use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: i32,
    pub user_id: i32,
    pub category_id: Option<i32>, // optional
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub kind: String,
    pub completed: bool,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}
