use sea_orm::entity::prelude::*;

/// Canonical tournament metadata.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tournament")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub duration_days: i32,
    pub city: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub logo_path: Option<String>,
    pub status: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub ranking_id: Option<i32>,
    pub location: String,
    pub contact: String,
    pub organizing_committee: String,
    pub officials: String,
    pub banner: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::tournament_day::Entity")]
    TournamentDay,
    #[sea_orm(has_many = "super::matches::Entity")]
    Match,
    #[sea_orm(has_many = "super::tournament_champion::Entity")]
    TournamentChampion,
    #[sea_orm(has_many = "super::medal_ceremony::Entity")]
    MedalCeremony,
    #[sea_orm(has_many = "super::octagon::Entity")]
    Octagon,
}

impl Related<super::tournament_day::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TournamentDay.def()
    }
}

impl Related<super::matches::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Match.def()
    }
}

impl Related<super::tournament_champion::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TournamentChampion.def()
    }
}

impl Related<super::medal_ceremony::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MedalCeremony.def()
    }
}

impl Related<super::octagon::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Octagon.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
