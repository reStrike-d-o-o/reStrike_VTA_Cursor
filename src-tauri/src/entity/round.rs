use sea_orm::entity::prelude::*;

/// Round summary for a given match.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "round")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub match_id: i32,
    pub round_number: i32,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub duration_seconds: Option<i32>,
    pub winner_side: Option<String>,
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
    #[sea_orm(has_many = "super::score::Entity")]
    Score,
    #[sea_orm(has_many = "super::event::Entity")]
    Event,
}

impl Related<super::matches::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Match.def()
    }
}

impl Related<super::score::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Score.def()
    }
}

impl Related<super::event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Event.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
