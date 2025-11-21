use crate::core::app::App;
use anyhow;
use std::sync::Arc;
use tauri::State;
use crate::types::TauriError;

// ============================================================================
// Resource/Flag Commands
// ============================================================================

/// Get flag mappings data from the database
#[tauri::command]
pub async fn get_flag_mappings_data(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting flag mappings data");

    let conn = match app.database_plugin().get_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to get database connection: {}", e)
            }))
        }
    };

    // Check if flag_mappings table exists and get its data
    let table_exists: i32 = match conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='flag_mappings'",
        [],
        |row| row.get(0),
    ) {
        Ok(count) => count,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to check table existence: {}", e)
            }))
        }
    };

    if table_exists == 0 {
        return Ok(serde_json::json!({
            "success": false,
            "error": "flag_mappings table does not exist",
            "table_exists": false
        }));
    }

    // Get the data from flag_mappings table
    let mappings: Vec<serde_json::Value> = match conn.prepare(
        "SELECT id, pss_code, ioc_code, country_name, is_custom, created, updated FROM flag_mappings ORDER BY pss_code"
    ) {
        Ok(mut stmt) => {
            let mut mapping_data = Vec::new();
            let rows = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "pss_code": row.get::<_, Option<String>>(1)?,
                    "ioc_code": row.get::<_, Option<String>>(2)?,
                    "country_name": row.get::<_, Option<String>>(3)?,
                    "is_custom": row.get::<_, bool>(4)?,
                    "created": row.get::<_, Option<i64>>(5)?,
                    "updated": row.get::<_, Option<i64>>(6)?
                }))
            }).map_err(|e| TauriError::from(anyhow::anyhow!("Failed to query flag mappings: {e}")))?;

            for row in rows {
                let mapping = row.map_err(|e| TauriError::from(anyhow::anyhow!("Failed to get mapping data: {e}")))?;
                mapping_data.push(mapping);
            }
            mapping_data
        },
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to prepare flag mappings query: {}", e)
        }))
    };

    Ok(serde_json::json!({
        "success": true,
        "table_exists": true,
        "mappings": mappings,
        "count": mappings.len()
    }))
}

/// Scan SVG flags directory and populate the flags database table
#[tauri::command]
pub async fn scan_and_populate_flags(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Scanning and populating flags table");

    let conn = match app.database_plugin().get_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to get database connection: {}", e)
            }))
        }
    };

    // Path to the SVG flags directory (relative to project root)
    let flags_dir = std::path::Path::new("../ui/public/assets/flags/svg");

    if !flags_dir.exists() {
        return Ok(serde_json::json!({
            "success": false,
            "error": "Flags directory does not exist: ../ui/public/assets/flags/svg"
        }));
    }

    let mut processed_count = 0;
    let mut skipped_count = 0;
    let mut errors = Vec::new();

    // Read directory entries
    let entries = match std::fs::read_dir(flags_dir) {
        Ok(entries) => entries,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to read flags directory: {}", e)
            }))
        }
    };

    let current_time = chrono::Utc::now().to_rfc3339();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                errors.push(format!("Failed to read directory entry: {e}"));
                continue;
            }
        };

        let path = entry.path();

        // Only process SVG files
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("svg") {
            continue;
        }

        let filename = match path.file_name().and_then(|name| name.to_str()) {
            Some(name) => name,
            None => {
                errors.push(format!("Invalid filename for path: {}", path.display()));
                continue;
            }
        };

        // Skip report files
        if filename.contains("REPORT") || filename.contains("report") {
            skipped_count += 1;
            continue;
        }

        // Extract IOC code from filename (e.g., "USA.svg" -> "USA")
        let ioc_code = filename.trim_end_matches(".svg").to_uppercase();

        // Get file metadata
        let metadata = match std::fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(e) => {
                errors.push(format!("Failed to get metadata for {filename}: {e}"));
                continue;
            }
        };

        let file_size = metadata.len() as i64;
        let file_path = path.to_string_lossy().to_string();

        // Try to get country name from flag_mappings table
        let country_name: Option<String> = conn
            .query_row(
                "SELECT country_name FROM flag_mappings WHERE ioc_code = ? OR pss_code = ?",
                [&ioc_code, &ioc_code],
                |row| row.get(0),
            )
            .unwrap_or(None);

        let recognition_status = if country_name.is_some() {
            "recognized"
        } else {
            "pending"
        };
        let is_recognized = country_name.is_some();
        let recognition_confidence = if is_recognized { Some(1.0) } else { None };

        // Check if this flag already exists in the database
        let exists: i32 = conn.query_row(
            "SELECT COUNT(*) FROM flag WHERE filename = ? OR (ioc_code = ? AND ioc_code IS NOT NULL)",
            [filename, &ioc_code],
            |row| row.get(0),
        ).unwrap_or(0);

        if exists > 0 {
            skipped_count += 1;
            continue;
        }

        // Insert the flag into the database
        let result = conn.execute(
            "INSERT INTO flag (filename, ioc_code, country_name, recognition_status, recognition_confidence, upload_date, last_modified, file_size, file_path, is_recognized) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                filename,
                if ioc_code.is_empty() { None } else { Some(ioc_code.clone()) },
                country_name,
                recognition_status,
                recognition_confidence,
                &current_time,
                &current_time,
                file_size,
                file_path,
                is_recognized
            ],
        );

        match result {
            Ok(_) => {
                processed_count += 1;
                log::info!("Added flag to database: {filename} -> {ioc_code}");
            }
            Err(e) => {
                errors.push(format!("Failed to insert flag {filename}: {e}"));
            }
        }
    }

    log::info!(
        "Flag scanning completed: {} processed, {} skipped, {} errors",
        processed_count,
        skipped_count,
        errors.len()
    );

    Ok(serde_json::json!({
        "success": true,
        "processed_count": processed_count,
        "skipped_count": skipped_count,
        "errors": errors,
        "message": format!("Successfully processed {} flag files", processed_count)
    }))
}

