use sea_orm::entity::prelude::*;

/// Reference table for PSS event codes.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "event_type")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub is_active: bool,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::event::Entity")]
    Event,
    #[sea_orm(has_many = "super::event_statistic::Entity")]
    EventStatistic,
}

impl Related<super::event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Event.def()
    }
}

impl Related<super::event_statistic::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EventStatistic.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
