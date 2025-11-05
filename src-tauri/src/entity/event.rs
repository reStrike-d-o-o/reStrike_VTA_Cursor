use sea_orm::entity::prelude::*;

/// Raw PSS event stream entries.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "event")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub session_id: i32,
    pub match_id: Option<i32>,
    pub round_id: Option<i32>,
    pub event_type_id: i32,
    pub timestamp: String,
    pub raw_data: String,
    pub parsed_data: Option<String>,
    pub event_sequence: Option<i32>,
    pub processing_time_ms: Option<i32>,
    pub is_valid: bool,
    pub error_message: Option<String>,
    pub recognition_status: String,
    pub protocol_version: Option<String>,
    pub parser_confidence: Option<f64>,
    pub validation_errors: Option<String>,
    pub tournament_uuid: Option<String>,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::matches::Entity",
        from = "Column::MatchId",
        to = "super::matches::Column::Id"
    )]
    Match,
    #[sea_orm(
        belongs_to = "super::round::Entity",
        from = "Column::RoundId",
        to = "super::round::Column::Id"
    )]
    Round,
    #[sea_orm(
        belongs_to = "super::event_type::Entity",
        from = "Column::EventTypeId",
        to = "super::event_type::Column::Id"
    )]
    EventType,
    #[sea_orm(has_many = "super::event_detail::Entity")]
    EventDetail,
    #[sea_orm(has_many = "super::event_recognition_history::Entity")]
    EventRecognitionHistory,
    #[sea_orm(has_many = "super::event_validation_result::Entity")]
    EventValidationResult,
}

impl Related<super::matches::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Match.def()
    }
}

impl Related<super::round::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Round.def()
    }
}

impl Related<super::event_type::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EventType.def()
    }
}

impl Related<super::event_detail::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EventDetail.def()
    }
}

impl Related<super::event_recognition_history::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EventRecognitionHistory.def()
    }
}

impl Related<super::event_validation_result::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EventValidationResult.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
