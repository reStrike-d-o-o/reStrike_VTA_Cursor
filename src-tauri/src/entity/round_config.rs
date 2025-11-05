use sea_orm::entity::prelude::*;

/// Preset round configuration options.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "round_config")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub rounds: i32,
    pub round_duration: Option<i32>,
    pub rest_duration: Option<i32>,
    pub golden_round: bool,
    pub created_at: DateTime,
    pub golden_round_duration: Option<i32>,
    pub kyeshi_duration: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
