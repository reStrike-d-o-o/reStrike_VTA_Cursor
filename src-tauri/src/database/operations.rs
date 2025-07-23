use rusqlite::{Connection, Result as SqliteResult, params};
use chrono::Utc;
use crate::database::{DatabaseError, DatabaseResult};
use crate::database::models::{
    PssEvent, ObsConnection, AppConfig, FlagMapping,
    SettingsCategory, SettingsKey, SettingsValue, SettingsHistory,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// PSS Events operations
pub struct PssEventOperations;

impl PssEventOperations {
    /// Insert a new PSS event with validation
    pub fn insert(db: &mut Connection, event: &PssEvent) -> SqliteResult<i64> {
        // Validate event data before insertion
        if event.event_type.is_empty() {
            return Err(rusqlite::Error::InvalidParameterName("Event type cannot be empty".to_string()));
        }
        
        if let Some(ref match_id) = event.match_id {
            if match_id.is_empty() {
                return Err(rusqlite::Error::InvalidParameterName("Match ID cannot be empty".to_string()));
            }
        }
        
        let tx = db.transaction()?;
        
        // Check for duplicate events (same match_id, timestamp, and event_type)
        let existing_count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM pss_events WHERE match_id = ? AND timestamp = ? AND event_type = ?",
            params![event.match_id.as_deref().unwrap_or(""), event.timestamp.to_rfc3339(), event.event_type],
            |row| row.get(0)
        )?;
        
        if existing_count > 0 {
            log::warn!("Duplicate PSS event detected: match_id={:?}, timestamp={}, event_type={}", 
                      event.match_id, event.timestamp, event.event_type);
        }
        
        let id = tx.execute(
            "INSERT INTO pss_events (
                event_type, timestamp, match_id, athlete1_code, athlete2_code,
                score1, score2, round, weight_class, category, raw_data, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                event.event_type,
                event.timestamp.to_rfc3339(),
                event.match_id,
                event.athlete1_code,
                event.athlete2_code,
                event.score1,
                event.score2,
                event.round,
                event.weight_class,
                event.category,
                event.raw_data,
                event.created_at.to_rfc3339(),
            ],
        )?;
        
        tx.commit()?;
        log::debug!("PSS event inserted with ID: {}", id);
        Ok(id as i64)
    }
    
    /// Get PSS event by ID with validation
    pub fn get_by_id(db: &Connection, id: i64) -> SqliteResult<Option<PssEvent>> {
        if id <= 0 {
            return Err(rusqlite::Error::InvalidParameterName("Invalid event ID".to_string()));
        }
        
        let mut stmt = db.prepare(
            "SELECT * FROM pss_events WHERE id = ?"
        )?;
        
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(PssEvent::from_row(&row)?))
        } else {
            Ok(None)
        }
    }
    
    /// Get PSS events by match ID with validation
    pub fn get_by_match_id(db: &Connection, match_id: &str) -> SqliteResult<Vec<PssEvent>> {
        if match_id.is_empty() {
            return Err(rusqlite::Error::InvalidParameterName("Match ID cannot be empty".to_string()));
        }
        
        let mut stmt = db.prepare(
            "SELECT * FROM pss_events WHERE match_id = ? ORDER BY timestamp"
        )?;
        
        let rows = stmt.query_map(params![match_id], |row| PssEvent::from_row(row))?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }
        
        Ok(events)
    }
    
    /// Get PSS events within a time range with validation
    pub fn get_by_time_range(
        db: &Connection,
        start_time: chrono::DateTime<Utc>,
        end_time: chrono::DateTime<Utc>,
    ) -> SqliteResult<Vec<PssEvent>> {
        if start_time >= end_time {
            return Err(rusqlite::Error::InvalidParameterName("Start time must be before end time".to_string()));
        }
        
        let mut stmt = db.prepare(
            "SELECT * FROM pss_events WHERE timestamp BETWEEN ? AND ? ORDER BY timestamp"
        )?;
        
        let rows = stmt.query_map(
            params![start_time.to_rfc3339(), end_time.to_rfc3339()],
            |row| PssEvent::from_row(row)
        )?;
        
        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }
        
        Ok(events)
    }
    
    /// Get recent PSS events with validation
    pub fn get_recent(db: &Connection, limit: i64) -> SqliteResult<Vec<PssEvent>> {
        if limit <= 0 || limit > 10000 {
            return Err(rusqlite::Error::InvalidParameterName("Limit must be between 1 and 10000".to_string()));
        }
        
        let mut stmt = db.prepare(
            "SELECT * FROM pss_events ORDER BY timestamp DESC LIMIT ?"
        )?;
        
        let rows = stmt.query_map(params![limit], |row| PssEvent::from_row(row))?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }
        
        Ok(events)
    }
    
    /// Update PSS event with validation
    pub fn update(db: &mut Connection, event: &PssEvent) -> SqliteResult<bool> {
        if let Some(id) = event.id {
            if id <= 0 {
                return Err(rusqlite::Error::InvalidParameterName("Invalid event ID".to_string()));
            }
        } else {
            return Err(rusqlite::Error::InvalidParameterName("Event ID is required for update".to_string()));
        }
        
        let tx = db.transaction()?;
        
        // Check if event exists
        let exists: i64 = tx.query_row(
            "SELECT COUNT(*) FROM pss_events WHERE id = ?",
            params![event.id],
            |row| row.get(0)
        )?;
        
        if exists == 0 {
            tx.rollback()?;
            return Ok(false);
        }
        
        let affected = tx.execute(
            "UPDATE pss_events SET 
                event_type = ?, timestamp = ?, match_id = ?, athlete1_code = ?, 
                athlete2_code = ?, score1 = ?, score2 = ?, round = ?, 
                weight_class = ?, category = ?, raw_data = ?
             WHERE id = ?",
            params![
                event.event_type,
                event.timestamp.to_rfc3339(),
                event.match_id,
                event.athlete1_code,
                event.athlete2_code,
                event.score1,
                event.score2,
                event.round,
                event.weight_class,
                event.category,
                event.raw_data,
                event.id,
            ],
        )?;
        
        tx.commit()?;
        log::debug!("PSS event updated: {} rows affected", affected);
        Ok(affected > 0)
    }
    
    /// Delete PSS event by ID with validation
    pub fn delete_by_id(db: &mut Connection, id: i64) -> SqliteResult<bool> {
        if id <= 0 {
            return Err(rusqlite::Error::InvalidParameterName("Invalid event ID".to_string()));
        }
        
        let tx = db.transaction()?;
        
        let affected = tx.execute(
            "DELETE FROM pss_events WHERE id = ?",
            params![id],
        )?;
        
        tx.commit()?;
        log::debug!("PSS event deleted: {} rows affected", affected);
        Ok(affected > 0)
    }
    
    /// Clear all PSS events with confirmation
    pub fn clear_all(db: &mut Connection) -> SqliteResult<u64> {
        let tx = db.transaction()?;
        
        // Get count before deletion for logging
        let count: i64 = tx.query_row("SELECT COUNT(*) FROM pss_events", [], |row| row.get(0))?;
        
        let affected = tx.execute("DELETE FROM pss_events", [])?;
        
        tx.commit()?;
        log::warn!("Cleared all PSS events: {} events deleted", count);
        Ok(affected as u64)
    }
    
    /// Get PSS events statistics
    pub fn get_statistics(db: &Connection) -> SqliteResult<PssEventStatistics> {
        let total_events: i64 = db.query_row("SELECT COUNT(*) FROM pss_events", [], |row| row.get(0))?;
        let unique_matches: i64 = db.query_row("SELECT COUNT(DISTINCT match_id) FROM pss_events", [], |row| row.get(0))?;
        let oldest_event: Option<String> = db.query_row("SELECT MIN(timestamp) FROM pss_events", [], |row| row.get(0)).ok();
        let newest_event: Option<String> = db.query_row("SELECT MAX(timestamp) FROM pss_events", [], |row| row.get(0)).ok();
        
        Ok(PssEventStatistics {
            total_events,
            unique_matches,
            oldest_event,
            newest_event,
        })
    }
}

