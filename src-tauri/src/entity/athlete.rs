use sea_orm::entity::prelude::*;

/// Canonical athlete roster entry.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "athlete")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub uuid: String,
    pub wt_id: Option<String>,
    pub pss_code: Option<String>,
    pub country: Option<String>,
    pub short_name: Option<String>,
    pub display_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub country_code: Option<String>,
    pub ioc_code: Option<String>,
    pub flag_id: Option<i32>,
    pub gender_id: Option<i32>,
    pub division_id: Option<i32>,
    pub weight_class_id: Option<i32>,
    pub age_group_id: Option<i32>,
    pub image: Option<String>,
    pub history: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::flag::Entity",
        from = "Column::FlagId",
        to = "super::flag::Column::Id"
    )]
    Flag,
    #[sea_orm(
        belongs_to = "super::gender::Entity",
        from = "Column::GenderId",
        to = "super::gender::Column::Id"
    )]
    Gender,
    #[sea_orm(
        belongs_to = "super::division::Entity",
        from = "Column::DivisionId",
        to = "super::division::Column::Id"
    )]
    Division,
    #[sea_orm(
        belongs_to = "super::weight_class::Entity",
        from = "Column::WeightClassId",
        to = "super::weight_class::Column::Id"
    )]
    WeightClass,
    #[sea_orm(
        belongs_to = "super::age_group::Entity",
        from = "Column::AgeGroupId",
        to = "super::age_group::Column::Id"
    )]
    AgeGroup,
    #[sea_orm(has_many = "super::match_participant::Entity")]
    MatchParticipant,
    #[sea_orm(has_many = "super::medal_ceremony_medalist::Entity")]
    MedalCeremonyMedalist,
}

impl Related<super::flag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Flag.def()
    }
}

impl Related<super::gender::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Gender.def()
    }
}

impl Related<super::division::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Division.def()
    }
}

impl Related<super::weight_class::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WeightClass.def()
    }
}

impl Related<super::age_group::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AgeGroup.def()
    }
}

impl Related<super::match_participant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MatchParticipant.def()
    }
}

impl Related<super::medal_ceremony_medalist::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MedalCeremonyMedalist.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
