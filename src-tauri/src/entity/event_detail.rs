use sea_orm::entity::prelude::*;

/// Key-value metadata attached to an event.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "event_detail")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub event_id: i32,
    pub detail_key: String,
    pub detail_value: Option<String>,
    pub detail_type: String,
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
