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

/// PSS and UDP Subsystem Operations
pub struct PssUdpOperations;

impl PssUdpOperations {
    // Network Interface Operations
    
    /// Get all network interfaces
    pub fn get_network_interfaces(conn: &Connection) -> DatabaseResult<Vec<crate::database::models::NetworkInterface>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM network_interfaces ORDER BY is_recommended DESC, is_active DESC, name"
        )?;
        
        let interfaces = stmt.query_map([], |row| {
            crate::database::models::NetworkInterface::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(interfaces)
    }
    
    /// Get recommended network interface
    pub fn get_recommended_interface(conn: &Connection) -> DatabaseResult<Option<crate::database::models::NetworkInterface>> {
        let interface = conn.query_row(
            "SELECT * FROM network_interfaces WHERE is_recommended = 1 AND is_active = 1 LIMIT 1",
            [],
            |row| crate::database::models::NetworkInterface::from_row(row)
        ).optional()?;
        
        Ok(interface)
    }
    
    /// Add or update network interface
    pub fn upsert_network_interface(conn: &mut Connection, interface: &crate::database::models::NetworkInterface) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;
        
        let interface_id = if let Some(id) = interface.id {
            // Update existing interface
            tx.execute(
                "UPDATE network_interfaces SET 
                    name = ?, address = ?, netmask = ?, broadcast = ?, is_loopback = ?, 
                    is_active = ?, is_recommended = ?, speed_mbps = ?, mtu = ?, 
                    mac_address = ?, interface_type = ?, updated_at = ?
                WHERE id = ?",
                params![
                    interface.name,
                    interface.address,
                    interface.netmask,
                    interface.broadcast,
                    interface.is_loopback,
                    interface.is_active,
                    interface.is_recommended,
                    interface.speed_mbps,
                    interface.mtu,
                    interface.mac_address,
                    interface.interface_type,
                    Utc::now().to_rfc3339(),
                    id
                ]
            )?;
            id
        } else {
            // Insert new interface
            tx.execute(
                "INSERT INTO network_interfaces (
                    name, address, netmask, broadcast, is_loopback, is_active, is_recommended,
                    speed_mbps, mtu, mac_address, interface_type, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    interface.name,
                    interface.address,
                    interface.netmask,
                    interface.broadcast,
                    interface.is_loopback,
                    interface.is_active,
                    interface.is_recommended,
                    interface.speed_mbps,
                    interface.mtu,
                    interface.mac_address,
                    interface.interface_type,
                    interface.created_at.to_rfc3339(),
                    interface.updated_at.to_rfc3339()
                ]
            )?;
            tx.last_insert_rowid()
        };
        
        tx.commit()?;
        Ok(interface_id)
    }
    
    // UDP Server Configuration Operations
    
    /// Get all UDP server configurations
    pub fn get_udp_server_configs(conn: &Connection) -> DatabaseResult<Vec<crate::database::models::UdpServerConfig>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM udp_server_configs ORDER BY name"
        )?;
        
        let configs = stmt.query_map([], |row| {
            crate::database::models::UdpServerConfig::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(configs)
    }
    
    /// Get UDP server configuration by ID
    pub fn get_udp_server_config(conn: &Connection, config_id: i64) -> DatabaseResult<Option<crate::database::models::UdpServerConfig>> {
        let config = conn.query_row(
            "SELECT * FROM udp_server_configs WHERE id = ?",
            params![config_id],
            |row| crate::database::models::UdpServerConfig::from_row(row)
        ).optional()?;
        
        Ok(config)
    }
    
    /// Add or update UDP server configuration
    pub fn upsert_udp_server_config(conn: &mut Connection, config: &crate::database::models::UdpServerConfig) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;
        
        let config_id = if let Some(id) = config.id {
            // Update existing config
            tx.execute(
                "UPDATE udp_server_configs SET 
                    name = ?, port = ?, bind_address = ?, network_interface_id = ?, 
                    enabled = ?, auto_start = ?, max_packet_size = ?, buffer_size = ?, 
                    timeout_ms = ?, updated_at = ?
                WHERE id = ?",
                params![
                    config.name,
                    config.port,
                    config.bind_address,
                    config.network_interface_id,
                    config.enabled,
                    config.auto_start,
                    config.max_packet_size,
                    config.buffer_size,
                    config.timeout_ms,
                    Utc::now().to_rfc3339(),
                    id
                ]
            )?;
            id
        } else {
            // Check if config with same name exists
            let existing_id: Option<i64> = tx.query_row(
                "SELECT id FROM udp_server_configs WHERE name = ?",
                params![config.name],
                |row| row.get(0)
            ).optional()?;
            
            if let Some(existing_id) = existing_id {
                // Update existing config
                tx.execute(
                    "UPDATE udp_server_configs SET 
                        port = ?, bind_address = ?, network_interface_id = ?, 
                        enabled = ?, auto_start = ?, max_packet_size = ?, buffer_size = ?, 
                        timeout_ms = ?, updated_at = ?
                    WHERE id = ?",
                    params![
                        config.port,
                        config.bind_address,
                        config.network_interface_id,
                        config.enabled,
                        config.auto_start,
                        config.max_packet_size,
                        config.buffer_size,
                        config.timeout_ms,
                        Utc::now().to_rfc3339(),
                        existing_id
                    ]
                )?;
                existing_id
            } else {
                // Insert new config
                tx.execute(
                    "INSERT INTO udp_server_configs (
                        name, port, bind_address, network_interface_id, enabled, auto_start,
                        max_packet_size, buffer_size, timeout_ms, created_at, updated_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        config.name,
                        config.port,
                        config.bind_address,
                        config.network_interface_id,
                        config.enabled,
                        config.auto_start,
                        config.max_packet_size,
                        config.buffer_size,
                        config.timeout_ms,
                        config.created_at.to_rfc3339(),
                        config.updated_at.to_rfc3339()
                    ]
                )?;
                tx.last_insert_rowid()
            }
        };
        
        tx.commit()?;
        Ok(config_id)
    }
    
    // UDP Server Session Operations
    
    /// Create new UDP server session
    pub fn create_udp_server_session(conn: &mut Connection, server_config_id: i64) -> DatabaseResult<i64> {
        let session = crate::database::models::UdpServerSession::new(server_config_id);
        
        let session_id = conn.execute(
            "INSERT INTO udp_server_sessions (
                server_config_id, start_time, status, packets_received, packets_parsed,
                parse_errors, total_bytes_received, average_packet_size, max_packet_size_seen,
                min_packet_size_seen, unique_clients_count
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                session.server_config_id,
                session.start_time.to_rfc3339(),
                session.status,
                session.packets_received,
                session.packets_parsed,
                session.parse_errors,
                session.total_bytes_received,
                session.average_packet_size,
                session.max_packet_size_seen,
                session.min_packet_size_seen,
                session.unique_clients_count
            ]
        )?;
        
        Ok(session_id as i64)
    }
    
    /// Update UDP server session statistics
    pub fn update_udp_server_session_stats(
        conn: &mut Connection,
        session_id: i64,
        packets_received: i32,
        packets_parsed: i32,
        parse_errors: i32,
        total_bytes_received: i32,
        average_packet_size: f64,
        max_packet_size_seen: i32,
        min_packet_size_seen: i32,
        unique_clients_count: i32,
    ) -> DatabaseResult<()> {
        conn.execute(
            "UPDATE udp_server_sessions SET 
                packets_received = ?, packets_parsed = ?, parse_errors = ?, 
                total_bytes_received = ?, average_packet_size = ?, max_packet_size_seen = ?,
                min_packet_size_seen = ?, unique_clients_count = ?
            WHERE id = ?",
            params![
                packets_received,
                packets_parsed,
                parse_errors,
                total_bytes_received,
                average_packet_size,
                max_packet_size_seen,
                min_packet_size_seen,
                unique_clients_count,
                session_id
            ]
        )?;
        
        Ok(())
    }
    
    /// End UDP server session
    pub fn end_udp_server_session(conn: &mut Connection, session_id: i64, status: &str, error_message: Option<&str>) -> DatabaseResult<()> {
        conn.execute(
            "UPDATE udp_server_sessions SET 
                end_time = ?, status = ?, error_message = ?
            WHERE id = ?",
            params![
                Utc::now().to_rfc3339(),
                status,
                error_message,
                session_id
            ]
        )?;
        
        Ok(())
    }
    
    /// Get UDP server session by ID
    pub fn get_udp_server_session(conn: &Connection, session_id: i64) -> DatabaseResult<Option<crate::database::models::UdpServerSession>> {
        let session = conn.query_row(
            "SELECT * FROM udp_server_sessions WHERE id = ?",
            params![session_id],
            |row| crate::database::models::UdpServerSession::from_row(row)
        ).optional()?;
        
        Ok(session)
    }
    
    /// Get recent UDP server sessions
    pub fn get_recent_udp_server_sessions(conn: &Connection, limit: i64) -> DatabaseResult<Vec<crate::database::models::UdpServerSession>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM udp_server_sessions ORDER BY start_time DESC LIMIT ?"
        )?;
        
        let sessions = stmt.query_map(params![limit], |row| {
            crate::database::models::UdpServerSession::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(sessions)
    }
    
    // UDP Client Connection Operations
    
    /// Add or update UDP client connection
    pub fn upsert_udp_client_connection(conn: &mut Connection, client: &crate::database::models::UdpClientConnection) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;
        
        let client_id = if let Some(id) = client.id {
            // Update existing client connection
            tx.execute(
                "UPDATE udp_client_connections SET 
                    last_seen = ?, packets_received = ?, total_bytes_received = ?, is_active = ?
                WHERE id = ?",
                params![
                    client.last_seen.to_rfc3339(),
                    client.packets_received,
                    client.total_bytes_received,
                    client.is_active,
                    id
                ]
            )?;
            id
        } else {
            // Insert new client connection
            tx.execute(
                "INSERT INTO udp_client_connections (
                    session_id, client_address, client_port, first_seen, last_seen,
                    packets_received, total_bytes_received, is_active
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    client.session_id,
                    client.client_address,
                    client.client_port,
                    client.first_seen.to_rfc3339(),
                    client.last_seen.to_rfc3339(),
                    client.packets_received,
                    client.total_bytes_received,
                    client.is_active
                ]
            )?;
            tx.last_insert_rowid()
        };
        
        tx.commit()?;
        Ok(client_id)
    }
    
    /// Get active client connections for a session
    pub fn get_active_client_connections(conn: &Connection, session_id: i64) -> DatabaseResult<Vec<crate::database::models::UdpClientConnection>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM udp_client_connections WHERE session_id = ? AND is_active = 1 ORDER BY last_seen DESC"
        )?;
        
        let clients = stmt.query_map(params![session_id], |row| {
            crate::database::models::UdpClientConnection::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(clients)
    }
    
    // PSS Event Type Operations
    
    /// Get all PSS event types
    pub fn get_pss_event_types(conn: &Connection) -> DatabaseResult<Vec<crate::database::models::PssEventType>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_event_types WHERE is_active = 1 ORDER BY category, event_code"
        )?;
        
        let event_types = stmt.query_map([], |row| {
            crate::database::models::PssEventType::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(event_types)
    }
    
    /// Get PSS event type by code
    pub fn get_pss_event_type_by_code(conn: &Connection, event_code: &str) -> DatabaseResult<Option<crate::database::models::PssEventType>> {
        let event_type = conn.query_row(
            "SELECT * FROM pss_event_types WHERE event_code = ? AND is_active = 1",
            params![event_code],
            |row| crate::database::models::PssEventType::from_row(row)
        ).optional()?;
        
        Ok(event_type)
    }
    
    // PSS Match Operations
    
    /// Get or create PSS match
    pub fn get_or_create_pss_match(conn: &mut Connection, match_id: &str) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;
        
        // Try to get existing match
        let existing_match_id: Option<i64> = tx.query_row(
            "SELECT id FROM pss_matches WHERE match_id = ?",
            params![match_id],
            |row| row.get(0)
        ).optional()?;
        
        let match_id = if let Some(id) = existing_match_id {
            id
        } else {
            // Create new match
            let match_obj = crate::database::models::PssMatch::new(match_id.to_string());
            tx.execute(
                "INSERT INTO pss_matches (
                    match_id, total_rounds, created_at, updated_at
                ) VALUES (?, ?, ?, ?)",
                params![
                    match_obj.match_id,
                    match_obj.total_rounds,
                    match_obj.created_at.to_rfc3339(),
                    match_obj.updated_at.to_rfc3339()
                ]
            )?;
            tx.last_insert_rowid()
        };
        
        tx.commit()?;
        Ok(match_id)
    }
    
    /// Update PSS match information
    pub fn update_pss_match(conn: &mut Connection, match_id: i64, match_data: &crate::database::models::PssMatch) -> DatabaseResult<()> {
        conn.execute(
            "UPDATE pss_matches SET 
                match_number = ?, category = ?, weight_class = ?, division = ?,
                total_rounds = ?, round_duration = ?, countdown_type = ?, format_type = ?, updated_at = ?
            WHERE id = ?",
            params![
                match_data.match_number,
                match_data.category,
                match_data.weight_class,
                match_data.division,
                match_data.total_rounds,
                match_data.round_duration,
                match_data.countdown_type,
                match_data.format_type,
                Utc::now().to_rfc3339(),
                match_id
            ]
        )?;
        
        Ok(())
    }
    
    // PSS Athlete Operations
    
    /// Get or create PSS athlete
    pub fn get_or_create_pss_athlete(conn: &mut Connection, athlete_code: &str, short_name: &str) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;
        
        // Try to get existing athlete
        let existing_athlete_id: Option<i64> = tx.query_row(
            "SELECT id FROM pss_athletes WHERE athlete_code = ?",
            params![athlete_code],
            |row| row.get(0)
        ).optional()?;
        
        let athlete_id = if let Some(id) = existing_athlete_id {
            id
        } else {
            // Create new athlete
            let athlete = crate::database::models::PssAthlete::new(athlete_code.to_string(), short_name.to_string());
            tx.execute(
                "INSERT INTO pss_athletes (
                    athlete_code, short_name, created_at, updated_at
                ) VALUES (?, ?, ?, ?)",
                params![
                    athlete.athlete_code,
                    athlete.short_name,
                    athlete.created_at.to_rfc3339(),
                    athlete.updated_at.to_rfc3339()
                ]
            )?;
            tx.last_insert_rowid()
        };
        
        tx.commit()?;
        Ok(athlete_id)
    }
    
    /// Update PSS athlete information
    pub fn update_pss_athlete(conn: &mut Connection, athlete_id: i64, athlete_data: &crate::database::models::PssAthlete) -> DatabaseResult<()> {
        conn.execute(
            "UPDATE pss_athletes SET 
                long_name = ?, country_code = ?, flag_id = ?, updated_at = ?
            WHERE id = ?",
            params![
                athlete_data.long_name,
                athlete_data.country_code,
                athlete_data.flag_id,
                Utc::now().to_rfc3339(),
                athlete_id
            ]
        )?;
        
        Ok(())
    }
    
    // PSS Event Operations
    
    /// Store PSS event
    pub fn store_pss_event(conn: &mut Connection, event: &crate::database::models::PssEventV2) -> DatabaseResult<i64> {
        let event_id = conn.execute(
            "INSERT INTO pss_events_v2 (
                session_id, match_id, round_id, event_type_id, timestamp, raw_data,
                parsed_data, event_sequence, processing_time_ms, is_valid, error_message, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                event.session_id,
                event.match_id,
                event.round_id,
                event.event_type_id,
                event.timestamp.to_rfc3339(),
                event.raw_data,
                event.parsed_data,
                event.event_sequence,
                event.processing_time_ms,
                event.is_valid,
                event.error_message,
                event.created_at.to_rfc3339()
            ]
        )?;
        
        Ok(event_id as i64)
    }
    
    /// Get PSS events for a session
    pub fn get_pss_events_for_session(conn: &Connection, session_id: i64, limit: Option<i64>) -> DatabaseResult<Vec<crate::database::models::PssEventV2>> {
        let limit = limit.unwrap_or(100);
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_events_v2 WHERE session_id = ? ORDER BY event_sequence DESC LIMIT ?"
        )?;
        
        let events = stmt.query_map(params![session_id, limit], |row| {
            crate::database::models::PssEventV2::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(events)
    }
    
    /// Get PSS events for a match
    pub fn get_pss_events_for_match(conn: &Connection, match_id: i64, limit: Option<i64>) -> DatabaseResult<Vec<crate::database::models::PssEventV2>> {
        let limit = limit.unwrap_or(100);
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_events_v2 WHERE match_id = ? ORDER BY timestamp DESC LIMIT ?"
        )?;
        
        let events = stmt.query_map(params![match_id, limit], |row| {
            crate::database::models::PssEventV2::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(events)
    }
    
    // PSS Event Detail Operations
    
    /// Store PSS event details
    pub fn store_pss_event_details(conn: &mut Connection, event_id: i64, details: &[(String, Option<String>, String)]) -> DatabaseResult<()> {
        let tx = conn.transaction()?;
        
        for (key, value, detail_type) in details {
            let detail = crate::database::models::PssEventDetail::new(
                event_id,
                key.clone(),
                value.clone(),
                detail_type.clone(),
            );
            
            tx.execute(
                "INSERT INTO pss_event_details (event_id, detail_key, detail_value, detail_type, created_at) VALUES (?, ?, ?, ?, ?)",
                params![
                    detail.event_id,
                    detail.detail_key,
                    detail.detail_value,
                    detail.detail_type,
                    detail.created_at.to_rfc3339()
                ]
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }
    
    /// Get PSS event details
    pub fn get_pss_event_details(conn: &Connection, event_id: i64) -> DatabaseResult<Vec<crate::database::models::PssEventDetail>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_event_details WHERE event_id = ? ORDER BY detail_key"
        )?;
        
        let details = stmt.query_map(params![event_id], |row| {
            crate::database::models::PssEventDetail::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(details)
    }
    
    // PSS Score Operations
    
    /// Store PSS score
    pub fn store_pss_score(conn: &mut Connection, score: &crate::database::models::PssScore) -> DatabaseResult<i64> {
        let score_id = conn.execute(
            "INSERT INTO pss_scores (
                match_id, round_id, athlete_position, score_type, score_value, timestamp, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                score.match_id,
                score.round_id,
                score.athlete_position,
                score.score_type,
                score.score_value,
                score.timestamp.to_rfc3339(),
                score.created_at.to_rfc3339()
            ]
        )?;
        
        Ok(score_id as i64)
    }
    
    /// Get current scores for a match
    pub fn get_current_scores_for_match(conn: &Connection, match_id: i64) -> DatabaseResult<Vec<crate::database::models::PssScore>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_scores WHERE match_id = ? AND score_type = 'current' ORDER BY timestamp DESC LIMIT 2"
        )?;
        
        let scores = stmt.query_map(params![match_id], |row| {
            crate::database::models::PssScore::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(scores)
    }
    
    // PSS Warning Operations
    
    /// Store PSS warning
    pub fn store_pss_warning(conn: &mut Connection, warning: &crate::database::models::PssWarning) -> DatabaseResult<i64> {
        let warning_id = conn.execute(
            "INSERT INTO pss_warnings (
                match_id, round_id, athlete_position, warning_type, warning_count, timestamp, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                warning.match_id,
                warning.round_id,
                warning.athlete_position,
                warning.warning_type,
                warning.warning_count,
                warning.timestamp.to_rfc3339(),
                warning.created_at.to_rfc3339()
            ]
        )?;
        
        Ok(warning_id as i64)
    }
    
    /// Get current warnings for a match
    pub fn get_current_warnings_for_match(conn: &Connection, match_id: i64) -> DatabaseResult<Vec<crate::database::models::PssWarning>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_warnings WHERE match_id = ? ORDER BY timestamp DESC"
        )?;
        
        let warnings = stmt.query_map(params![match_id], |row| {
            crate::database::models::PssWarning::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(warnings)
    }
    
    // Statistics and Analytics
    
    /// Get UDP server statistics
    pub fn get_udp_server_statistics(conn: &Connection) -> DatabaseResult<serde_json::Value> {
        // Get total sessions
        let total_sessions: i64 = conn.query_row(
            "SELECT COUNT(*) FROM udp_server_sessions",
            [],
            |row| row.get(0)
        )?;
        
        // Get active sessions
        let active_sessions: i64 = conn.query_row(
            "SELECT COUNT(*) FROM udp_server_sessions WHERE status = 'running'",
            [],
            |row| row.get(0)
        )?;
        
        // Get total events
        let total_events: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pss_events_v2",
            [],
            |row| row.get(0)
        )?;
        
        // Get total matches
        let total_matches: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pss_matches",
            [],
            |row| row.get(0)
        )?;
        
        // Get recent activity (last 24 hours)
        let recent_events: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pss_events_v2 WHERE created_at > datetime('now', '-1 day')",
            [],
            |row| row.get(0)
        )?;
        
        Ok(serde_json::json!({
            "total_sessions": total_sessions,
            "active_sessions": active_sessions,
            "total_events": total_events,
            "total_matches": total_matches,
            "recent_events_24h": recent_events
        }))
    }
} 

