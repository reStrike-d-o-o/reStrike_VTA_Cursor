use sea_orm::entity::prelude::*;

/// OBS scenes discovered through WebSocket polling.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "obs_scene")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub scene_name: String,
    pub scene_id: String,
    pub is_active: bool,
    pub last_seen_at: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
