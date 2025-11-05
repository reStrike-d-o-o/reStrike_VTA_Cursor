use sea_orm::entity::prelude::*;

/// Core match metadata (canonical replacement for `pss_matches`).
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "match")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub uuid: String,
    pub tournament_id: Option<i32>,
    pub match_code: String,
    pub match_number: Option<String>,
    pub category: Option<String>,
    pub division_code: Option<String>,
    pub weight_class_code: Option<String>,
    pub total_rounds: Option<i32>,
    pub round_duration: Option<i32>,
    pub countdown_type: Option<String>,
    pub format_type: Option<i32>,
    pub creation_mode: Option<String>,
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
    #[sea_orm(has_many = "super::match_participant::Entity")]
    MatchParticipant,
    #[sea_orm(has_many = "super::tournament_champion::Entity")]
    TournamentChampion,
}

impl Related<super::tournament::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tournament.def()
    }
}

impl Related<super::match_participant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MatchParticipant.def()
    }
}

impl Related<super::tournament_champion::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TournamentChampion.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
