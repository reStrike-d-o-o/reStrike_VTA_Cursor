use rusqlite::{Connection, Result as SqliteResult, params, OptionalExtension};
use chrono::Utc;
use crate::database::{
    DatabaseResult,
    models::{SettingsKey, SettingsValue, SettingsHistory, SettingsCategory},
};

/// UI Settings Operations for managing UI configuration
pub struct UiSettingsOperations;

impl UiSettingsOperations {
    /// Initialize UI settings in the database
    pub fn initialize_ui_settings(conn: &mut Connection) -> DatabaseResult<()> {
        let tx = conn.transaction()?;
        
        // Get or create UI category
        let ui_category_id = Self::get_or_create_category(&tx, "ui", "User Interface Settings", 5)?;
        
        // Define UI setting keys
        let ui_settings = vec![
            // Window settings
            ("window.position.x", "Window X Position", "integer", Some("100"), Some(r#"{"min": 0, "max": 9999}"#)),
            ("window.position.y", "Window Y Position", "integer", Some("100"), Some(r#"{"min": 0, "max": 9999}"#)),
            ("window.size.width", "Window Width", "integer", Some("1200"), Some(r#"{"min": 350, "max": 9999}"#)),
            ("window.size.height", "Window Height", "integer", Some("800"), Some(r#"{"min": 600, "max": 9999}"#)),
            ("window.fullscreen", "Fullscreen Mode", "boolean", Some("false"), None),
            ("window.compact", "Compact Mode", "boolean", Some("false"), None),
            
            // Theme settings
            ("theme.current", "Current Theme", "string", Some("dark"), Some(r#"{"enum": ["dark", "light", "auto"]}"#)),
            ("theme.auto_theme", "Auto Theme", "boolean", Some("false"), None),
            ("theme.high_contrast", "High Contrast", "boolean", Some("false"), None),
            
            // Layout settings
            ("layout.sidebar_position", "Sidebar Position", "string", Some("left"), Some(r#"{"enum": ["left", "right"]}"#)),
            ("layout.sidebar_width", "Sidebar Width", "integer", Some("300"), Some(r#"{"min": 200, "max": 500}"#)),
            ("layout.status_bar_visible", "Status Bar Visible", "boolean", Some("true"), None),
            ("layout.task_bar_visible", "Task Bar Visible", "boolean", Some("true"), None),
            
            // Advanced panel settings
            ("advanced.show_advanced_panel", "Show Advanced Panel", "boolean", Some("false"), None),
            ("advanced.debug_mode", "Debug Mode", "boolean", Some("false"), None),
            ("advanced.verbose_logging", "Verbose Logging", "boolean", Some("false"), None),
            
            // Animation settings
            ("animations.enabled", "Animations Enabled", "boolean", Some("true"), None),
            ("animations.duration_ms", "Animation Duration", "integer", Some("300"), Some(r#"{"min": 0, "max": 2000}"#)),
            ("animations.reduce_motion", "Reduce Motion", "boolean", Some("false"), None),
        ];
        
        // Create setting keys
        for (key_name, display_name, data_type, default_value, validation_rules) in ui_settings {
            Self::create_setting_key_if_not_exists(
                &tx,
                ui_category_id,
                key_name,
                display_name,
                data_type,
                default_value,
                validation_rules,
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }
    
    /// Get or create a settings category
    fn get_or_create_category(conn: &Connection, name: &str, description: &str, display_order: i32) -> DatabaseResult<i64> {
        // Try to get existing category
        let category_id: Option<i64> = conn.query_row(
            "SELECT id FROM settings_categories WHERE name = ?",
            params![name],
            |row| row.get(0)
        ).optional()?;
        
        if let Some(id) = category_id {
            Ok(id)
        } else {
            // Create new category
            let category = SettingsCategory::new(
                name.to_string(),
                Some(description.to_string()),
                display_order,
            );
            
            let category_id = conn.execute(
                "INSERT INTO settings_categories (name, description, display_order, created_at) VALUES (?, ?, ?, ?)",
                params![
                    category.name,
                    category.description,
                    category.display_order,
                    category.created_at.to_rfc3339()
                ]
            )?;
            
            Ok(category_id as i64)
        }
    }
    
    /// Create a setting key if it doesn't exist
    fn create_setting_key_if_not_exists(
        conn: &Connection,
        category_id: i64,
        key_name: &str,
        display_name: &str,
        data_type: &str,
        default_value: Option<&str>,
        validation_rules: Option<&str>,
    ) -> DatabaseResult<()> {
        // Check if key already exists
        let exists: i32 = conn.query_row(
            "SELECT COUNT(*) FROM settings_keys WHERE key_name = ?",
            params![key_name],
            |row| row.get(0)
        )?;
        
        if exists == 0 {
            // Create new setting key
            let setting_key = SettingsKey::new(
                category_id,
                key_name.to_string(),
                display_name.to_string(),
                Some(format!("UI setting for {}", display_name)),
                data_type.to_string(),
                default_value.map(|s| s.to_string()),
                validation_rules.map(|s| s.to_string()),
                false, // not required
                false, // not sensitive
            );
            
            let key_id = conn.execute(
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
                
                conn.execute(
                    "INSERT INTO settings_values (key_id, value, created_at, updated_at) VALUES (?, ?, ?, ?)",
                    params![
                        setting_value.key_id,
                        setting_value.value,
                        setting_value.created_at.to_rfc3339(),
                        setting_value.updated_at.to_rfc3339()
                    ]
                )?;
            }
        }
        
        Ok(())
    }
    
    /// Get a UI setting value
    pub fn get_ui_setting(conn: &Connection, key_name: &str) -> DatabaseResult<Option<String>> {
        let value: Option<String> = conn.query_row(
            "SELECT sv.value FROM settings_values sv 
             JOIN settings_keys sk ON sv.key_id = sk.id 
             WHERE sk.key_name = ?",
            params![key_name],
            |row| row.get(0)
        ).optional()?;
        
        Ok(value)
    }
    
    /// Set a UI setting value
    pub fn set_ui_setting(
        conn: &mut Connection,
        key_name: &str,
        value: &str,
        changed_by: &str,
        change_reason: Option<&str>,
    ) -> DatabaseResult<()> {
        let tx = conn.transaction()?;
        
        // Get the setting key
        let setting_key: SettingsKey = tx.query_row(
            "SELECT * FROM settings_keys WHERE key_name = ?",
            params![key_name],
            |row| SettingsKey::from_row(row)
        )?;
        
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
            
            tx.execute(
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
        
        tx.commit()?;
        Ok(())
    }
    
    /// Get all UI settings
    pub fn get_all_ui_settings(conn: &Connection) -> DatabaseResult<Vec<(String, String)>> {
        let mut stmt = conn.prepare(
            "SELECT sk.key_name, sv.value FROM settings_keys sk 
             LEFT JOIN settings_values sv ON sk.id = sv.key_id 
             JOIN settings_categories sc ON sk.category_id = sc.id 
             WHERE sc.name = 'ui' 
             ORDER BY sk.key_name"
        )?;
        
        let settings = stmt.query_map([], |row| {
            let key_name: String = row.get(0)?;
            let value: Option<String> = row.get(1)?;
            Ok((key_name, value.unwrap_or_default()))
        })?
        .collect::<SqliteResult<Vec<_>>>()?;
        
        Ok(settings)
    }
} 