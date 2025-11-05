use sea_orm::entity::prelude::*;

/// Category metadata supplied by overlay providers.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "overlay_category")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub overlay_tournament_id: i32,
    pub discipline: Option<String>,
    pub age_group: Option<String>,
    pub gender: Option<String>,
    pub division: Option<String>,
    pub weight_class: Option<String>,
    pub bracket_stage: Option<String>,
    pub provider_raw: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::overlay_tournament::Entity",
        from = "Column::OverlayTournamentId",
        to = "super::overlay_tournament::Column::Id"
    )]
    OverlayTournament,
}

impl Related<super::overlay_tournament::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OverlayTournament.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
