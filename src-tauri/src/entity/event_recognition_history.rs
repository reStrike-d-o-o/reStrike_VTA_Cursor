use sea_orm::entity::prelude::*;

/// Tracks changes to event recognition status over time.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "event_recognition_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub event_id: i32,
    pub old_status: String,
    pub new_status: String,
    pub changed_by: String,
    pub change_reason: Option<String>,
    pub protocol_version: Option<String>,
    pub raw_data: String,
    pub parsed_data: Option<String>,
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
}

impl Related<super::event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Event.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