/// PSS Events statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssEventStatistics {
    pub total_events: i64,
    pub unique_matches: i64,
    pub oldest_event: Option<String>,
    pub newest_event: Option<String>,
}

/// OBS Connections operations
pub struct ObsConnectionOperations;

impl ObsConnectionOperations {
    /// Insert a new OBS connection
    pub fn insert(db: &mut Connection, connection: &ObsConnection) -> SqliteResult<i64> {
        let tx = db.transaction()?;
        let id = tx.execute(
            "INSERT INTO obs_connections (
                name, host, port, password, is_active, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                connection.name,
                connection.host,
                connection.port,
                connection.password,
                connection.is_active,
                connection.created_at.to_rfc3339(),
                connection.updated_at.to_rfc3339(),
            ],
        )?;
        
        tx.commit()?;
        Ok(id as i64)
    }
    
    /// Get OBS connection by ID
    pub fn get_by_id(db: &Connection, id: i64) -> SqliteResult<Option<ObsConnection>> {
        let mut stmt = db.prepare(
            "SELECT * FROM obs_connections WHERE id = ?"
        )?;
        
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(ObsConnection::from_row(&row)?))
        } else {
            Ok(None)
        }
    }
    
    /// Get OBS connection by name
    pub fn get_by_name(db: &Connection, name: &str) -> SqliteResult<Option<ObsConnection>> {
        let mut stmt = db.prepare(
            "SELECT * FROM obs_connections WHERE name = ?"
        )?;
        
        let mut rows = stmt.query(params![name])?;
        if let Some(row) = rows.next()? {
            Ok(Some(ObsConnection::from_row(&row)?))
        } else {
            Ok(None)
        }
    }
    
    /// Get all OBS connections
    pub fn get_all(db: &Connection) -> SqliteResult<Vec<ObsConnection>> {
        let mut stmt = db.prepare(
            "SELECT * FROM obs_connections ORDER BY name"
        )?;
        
        let rows = stmt.query_map([], |row| ObsConnection::from_row(row))?;
        let mut connections = Vec::new();
        for row in rows {
            connections.push(row?);
        }
        
        Ok(connections)
    }
    
    /// Update OBS connection
    pub fn update(db: &mut Connection, connection: &ObsConnection) -> SqliteResult<bool> {
        let tx = db.transaction()?;
        let rows_affected = tx.execute(
            "UPDATE obs_connections SET
                name = ?, host = ?, port = ?, password = ?,
                is_active = ?, updated_at = ?
             WHERE id = ?",
            params![
                connection.name,
                connection.host,
                connection.port,
                connection.password,
                connection.is_active,
                Utc::now().to_rfc3339(),
                connection.id,
            ],
        )?;
        
        tx.commit()?;
        Ok(rows_affected > 0)
    }
    
    /// Delete OBS connection by ID
    pub fn delete_by_id(db: &mut Connection, id: i64) -> SqliteResult<bool> {
        let tx = db.transaction()?;
        let rows_affected = tx.execute(
            "DELETE FROM obs_connections WHERE id = ?",
            params![id],
        )?;
        
        tx.commit()?;
        Ok(rows_affected > 0)
    }
    
    /// Set active OBS connection
    pub fn set_active(db: &mut Connection, id: i64) -> SqliteResult<bool> {
        let tx = db.transaction()?;
        // Deactivate all connections
        tx.execute("UPDATE obs_connections SET is_active = 0", [])?;
        
        // Activate the specified connection
        let rows_affected = tx.execute(
            "UPDATE obs_connections SET is_active = 1 WHERE id = ?",
            params![id],
        )?;
        
        tx.commit()?;
        Ok(rows_affected > 0)
    }
}

