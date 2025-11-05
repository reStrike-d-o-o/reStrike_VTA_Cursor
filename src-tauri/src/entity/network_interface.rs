use sea_orm::entity::prelude::*;

/// Physical or virtual network interface available to the desktop app for UDP bind.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "network_interfaces")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub address: String,
    pub netmask: Option<String>,
    pub broadcast: Option<String>,
    pub is_loopback: bool,
    pub is_active: bool,
    pub is_recommended: bool,
    pub speed_mbps: Option<i32>,
    pub mtu: Option<i32>,
    pub mac_address: Option<String>,
    pub interface_type: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub created: Option<i64>,
    pub updated: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
