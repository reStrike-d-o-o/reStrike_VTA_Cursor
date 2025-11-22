use crate::core::app::App;
use anyhow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use crate::types::TauriError;

// ============================================================================
// Database Commands
// ============================================================================

/// Purge all tournament PSS data from the database
#[tauri::command]
pub async fn db_purge_all_tournament_pss_data(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    let mut conn = app
        .database_plugin()
        .get_connection()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(format!("DB connection error: {e}"))))?;
    use crate::database::operations::PssUdpOperations as Ops;
    Ops::purge_all_tournament_pss_data(&mut conn)
        .map_err(|e| TauriError::from(anyhow::anyhow!(format!("purge failed: {e}"))))?;
    Ok(serde_json::json!({ "purged": true }))
}

// UI Settings Commands
#[tauri::command]
pub async fn db_initialize_ui_settings(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Initializing UI settings in database");

    match app.database_plugin().initialize_ui_settings().await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "UI settings initialized in database successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
pub async fn db_get_ui_setting(
    key: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting UI setting: {key}");

    match app.database_plugin().get_ui_setting(&key).await {
        Ok(value) => Ok(serde_json::json!({
            "success": true,
            "key": key,
            "value": value
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
pub async fn db_set_ui_setting(
    key: String,
    value: String,
    changed_by: String,
    change_reason: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Setting UI setting: {key} = {value}");

    match app
        .database_plugin()
        .set_ui_setting(&key, &value, &changed_by, change_reason.as_deref())
        .await
    {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("UI setting '{}' set successfully", key)
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
pub async fn db_get_all_ui_settings(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting all UI settings");

    match app.database_plugin().get_all_ui_settings().await {
        Ok(settings) => {
            let settings_map: HashMap<String, String> =
                settings.into_iter().collect();
            Ok(serde_json::json!({
                "success": true,
                "settings": settings_map
            }))
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
pub async fn db_get_database_info(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting database information");

    let is_accessible = app.database_plugin().is_accessible().await;
    let file_size = app.database_plugin().get_file_size();
    let database_path = app.database_plugin().get_database_path();
    let settings_count = app
        .database_plugin()
        .get_all_ui_settings()
        .await
        .map(|s| s.len())
        .unwrap_or(0);

    let file_size_value = match file_size {
        Ok(size) => serde_json::Value::Number(serde_json::Number::from(size)),
        Err(_) => serde_json::Value::Null,
    };

    let path_value = match database_path {
        Ok(path) => serde_json::Value::String(path),
        Err(_) => serde_json::Value::String("Unknown".to_string()),
    };

    // Get tables count
    let tables_count = match app.database_plugin().get_connection().await {
        Ok(conn) => match conn.prepare("SELECT name FROM sqlite_master WHERE type='table'") {
            Ok(mut stmt) => match stmt.query_map([], |row| row.get::<_, String>(0)) {
                Ok(rows) => {
                    let tables: Result<Vec<String>, _> = rows.collect();
                    match tables {
                        Ok(tables) => tables.len(),
                        Err(_) => 0,
                    }
                }
                Err(_) => 0,
            },
            Err(_) => 0,
        },
        Err(_) => 0,
    };

    let status = if is_accessible { "Active" } else { "Inactive" };
    let last_modified = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    Ok(serde_json::json!({
        "success": true,
        "path": path_value,
        "size": file_size_value,
        "tables": tables_count,
        "settings_count": settings_count,
        "last_modified": last_modified,
        "status": status,
        "is_accessible": is_accessible
    }))
}

// Database Migration Commands
#[tauri::command]
pub async fn migrate_json_to_database(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Starting JSON to database migration");

    match app.database_plugin().migrate_json_to_database().await {
        Ok(result) => Ok(serde_json::json!({
            "success": true,
            "result": {
                "total_settings": result.total_settings,
                "migrated_settings": result.migrated_settings,
                "failed_settings": result.failed_settings,
                "success_rate": result.success_rate(),
                "errors": result.errors
            }
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
pub async fn db_run_migrations(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    match app.database_plugin().run_migrations().await {
        Ok(_) => Ok(
            serde_json::json!({ "success": true, "message": "Database migrations ran successfully" }),
        ),
        Err(e) => Ok(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

// SQLite Backup Commands
#[derive(serde::Serialize)]
pub struct SqliteBackupInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: String,
}

#[tauri::command]
pub async fn db_create_sqlite_backup(
    app: State<'_, Arc<App>>,
    name: Option<String>,
) -> Result<serde_json::Value, TauriError> {
    let conn = app.database_plugin().get_database_connection();
    match conn.create_backup(name.as_deref()) {
        Ok(path) => {
            Ok(serde_json::json!({ "success": true, "path": path.to_string_lossy().to_string() }))
        }
        Err(e) => Ok(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

#[tauri::command]
pub async fn db_list_sqlite_backups(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    let conn = app.database_plugin().get_database_connection();
    match conn.list_backups() {
        Ok(paths) => {
            let mut items: Vec<SqliteBackupInfo> = Vec::new();
            for p in paths {
                let md = std::fs::metadata(&p).ok();
                let size = md.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified = md
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        chrono::DateTime::<chrono::Local>::from(t)
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string()
                    })
                    .unwrap_or_default();
                items.push(SqliteBackupInfo {
                    name: p
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string(),
                    path: p.to_string_lossy().to_string(),
                    size,
                    modified,
                });
            }
            Ok(serde_json::json!({ "success": true, "backups": items }))
        }
        Err(e) => Ok(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

#[tauri::command]
pub async fn db_restore_sqlite_backup(
    app: State<'_, Arc<App>>,
    backup_path: String,
) -> Result<serde_json::Value, TauriError> {
    let conn = app.database_plugin().get_database_connection();
    let pb = PathBuf::from(backup_path);
    match conn.restore_from_backup(&pb).await {
        Ok(_) => Ok(serde_json::json!({ "success": true })),
        Err(e) => Ok(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

#[tauri::command]
pub async fn get_migration_status(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting migration status");

    // Get database status
    let db_status = match app.database_plugin().get_migration_status().await {
        Ok(status) => status,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    };

    // Check for backup files in external directory
    let backup_dir = match dirs::data_dir() {
        Some(data_dir) => data_dir.join("reStrikeVTA").join("backups"),
        None => PathBuf::from("backups"),
    };
    let backup_files_exist = std::fs::read_dir(&backup_dir)
        .map(|entries| entries.filter_map(|entry| entry.ok()).count() > 0)
        .unwrap_or(false);

    // Get actual database settings count
    let db_settings_count = match app.database_plugin().get_all_ui_settings().await {
        Ok(settings) => settings.len(),
        Err(_) => 0,
    };

    // Get JSON settings count - simplified for now
    let json_settings_count = 0;

    Ok(serde_json::json!({
        "success": true,
        "status": {
            "database_enabled": db_status.database_enabled,
            "json_fallback_enabled": db_status.json_fallback_enabled,
            "migration_completed": db_status.migration_completed,
            "last_migration": db_status.last_migration,
            "settings_count": db_settings_count,
            "backup_created": backup_files_exist,
            "json_settings_count": json_settings_count,
            "database_settings_count": db_settings_count
        }
    }))
}

#[tauri::command]
pub async fn enable_database_mode(
    app: State<'_, Arc<App>>,
    enabled: bool,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Setting database mode to: {enabled}");

    match app.database_plugin().set_database_mode(enabled).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("Database mode {}", if enabled { "enabled" } else { "disabled" })
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
pub async fn get_database_preview(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting database preview");

    // Get all UI settings from database
    let db_settings = match app.database_plugin().get_all_ui_settings().await {
        Ok(settings) => settings,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to get database settings: {}", e)
            }))
        }
    };

    // Get JSON settings for comparison - simplified for now
    let json_settings: HashMap<String, String> = HashMap::new();

    // Convert settings to preview format
    let db_preview: Vec<serde_json::Value> = db_settings
        .iter()
        .map(|(key, value)| {
            serde_json::json!({
                "key": key,
                "value": value,
                "source": "database"
            })
        })
        .collect();

    let json_preview: Vec<serde_json::Value> = json_settings
        .iter()
        .map(|(key, value)| {
            serde_json::json!({
                "key": key,
                "value": value,
                "source": "json"
            })
        })
        .collect();

    Ok(serde_json::json!({
        "success": true,
        "database_settings": db_preview,
        "json_settings": json_preview,
        "database_count": db_settings.len(),
        "json_count": json_settings.len()
    }))
}

#[tauri::command]
pub async fn get_database_tables(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting database tables");

    let conn = match app.database_plugin().get_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to get database connection: {}", e)
            }))
        }
    };

    // Query to get all table names
    let tables: Vec<String> = match conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
    {
        Ok(mut stmt) => {
            let mut table_names = Vec::new();
            let rows = stmt
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to query tables: {e}")))?;

            for row in rows {
                let table_name = row.map_err(|e| {
                    TauriError::from(anyhow::anyhow!("Failed to get table name: {e}"))
                })?;
                table_names.push(table_name);
            }
            table_names
        }
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to prepare table query: {}", e)
            }))
        }
    };

    Ok(serde_json::json!({
        "success": true,
        "tables": tables
    }))
}

#[tauri::command]
pub async fn get_table_data(
    app: State<'_, Arc<App>>,
    table_name: String,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting data for table: {table_name}");

    let conn = match app.database_plugin().get_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to get database connection: {}", e)
            }))
        }
    };

    // First, get the table schema to understand the columns
    let schema_query = format!("PRAGMA table_info({table_name})");
    let columns: Vec<serde_json::Value> = match conn.prepare(&schema_query) {
        Ok(mut stmt) => {
            let mut column_info = Vec::new();
            let rows = stmt
                .query_map([], |row| {
                    Ok(serde_json::json!({
                        "name": row.get::<_, String>(1)?,
                        "type": row.get::<_, String>(2)?,
                        "not_null": row.get::<_, i32>(3)? == 1,
                        "primary_key": row.get::<_, i32>(5)? == 1
                    }))
                })
                .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to query schema: {e}")))?;

            for row in rows {
                let column = row.map_err(|e| {
                    TauriError::from(anyhow::anyhow!("Failed to get column info: {e}"))
                })?;
                column_info.push(column);
            }
            column_info
        }
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to prepare schema query: {}", e)
            }))
        }
    };

    // Get the data from the table (no limit to show all rows)
    let data_query = format!("SELECT * FROM {table_name}");
    let rows: Vec<serde_json::Value> = match conn.prepare(&data_query) {
        Ok(mut stmt) => {
            let mut table_data = Vec::new();
            let rows = stmt
                .query_map([], |row| {
                    let mut row_data = serde_json::Map::new();
                    for (i, column) in columns.iter().enumerate() {
                        let column_name = column["name"].as_str().unwrap_or("unknown");
                        let value = match row.get::<_, rusqlite::types::Value>(i) {
                            Ok(val) => match val {
                                rusqlite::types::Value::Null => serde_json::Value::Null,
                                rusqlite::types::Value::Integer(i) => {
                                    serde_json::Value::Number(serde_json::Number::from(i))
                                }
                                rusqlite::types::Value::Real(f) => serde_json::Value::Number(
                                    serde_json::Number::from_f64(f)
                                        .unwrap_or(serde_json::Number::from(0)),
                                ),
                                rusqlite::types::Value::Text(s) => serde_json::Value::String(s),
                                rusqlite::types::Value::Blob(b) => {
                                    serde_json::Value::String(format!("[BLOB: {} bytes]", b.len()))
                                }
                            },
                            Err(_) => serde_json::Value::Null,
                        };
                        row_data.insert(column_name.to_string(), value);
                    }
                    Ok(serde_json::Value::Object(row_data))
                })
                .map_err(|e| {
                    TauriError::from(anyhow::anyhow!("Failed to query table data: {e}"))
                })?;

            for row in rows {
                let row_data = row.map_err(|e| {
                    TauriError::from(anyhow::anyhow!("Failed to get row data: {e}"))
                })?;
                table_data.push(row_data);
            }
            table_data
        }
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to prepare data query: {}", e)
            }))
        }
    };

    Ok(serde_json::json!({
        "success": true,
        "table_name": table_name,
        "columns": columns,
        "rows": rows,
        "row_count": rows.len()
    }))
}

