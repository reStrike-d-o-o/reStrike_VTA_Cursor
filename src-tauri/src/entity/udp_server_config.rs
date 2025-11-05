use sea_orm::entity::prelude::*;

/// UDP server configuration used for PSS ingest.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "udp_server_config")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub port: i32,
    pub bind_address: String,
    pub network_interface_id: Option<i32>,
    pub enabled: bool,
    pub auto_start: bool,
    pub max_packet_size: i32,
    pub buffer_size: i32,
    pub timeout_ms: i32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::udp_server_session::Entity")]
    UdpServerSession,
}

impl Related<super::udp_server_session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UdpServerSession.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
