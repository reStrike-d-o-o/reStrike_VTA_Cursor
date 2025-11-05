use sea_orm::entity::prelude::*;

/// Stored video recording metadata.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "video")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub match_id: i32,
    pub event_id: Option<i32>,
    pub tournament_uuid: Option<String>,
    pub r#type: String,
    pub file_path: Option<String>,
    pub directory: Option<String>,
    pub filename_formatting: Option<String>,
    pub start_time: String,
    pub duration_seconds: Option<i32>,
    pub file_size: Option<i64>,
    pub checksum: Option<String>,
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
        belongs_to = "super::event::Entity",
        from = "Column::EventId",
        to = "super::event::Column::Id"
    )]
    Event,
    #[sea_orm(has_many = "super::video_event::Entity")]
    VideoEvent,
}

impl Related<super::matches::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Match.def()
    }
}

impl Related<super::event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Event.def()
    }
}

impl Related<super::video_event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::VideoEvent.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
