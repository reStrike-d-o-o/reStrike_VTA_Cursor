use sea_orm::entity::prelude::*;

/// Tracks individual UDP clients during a session.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "udp_client_connection")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub server_session_id: i32,
    pub client_address: String,
    pub client_port: i32,
    pub first_seen: String,
    pub last_seen: String,
    pub packets_received: i32,
    pub total_bytes_received: i64,
    pub is_active: bool,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::udp_server_session::Entity",
        from = "Column::ServerSessionId",
        to = "super::udp_server_session::Column::Id"
    )]
    UdpServerSession,
}

impl Related<super::udp_server_session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UdpServerSession.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
