use sea_orm::entity::prelude::*;

/// Animated flag assets for overlay playback.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "overlay_flag_animation")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub ioc_code: String,
    pub file_name: String,
    pub file_path: String,
    pub display_name: Option<String>,
    pub duration_ms: Option<i32>,
    pub is_default: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
