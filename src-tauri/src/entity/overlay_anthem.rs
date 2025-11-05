use sea_orm::entity::prelude::*;

/// National anthem media metadata for overlays.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "overlay_anthem")]
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

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