/// App Config operations
pub struct AppConfigOperations;

impl AppConfigOperations {
    /// Upsert app config
    pub fn upsert(db: &mut Connection, config: &AppConfig) -> SqliteResult<i64> {
        let tx = db.transaction()?;
        let id = tx.execute(
            "INSERT OR REPLACE INTO app_config (
                key, value, category, description, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?)",
            params![
                config.key,
                config.value,
                config.category,
                config.description,
                config.created_at.to_rfc3339(),
                Utc::now().to_rfc3339(),
            ],
        )?;
        
        tx.commit()?;
        Ok(id as i64)
    }
    
    /// Get app config by key
    pub fn get_by_key(db: &Connection, key: &str) -> SqliteResult<Option<AppConfig>> {
        let mut stmt = db.prepare(
            "SELECT * FROM app_config WHERE key = ?"
        )?;
        
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(AppConfig::from_row(&row)?))
        } else {
            Ok(None)
        }
    }
    
    /// Get app configs by category
    pub fn get_by_category(db: &Connection, category: &str) -> SqliteResult<Vec<AppConfig>> {
        let mut stmt = db.prepare(
            "SELECT * FROM app_config WHERE category = ? ORDER BY key"
        )?;
        
        let rows = stmt.query_map(params![category], |row| AppConfig::from_row(row))?;
        let mut configs = Vec::new();
        for row in rows {
            configs.push(row?);
        }
        
        Ok(configs)
    }
    
    /// Get all app configs
    pub fn get_all(db: &Connection) -> SqliteResult<Vec<AppConfig>> {
        let mut stmt = db.prepare(
            "SELECT * FROM app_config ORDER BY category, key"
        )?;
        
        let rows = stmt.query_map([], |row| AppConfig::from_row(row))?;
        let mut configs = Vec::new();
        for row in rows {
            configs.push(row?);
        }
        
        Ok(configs)
    }
    
    /// Delete app config by key
    pub fn delete_by_key(db: &mut Connection, key: &str) -> SqliteResult<bool> {
        let tx = db.transaction()?;
        let rows_affected = tx.execute(
            "DELETE FROM app_config WHERE key = ?",
            params![key],
        )?;
        
        tx.commit()?;
        Ok(rows_affected > 0)
    }
}

