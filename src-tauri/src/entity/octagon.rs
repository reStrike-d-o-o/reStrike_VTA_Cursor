use sea_orm::entity::prelude::*;

/// Physical ring assignment metadata.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "octagon")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub tournament_id: i32,
    pub tournament_day_id: Option<i32>,
    pub number: String,
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
        belongs_to = "super::tournament_day::Entity",
        from = "Column::TournamentDayId",
        to = "super::tournament_day::Column::Id"
    )]
    TournamentDay,
}

impl Related<super::tournament::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tournament.def()
    }
}

impl Related<super::tournament_day::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TournamentDay.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