/// Store PSS event command (from UI/frontend)
// store_pss_event_cmd moved to commands/pss.rs

#[tauri::command]
pub async fn validate_tournament_pss_integrity(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    let conn = app
        .database_plugin()
        .get_connection()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(format!("DB connection error: {e}"))))?;
    let conn_ref = &*conn;
    // Counts
    let counts = |sql: &str| -> i64 {
        conn_ref
            .query_row(sql, [], |r| r.get::<_, i64>(0))
            .unwrap_or(0)
    };
    let c_tournaments = counts("SELECT COUNT(*) FROM tournaments");
    let c_days = counts("SELECT COUNT(*) FROM tournament_days");
    let c_matches = counts("SELECT COUNT(*) FROM pss_matches");
    let c_events = counts("SELECT COUNT(*) FROM pss_events");
    let c_vids = counts("SELECT COUNT(*) FROM recorded_videos");
    let c_links = counts("SELECT COUNT(*) FROM recorded_video_events");
    // Orphans
    let orphan_matches = counts("SELECT COUNT(*) FROM pss_matches m LEFT JOIN tournaments t ON t.id = m.tournament_id WHERE m.tournament_id IS NOT NULL AND t.id IS NULL");
    let orphan_events = counts("SELECT COUNT(*) FROM pss_events e LEFT JOIN pss_matches m ON m.id = e.match_id WHERE e.match_id IS NOT NULL AND m.id IS NULL");
    let orphan_vids = counts("SELECT COUNT(*) FROM recorded_videos rv LEFT JOIN pss_matches m ON m.id = rv.match_id WHERE rv.match_id IS NOT NULL AND m.id IS NULL");
    // Missing context
    let events_missing_ctx = counts("SELECT COUNT(*) FROM pss_events WHERE tournament_id IS NULL");
    let matches_missing_ctx =
        counts("SELECT COUNT(*) FROM pss_matches WHERE tournament_id IS NULL");
    Ok(serde_json::json!({
        "counts": {"tournaments": c_tournaments, "days": c_days, "matches": c_matches, "events": c_events, "videos": c_vids, "video_links": c_links},
        "orphans": {"matches": orphan_matches, "events": orphan_events, "videos": orphan_vids},
        "missing_context": {"events": events_missing_ctx, "matches": matches_missing_ctx}
    }))
}

