use sea_orm::entity::prelude::*;

/// Stores medal results for completed matches within a tournament.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tournament_champion")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub tournament_id: Option<i32>,
    pub match_id: Option<i32>,
    pub match_uuid: Option<String>,
    pub match_code: Option<String>,
    pub category: Option<String>,
    pub winner_side: Option<String>,
    pub winner_name: Option<String>,
    pub winner_country_code: Option<String>,
    pub blue_score: Option<i32>,
    pub red_score: Option<i32>,
    pub medal_type: String,
    pub medal_rank: i32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tournament::Entity",
        from = "Column::TournamentId",
        to = "super::tournament::Column::Id"
    )]
    Tournament,
    #[sea_orm(
        belongs_to = "super::matches::Entity",
        from = "Column::MatchId",
        to = "super::matches::Column::Id"
    )]
    Match,
}

impl Related<super::tournament::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tournament.def()
    }
}

impl Related<super::matches::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Match.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
