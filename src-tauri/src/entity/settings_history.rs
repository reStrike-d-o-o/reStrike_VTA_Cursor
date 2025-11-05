use sea_orm::entity::prelude::*;

/// Audit trail entry for a setting value change.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "settings_history")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub key_id: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: String,
    pub change_reason: Option<String>,
    pub created: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::settings_key::Entity",
        from = "Column::KeyId",
        to = "super::settings_key::Column::Id"
    )]
    Key,
}

impl Related<super::settings_key::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Key.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