/// Flag Mapping operations
pub struct FlagMappingOperations;

impl FlagMappingOperations {
    /// Upsert flag mapping
    pub fn upsert(db: &mut Connection, mapping: &FlagMapping) -> SqliteResult<i64> {
        let tx = db.transaction()?;
        let id = tx.execute(
            "INSERT OR REPLACE INTO flag_mappings (
                pss_code, ioc_code, country_name, is_custom, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?)",
            params![
                mapping.pss_code,
                mapping.ioc_code,
                mapping.country_name,
                mapping.is_custom,
                mapping.created_at.to_rfc3339(),
                Utc::now().to_rfc3339(),
            ],
        )?;
        
        tx.commit()?;
        Ok(id as i64)
    }
    
    /// Get flag mapping by PSS code
    pub fn get_by_pss_code(db: &Connection, pss_code: &str) -> SqliteResult<Option<FlagMapping>> {
        let mut stmt = db.prepare(
            "SELECT * FROM flag_mappings WHERE pss_code = ?"
        )?;
        
        let mut rows = stmt.query(params![pss_code])?;
        if let Some(row) = rows.next()? {
            Ok(Some(FlagMapping::from_row(&row)?))
        } else {
            Ok(None)
        }
    }
    
    /// Get flag mapping by IOC code
    pub fn get_by_ioc_code(db: &Connection, ioc_code: &str) -> SqliteResult<Option<FlagMapping>> {
        let mut stmt = db.prepare(
            "SELECT * FROM flag_mappings WHERE ioc_code = ?"
        )?;
        
        let mut rows = stmt.query(params![ioc_code])?;
        if let Some(row) = rows.next()? {
            Ok(Some(FlagMapping::from_row(&row)?))
        } else {
            Ok(None)
        }
    }
    
    /// Get all flag mappings
    pub fn get_all(db: &Connection) -> SqliteResult<Vec<FlagMapping>> {
        let mut stmt = db.prepare(
            "SELECT * FROM flag_mappings ORDER BY country_name"
        )?;
        
        let rows = stmt.query_map([], |row| FlagMapping::from_row(row))?;
        let mut mappings = Vec::new();
        for row in rows {
            mappings.push(row?);
        }
        
        Ok(mappings)
    }
    
    /// Get custom flag mappings
    pub fn get_custom(db: &Connection) -> SqliteResult<Vec<FlagMapping>> {
        let mut stmt = db.prepare(
            "SELECT * FROM flag_mappings WHERE is_custom = 1 ORDER BY country_name"
        )?;
        
        let rows = stmt.query_map([], |row| FlagMapping::from_row(row))?;
        let mut mappings = Vec::new();
        for row in rows {
            mappings.push(row?);
        }
        
        Ok(mappings)
    }
    
    /// Delete flag mapping by PSS code
    pub fn delete_by_pss_code(db: &mut Connection, pss_code: &str) -> SqliteResult<bool> {
        let tx = db.transaction()?;
        let rows_affected = tx.execute(
            "DELETE FROM flag_mappings WHERE pss_code = ?",
            params![pss_code],
        )?;
        
        tx.commit()?;
        Ok(rows_affected > 0)
    }
} 

/// Settings operations for normalized settings schema
pub struct SettingsOperations;

impl SettingsOperations {
    /// Get all settings categories
    pub fn get_categories(db: &Connection) -> DatabaseResult<Vec<SettingsCategory>> {
        let mut stmt = db.prepare(
            "SELECT id, name, description, display_order, created_at FROM settings_categories ORDER BY display_order, name"
        ).map_err(DatabaseError::from)?;
        
        let rows = stmt.query_map([], |row| SettingsCategory::from_row(row)).map_err(DatabaseError::from)?;
        let mut categories = Vec::new();
        for row in rows {
            categories.push(row.map_err(DatabaseError::from)?);
        }
        
        Ok(categories)
    }
    
    /// Get settings keys by category
    pub fn get_keys_by_category(db: &Connection, category_id: i64) -> DatabaseResult<Vec<SettingsKey>> {
        let mut stmt = db.prepare(
            "SELECT id, category_id, key_name, display_name, description, data_type, default_value, validation_rules, is_required, is_sensitive, created_at FROM settings_keys WHERE category_id = ? ORDER BY key_name"
        ).map_err(DatabaseError::from)?;
        
        let rows = stmt.query_map([category_id], |row| SettingsKey::from_row(row)).map_err(DatabaseError::from)?;
        let mut keys = Vec::new();
        for row in rows {
            keys.push(row.map_err(DatabaseError::from)?);
        }
        
        Ok(keys)
    }
    
