use sea_orm::entity::prelude::*;

/// Logical grouping for application settings (for example UI, logging, UDP).
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "settings_categories")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub display_order: i32,
    #[sea_orm(column_name = "created")]
    pub created: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::settings_key::Entity")]
    SettingsKey,
}

impl Related<super::settings_key::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SettingsKey.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
