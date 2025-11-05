use sea_orm::entity::prelude::*;

/// Persistent configuration for medal ceremonies.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "medal_ceremony")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub tournament_id: Option<i32>,
    pub name: String,
    pub background_path: Option<String>,
    pub break_path: Option<String>,
    pub animation_duration: i32,
    pub animation_speed: f64,
    pub photo_time: i32,
    pub prepared_at: Option<String>,
    pub prepared_version: i32,
    pub show_external: bool,
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
    #[sea_orm(has_many = "super::medal_ceremony_division::Entity")]
    Division,
}

impl Related<super::tournament::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tournament.def()
    }
}

impl Related<super::medal_ceremony_division::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Division.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
