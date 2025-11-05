use sea_orm::entity::prelude::*;

/// Cross reference between recorded video and events within it.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "video_event")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub video_id: i32,
    pub event_id: i32,
    pub offset_ms: i32,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::video::Entity",
        from = "Column::VideoId",
        to = "super::video::Column::Id"
    )]
    Video,
    #[sea_orm(
        belongs_to = "super::event::Entity",
        from = "Column::EventId",
        to = "super::event::Column::Id"
    )]
    Event,
}

impl Related<super::video::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Video.def()
    }
}

impl Related<super::event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Event.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