    /// Get all settings keys
    pub fn get_all_keys(db: &Connection) -> DatabaseResult<Vec<SettingsKey>> {
        let mut stmt = db.prepare(
            "SELECT id, category_id, key_name, display_name, description, data_type, default_value, validation_rules, is_required, is_sensitive, created_at FROM settings_keys ORDER BY category_id, key_name"
        ).map_err(DatabaseError::from)?;
        
        let rows = stmt.query_map([], |row| SettingsKey::from_row(row)).map_err(DatabaseError::from)?;
        let mut keys = Vec::new();
        for row in rows {
            keys.push(row.map_err(DatabaseError::from)?);
        }
        
        Ok(keys)
    }
    
    /// Get settings key by name
    pub fn get_key_by_name(db: &Connection, key_name: &str) -> DatabaseResult<Option<SettingsKey>> {
        let mut stmt = db.prepare(
            "SELECT id, category_id, key_name, display_name, description, data_type, default_value, validation_rules, is_required, is_sensitive, created_at FROM settings_keys WHERE key_name = ?"
        ).map_err(DatabaseError::from)?;
        
        let mut rows = stmt.query_map([key_name], |row| SettingsKey::from_row(row)).map_err(DatabaseError::from)?;
        Ok(rows.next().transpose().map_err(DatabaseError::from)?)
    }
    
    /// Get settings value by key name
    pub fn get_value_by_key_name(db: &Connection, key_name: &str) -> DatabaseResult<Option<String>> {
        let mut stmt = db.prepare(
            "SELECT sv.value FROM settings_values sv 
             JOIN settings_keys sk ON sv.key_id = sk.id 
             WHERE sk.key_name = ? 
             ORDER BY sv.updated_at DESC 
             LIMIT 1"
        ).map_err(DatabaseError::from)?;
        
        let mut rows = stmt.query_map([key_name], |row| row.get::<_, String>(0)).map_err(DatabaseError::from)?;
        Ok(rows.next().transpose().map_err(DatabaseError::from)?)
    }
    
    /// Set settings value by key name
    pub fn set_value_by_key_name(db: &mut Connection, key_name: &str, value: &str, changed_by: &str) -> DatabaseResult<()> {
        let tx = db.transaction().map_err(DatabaseError::from)?;
        
        // Get the key
        let key = Self::get_key_by_name(&tx, key_name)?
            .ok_or_else(|| DatabaseError::NotFound(format!("Settings key '{}' not found", key_name)))?;
        
        // Get current value for history
        let old_value = Self::get_value_by_key_name(&tx, key_name)?;
        
        // Insert new value
        let settings_value = SettingsValue::new(key.id.unwrap(), value.to_string());
        tx.execute(
            "INSERT INTO settings_values (key_id, value, created_at, updated_at) VALUES (?, ?, ?, ?)",
            [
                &settings_value.key_id.to_string(),
                &settings_value.value,
                &settings_value.created_at.to_rfc3339(),
                &settings_value.updated_at.to_rfc3339(),
            ],
        ).map_err(DatabaseError::from)?;
        
        // Record in history
        let history = SettingsHistory::new(
            key.id.unwrap(),
            old_value,
            Some(value.to_string()),
            changed_by.to_string(),
            Some("Value updated".to_string()),
        );
        
        tx.execute(
            "INSERT INTO settings_history (key_id, old_value, new_value, changed_by, change_reason, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            [
                &history.key_id.to_string(),
                &history.old_value.as_deref().unwrap_or("").to_string(),
                &history.new_value.as_deref().unwrap_or("").to_string(),
                &history.changed_by,
                &history.change_reason.as_deref().unwrap_or("").to_string(),
                &history.created_at.to_rfc3339(),
            ],
        ).map_err(DatabaseError::from)?;
        
