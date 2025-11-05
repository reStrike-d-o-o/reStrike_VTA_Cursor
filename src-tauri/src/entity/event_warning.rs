use sea_orm::entity::prelude::*;

/// Warning entries (gam-jeom etc.) tied to a match.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "event_warning")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub match_id: i32,
    pub round_id: Option<i32>,
    pub side: String,
    pub warning_type: String,
    pub warning_count: i32,
    pub timestamp: Option<String>,
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

impl ActiveModelBehavior for ActiveModel {}
