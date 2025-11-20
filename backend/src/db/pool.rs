// use sqlx::{PgPool, postgres::PgPoolOptions};
// use std::env;

// pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
//     PgPoolOptions::new()
//         .max_connections(5)
//         .connect(database_url)
//         .await
// }

use sea_orm::{Database, DatabaseConnection};

pub async fn create_pool(database_url: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    Database::connect(database_url).await
}
