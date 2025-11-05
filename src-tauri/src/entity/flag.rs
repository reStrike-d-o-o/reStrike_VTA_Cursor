use sea_orm::entity::prelude::*;

/// Static flag assets recognised by the system.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "flag")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub filename: String,
    pub ioc_code: Option<String>,
    pub country_name: Option<String>,
    pub recognition_status: Option<String>,
    pub recognition_confidence: Option<f64>,
    pub upload_date: Option<DateTime>,
    pub last_modified: Option<DateTime>,
    pub file_size: Option<i64>,
    pub file_path: String,
    pub is_recognized: bool,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
