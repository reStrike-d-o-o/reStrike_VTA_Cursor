use sea_orm::entity::prelude::*;

/// Participant assignment linking athletes to matches.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "match_participant")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub match_id: i32,
    pub athlete_id: i32,
    pub side: String,
    pub bg_color: Option<String>,
    pub fg_color: Option<String>,
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
        belongs_to = "super::athlete::Entity",
        from = "Column::AthleteId",
        to = "super::athlete::Column::Id"
    )]
    Athlete,
}

impl Related<super::matches::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Match.def()
    }
}

impl Related<super::athlete::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Athlete.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
