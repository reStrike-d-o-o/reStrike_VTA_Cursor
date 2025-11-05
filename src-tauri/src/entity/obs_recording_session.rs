use sea_orm::entity::prelude::*;

/// Captures individual OBS recording or replay buffer sessions.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "obs_recording_session")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub obs_connection_name: String,
    pub tournament_id: Option<i32>,
    pub match_code: Option<String>,
    pub match_number: Option<String>,
    pub player1_name: Option<String>,
    pub player1_flag: Option<String>,
    pub player2_name: Option<String>,
    pub player2_flag: Option<String>,
    pub recording_path: String,
    pub recording_filename: String,
    pub recording_start_time: Option<String>,
    pub recording_end_time: Option<String>,
    pub recording_duration: Option<i32>,
    pub recording_size_bytes: Option<i64>,
    pub replay_buffer_start_time: Option<String>,
    pub replay_buffer_end_time: Option<String>,
    pub replay_buffer_saved: bool,
    pub replay_buffer_filename: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tournament::Entity",
        from = "Column::TournamentId",
        to = "super::tournament::Column::Id"
    )]
    Tournament,
}

impl Related<super::tournament::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tournament.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
