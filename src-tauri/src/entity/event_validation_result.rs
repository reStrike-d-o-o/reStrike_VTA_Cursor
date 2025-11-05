use sea_orm::entity::prelude::*;

/// Stores outcome of applying a validation rule to an event.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "event_validation_result")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub event_id: i32,
    pub rule_id: i32,
    pub validation_passed: bool,
    pub error_message: Option<String>,
    pub validation_time_ms: Option<i32>,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::event::Entity",
        from = "Column::EventId",
        to = "super::event::Column::Id"
    )]
    Event,
    #[sea_orm(
        belongs_to = "super::event_validation_rule::Entity",
        from = "Column::RuleId",
        to = "super::event_validation_rule::Column::Id"
    )]
    Rule,
}

impl Related<super::event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Event.def()
    }
}

impl Related<super::event_validation_rule::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rule.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
