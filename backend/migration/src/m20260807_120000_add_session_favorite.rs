use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260807_120000_add_session_favorite"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Sessions::Table)
                    .add_column(
                        ColumnDef::new(Sessions::Favorite)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Sessions::Table)
                    .drop_column(Sessions::Favorite)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Sessions {
    Table,
    Favorite,
}
