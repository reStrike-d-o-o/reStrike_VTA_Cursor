use sea_orm::entity::prelude::*;

/// Medal recipient entry for a ceremony division.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "medal_ceremony_medalist")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub division_entry_id: String,
    pub medal_type: String,
    pub medal_rank: i32,
    pub athlete_id: Option<i32>,
    pub athlete_name: String,
    pub athlete_short_name: Option<String>,
    pub ioc_code: Option<String>,
    pub flag_asset: Option<String>,
    pub anthem_asset: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::medal_ceremony_division::Entity",
        from = "Column::DivisionEntryId",
        to = "super::medal_ceremony_division::Column::Id"
    )]
    DivisionEntry,
    #[sea_orm(
        belongs_to = "super::athlete::Entity",
        from = "Column::AthleteId",
        to = "super::athlete::Column::Id"
    )]
    Athlete,
}

impl Related<super::medal_ceremony_division::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DivisionEntry.def()
    }
}

impl Related<super::athlete::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Athlete.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
