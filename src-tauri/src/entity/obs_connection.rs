use sea_orm::entity::prelude::*;

/// Configured OBS WebSocket connections.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "obs_connection")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub password: Option<String>,
    pub is_active: bool,
    pub status: String,
    pub error: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::obs_recording_session::Entity")]
    RecordingSession,
}

impl Related<super::obs_recording_session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RecordingSession.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
