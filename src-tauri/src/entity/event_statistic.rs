use sea_orm::entity::prelude::*;

/// Aggregated statistics per event type/session.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "event_statistic")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub session_id: i32,
    pub event_type_id: Option<i32>,
    pub total_events: i32,
    pub recognized_events: i32,
    pub unknown_events: i32,
    pub partial_events: i32,
    pub deprecated_events: i32,
    pub validation_errors: i32,
    pub parsing_errors: i32,
    pub average_processing_time_ms: f64,
    pub min_processing_time_ms: Option<i32>,
    pub max_processing_time_ms: Option<i32>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::event_type::Entity",
        from = "Column::EventTypeId",
        to = "super::event_type::Column::Id"
    )]
    EventType,
}

impl Related<super::event_type::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EventType.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
