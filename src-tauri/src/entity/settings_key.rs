use sea_orm::entity::prelude::*;

/// Defines an individual application setting and its metadata.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "settings_keys")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub category_id: String,
    pub key_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub data_type: String,
    pub default_value: Option<String>,
    pub validation_rules: Option<String>,
    pub is_required: bool,
    pub is_sensitive: bool,
    #[sea_orm(column_name = "created")]
    pub created: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::settings_category::Entity",
        from = "Column::CategoryId",
        to = "super::settings_category::Column::Id"
    )]
    Category,
    #[sea_orm(has_many = "super::settings_value::Entity")]
    Value,
    #[sea_orm(has_many = "super::settings_history::Entity")]
    History,
}

impl Related<super::settings_category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}

impl Related<super::settings_value::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Value.def()
    }
}

impl Related<super::settings_history::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::History.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