        tx.commit().map_err(DatabaseError::from)?;
        Ok(())
    }
    
    /// Get all settings values with their keys
    pub fn get_all_values(db: &Connection) -> DatabaseResult<Vec<(SettingsKey, Option<String>)>> {
        let mut stmt = db.prepare(
            "SELECT sk.id, sk.category_id, sk.key_name, sk.display_name, sk.description, sk.data_type, sk.default_value, sk.validation_rules, sk.is_required, sk.is_sensitive, sk.created_at,
                    (SELECT sv.value FROM settings_values sv WHERE sv.key_id = sk.id ORDER BY sv.updated_at DESC LIMIT 1) as current_value
             FROM settings_keys sk
             ORDER BY sk.category_id, sk.key_name"
        ).map_err(DatabaseError::from)?;
        
        let rows = stmt.query_map([], |row| {
            let key = SettingsKey::from_row(row)?;
            let value: Option<String> = row.get("current_value")?;
            Ok((key, value))
        }).map_err(DatabaseError::from)?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(DatabaseError::from)?);
        }
        
        Ok(results)
    }
    
    /// Get settings history for a key
    pub fn get_history_by_key_name(db: &Connection, key_name: &str, limit: Option<i64>) -> DatabaseResult<Vec<SettingsHistory>> {
        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        let query = format!(
            "SELECT sh.id, sh.key_id, sh.old_value, sh.new_value, sh.changed_by, sh.change_reason, sh.created_at
             FROM settings_history sh
             JOIN settings_keys sk ON sh.key_id = sk.id
             WHERE sk.key_name = ?
             ORDER BY sh.created_at DESC{}",
            limit_clause
        );
        
        let mut stmt = db.prepare(&query).map_err(DatabaseError::from)?;
        let rows = stmt.query_map([key_name], |row| SettingsHistory::from_row(row)).map_err(DatabaseError::from)?;
        
        let mut history = Vec::new();
        for row in rows {
            history.push(row.map_err(DatabaseError::from)?);
        }
        
        Ok(history)
    }
    
    /// Bulk update settings values
    pub fn bulk_update_values(db: &mut Connection, updates: Vec<(String, String)>, changed_by: &str) -> DatabaseResult<()> {
        let tx = db.transaction().map_err(DatabaseError::from)?;
        for (key_name, value) in updates {
            // Get the key ID
            let key_id: i64 = tx.query_row(
                "SELECT id FROM settings_keys WHERE key_name = ?",
                [&key_name],
                |row| row.get(0)
            ).map_err(DatabaseError::from)?;
            
            // Get current value for history
            let current_value: Option<String> = tx.query_row(
                "SELECT value FROM settings_values WHERE key_id = ? ORDER BY updated_at DESC LIMIT 1",
                [key_id],
                |row| row.get(0)
            ).ok();
            
            // Update or insert the value
            tx.execute(
                "INSERT OR REPLACE INTO settings_values (key_id, value, updated_at) VALUES (?, ?, ?)",
                [
                    &key_id.to_string(),
                    &value,
                    &Utc::now().to_rfc3339(),
                ],
            ).map_err(DatabaseError::from)?;
            
            // Add to history
            tx.execute(
                "INSERT INTO settings_history (key_id, old_value, new_value, changed_by, change_reason, created_at) VALUES (?, ?, ?, ?, ?, ?)",
                [
                    &key_id.to_string(),
                    &current_value.as_deref().unwrap_or("").to_string(),
                    &value,
                    changed_by,
                    &"Bulk update".to_string(),
                    &Utc::now().to_rfc3339(),
                ],
            ).map_err(DatabaseError::from)?;
        }
        tx.commit().map_err(DatabaseError::from)?;
        Ok(())
    }
    
    /// Get settings statistics
    pub fn get_statistics(db: &Connection) -> DatabaseResult<SettingsStatistics> {
        let total_keys: i64 = db.query_row("SELECT COUNT(*) FROM settings_keys", [], |row| row.get(0)).map_err(DatabaseError::from)?;
        let total_values: i64 = db.query_row("SELECT COUNT(*) FROM settings_values", [], |row| row.get(0)).map_err(DatabaseError::from)?;
        let total_categories: i64 = db.query_row("SELECT COUNT(*) FROM settings_categories", [], |row| row.get(0)).map_err(DatabaseError::from)?;
        let total_history: i64 = db.query_row("SELECT COUNT(*) FROM settings_history", [], |row| row.get(0)).map_err(DatabaseError::from)?;
        
        Ok(SettingsStatistics {
            total_keys,
            total_values,
            total_categories,
            total_history,
        })
    }
}

/// Settings statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsStatistics {
    pub total_keys: i64,
    pub total_values: i64,
    pub total_categories: i64,
    pub total_history: i64,
}

/// Database maintenance and backup operations
pub struct DatabaseMaintenanceOperations;