/// Tournament Operations for managing tournaments and tournament days
pub struct TournamentOperations;

impl TournamentOperations {
    /// Check if tournament name already exists
    pub fn tournament_name_exists(conn: &Connection, name: &str) -> DatabaseResult<bool> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM tournaments WHERE name = ?",
            params![name],
            |row| row.get(0)
        )?;
        
        Ok(count > 0)
    }

    /// Create a new tournament
    pub fn create_tournament(conn: &mut Connection, tournament: &crate::database::models::Tournament) -> DatabaseResult<i64> {
        // Check if tournament name already exists
        if Self::tournament_name_exists(conn, &tournament.name)? {
            return Err(crate::database::DatabaseError::Sqlite(
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE),
                    Some("Tournament name already exists".to_string())
                )
            ));
        }

        let tournament_id = conn.execute(
            "INSERT INTO tournaments (name, duration_days, city, country, country_code, logo_path, status, start_date, end_date, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                tournament.name,
                tournament.duration_days,
                tournament.city,
                tournament.country,
                tournament.country_code,
                tournament.logo_path,
                tournament.status,
                tournament.start_date.map(|d| d.to_rfc3339()),
                tournament.end_date.map(|d| d.to_rfc3339()),
                tournament.created_at.to_rfc3339(),
                tournament.updated_at.to_rfc3339(),
            ]
        )?;
        
        Ok(tournament_id as i64)
    }
    
    /// Get all tournaments
    pub fn get_tournaments(conn: &Connection) -> DatabaseResult<Vec<crate::database::models::Tournament>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, duration_days, city, country, country_code, logo_path, status, start_date, end_date, created_at, updated_at FROM tournaments ORDER BY created_at DESC"
        )?;
        
        let rows = stmt.query_map([], |row| crate::database::models::Tournament::from_row(row))?;
        
        let mut tournaments = Vec::new();
        for row in rows {
            tournaments.push(row?);
        }
        
        Ok(tournaments)
    }
    
    /// Get tournament by ID
    pub fn get_tournament(conn: &Connection, tournament_id: i64) -> DatabaseResult<Option<crate::database::models::Tournament>> {
        let tournament = conn.query_row(
            "SELECT id, name, duration_days, city, country, country_code, logo_path, status, start_date, end_date, created_at, updated_at FROM tournaments WHERE id = ?",
            params![tournament_id],
            |row| crate::database::models::Tournament::from_row(row)
        ).optional()?;
        
        Ok(tournament)
    }
    
    /// Update tournament
    pub fn update_tournament(conn: &mut Connection, tournament_id: i64, tournament: &crate::database::models::Tournament) -> DatabaseResult<()> {
        conn.execute(
            "UPDATE tournaments SET name = ?, duration_days = ?, city = ?, country = ?, country_code = ?, logo_path = ?, status = ?, start_date = ?, end_date = ?, updated_at = ? WHERE id = ?",
            params![
                tournament.name,
                tournament.duration_days,
                tournament.city,
                tournament.country,
                tournament.country_code,
                tournament.logo_path,
                tournament.status,
                tournament.start_date.map(|d| d.to_rfc3339()),
                tournament.end_date.map(|d| d.to_rfc3339()),
                Utc::now().to_rfc3339(),
                tournament_id,
            ]
        )?;
        
        Ok(())
    }
    
    /// Delete tournament
    pub fn delete_tournament(conn: &mut Connection, tournament_id: i64) -> DatabaseResult<()> {
        conn.execute("DELETE FROM tournaments WHERE id = ?", params![tournament_id])?;
        Ok(())
    }
    
    /// Create tournament days for a tournament
    pub fn create_tournament_days(conn: &mut Connection, tournament_id: i64, start_date: chrono::DateTime<chrono::Utc>, duration_days: i32) -> DatabaseResult<()> {
        let tx = conn.transaction()?;
        
        for day_number in 1..=duration_days {
            let day_date = start_date + chrono::Duration::days((day_number - 1) as i64);
            let tournament_day = crate::database::models::TournamentDay::new(tournament_id, day_number, day_date);
            
            tx.execute(
                "INSERT INTO tournament_days (tournament_id, day_number, date, status, start_time, end_time, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    tournament_day.tournament_id,
                    tournament_day.day_number,
                    tournament_day.date.to_rfc3339(),
                    tournament_day.status,
                    tournament_day.start_time.map(|t| t.to_rfc3339()),
                    tournament_day.end_time.map(|t| t.to_rfc3339()),
                    tournament_day.created_at.to_rfc3339(),
                    tournament_day.updated_at.to_rfc3339(),
                ]
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }
    
    /// Get tournament days for a tournament
    pub fn get_tournament_days(conn: &Connection, tournament_id: i64) -> DatabaseResult<Vec<crate::database::models::TournamentDay>> {
        let mut stmt = conn.prepare(
            "SELECT id, tournament_id, day_number, date, status, start_time, end_time, created_at, updated_at FROM tournament_days WHERE tournament_id = ? ORDER BY day_number"
        )?;
        
        let rows = stmt.query_map(params![tournament_id], |row| crate::database::models::TournamentDay::from_row(row))?;
        
        let mut days = Vec::new();
        for row in rows {
            days.push(row?);
        }
        
        Ok(days)
    }
    
    /// Start a tournament day
    pub fn start_tournament_day(conn: &mut Connection, tournament_day_id: i64) -> DatabaseResult<()> {
        let now = Utc::now();
        
        // Update the tournament day status
        conn.execute(
            "UPDATE tournament_days SET status = ?, start_time = ?, updated_at = ? WHERE id = ?",
            params!["active", now.to_rfc3339(), now.to_rfc3339(), tournament_day_id]
        )?;
        
        // Check if this is the first day and start the tournament
        let tournament_id: i64 = conn.query_row(
            "SELECT tournament_id FROM tournament_days WHERE id = ?",
            params![tournament_day_id],
            |row| row.get(0)
        )?;
        
        let day_number: i32 = conn.query_row(
            "SELECT day_number FROM tournament_days WHERE id = ?",
            params![tournament_day_id],
            |row| row.get(0)
        )?;
        
        if day_number == 1 {
            conn.execute(
                "UPDATE tournaments SET status = ?, start_date = ?, updated_at = ? WHERE id = ?",
                params!["active", now.to_rfc3339(), now.to_rfc3339(), tournament_id]
            )?;
        }
        
        Ok(())
    }
    
    /// End a tournament day
    pub fn end_tournament_day(conn: &mut Connection, tournament_day_id: i64) -> DatabaseResult<()> {
        let now = Utc::now();
        
        // Update the tournament day status
        conn.execute(
            "UPDATE tournament_days SET status = ?, end_time = ?, updated_at = ? WHERE id = ?",
            params!["completed", now.to_rfc3339(), now.to_rfc3339(), tournament_day_id]
        )?;
        
        // Check if this is the last day and end the tournament
        let tournament_id: i64 = conn.query_row(
            "SELECT tournament_id FROM tournament_days WHERE id = ?",
            params![tournament_day_id],
            |row| row.get(0)
        )?;
        
        let day_number: i32 = conn.query_row(
            "SELECT day_number FROM tournament_days WHERE id = ?",
            params![tournament_day_id],
            |row| row.get(0)
        )?;
        
        let total_days: i32 = conn.query_row(
            "SELECT duration_days FROM tournaments WHERE id = ?",
            params![tournament_id],
            |row| row.get(0)
        )?;
        
        if day_number == total_days {
            conn.execute(
                "UPDATE tournaments SET status = ?, end_date = ?, updated_at = ? WHERE id = ?",
                params!["ended", now.to_rfc3339(), now.to_rfc3339(), tournament_id]
            )?;
        }
        
        Ok(())
    }
    
    /// Get active tournament
    pub fn get_active_tournament(conn: &Connection) -> DatabaseResult<Option<crate::database::models::Tournament>> {
        let tournament = conn.query_row(
            "SELECT id, name, duration_days, city, country, country_code, logo_path, status, start_date, end_date, created_at, updated_at FROM tournaments WHERE status = 'active' ORDER BY created_at DESC LIMIT 1",
            [],
            |row| crate::database::models::Tournament::from_row(row)
        ).optional()?;
        
        Ok(tournament)
    }
    
    /// Get active tournament day
    pub fn get_active_tournament_day(conn: &Connection, tournament_id: i64) -> DatabaseResult<Option<crate::database::models::TournamentDay>> {
        let day = conn.query_row(
            "SELECT id, tournament_id, day_number, date, status, start_time, end_time, created_at, updated_at FROM tournament_days WHERE tournament_id = ? AND status = 'active' ORDER BY day_number DESC LIMIT 1",
            params![tournament_id],
            |row| crate::database::models::TournamentDay::from_row(row)
        ).optional()?;
        
        Ok(day)
    }
    
    /// Update tournament logo
    pub fn update_tournament_logo(conn: &mut Connection, tournament_id: i64, logo_path: &str) -> DatabaseResult<()> {
        conn.execute(
            "UPDATE tournaments SET logo_path = ?, updated_at = ? WHERE id = ?",
            params![logo_path, Utc::now().to_rfc3339(), tournament_id]
        )?;
        
        Ok(())
    }
} 