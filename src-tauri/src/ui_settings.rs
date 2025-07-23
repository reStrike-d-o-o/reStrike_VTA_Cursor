use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use serde_json::Value;

// Simple UI Settings Store (temporary until database plugin is ready)
static UI_SETTINGS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
    let mut settings = HashMap::new();
    // Initialize with default UI settings
    settings.insert("window.position.x".to_string(), "100".to_string());
    settings.insert("window.position.y".to_string(), "100".to_string());
    settings.insert("window.size.width".to_string(), "1200".to_string());
    settings.insert("window.size.height".to_string(), "800".to_string());
    settings.insert("window.fullscreen".to_string(), "false".to_string());
    settings.insert("window.compact".to_string(), "false".to_string());
    settings.insert("theme.current".to_string(), "dark".to_string());
    settings.insert("theme.auto_theme".to_string(), "false".to_string());
    settings.insert("theme.high_contrast".to_string(), "false".to_string());
    settings.insert("layout.sidebar_position".to_string(), "left".to_string());
    settings.insert("layout.sidebar_width".to_string(), "300".to_string());
    settings.insert("layout.status_bar_visible".to_string(), "true".to_string());
    settings.insert("layout.task_bar_visible".to_string(), "true".to_string());
    settings.insert("advanced.show_advanced_panel".to_string(), "false".to_string());
    settings.insert("advanced.debug_mode".to_string(), "false".to_string());
    settings.insert("advanced.verbose_logging".to_string(), "false".to_string());
    settings.insert("animations.enabled".to_string(), "true".to_string());
    settings.insert("animations.duration_ms".to_string(), "300".to_string());
    settings.insert("animations.reduce_motion".to_string(), "false".to_string());
    Mutex::new(settings)
});

/// UI Settings Manager
pub struct UiSettingsManager;

impl UiSettingsManager {
    /// Initialize UI settings
    pub fn initialize() -> Result<Value, String> {
        log::info!("Initializing UI settings");
        
        let settings_count = UI_SETTINGS.lock().unwrap().len();
        
        Ok(serde_json::json!({
            "success": true,
            "message": "UI settings initialized successfully",
            "settings_count": settings_count
        }))
    }
    
    /// Get a UI setting
    pub fn get_setting(key_name: &str) -> Result<Value, String> {
        log::info!("Getting UI setting: {}", key_name);
        
        let settings = UI_SETTINGS.lock().unwrap();
        let value = settings.get(key_name).cloned();
        
        Ok(serde_json::json!({
            "success": true,
            "value": value,
            "key": key_name
        }))
    }
    
    /// Set a UI setting
    pub fn set_setting(key_name: &str, value: &str, changed_by: &str, change_reason: Option<&str>) -> Result<Value, String> {
        log::info!("Setting UI setting: {} = {} (by: {}, reason: {:?})", 
                   key_name, value, changed_by, change_reason);
        
        let mut settings = UI_SETTINGS.lock().unwrap();
        let old_value = settings.get(key_name).cloned();
        settings.insert(key_name.to_string(), value.to_string());
        
        Ok(serde_json::json!({
            "success": true,
            "message": "UI setting updated successfully",
            "key": key_name,
            "old_value": old_value,
            "new_value": value,
            "changed_by": changed_by,
            "change_reason": change_reason
        }))
    }
    
    /// Get all UI settings
    pub fn get_all_settings() -> Result<Value, String> {
        log::info!("Getting all UI settings");
        
        let settings = UI_SETTINGS.lock().unwrap();
        let settings_map: HashMap<String, String> = settings.clone();
        
        Ok(serde_json::json!({
            "success": true,
            "settings": settings_map,
            "count": settings.len()
        }))
    }
    
    /// Migrate settings from JSON (placeholder for future implementation)
    pub fn migrate_from_json() -> Result<Value, String> {
        log::info!("Migrating settings from JSON");
        
        let settings_count = UI_SETTINGS.lock().unwrap().len();
        
        Ok(serde_json::json!({
            "success": true,
            "migrated_count": settings_count,
            "errors": [],
            "message": format!("{} settings available in memory store", settings_count)
        }))
    }
} 