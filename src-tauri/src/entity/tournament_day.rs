use sea_orm::entity::prelude::*;

/// Represents a single competition day within a tournament.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tournament_day")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub uuid: String,
    pub tournament_id: i32,
    pub day_number: i32,
    pub date: String,
    pub status: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
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
    #[sea_orm(has_many = "super::octagon::Entity")]
    Octagon,
}

impl Related<super::tournament::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tournament.def()
    }
}

impl Related<super::octagon::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Octagon.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
