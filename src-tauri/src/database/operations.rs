use rusqlite::{params};
use chrono::Utc;
use crate::database::{
    DatabaseConnection, DatabaseResult,
    PssEvent, ObsConnection, AppConfig, FlagMapping,
};

/// PSS Events operations
pub struct PssEventOperations;

impl PssEventOperations {
    /// Insert a new PSS event
    pub fn insert(db: &DatabaseConnection, event: &PssEvent) -> DatabaseResult<i64> {
        db.transaction(|conn| {
            let id = conn.execute(
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
            
            Ok(id as i64)
        })
    }
    
    /// Get PSS event by ID
    pub fn get_by_id(db: &DatabaseConnection, id: i64) -> DatabaseResult<Option<PssEvent>> {
        db.read_transaction(|conn| {
            let mut stmt = conn.prepare(
                "SELECT * FROM pss_events WHERE id = ?"
            )?;
            
            let mut rows = stmt.query(params![id])?;
            if let Some(row) = rows.next()? {
                Ok(Some(PssEvent::from_row(&row)?))
            } else {
                Ok(None)
            }
        })
    }
    
    /// Get PSS events by match ID
    pub fn get_by_match_id(db: &DatabaseConnection, match_id: &str) -> DatabaseResult<Vec<PssEvent>> {
        db.read_transaction(|conn| {
            let mut stmt = conn.prepare(
                "SELECT * FROM pss_events WHERE match_id = ? ORDER BY timestamp"
            )?;
            
            let rows = stmt.query_map(params![match_id], |row| PssEvent::from_row(row))?;
            let mut events = Vec::new();
            for row in rows {
                events.push(row?);
            }
            
            Ok(events)
        })
    }
    
    /// Get PSS events within a time range
    pub fn get_by_time_range(
        db: &DatabaseConnection,
        start_time: chrono::DateTime<Utc>,
        end_time: chrono::DateTime<Utc>,
    ) -> DatabaseResult<Vec<PssEvent>> {
        db.read_transaction(|conn| {
            let mut stmt = conn.prepare(
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
        })
    }
    
    /// Get recent PSS events
    pub fn get_recent(db: &DatabaseConnection, limit: i64) -> DatabaseResult<Vec<PssEvent>> {
        db.read_transaction(|conn| {
            let mut stmt = conn.prepare(
                "SELECT * FROM pss_events ORDER BY timestamp DESC LIMIT ?"
            )?;
            
            let rows = stmt.query_map(params![limit], |row| PssEvent::from_row(row))?;
            let mut events = Vec::new();
            for row in rows {
                events.push(row?);
            }
            
            Ok(events)
        })
    }
    
    /// Delete PSS event by ID
    pub fn delete_by_id(db: &DatabaseConnection, id: i64) -> DatabaseResult<bool> {
        db.transaction(|conn| {
            let rows_affected = conn.execute(
                "DELETE FROM pss_events WHERE id = ?",
                params![id],
            )?;
            
            Ok(rows_affected > 0)
        })
    }
    
    /// Clear all PSS events
    pub fn clear_all(db: &DatabaseConnection) -> DatabaseResult<u64> {
        db.transaction(|conn| {
            let rows_affected = conn.execute("DELETE FROM pss_events", [])?;
            Ok(rows_affected as u64)
        })
    }
}

/// OBS Connections operations
pub struct ObsConnectionOperations;

impl ObsConnectionOperations {
    /// Insert a new OBS connection
    pub fn insert(db: &DatabaseConnection, connection: &ObsConnection) -> DatabaseResult<i64> {
        db.transaction(|conn| {
            let id = conn.execute(
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
            
            Ok(id as i64)
        })
    }
    
    /// Get OBS connection by ID
    pub fn get_by_id(db: &DatabaseConnection, id: i64) -> DatabaseResult<Option<ObsConnection>> {
        db.read_transaction(|conn| {
            let mut stmt = conn.prepare(
                "SELECT * FROM obs_connections WHERE id = ?"
            )?;
            
            let mut rows = stmt.query(params![id])?;
            if let Some(row) = rows.next()? {
                Ok(Some(ObsConnection::from_row(&row)?))
            } else {
                Ok(None)
            }
        })
    }
    
    /// Get OBS connection by name
    pub fn get_by_name(db: &DatabaseConnection, name: &str) -> DatabaseResult<Option<ObsConnection>> {
        db.read_transaction(|conn| {
            let mut stmt = conn.prepare(
                "SELECT * FROM obs_connections WHERE name = ?"
            )?;
            
            let mut rows = stmt.query(params![name])?;
            if let Some(row) = rows.next()? {
                Ok(Some(ObsConnection::from_row(&row)?))
            } else {
                Ok(None)
            }
        })
    }
    
    /// Get all OBS connections
    pub fn get_all(db: &DatabaseConnection) -> DatabaseResult<Vec<ObsConnection>> {
        db.read_transaction(|conn| {
            let mut stmt = conn.prepare(
                "SELECT * FROM obs_connections ORDER BY name"
            )?;
            
            let rows = stmt.query_map([], |row| ObsConnection::from_row(row))?;
            let mut connections = Vec::new();
            for row in rows {
                connections.push(row?);
            }
            
            Ok(connections)
        })
    }
    
    /// Update OBS connection
    pub fn update(db: &DatabaseConnection, connection: &ObsConnection) -> DatabaseResult<bool> {
        db.transaction(|conn| {
            let rows_affected = conn.execute(
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
            
            Ok(rows_affected > 0)
        })
    }
    
    /// Delete OBS connection by ID
    pub fn delete_by_id(db: &DatabaseConnection, id: i64) -> DatabaseResult<bool> {
        db.transaction(|conn| {
            let rows_affected = conn.execute(
                "DELETE FROM obs_connections WHERE id = ?",
                params![id],
            )?;
            
            Ok(rows_affected > 0)
        })
    }
    
    /// Set active OBS connection
    pub fn set_active(db: &DatabaseConnection, id: i64) -> DatabaseResult<bool> {
        db.transaction(|conn| {
            // Deactivate all connections
            conn.execute("UPDATE obs_connections SET is_active = 0", [])?;
            
            // Activate the specified connection
            let rows_affected = conn.execute(
                "UPDATE obs_connections SET is_active = 1 WHERE id = ?",
                params![id],
            )?;
            
            Ok(rows_affected > 0)
        })
    }
}

/// App Config operations
pub struct AppConfigOperations;

impl AppConfigOperations {
    /// Insert or update app config
    pub fn upsert(db: &DatabaseConnection, config: &AppConfig) -> DatabaseResult<i64> {
        db.transaction(|conn| {
            let id = conn.execute(
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
            
            Ok(id as i64)
        })
    }
    
    /// Get app config by key
    pub fn get_by_key(db: &DatabaseConnection, key: &str) -> DatabaseResult<Option<AppConfig>> {
        db.read_transaction(|conn| {
            let mut stmt = conn.prepare(
                "SELECT * FROM app_config WHERE key = ?"
            )?;
            
            let mut rows = stmt.query(params![key])?;
            if let Some(row) = rows.next()? {
                Ok(Some(AppConfig::from_row(&row)?))
            } else {
                Ok(None)
            }
        })
    }
    
    /// Get app configs by category
    pub fn get_by_category(db: &DatabaseConnection, category: &str) -> DatabaseResult<Vec<AppConfig>> {
        db.read_transaction(|conn| {
            let mut stmt = conn.prepare(
                "SELECT * FROM app_config WHERE category = ? ORDER BY key"
            )?;
            
            let rows = stmt.query_map(params![category], |row| AppConfig::from_row(row))?;
            let mut configs = Vec::new();
            for row in rows {
                configs.push(row?);
            }
            
            Ok(configs)
        })
    }
    
    /// Get all app configs
    pub fn get_all(db: &DatabaseConnection) -> DatabaseResult<Vec<AppConfig>> {
        db.read_transaction(|conn| {
            let mut stmt = conn.prepare(
                "SELECT * FROM app_config ORDER BY category, key"
            )?;
            
            let rows = stmt.query_map([], |row| AppConfig::from_row(row))?;
            let mut configs = Vec::new();
            for row in rows {
                configs.push(row?);
            }
            
            Ok(configs)
        })
    }
    
    /// Delete app config by key
    pub fn delete_by_key(db: &DatabaseConnection, key: &str) -> DatabaseResult<bool> {
        db.transaction(|conn| {
            let rows_affected = conn.execute(
                "DELETE FROM app_config WHERE key = ?",
                params![key],
            )?;
            
            Ok(rows_affected > 0)
        })
    }
}

/// Flag Mapping operations
pub struct FlagMappingOperations;

impl FlagMappingOperations {
    /// Insert or update flag mapping
    pub fn upsert(db: &DatabaseConnection, mapping: &FlagMapping) -> DatabaseResult<i64> {
        db.transaction(|conn| {
            let id = conn.execute(
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
            
            Ok(id as i64)
        })
    }
    
    /// Get flag mapping by PSS code
    pub fn get_by_pss_code(db: &DatabaseConnection, pss_code: &str) -> DatabaseResult<Option<FlagMapping>> {
        db.read_transaction(|conn| {
            let mut stmt = conn.prepare(
                "SELECT * FROM flag_mappings WHERE pss_code = ?"
            )?;
            
            let mut rows = stmt.query(params![pss_code])?;
            if let Some(row) = rows.next()? {
                Ok(Some(FlagMapping::from_row(&row)?))
            } else {
                Ok(None)
            }
        })
    }
    
    /// Get flag mapping by IOC code
    pub fn get_by_ioc_code(db: &DatabaseConnection, ioc_code: &str) -> DatabaseResult<Option<FlagMapping>> {
        db.read_transaction(|conn| {
            let mut stmt = conn.prepare(
                "SELECT * FROM flag_mappings WHERE ioc_code = ?"
            )?;
            
            let mut rows = stmt.query(params![ioc_code])?;
            if let Some(row) = rows.next()? {
                Ok(Some(FlagMapping::from_row(&row)?))
            } else {
                Ok(None)
            }
        })
    }
    
    /// Get all flag mappings
    pub fn get_all(db: &DatabaseConnection) -> DatabaseResult<Vec<FlagMapping>> {
        db.read_transaction(|conn| {
            let mut stmt = conn.prepare(
                "SELECT * FROM flag_mappings ORDER BY country_name"
            )?;
            
            let rows = stmt.query_map([], |row| FlagMapping::from_row(row))?;
            let mut mappings = Vec::new();
            for row in rows {
                mappings.push(row?);
            }
            
            Ok(mappings)
        })
    }
    
    /// Get custom flag mappings only
    pub fn get_custom(db: &DatabaseConnection) -> DatabaseResult<Vec<FlagMapping>> {
        db.read_transaction(|conn| {
            let mut stmt = conn.prepare(
                "SELECT * FROM flag_mappings WHERE is_custom = 1 ORDER BY country_name"
            )?;
            
            let rows = stmt.query_map([], |row| FlagMapping::from_row(row))?;
            let mut mappings = Vec::new();
            for row in rows {
                mappings.push(row?);
            }
            
            Ok(mappings)
        })
    }
    
    /// Delete flag mapping by PSS code
    pub fn delete_by_pss_code(db: &DatabaseConnection, pss_code: &str) -> DatabaseResult<bool> {
        db.transaction(|conn| {
            let rows_affected = conn.execute(
                "DELETE FROM flag_mappings WHERE pss_code = ?",
                params![pss_code],
            )?;
            
            Ok(rows_affected > 0)
        })
    }
} 