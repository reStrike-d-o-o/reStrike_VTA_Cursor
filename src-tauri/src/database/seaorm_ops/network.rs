use crate::{database::models::NetworkInterface, entity::network_interface};
use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, QueryOrder,
};

fn map_interface(model: network_interface::Model) -> NetworkInterface {
    NetworkInterface {
        id: Some(model.id as i64),
        name: model.name,
        address: model.address,
        netmask: model.netmask,
        broadcast: model.broadcast,
        is_loopback: model.is_loopback,
        is_active: model.is_active,
        is_recommended: model.is_recommended,
        speed_mbps: model.speed_mbps,
        mtu: model.mtu,
        mac_address: model.mac_address,
        interface_type: model.interface_type,
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    }
}

/// Fetch all known interfaces ordered by recommendation and active status.
pub async fn get_network_interfaces(
    conn: &DatabaseConnection,
) -> Result<Vec<NetworkInterface>, DbErr> {
    let rows = network_interface::Entity::find()
        .order_by_desc(network_interface::Column::IsRecommended)
        .order_by_desc(network_interface::Column::IsActive)
        .order_by_asc(network_interface::Column::Name)
        .all(conn)
        .await?;

    Ok(rows.into_iter().map(map_interface).collect())
}

/// Return the preferred interface flagged as active and recommended.
pub async fn get_recommended_interface(
    conn: &DatabaseConnection,
) -> Result<Option<NetworkInterface>, DbErr> {
    let record = network_interface::Entity::find()
        .filter(network_interface::Column::IsRecommended.eq(true))
        .filter(network_interface::Column::IsActive.eq(true))
        .order_by_desc(network_interface::Column::UpdatedAt)
        .one(conn)
        .await?;

    Ok(record.map(map_interface))
}

/// Insert or update an interface record and return its primary key.
pub async fn upsert_network_interface(
    conn: &DatabaseConnection,
    interface: &NetworkInterface,
) -> Result<i64, DbErr> {
    if let Some(id) = interface.id {
        let now = Utc::now();
        let Some(existing) = network_interface::Entity::find_by_id(id as i32)
            .one(conn)
            .await?
        else {
            return Err(DbErr::RecordNotFound(format!(
                "Network interface {id} not found"
            )));
        };

        let mut active: network_interface::ActiveModel = existing.into();
        active.name = Set(interface.name.clone());
        active.address = Set(interface.address.clone());
        active.netmask = Set(interface.netmask.clone());
        active.broadcast = Set(interface.broadcast.clone());
        active.is_loopback = Set(interface.is_loopback);
        active.is_active = Set(interface.is_active);
        active.is_recommended = Set(interface.is_recommended);
        active.speed_mbps = Set(interface.speed_mbps);
        active.mtu = Set(interface.mtu);
        active.mac_address = Set(interface.mac_address.clone());
        active.interface_type = Set(interface.interface_type.clone());
        active.updated_at = Set(now.naive_utc());
        active.update(conn).await?;
        Ok(id)
    } else {
        let created_at = interface.created_at.naive_utc();
        let updated_at = interface.updated_at.naive_utc();
        let active = network_interface::ActiveModel {
            name: Set(interface.name.clone()),
            address: Set(interface.address.clone()),
            netmask: Set(interface.netmask.clone()),
            broadcast: Set(interface.broadcast.clone()),
            is_loopback: Set(interface.is_loopback),
            is_active: Set(interface.is_active),
            is_recommended: Set(interface.is_recommended),
            speed_mbps: Set(interface.speed_mbps),
            mtu: Set(interface.mtu),
            mac_address: Set(interface.mac_address.clone()),
            interface_type: Set(interface.interface_type.clone()),
            created_at: Set(created_at),
            updated_at: Set(updated_at),
            ..Default::default()
        };

        let inserted = active.insert(conn).await?;
        Ok(inserted.id as i64)
    }
}