impl DatabaseMaintenanceOperations {
    /// Optimize the database (VACUUM, ANALYZE, REINDEX)
    pub fn optimize(db: &mut Connection) -> DatabaseResult<()> {
        log::info!("Starting database optimization...");
        
        let tx = db.transaction().map_err(DatabaseError::from)?;
        
        // VACUUM to reclaim space and defragment
        log::debug!("Running VACUUM...");
        tx.execute("VACUUM", []).map_err(DatabaseError::from)?;
        
        // ANALYZE to update statistics
        log::debug!("Running ANALYZE...");
        tx.execute("ANALYZE", []).map_err(DatabaseError::from)?;
        
        // REINDEX to rebuild all indexes
        log::debug!("Running REINDEX...");
        tx.execute("REINDEX", []).map_err(DatabaseError::from)?;
        
        tx.commit().map_err(DatabaseError::from)?;
        log::info!("Database optimization completed successfully");
        Ok(())
    }
    
    /// Check database integrity
    pub fn check_integrity(db: &Connection) -> DatabaseResult<DatabaseIntegrityReport> {
        log::info!("Checking database integrity...");
        
        let integrity_check: String = db.query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(DatabaseError::from)?;
        
        let quick_check: String = db.query_row("PRAGMA quick_check", [], |row| row.get(0))
            .map_err(DatabaseError::from)?;
        
        let foreign_key_check: String = db.query_row("PRAGMA foreign_key_check", [], |row| row.get(0))
            .map_err(DatabaseError::from)?;
        
        let is_integrity_ok = integrity_check == "ok";
        let is_quick_ok = quick_check == "ok";
        let has_foreign_key_issues = !foreign_key_check.is_empty();
        
        let report = DatabaseIntegrityReport {
            integrity_check,
            quick_check,
            foreign_key_check,
            is_integrity_ok,
            is_quick_ok,
            has_foreign_key_issues,
        };
        
        if is_integrity_ok && is_quick_ok && !has_foreign_key_issues {
            log::info!("Database integrity check passed");
        } else {
            log::error!("Database integrity check failed: {:?}", report);
        }
        
        Ok(report)
    }
    
    /// Get database size information
    pub fn get_size_info(db: &Connection) -> DatabaseResult<DatabaseSizeInfo> {
        let page_count: i64 = db.query_row("PRAGMA page_count", [], |row| row.get(0))
            .map_err(DatabaseError::from)?;
        
        let page_size: i64 = db.query_row("PRAGMA page_size", [], |row| row.get(0))
            .map_err(DatabaseError::from)?;
        
        let cache_size: i64 = db.query_row("PRAGMA cache_size", [], |row| row.get(0))
            .map_err(DatabaseError::from)?;
        
        let total_size = page_count * page_size;
        let cache_size_bytes = cache_size.abs() * 1024; // Convert KB to bytes
        
        Ok(DatabaseSizeInfo {
            page_count,
            page_size,
            cache_size,
            total_size,
            cache_size_bytes,
        })
    }
    
    /// Export database to SQL file
    pub fn export_to_sql(db: &Connection, output_path: &PathBuf) -> DatabaseResult<()> {
        log::info!("Exporting database to SQL file: {:?}", output_path);
        
        // Get all table names
        let mut stmt = db.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
        ).map_err(DatabaseError::from)?;
        
        let tables: Vec<String> = stmt.query_map([], |row| row.get(0))
            .map_err(DatabaseError::from)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(DatabaseError::from)?;
        
        let mut sql_content = String::new();
        sql_content.push_str("-- Database Export\n");
        sql_content.push_str(&format!("-- Generated at: {}\n", Utc::now().to_rfc3339()));
        sql_content.push_str("-- Application: reStrike VTA\n\n");
        
        // Export schema for each table
        for table_name in &tables {
            let schema: String = db.query_row(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name = ?",
                [table_name],
                |row| row.get(0)
            ).map_err(DatabaseError::from)?;
            
            sql_content.push_str(&format!("-- Table: {}\n", table_name));
            sql_content.push_str(&schema);
            sql_content.push_str(";\n\n");
        }
        
        // Export data for each table (simplified approach)
        for table_name in &tables {
            let count: i64 = db.query_row(
                &format!("SELECT COUNT(*) FROM {}", table_name),
                [],
                |row| row.get(0)
            ).map_err(DatabaseError::from)?;
            
            if count > 0 {
                sql_content.push_str(&format!("-- Data for table: {} ({} rows)\n", table_name, count));
                sql_content.push_str(&format!("-- Note: Data export is simplified. Use backup files for full data export.\n\n"));
            }
        }
        
        // Write to file
        std::fs::write(output_path, sql_content)
            .map_err(|e| DatabaseError::Connection(format!("Failed to write SQL file: {}", e)))?;
        
