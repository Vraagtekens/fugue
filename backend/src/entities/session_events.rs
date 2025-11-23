use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum EventType {
    #[sea_orm(string_value = "pause")]
    Pause,

    #[sea_orm(string_value = "resume")]
    Resume,

    #[sea_orm(string_value = "cancel")]
    Cancel,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "session_events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub session_id: i32,

    pub event_type: EventType,

    pub created_at: Option<DateTime>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
