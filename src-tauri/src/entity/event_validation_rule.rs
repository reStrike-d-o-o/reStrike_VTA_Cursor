use sea_orm::entity::prelude::*;

/// Validation rule metadata for incoming events.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "event_validation_rule")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub event_code: String,
    pub protocol_version: String,
    pub rule_name: String,
    pub rule_type: String,
    pub rule_definition: String,
    pub error_message: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::event_validation_result::Entity")]
    EventValidationResult,
}

impl Related<super::event_validation_result::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EventValidationResult.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
