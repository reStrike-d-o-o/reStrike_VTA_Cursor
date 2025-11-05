use sea_orm::entity::prelude::*;

/// Unclassified UDP packets captured for later analysis.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "event_unknown")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub session_id: i32,
    pub raw_data: String,
    pub first_seen: String,
    pub last_seen: String,
    pub occurrence_count: i32,
    pub pattern_hash: Option<String>,
    pub suggested_event_type: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
