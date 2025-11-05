use sea_orm::entity::prelude::*;

/// Lookup catalog for age group classifications.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "age_group")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub code: String,
    pub name: String,
    pub min_age: Option<i32>,
    pub max_age: Option<i32>,
    pub authority: Option<String>,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
