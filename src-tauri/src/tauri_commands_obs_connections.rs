use crate::database::models::ObsConnection;
use chrono::Utc;
use std::sync::Arc;
use tauri::{command, Error as TauriError, State};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObsConnectionPayload {
    pub id: Option<i64>,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub is_active: bool,
    pub status: String,
    pub error: Option<String>,
}

#[command]
pub async fn obs_connections_get_all(
    app: State<'_, Arc<crate::App>>,
) -> Result<Vec<ObsConnection>, TauriError> {
    let plugin = app.database_plugin();

    let connections = plugin
        .list_obs_connections()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    Ok(connections)
}

#[command]
pub async fn obs_connections_get_active(
    app: State<'_, Arc<crate::App>>,
) -> Result<Vec<ObsConnection>, TauriError> {
    let plugin = app.database_plugin();

    let connections = plugin
        .list_active_obs_connections()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    Ok(connections)
}

#[command]
pub async fn obs_connections_save(
    app: State<'_, Arc<crate::App>>,
    connection: ObsConnectionPayload,
) -> Result<ObsConnection, TauriError> {
    let plugin = app.database_plugin();

    let obs_connection = ObsConnection {
        id: connection.id,
        name: connection.name,
        host: connection.host,
        port: connection.port,
        password: connection.password,
        is_active: connection.is_active,
        status: connection.status,
        error: connection.error,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let saved = plugin
        .save_obs_connection(&obs_connection)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    Ok(saved)
}

#[command]
pub async fn obs_connections_update_status(
    app: State<'_, Arc<crate::App>>,
    name: String,
    status: String,
    error: Option<String>,
) -> Result<(), TauriError> {
    let plugin = app.database_plugin();

    plugin
        .update_obs_connection_status(&name, &status, error.as_deref())
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    Ok(())
}

#[command]
pub async fn obs_connections_delete(
    app: State<'_, Arc<crate::App>>,
    name: String,
) -> Result<(), TauriError> {
    let plugin = app.database_plugin();

    plugin
        .delete_obs_connection(&name)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    Ok(())
}

#[command]
pub async fn obs_connections_clear_all(app: State<'_, Arc<crate::App>>) -> Result<(), TauriError> {
    let plugin = app.database_plugin();

    plugin
        .clear_obs_connections()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    Ok(())
}

#[command]
pub async fn obs_connections_sync_from_config(
    app: State<'_, Arc<crate::App>>,
) -> Result<Vec<ObsConnection>, TauriError> {
    let plugin = app.database_plugin();

    // Get connections from config manager
    let config_connections = app.config_manager().get_obs_connections().await;

    // Clear existing database connections
    plugin
        .clear_obs_connections()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    // Convert config connections to database connections
    let mut db_connections = Vec::new();
    for config_conn in config_connections {
        let obs_connection = ObsConnection {
            id: None,
            name: config_conn.name,
            host: config_conn.host,
            port: config_conn.port,
            password: config_conn.password,
            is_active: config_conn.enabled,
            status: "disconnected".to_string(),
            error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let saved = plugin
            .save_obs_connection(&obs_connection)
            .await
            .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

        db_connections.push(saved);
    }

    Ok(db_connections)
}
