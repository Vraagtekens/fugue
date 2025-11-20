use sea_orm::entity::prelude::*;
use sea_orm_migration::prelude::*;
use sea_query::{ColumnDef, Table};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, TS, Serialize, Deserialize)]
#[sea_orm(table_name = "sessions")]
#[ts(export)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub user_id: i32,
    pub category_id: Option<i32>,
    pub kind: String,
    pub completed: bool,

    #[ts(type = "number")]
    pub start_time: DateTimeUtc,
    #[ts(type = "number")]
    pub end_time: Option<DateTimeUtc>,

    #[ts(type = "number")]
    pub created_at: DateTimeUtc,
    #[ts(type = "number")]
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// Enum for column names to avoid repeating strings
#[derive(Iden)]
pub enum Session {
    Table,
    Id,
    UserId,
    CategoryId,
    Kind,
    Completed,
    StartTime,
    EndTime,
    CreatedAt,
    UpdatedAt,
}

impl Model {
    /// Generate a TableCreateStatement from the model
    pub fn create_table_statement() -> TableCreateStatement {
        Table::create()
            .table(Session::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Session::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Session::UserId).integer().not_null())
            .col(ColumnDef::new(Session::CategoryId).integer())
            .col(
                ColumnDef::new(Session::Kind)
                    .string()
                    .not_null()
                    .default("pomodoro"),
            )
            .col(
                ColumnDef::new(Session::Completed)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(ColumnDef::new(Session::StartTime).timestamp().not_null())
            .col(ColumnDef::new(Session::EndTime).timestamp())
            .col(
                ColumnDef::new(Session::CreatedAt)
                    .timestamp()
                    .not_null()
                    .default("CURRENT_TIMESTAMP"),
            )
            .col(
                ColumnDef::new(Session::UpdatedAt)
                    .timestamp()
                    .not_null()
                    .default("CURRENT_TIMESTAMP"),
            )
            .to_owned()
    }
}