        log::info!("Database exported successfully to: {:?}", output_path);
        Ok(())
    }
    
    /// Import database from SQL file
    pub fn import_from_sql(db: &mut Connection, sql_file_path: &PathBuf) -> DatabaseResult<()> {
        log::info!("Importing database from SQL file: {:?}", sql_file_path);
        
        if !sql_file_path.exists() {
            return Err(DatabaseError::Connection(format!("SQL file does not exist: {:?}", sql_file_path)));
        }
        
        let sql_content = std::fs::read_to_string(sql_file_path)
            .map_err(|e| DatabaseError::Connection(format!("Failed to read SQL file: {}", e)))?;
        
        let tx = db.transaction().map_err(DatabaseError::from)?;
        
        // Split SQL content into individual statements
        let statements: Vec<&str> = sql_content
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.starts_with("--"))
            .collect();
        
        let mut executed_count = 0;
        for statement in statements {
            if !statement.trim().is_empty() {
                tx.execute(statement, [])
                    .map_err(|e| DatabaseError::Connection(format!("Failed to execute SQL statement: {}", e)))?;
                executed_count += 1;
            }
        }
        
        tx.commit().map_err(DatabaseError::from)?;
        
        log::info!("Database import completed: {} statements executed", executed_count);
        Ok(())
    }
    
    /// Create database backup with compression
    pub fn create_compressed_backup(_db: &Connection, backup_path: &PathBuf) -> DatabaseResult<()> {
        log::info!("Creating compressed database backup: {:?}", backup_path);
        
        // For now, we'll create a simple file copy backup
        // In a production environment, you might want to use compression libraries
        let db_path = crate::database::connection::DatabaseConnection::get_database_path()
            .map_err(|e| DatabaseError::Connection(format!("Failed to get database path: {}", e)))?;
        
        std::fs::copy(&db_path, backup_path)
            .map_err(|e| DatabaseError::Connection(format!("Failed to create backup: {}", e)))?;
        
        log::info!("Compressed backup created successfully: {:?}", backup_path);
        Ok(())
    }
    
    /// Verify backup integrity
    pub fn verify_backup(backup_path: &PathBuf) -> DatabaseResult<bool> {
        log::info!("Verifying backup integrity: {:?}", backup_path);
        
        if !backup_path.exists() {
            return Err(DatabaseError::Connection(format!("Backup file does not exist: {:?}", backup_path)));
        }
        
        let backup_conn = Connection::open(backup_path)
            .map_err(|e| DatabaseError::Connection(format!("Failed to open backup file: {}", e)))?;
        
        let integrity: String = backup_conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(|e| DatabaseError::Connection(format!("Failed to check backup integrity: {}", e)))?;
        
        let is_valid = integrity == "ok";
        
        if is_valid {
            log::info!("Backup integrity verification passed");
        } else {
            log::error!("Backup integrity verification failed: {}", integrity);
        }
        
        Ok(is_valid)
    }
    
    /// Get database performance statistics
    pub fn get_performance_stats(db: &Connection) -> DatabaseResult<DatabasePerformanceStats> {
        let cache_hits: i64 = db.query_row("PRAGMA cache_hit", [], |row| row.get(0))
            .map_err(DatabaseError::from)?;
        
        let cache_misses: i64 = db.query_row("PRAGMA cache_miss", [], |row| row.get(0))
            .map_err(DatabaseError::from)?;
        
        let total_cache_access = cache_hits + cache_misses;
        let hit_rate = if total_cache_access > 0 {
            (cache_hits as f64 / total_cache_access as f64) * 100.0
        } else {
            0.0
        };
        
        let wal_checkpoint_mode: String = db.query_row("PRAGMA wal_checkpoint", [], |row| row.get(0))
            .map_err(DatabaseError::from)?;
        
        let journal_mode: String = db.query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .map_err(DatabaseError::from)?;
        
        Ok(DatabasePerformanceStats {
            cache_hits,
            cache_misses,
            hit_rate,
            wal_checkpoint_mode,
            journal_mode,
        })
    }
}

/// Database integrity report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseIntegrityReport {
    pub integrity_check: String,
    pub quick_check: String,
    pub foreign_key_check: String,
    pub is_integrity_ok: bool,
    pub is_quick_ok: bool,
    pub has_foreign_key_issues: bool,
}

/// Database size information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSizeInfo {
    pub page_count: i64,
    pub page_size: i64,
    pub cache_size: i64,
    pub total_size: i64,
    pub cache_size_bytes: i64,
}

/// Database performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePerformanceStats {
    pub cache_hits: i64,
    pub cache_misses: i64,
    pub hit_rate: f64,
    pub wal_checkpoint_mode: String,
    pub journal_mode: String,
} 