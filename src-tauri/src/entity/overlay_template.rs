use sea_orm::entity::prelude::*;

/// HTML/CSS overlay templates used for OBS automations.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "overlay_templates")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub theme: String,
    pub colors: Option<String>,
    pub animation_type: String,
    pub duration_ms: i32,
    pub is_active: bool,
    pub url: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