// Database optimization commands
#[tauri::command]
pub async fn database_run_vacuum(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Running database VACUUM operation");

    let db_conn = app.database_plugin().get_database_connection();
    let mut maintenance = crate::database::DatabaseMaintenance::new_default();

    match maintenance.run_vacuum(&db_conn).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Database VACUUM completed successfully"
        })),
        Err(e) => {
            log::error!("Database VACUUM failed: {e}");
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

#[tauri::command]
pub async fn database_run_integrity_check(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Running database integrity check");

    let db_conn = app.database_plugin().get_database_connection();
    let mut maintenance = crate::database::DatabaseMaintenance::new_default();

    match maintenance.run_integrity_check(&db_conn).await {
        Ok(integrity_ok) => {
            if integrity_ok {
                Ok(serde_json::json!({
                    "success": true,
                    "message": "Database integrity check passed",
                    "integrity_ok": true
                }))
            } else {
                Ok(serde_json::json!({
                    "success": false,
                    "message": "Database integrity check failed",
                    "integrity_ok": false
                }))
            }
        }
        Err(e) => {
            log::error!("Database integrity check error: {e}");
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

#[tauri::command]
pub async fn database_run_analyze(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Running database ANALYZE operation");

    let db_conn = app.database_plugin().get_database_connection();
    let mut maintenance = crate::database::DatabaseMaintenance::new_default();

    match maintenance.run_analyze(&db_conn).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Database ANALYZE completed successfully"
        })),
        Err(e) => {
            log::error!("Database ANALYZE failed: {e}");
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

#[tauri::command]
pub async fn database_run_optimize(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Running database OPTIMIZE operation");

    let db_conn = app.database_plugin().get_database_connection();
    let mut maintenance = crate::database::DatabaseMaintenance::new_default();

    match maintenance.run_optimize(&db_conn).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Database OPTIMIZE completed successfully"
        })),
        Err(e) => {
            log::error!("Database OPTIMIZE failed: {e}");
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

#[tauri::command]
pub async fn database_run_full_maintenance(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Running full database maintenance");

    let db_conn = app.database_plugin().get_database_connection();
    let mut maintenance = crate::database::DatabaseMaintenance::new_default();

    match maintenance.run_full_maintenance(&db_conn).await {
        Ok(result) => Ok(serde_json::json!({
            "success": true,
            "message": "Full database maintenance completed",
            "result": {
                "integrity_check_passed": result.integrity_check_passed,
                "analyze_success": result.analyze_success,
                "optimize_success": result.optimize_success,
                "vacuum_success": result.vacuum_success,
                "total_duration_secs": result.total_duration.as_secs()
            }
        })),
        Err(e) => {
            log::error!("Full database maintenance failed: {e}");
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

#[tauri::command]
pub async fn database_get_maintenance_status(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting database maintenance status");

    let maintenance = crate::database::DatabaseMaintenance::new_default();
    let needed = maintenance.check_maintenance_needed();
    let stats = maintenance.get_statistics();
    let config = maintenance.get_config();

    Ok(serde_json::json!({
        "success": true,
        "maintenance_needed": {
            "vacuum_needed": needed.vacuum_needed,
            "integrity_check_needed": needed.integrity_check_needed,
            "analyze_needed": needed.analyze_needed,
            "optimize_needed": needed.optimize_needed,
            "any_needed": needed.any_needed()
        },
        "statistics": {
            "last_vacuum": stats.last_vacuum,
            "last_integrity_check": stats.last_integrity_check,
            "last_analyze": stats.last_analyze,
            "last_optimize": stats.last_optimize,
            "vacuum_count": stats.vacuum_count,
            "integrity_check_count": stats.integrity_check_count,
            "analyze_count": stats.analyze_count,
            "optimize_count": stats.optimize_count,
            "total_maintenance_time_secs": stats.total_maintenance_time_secs
        },
        "config": {
            "vacuum_interval_secs": config.vacuum_interval.as_secs(),
            "integrity_check_interval_secs": config.integrity_check_interval.as_secs(),
            "analyze_interval_secs": config.analyze_interval.as_secs(),
            "optimize_interval_secs": config.optimize_interval.as_secs(),
            "max_vacuum_time_secs": config.max_vacuum_time.as_secs(),
            "backup_before_maintenance": config.backup_before_maintenance
        }
    }))
}

/// Get comprehensive event statistics with status breakdown
#[tauri::command]
pub async fn get_comprehensive_event_statistics(
    session_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting comprehensive event statistics for session {session_id}");

    match app
        .database_plugin()
        .get_comprehensive_event_statistics(session_id)
        .await
    {
        Ok(stats) => Ok(stats),
        Err(e) => {
            log::error!("Failed to get comprehensive event statistics: {e}");
            Err(TauriError::from(anyhow::anyhow!(
                "Failed to get event statistics: {e}"
            )))
        }
    }
}

/// Get events by recognition status
#[tauri::command]
pub async fn get_events_by_status(
    session_id: i64,
    recognition_status: String,
    limit: Option<i64>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting events by status: {recognition_status} for session {session_id}");

    match app
        .database_plugin()
        .get_events_by_status(session_id, &recognition_status, limit)
        .await
    {
        Ok(events) => {
            let events_json: Vec<serde_json::Value> = events
                .into_iter()
                .map(|event| serde_json::to_value(event).unwrap_or_default())
                .collect();
            Ok(serde_json::json!({
                "success": true,
                "events": events_json,
                "count": events_json.len()
            }))
        }
        Err(e) => {
            log::error!("Failed to get events by status: {e}");
            Err(TauriError::from(anyhow::anyhow!(
                "Failed to get events by status: {e}"
            )))
        }
    }
}

/// Get unknown events for analysis
#[tauri::command]
pub async fn get_unknown_events(
    session_id: Option<i64>,
    limit: Option<i64>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting unknown events");

    match app
        .database_plugin()
        .get_unknown_events(session_id, limit)
        .await
    {
        Ok(events) => {
            let events_json: Vec<serde_json::Value> = events
                .into_iter()
                .map(|event| serde_json::to_value(event).unwrap_or_default())
                .collect();
            Ok(serde_json::json!({
                "success": true,
                "events": events_json,
                "count": events_json.len()
            }))
        }
        Err(e) => {
            log::error!("Failed to get unknown events: {e}");
            Err(TauriError::from(anyhow::anyhow!(
                "Failed to get unknown events: {e}"
            )))
        }
    }
}
