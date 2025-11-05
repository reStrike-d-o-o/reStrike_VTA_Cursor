use sea_orm::entity::prelude::*;

/// Metadata for tournaments fetched from overlay providers.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "overlay_tournament")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub provider_id: i32,
    pub provider_tournament_id: String,
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub url: Option<String>,
    pub status: Option<String>,
    pub last_seen_at: Option<String>,
    pub hash: Option<String>,
    pub etag: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::overlay_provider::Entity",
        from = "Column::ProviderId",
        to = "super::overlay_provider::Column::Id"
    )]
    Provider,
    #[sea_orm(has_many = "super::overlay_category::Entity")]
    OverlayCategory,
    #[sea_orm(has_many = "super::overlay_tournament_map::Entity")]
    OverlayTournamentMap,
}

impl Related<super::overlay_provider::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Provider.def()
    }
}

impl Related<super::overlay_category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OverlayCategory.def()
    }
}

impl Related<super::overlay_tournament_map::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OverlayTournamentMap.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
