// use migration::Categories;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_query::Table;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251120_160901_create_sessions" // Make sure this matches with the file name
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
                    .col(ColumnDef::new(Sessions::UserId).uuid().not_null())
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk_sessions_users")
                            .from_tbl(Sessions::Table)
                            .from_col(Sessions::UserId)
                            .to_tbl("users")
                            .to_col("id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(Sessions::Title)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Sessions::StartTime).timestamp().not_null())
                    .col(ColumnDef::new(Sessions::EndTime).timestamp())
                    .col(
                        ColumnDef::new(Sessions::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Sessions::UpdatedAt)
                            .timestamp_with_time_zone()
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
    Title,
    StartTime,
    EndTime,
    CreatedAt,
    UpdatedAt,
}
