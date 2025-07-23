use rusqlite::{Result as SqliteResult, params, OptionalExtension};
use serde_json::Value;
use chrono::Utc;
use crate::database::{
    DatabaseError, DatabaseResult,
    models::{SettingsKey, SettingsValue, SettingsHistory, SettingsCategory},
    connection::DatabaseConnection,
};

/// Settings Manager for enhanced settings management
pub struct SettingsManager {
    conn: DatabaseConnection,
}

impl SettingsManager {
    /// Create a new settings manager
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
    
    /// Get a setting value by key name
    pub fn get_setting(&self, key_name: &str) -> DatabaseResult<Option<String>> {
        let conn = self.conn.get_connection()?;
        
        let value: Option<String> = conn.query_row(
            "SELECT sv.value FROM settings_values sv 
             JOIN settings_keys sk ON sv.key_id = sk.id 
             WHERE sk.key_name = ?",
            params![key_name],
            |row| row.get(0)
        ).optional()?;
        
        Ok(value)
    }
    
    /// Set a setting value with validation and history tracking
    pub fn set_setting(
        &self,
        key_name: &str,
        value: &str,
        changed_by: &str,
        change_reason: Option<&str>,
    ) -> DatabaseResult<()> {
        let mut conn = self.conn.get_connection()?;
        
        // Start transaction
        let tx = conn.transaction()?;
        
        // Get the setting key
        let setting_key: Option<SettingsKey> = tx.query_row(
            "SELECT * FROM settings_keys WHERE key_name = ?",
            params![key_name],
            |row| SettingsKey::from_row(row)
        ).optional()?;
        
        let setting_key = setting_key.ok_or_else(|| {
            DatabaseError::InvalidData(format!("Setting key '{}' not found", key_name))
        })?;
        
        // Validate the setting if validation rules exist
        if let Some(validation_rules) = &setting_key.validation_rules {
            self.validate_setting_value(&setting_key.data_type, value, validation_rules)?;
        }
        
        // Check if setting value exists
        let existing_value: Option<SettingsValue> = tx.query_row(
            "SELECT * FROM settings_values WHERE key_id = ?",
            params![setting_key.id.unwrap()],
            |row| SettingsValue::from_row(row)
        ).optional()?;
        
        if let Some(existing) = existing_value {
            // Update existing value
            let old_value = existing.value.clone();
            
            tx.execute(
                "UPDATE settings_values SET value = ?, updated_at = ? WHERE id = ?",
                params![value, Utc::now().to_rfc3339(), existing.id.unwrap()]
            )?;
            
            // Record history
            let history = SettingsHistory::new(
                setting_key.id.unwrap(),
                Some(old_value),
                Some(value.to_string()),
                changed_by.to_string(),
                change_reason.map(|s| s.to_string()),
            );
            
            tx.execute(
                "INSERT INTO settings_history (key_id, old_value, new_value, changed_by, change_reason, created_at) VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    history.key_id,
                    history.old_value,
                    history.new_value,
                    history.changed_by,
                    history.change_reason,
                    history.created_at.to_rfc3339()
                ]
            )?;
        } else {
            // Create new value
            let setting_value = SettingsValue::new(
                setting_key.id.unwrap(),
                value.to_string(),
            );
            
            let _value_id = tx.execute(
                "INSERT INTO settings_values (key_id, value, created_at, updated_at) VALUES (?, ?, ?, ?)",
                params![
                    setting_value.key_id,
                    setting_value.value,
                    setting_value.created_at.to_rfc3339(),
                    setting_value.updated_at.to_rfc3339()
                ]
            )?;
            
            // Record history for new setting
            let history = SettingsHistory::new(
                setting_key.id.unwrap(),
                None,
                Some(value.to_string()),
                changed_by.to_string(),
                change_reason.map(|s| s.to_string()),
            );
            
            tx.execute(
                "INSERT INTO settings_history (key_id, old_value, new_value, changed_by, change_reason, created_at) VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    history.key_id,
                    history.old_value,
                    history.new_value,
                    history.changed_by,
                    history.change_reason,
                    history.created_at.to_rfc3339()
                ]
            )?;
        }
        
        // Commit transaction
        tx.commit()?;
        
        Ok(())
    }
    
    /// Get all settings by category
    pub fn get_settings_by_category(&self, category_name: &str) -> DatabaseResult<Vec<(SettingsKey, Option<String>)>> {
        let conn = self.conn.get_connection()?;
        
        let mut stmt = conn.prepare(
            "SELECT sk.*, sv.value FROM settings_keys sk 
             LEFT JOIN settings_values sv ON sk.id = sv.key_id 
             JOIN settings_categories sc ON sk.category_id = sc.id 
             WHERE sc.name = ? 
             ORDER BY sk.key_name"
        )?;
        
        let settings = stmt.query_map(params![category_name], |row| {
            let key = SettingsKey::from_row(row)?;
            let value: Option<String> = row.get("value")?;
            Ok((key, value))
        })?
        .collect::<SqliteResult<Vec<_>>>()?;
        
        Ok(settings)
    }
    
    /// Get settings history for a specific setting
    pub fn get_setting_history(&self, key_name: &str, limit: Option<i64>) -> DatabaseResult<Vec<SettingsHistory>> {
        let conn = self.conn.get_connection()?;
        
        let limit = limit.unwrap_or(50);
        
        let mut stmt = conn.prepare(
            "SELECT sh.* FROM settings_history sh 
             JOIN settings_keys sk ON sh.key_id = sk.id 
             WHERE sk.key_name = ? 
             ORDER BY sh.created_at DESC 
             LIMIT ?"
        )?;
        
        let history = stmt.query_map(params![key_name, limit], |row| {
            SettingsHistory::from_row(row)
        })?
        .collect::<SqliteResult<Vec<_>>>()?;
        
        Ok(history)
    }
    
    /// Create a new setting key
    pub fn create_setting_key(
        &self,
        category_name: &str,
        key_name: &str,
        display_name: &str,
        description: Option<&str>,
        data_type: &str,
        default_value: Option<&str>,
        validation_rules: Option<&str>,
        is_required: bool,
        is_sensitive: bool,
    ) -> DatabaseResult<i64> {
        let mut conn = self.conn.get_connection()?;
        
        // Start transaction
        let tx = conn.transaction()?;
        
        // Get category ID
        let category_id: i64 = tx.query_row(
            "SELECT id FROM settings_categories WHERE name = ?",
            params![category_name],
            |row| row.get(0)
        )?;
        
        // Create setting key
        let setting_key = SettingsKey::new(
            category_id,
            key_name.to_string(),
            display_name.to_string(),
            description.map(|s| s.to_string()),
            data_type.to_string(),
            default_value.map(|s| s.to_string()),
            validation_rules.map(|s| s.to_string()),
            is_required,
            is_sensitive,
        );
        
        let key_id = tx.execute(
            "INSERT INTO settings_keys (category_id, key_name, display_name, description, data_type, default_value, validation_rules, is_required, is_sensitive, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                setting_key.category_id,
                setting_key.key_name,
                setting_key.display_name,
                setting_key.description,
                setting_key.data_type,
                setting_key.default_value,
                setting_key.validation_rules,
                setting_key.is_required,
                setting_key.is_sensitive,
                setting_key.created_at.to_rfc3339()
            ]
        )?;
        
        // Set default value if provided
        if let Some(default_val) = default_value {
            let setting_value = SettingsValue::new(key_id as i64, default_val.to_string());
            
            tx.execute(
                "INSERT INTO settings_values (key_id, value, created_at, updated_at) VALUES (?, ?, ?, ?)",
                params![
                    setting_value.key_id,
                    setting_value.value,
                    setting_value.created_at.to_rfc3339(),
                    setting_value.updated_at.to_rfc3339()
                ]
            )?;
        }
        
        // Commit transaction
        tx.commit()?;
        
        Ok(key_id as i64)
    }
    
    /// Validate a setting value against validation rules
    fn validate_setting_value(&self, data_type: &str, value: &str, validation_rules: &str) -> DatabaseResult<()> {
        match data_type {
            "json" => {
                // Validate JSON format
                serde_json::from_str::<Value>(value)
                    .map_err(|e| DatabaseError::InvalidData(format!("Invalid JSON format: {}", e)))?;
            }
            "boolean" => {
                // Validate boolean value
                if !["true", "false", "1", "0"].contains(&value.to_lowercase().as_str()) {
                    return Err(DatabaseError::InvalidData("Invalid boolean value".to_string()));
                }
            }
            "integer" => {
                // Validate integer value
                value.parse::<i64>()
                    .map_err(|_| DatabaseError::InvalidData("Invalid integer value".to_string()))?;
            }
            "float" => {
                // Validate float value
                value.parse::<f64>()
                    .map_err(|_| DatabaseError::InvalidData("Invalid float value".to_string()))?;
            }
            "range" => {
                // Validate range (assuming validation_rules contains min,max)
                if let Ok(rules) = serde_json::from_str::<Value>(validation_rules) {
                    if let (Some(min), Some(max)) = (rules.get("min"), rules.get("max")) {
                        if let (Some(min_val), Some(max_val)) = (min.as_f64(), max.as_f64()) {
                            if let Ok(value_float) = value.parse::<f64>() {
                                if value_float < min_val || value_float > max_val {
                                    return Err(DatabaseError::InvalidData(
                                        format!("Value must be between {} and {}", min_val, max_val)
                                    ));
                                }
                            } else {
                                return Err(DatabaseError::InvalidData("Value must be a number".to_string()));
                            }
                        }
                    }
                }
            }
            _ => {
                // String type or unknown type - no validation needed
            }
        }
        
        Ok(())
    }
    
    /// Get all categories
    pub fn get_categories(&self) -> DatabaseResult<Vec<SettingsCategory>> {
        let conn = self.conn.get_connection()?;
        
        let mut stmt = conn.prepare(
            "SELECT * FROM settings_categories ORDER BY display_order, name"
        )?;
        
        let categories = stmt.query_map([], |row| {
            SettingsCategory::from_row(row)
        })?
        .collect::<SqliteResult<Vec<_>>>()?;
        
        Ok(categories)
    }
} 