use sea_orm::entity::prelude::*;

/// Links overlay tournaments to local canonical tournaments.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "overlay_tournament_map")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub overlay_tournament_id: i32,
    pub tournament_id: i32,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::overlay_tournament::Entity",
        from = "Column::OverlayTournamentId",
        to = "super::overlay_tournament::Column::Id"
    )]
    OverlayTournament,
    #[sea_orm(
        belongs_to = "super::tournament::Entity",
        from = "Column::TournamentId",
        to = "super::tournament::Column::Id"
    )]
    Tournament,
}

impl Related<super::overlay_tournament::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OverlayTournament.def()
    }
}

impl Related<super::tournament::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tournament.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
