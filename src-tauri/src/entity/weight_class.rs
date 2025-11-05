use sea_orm::entity::prelude::*;

/// Weight class lookup keyed by discipline, gender, and age group.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "weight_class")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub discipline_id: i32,
    pub gender_id: i32,
    pub age_group_id: i32,
    pub code: String,
    pub name: String,
    pub min_kg: Option<f64>,
    pub max_kg: Option<f64>,
    pub authority: Option<String>,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::discipline::Entity",
        from = "Column::DisciplineId",
        to = "super::discipline::Column::Id"
    )]
    Discipline,
    #[sea_orm(
        belongs_to = "super::gender::Entity",
        from = "Column::GenderId",
        to = "super::gender::Column::Id"
    )]
    Gender,
    #[sea_orm(
        belongs_to = "super::age_group::Entity",
        from = "Column::AgeGroupId",
        to = "super::age_group::Column::Id"
    )]
    AgeGroup,
}

impl Related<super::discipline::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Discipline.def()
    }
}

impl Related<super::gender::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Gender.def()
    }
}

impl Related<super::age_group::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AgeGroup.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