/// Get all flags data from the database
#[tauri::command]
pub async fn get_flags_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting flags data");

    let conn = match app.database_plugin().get_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to get database connection: {}", e)
            }))
        }
    };

    // Get all flags from the database
    let flags: Vec<serde_json::Value> = match conn.prepare(
        "SELECT id, filename, ioc_code, country_name, recognition_status, recognition_confidence, upload_date, last_modified, file_size, file_path, is_recognized FROM flag ORDER BY filename"
    ) {
        Ok(mut stmt) => {
            let mut flag_data = Vec::new();
            let rows = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "filename": row.get::<_, String>(1)?,
                    "ioc_code": row.get::<_, Option<String>>(2)?,
                    "country_name": row.get::<_, Option<String>>(3)?,
                    "recognition_status": row.get::<_, String>(4)?,
                    "recognition_confidence": row.get::<_, Option<f64>>(5)?,
                    "upload_date": row.get::<_, String>(6)?,
                    "last_modified": row.get::<_, String>(7)?,
                    "file_size": row.get::<_, i64>(8)?,
                    "file_path": row.get::<_, String>(9)?,
                    "is_recognized": row.get::<_, bool>(10)?
                }))
            }).map_err(|e| TauriError::from(anyhow::anyhow!("Failed to query flags: {e}")))?;

            for row in rows {
                let flag = row.map_err(|e| TauriError::from(anyhow::anyhow!("Failed to get flag data: {e}")))?;
                flag_data.push(flag);
            }
            flag_data
        },
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to prepare flags query: {}", e)
        }))
    };


    let stats = match conn
        .prepare("SELECT recognition_status, COUNT(*) FROM flag GROUP BY recognition_status")
    {
        Ok(mut stmt) => {
            let mut stats_map = std::collections::HashMap::new();
            let rows = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                })
                .map_err(|e| {
                    TauriError::from(anyhow::anyhow!("Failed to query flag statistics: {e}"))
                })?;

            for row in rows {
                let (status, count) =
                    row.map_err(|e| TauriError::from(anyhow::anyhow!("Failed to get stats: {e}")))?;
                let key = status.to_lowercase();
                let entry = stats_map.entry(key).or_insert(0);
                *entry += count;
            }
            stats_map
        }
        Err(_) => std::collections::HashMap::new(),
    };

    Ok(serde_json::json!({
        "success": true,
        "flags": flags,
        "count": flags.len(),
        "statistics": {
            "total": flags.len(),
            "recognized": stats.get("recognized").copied().unwrap_or(0),
            "pending": stats.get("pending").copied().unwrap_or(0),
            "failed": stats.get("failed").or(stats.get("error")).copied().unwrap_or(0)
        }
    }))
}

/// Clear all entries from the flags table
#[tauri::command]
pub async fn clear_flags_table(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Clearing flags table");

    let conn = match app.database_plugin().get_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to get database connection: {}", e)
            }))
        }
    };

    match conn.execute("DELETE FROM flag", []) {
        Ok(deleted_count) => {
            log::info!("Cleared {deleted_count} entries from flags table");
            Ok(serde_json::json!({
                "success": true,
                "deleted_count": deleted_count,
                "message": format!("Cleared {} flag entries", deleted_count)
            }))
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to clear flags table: {}", e)
        })),
    }
}
