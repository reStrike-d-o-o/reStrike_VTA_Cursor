use sea_orm::entity::prelude::*;

/// Recording configuration per OBS connection.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "obs_recording_config")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub obs_connection_name: String,
    pub recording_root_path: String,
    pub recording_format: String,
    pub replay_buffer_enabled: bool,
    pub replay_buffer_duration: i32,
    pub auto_start_recording: bool,
    pub auto_start_replay_buffer: bool,
    pub filename_template: String,
    pub folder_pattern: String,
    pub is_active: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
