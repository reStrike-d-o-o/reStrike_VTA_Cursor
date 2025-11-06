use crate::database::models::ObsConnection as DbObsConnection;
use crate::entity::obs_connection;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, QueryOrder,
};

fn map_model(model: obs_connection::Model) -> DbObsConnection {
    DbObsConnection {
        id: Some(model.id as i64),
        name: model.name,
        host: model.host,
        port: u16::try_from(model.port).unwrap_or_default(),
        password: model.password,
        is_active: model.is_active,
        status: model.status,
        error: model.error,
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    }
}

/// Fetch all OBS connections sorted by name.
pub async fn list_connections(conn: &DatabaseConnection) -> Result<Vec<DbObsConnection>, DbErr> {
    let records = obs_connection::Entity::find()
        .order_by_asc(obs_connection::Column::Name)
        .all(conn)
        .await?;

    Ok(records.into_iter().map(map_model).collect())
}

/// Fetch only the OBS connections marked as active.
pub async fn list_active_connections(
    conn: &DatabaseConnection,
) -> Result<Vec<DbObsConnection>, DbErr> {
    let records = obs_connection::Entity::find()
        .filter(obs_connection::Column::IsActive.eq(true))
        .order_by_asc(obs_connection::Column::Name)
        .all(conn)
        .await?;

    Ok(records.into_iter().map(map_model).collect())
}

/// Look up an OBS connection by its unique name.
pub async fn find_connection_by_name(
    conn: &DatabaseConnection,
    name: &str,
) -> Result<Option<DbObsConnection>, DbErr> {
    let record = obs_connection::Entity::find()
        .filter(obs_connection::Column::Name.eq(name))
        .one(conn)
        .await?;

    Ok(record.map(map_model))
}

/// Insert or update an OBS connection using SeaORM.
pub async fn upsert_connection(
    conn: &DatabaseConnection,
    data: &DbObsConnection,
) -> Result<DbObsConnection, DbErr> {
    // Try to resolve the target row either by id or by unique name.
    let existing = if let Some(id) = data.id {
        obs_connection::Entity::find_by_id(id as i32)
            .one(conn)
            .await?
    } else {
        obs_connection::Entity::find()
            .filter(obs_connection::Column::Name.eq(data.name.clone()))
            .one(conn)
            .await?
    };

    let now = Utc::now().naive_utc();

    if let Some(model) = existing {
        let mut active: obs_connection::ActiveModel = model.into();
        active.name = Set(data.name.clone());
        active.host = Set(data.host.clone());
        active.port = Set(i32::from(data.port));
        active.password = Set(data.password.clone());
        active.is_active = Set(data.is_active);
        active.status = Set(data.status.clone());
        active.error = Set(data.error.clone());
        active.updated_at = Set(now);

        let saved = active.update(conn).await?;
        Ok(map_model(saved))
    } else {
        let created_at = data.created_at.naive_utc();
        let active = obs_connection::ActiveModel {
            name: Set(data.name.clone()),
            host: Set(data.host.clone()),
            port: Set(i32::from(data.port)),
            password: Set(data.password.clone()),
            is_active: Set(data.is_active),
            status: Set(data.status.clone()),
            error: Set(data.error.clone()),
            created_at: Set(created_at),
            updated_at: Set(now),
            ..Default::default()
        };

        let saved = active.insert(conn).await?;
        Ok(map_model(saved))
    }
}

/// Update the status/error fields for an OBS connection.
pub async fn update_connection_status(
    conn: &DatabaseConnection,
    name: &str,
    status: &str,
    error: Option<&str>,
) -> Result<DbObsConnection, DbErr> {
    let Some(model) = obs_connection::Entity::find()
        .filter(obs_connection::Column::Name.eq(name))
        .one(conn)
        .await?
    else {
        return Err(DbErr::RecordNotFound(format!(
            "OBS connection '{name}' not found"
        )));
    };

    let mut active: obs_connection::ActiveModel = model.into();
    active.status = Set(status.to_string());
    active.error = Set(error.map(|s| s.to_string()));
    active.updated_at = Set(Utc::now().naive_utc());

    let saved = active.update(conn).await?;
    Ok(map_model(saved))
}

/// Delete an OBS connection by name.
pub async fn delete_connection(conn: &DatabaseConnection, name: &str) -> Result<(), DbErr> {
    obs_connection::Entity::delete_many()
        .filter(obs_connection::Column::Name.eq(name))
        .exec(conn)
        .await?;
    Ok(())
}

/// Remove all OBS connections from the database.
pub async fn clear_connections(conn: &DatabaseConnection) -> Result<(), DbErr> {
    obs_connection::Entity::delete_many().exec(conn).await?;
    Ok(())
}
