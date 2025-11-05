use sea_orm::entity::prelude::*;

/// Ceremony division ordering entry.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "medal_ceremony_division")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub ceremony_id: String,
    pub division_id: Option<i32>,
    pub division_name: String,
    pub order_index: i32,
    pub played_at: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::medal_ceremony::Entity",
        from = "Column::CeremonyId",
        to = "super::medal_ceremony::Column::Id"
    )]
    MedalCeremony,
    #[sea_orm(has_many = "super::medal_ceremony_medalist::Entity")]
    Medalist,
}

impl Related<super::medal_ceremony::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MedalCeremony.def()
    }
}

impl Related<super::medal_ceremony_medalist::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Medalist.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
