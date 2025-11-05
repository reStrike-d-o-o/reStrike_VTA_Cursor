use sea_orm::entity::prelude::*;

/// External overlay provider configuration.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "overlay_provider")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub base_url: Option<String>,
    pub enabled: bool,
    pub rate_limit_ms: i32,
    pub last_refreshed_at: Option<String>,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::overlay_tournament::Entity")]
    OverlayTournament,
}

impl Related<super::overlay_tournament::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OverlayTournament.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
