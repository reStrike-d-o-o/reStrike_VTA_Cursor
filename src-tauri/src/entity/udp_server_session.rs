use sea_orm::entity::prelude::*;

/// Runtime telemetry for UDP server sessions.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "udp_server_session")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub server_config_id: i32,
    pub start_time: String,
    pub end_time: Option<String>,
    pub status: String,
    pub packets_received: i32,
    pub packets_parsed: i32,
    pub parse_errors: i32,
    pub total_bytes_received: i64,
    pub average_packet_size: f64,
    pub max_packet_size_seen: i32,
    pub min_packet_size_seen: i32,
    pub unique_clients_count: i32,
    pub error_message: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::udp_server_config::Entity",
        from = "Column::ServerConfigId",
        to = "super::udp_server_config::Column::Id"
    )]
    UdpServerConfig,
    #[sea_orm(has_many = "super::udp_client_connection::Entity")]
    UdpClientConnection,
}

impl Related<super::udp_server_config::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UdpServerConfig.def()
    }
}

impl Related<super::udp_client_connection::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UdpClientConnection.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
