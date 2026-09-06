// migration/src/lib.rs
pub use sea_orm_migration::prelude::*;

// Add each migration file as a module
mod m20251120_151556_create_users;
mod m20251120_160901_create_sessions;
mod m20260807_120000_add_session_favorite;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251120_151556_create_users::Migration),
            Box::new(m20251120_160901_create_sessions::Migration),
            Box::new(m20260807_120000_add_session_favorite::Migration),
        ]
    }
}
