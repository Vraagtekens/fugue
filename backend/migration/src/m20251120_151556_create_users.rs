use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_query::Table;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251120_151556_create_users"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::Email).text().not_null().unique_key())
                    .col(ColumnDef::new(Users::PasswordHash).text().not_null())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum Users {
    #[iden = "users"]
    Table,
    Id,
    Email,
    PasswordHash,
    CreatedAt,
    UpdatedAt,
}
