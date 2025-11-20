use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_query::Table;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251120_110901_create_sessions" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create sessions table
        manager
            .create_table(
                Table::create()
                    .table(Sessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Sessions::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Sessions::UserId).integer().not_null())
                    .col(ColumnDef::new(Sessions::CategoryId).integer())
                    .col(ColumnDef::new(Sessions::StartTime).timestamp().not_null())
                    .col(ColumnDef::new(Sessions::EndTime).timestamp())
                    .col(
                        ColumnDef::new(Sessions::Kind)
                            .text()
                            .not_null()
                            .default("pomodoro"),
                    )
                    .col(ColumnDef::new(Sessions::Completed).boolean().default(false))
                    .col(
                        ColumnDef::new(Sessions::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Sessions::UpdatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop session table
        manager
            .drop_table(Table::drop().table(Sessions::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum Sessions {
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
