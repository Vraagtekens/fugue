// migration/src/lib.rs

pub use sea_orm_migration::prelude::*;

// Add each migration file as a module
mod m20251120_110901_create_sessions;
mod m20251120_151556_create_users;
mod m20251120_151615_create_categories;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251120_110901_create_sessions::Migration),
            Box::new(m20251120_151556_create_users::Migration),
            Box::new(m20251120_151615_create_categories::Migration),
        ]
    }
}
