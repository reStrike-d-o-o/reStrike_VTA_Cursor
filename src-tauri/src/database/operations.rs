use rusqlite::{Connection, Result as SqliteResult, params, OptionalExtension};
use chrono::Utc;
use crate::database::{
    DatabaseResult,
    DatabaseConnection,
    models::{SettingsKey, SettingsValue, SettingsHistory, SettingsCategory,
        Tournament, TournamentDay, NetworkInterface, UdpServerConfig, UdpServerSession, 
        UdpClientConnection, PssEventType, PssMatch, PssAthlete, PssMatchAthlete, PssEventV2, PssEventDetail, 
        PssScore, PssWarning, PssUnknownEvent, PssEventValidationRule, PssEventValidationResult, 
        PssEventStatistics, PssEventRecognitionHistory, ObsScene, OverlayTemplate, EventTrigger,
        ObsConnection, ObsRecordingConfig, ObsRecordingSession
    },
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
    pub fn get_network_interfaces(conn: &Connection) -> DatabaseResult<Vec<NetworkInterface>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM network_interfaces ORDER BY is_recommended DESC, is_active DESC, name"
        )?;
        
        let interfaces = stmt.query_map([], |row| {
            NetworkInterface::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(interfaces)
    }
    
    /// Get recommended network interface
    pub fn get_recommended_interface(conn: &Connection) -> DatabaseResult<Option<NetworkInterface>> {
        let interface = conn.query_row(
            "SELECT * FROM network_interfaces WHERE is_recommended = 1 AND is_active = 1 LIMIT 1",
            [],
            |row| NetworkInterface::from_row(row)
        ).optional()?;
        
        Ok(interface)
    }
    
    /// Add or update network interface
    pub fn upsert_network_interface(conn: &mut Connection, interface: &NetworkInterface) -> DatabaseResult<i64> {
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
    pub fn get_udp_server_configs(conn: &Connection) -> DatabaseResult<Vec<UdpServerConfig>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM udp_server_configs ORDER BY name"
        )?;
        
        let configs = stmt.query_map([], |row| {
            UdpServerConfig::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(configs)
    }
    
    /// Get UDP server configuration by ID
    pub fn get_udp_server_config(conn: &Connection, config_id: i64) -> DatabaseResult<Option<UdpServerConfig>> {
        let config = conn.query_row(
            "SELECT * FROM udp_server_configs WHERE id = ?",
            params![config_id],
            |row| UdpServerConfig::from_row(row)
        ).optional()?;
        
        Ok(config)
    }
    
    /// Add or update UDP server configuration
    pub fn upsert_udp_server_config(conn: &mut Connection, config: &UdpServerConfig) -> DatabaseResult<i64> {
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
        let session = UdpServerSession::new(server_config_id);
        
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
    pub fn get_udp_server_session(conn: &Connection, session_id: i64) -> DatabaseResult<Option<UdpServerSession>> {
        let session = conn.query_row(
            "SELECT * FROM udp_server_sessions WHERE id = ?",
            params![session_id],
            |row| UdpServerSession::from_row(row)
        ).optional()?;
        
        Ok(session)
    }
    
    /// Get recent UDP server sessions
    pub fn get_recent_udp_server_sessions(conn: &Connection, limit: i64) -> DatabaseResult<Vec<UdpServerSession>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM udp_server_sessions ORDER BY start_time DESC LIMIT ?"
        )?;
        
        let sessions = stmt.query_map(params![limit], |row| {
            UdpServerSession::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(sessions)
    }
    
    // UDP Client Connection Operations
    
    /// Add or update UDP client connection
    pub fn upsert_udp_client_connection(conn: &mut Connection, client: &UdpClientConnection) -> DatabaseResult<i64> {
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
    pub fn get_active_client_connections(conn: &Connection, session_id: i64) -> DatabaseResult<Vec<UdpClientConnection>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM udp_client_connections WHERE session_id = ? AND is_active = 1 ORDER BY last_seen DESC"
        )?;
        
        let clients = stmt.query_map(params![session_id], |row| {
            UdpClientConnection::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(clients)
    }
    
    // PSS Event Type Operations
    
    /// Get all PSS event types
    pub fn get_pss_event_types(conn: &Connection) -> DatabaseResult<Vec<PssEventType>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_event_types WHERE is_active = 1 ORDER BY category, event_code"
        )?;
        
        let event_types = stmt.query_map([], |row| {
            PssEventType::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(event_types)
    }
    
    /// Get PSS event type by code
    pub fn get_pss_event_type_by_code(conn: &Connection, event_code: &str) -> DatabaseResult<Option<PssEventType>> {
        let event_type = conn.query_row(
            "SELECT * FROM pss_event_types WHERE event_code = ? AND is_active = 1",
            params![event_code],
            |row| PssEventType::from_row(row)
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
            let match_obj = PssMatch::new(match_id.to_string());
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
    pub fn update_pss_match(conn: &mut Connection, match_id: i64, match_data: &PssMatch) -> DatabaseResult<()> {
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
            let athlete = PssAthlete::new(athlete_code.to_string(), short_name.to_string());
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
    pub fn update_pss_athlete(conn: &mut Connection, athlete_id: i64, athlete_data: &PssAthlete) -> DatabaseResult<()> {
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
    pub fn store_pss_event(conn: &mut Connection, event: &PssEventV2) -> DatabaseResult<i64> {
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
    pub fn get_pss_events_for_session(conn: &Connection, session_id: i64, limit: Option<i64>) -> DatabaseResult<Vec<PssEventV2>> {
        let limit = limit.unwrap_or(100);
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_events_v2 WHERE session_id = ? ORDER BY event_sequence DESC LIMIT ?"
        )?;
        
        let events = stmt.query_map(params![session_id, limit], |row| {
            PssEventV2::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(events)
    }
    
    /// Get PSS events for a match
    pub fn get_pss_events_for_match(conn: &Connection, match_id: i64, limit: Option<i64>) -> DatabaseResult<Vec<PssEventV2>> {
        let limit = limit.unwrap_or(100);
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_events_v2 WHERE match_id = ? ORDER BY timestamp DESC LIMIT ?"
        )?;
        
        let events = stmt.query_map(params![match_id, limit], |row| {
            PssEventV2::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(events)
    }
    
    // PSS Event Detail Operations
    
    /// Store PSS event details
    pub fn store_pss_event_details(conn: &mut Connection, event_id: i64, details: &[(String, Option<String>, String)]) -> DatabaseResult<()> {
        let tx = conn.transaction()?;
        
        for (key, value, detail_type) in details {
            let detail = PssEventDetail::new(
                event_id,
                key.clone(),
                value.clone(),
                detail_type.clone(),
            );
            
            // Use INSERT OR REPLACE to handle duplicate key violations gracefully
            tx.execute(
                "INSERT OR REPLACE INTO pss_event_details (event_id, detail_key, detail_value, detail_type, created_at) VALUES (?, ?, ?, ?, ?)",
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
    pub fn get_pss_event_details(conn: &Connection, event_id: i64) -> DatabaseResult<Vec<PssEventDetail>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_event_details WHERE event_id = ? ORDER BY detail_key"
        )?;
        
        let details = stmt.query_map(params![event_id], |row| {
            PssEventDetail::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(details)
    }
    
    // PSS Score Operations
    
    /// Store PSS score
    pub fn store_pss_score(conn: &mut Connection, score: &PssScore) -> DatabaseResult<i64> {
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
    pub fn get_current_scores_for_match(conn: &Connection, match_id: i64) -> DatabaseResult<Vec<PssScore>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_scores WHERE match_id = ? AND score_type = 'current' ORDER BY timestamp DESC LIMIT 2"
        )?;
        
        let scores = stmt.query_map(params![match_id], |row| {
            PssScore::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(scores)
    }
    
    // PSS Warning Operations
    
    /// Store PSS warning
    pub fn store_pss_warning(conn: &mut Connection, warning: &PssWarning) -> DatabaseResult<i64> {
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
    pub fn get_current_warnings_for_match(conn: &Connection, match_id: i64) -> DatabaseResult<Vec<PssWarning>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_warnings WHERE match_id = ? ORDER BY timestamp DESC"
        )?;
        
        let warnings = stmt.query_map(params![match_id], |row| {
            PssWarning::from_row(row)
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

    pub fn get_pss_matches(conn: &Connection, limit: Option<i64>) -> DatabaseResult<Vec<PssMatch>> {
        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        let query = format!(
            "SELECT * FROM pss_matches ORDER BY created_at DESC{}",
            limit_clause
        );
        
        let mut stmt = conn.prepare(&query)?;
        let matches = stmt.query_map([], |row| PssMatch::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(matches)
    }

    pub fn get_pss_matches_by_creation_mode(conn: &Connection, creation_mode: &str) -> DatabaseResult<Vec<PssMatch>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_matches WHERE creation_mode = ? ORDER BY created_at DESC"
        )?;
        
        let matches = stmt.query_map([creation_mode], |row| PssMatch::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(matches)
    }

    pub fn insert_pss_match(conn: &Connection, pss_match: &PssMatch) -> DatabaseResult<i64> {
        let match_id = conn.execute(
            "INSERT INTO pss_matches (match_id, match_number, category, weight_class, division, total_rounds, round_duration, countdown_type, format_type, creation_mode, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                pss_match.match_id,
                pss_match.match_number,
                pss_match.category,
                pss_match.weight_class,
                pss_match.division,
                pss_match.total_rounds,
                pss_match.round_duration,
                pss_match.countdown_type,
                pss_match.format_type,
                pss_match.creation_mode,
                pss_match.created_at.to_rfc3339(),
                pss_match.updated_at.to_rfc3339(),
            ],
        )?;
        
        Ok(match_id as i64)
    }

    pub fn insert_pss_athlete(conn: &Connection, athlete: &PssAthlete) -> DatabaseResult<i64> {
        let athlete_id = conn.execute(
            "INSERT INTO pss_athletes (athlete_code, short_name, long_name, country_code, flag_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                athlete.athlete_code,
                athlete.short_name,
                athlete.long_name,
                athlete.country_code,
                athlete.flag_id,
                athlete.created_at.to_rfc3339(),
                athlete.updated_at.to_rfc3339(),
            ],
        )?;
        
        Ok(athlete_id as i64)
    }

    pub fn insert_pss_match_athlete(conn: &Connection, match_athlete: &PssMatchAthlete) -> DatabaseResult<i64> {
        let match_athlete_id = conn.execute(
            "INSERT INTO pss_match_athletes (match_id, athlete_id, athlete_position, bg_color, fg_color, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            params![
                match_athlete.match_id,
                match_athlete.athlete_id,
                match_athlete.athlete_position,
                match_athlete.bg_color,
                match_athlete.fg_color,
                match_athlete.created_at.to_rfc3339(),
            ],
        )?;
        
        Ok(match_athlete_id as i64)
    }

    pub fn get_all_settings(conn: &Connection) -> DatabaseResult<serde_json::Value> {
        // Get all settings from the normalized settings system
        let mut stmt = conn.prepare(
            "SELECT c.name as category, k.key_name, k.display_name, v.value, k.data_type 
             FROM settings_categories c
             JOIN settings_keys k ON c.id = k.category_id
             LEFT JOIN settings_values v ON k.id = v.key_id
             ORDER BY c.display_order, k.key_name"
        )?;
        
        let settings = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "category": row.get::<_, String>(0)?,
                "key": row.get::<_, String>(1)?,
                "display_name": row.get::<_, String>(2)?,
                "value": row.get::<_, Option<String>>(3)?,
                "data_type": row.get::<_, String>(4)?
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(serde_json::json!({
            "settings": settings
        }))
    }

    pub fn get_obs_connections(conn: &Connection) -> DatabaseResult<Vec<ObsConnection>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, host, port, password, is_active, status, error, created_at, updated_at 
             FROM obs_connections ORDER BY name"
        )?;
        
        let connections = stmt.query_map([], |row| ObsConnection::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(connections)
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
    pub fn create_tournament(conn: &mut Connection, tournament: &Tournament) -> DatabaseResult<i64> {
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
    pub fn get_tournaments(conn: &Connection) -> DatabaseResult<Vec<Tournament>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, duration_days, city, country, country_code, logo_path, status, start_date, end_date, created_at, updated_at FROM tournaments ORDER BY created_at DESC"
        )?;
        
        let rows = stmt.query_map([], |row| Tournament::from_row(row))?;
        
        let mut tournaments = Vec::new();
        for row in rows {
            tournaments.push(row?);
        }
        
        Ok(tournaments)
    }
    
    /// Get tournament by ID
    pub fn get_tournament(conn: &Connection, tournament_id: i64) -> DatabaseResult<Option<Tournament>> {
        let tournament = conn.query_row(
            "SELECT id, name, duration_days, city, country, country_code, logo_path, status, start_date, end_date, created_at, updated_at FROM tournaments WHERE id = ?",
            params![tournament_id],
            |row| Tournament::from_row(row)
        ).optional()?;
        
        Ok(tournament)
    }
    
    /// Update tournament
    pub fn update_tournament(conn: &mut Connection, tournament_id: i64, tournament: &Tournament) -> DatabaseResult<()> {
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
            let tournament_day = TournamentDay::new(tournament_id, day_number, day_date);
            
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
    pub fn get_tournament_days(conn: &Connection, tournament_id: i64) -> DatabaseResult<Vec<TournamentDay>> {
        let mut stmt = conn.prepare(
            "SELECT id, tournament_id, day_number, date, status, start_time, end_time, created_at, updated_at FROM tournament_days WHERE tournament_id = ? ORDER BY day_number"
        )?;
        
        let rows = stmt.query_map(params![tournament_id], |row| TournamentDay::from_row(row))?;
        
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
    pub fn get_active_tournament(conn: &Connection) -> DatabaseResult<Option<Tournament>> {
        let tournament = conn.query_row(
            "SELECT id, name, duration_days, city, country, country_code, logo_path, status, start_date, end_date, created_at, updated_at FROM tournaments WHERE status = 'active' ORDER BY created_at DESC LIMIT 1",
            [],
            |row| Tournament::from_row(row)
        ).optional()?;
        
        Ok(tournament)
    }
    
    /// Get active tournament day
    pub fn get_active_tournament_day(conn: &Connection, tournament_id: i64) -> DatabaseResult<Option<TournamentDay>> {
        let day = conn.query_row(
            "SELECT id, tournament_id, day_number, date, status, start_time, end_time, created_at, updated_at FROM tournament_days WHERE tournament_id = ? AND status = 'active' ORDER BY day_number DESC LIMIT 1",
            params![tournament_id],
            |row| TournamentDay::from_row(row)
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

/// PSS Event Status Mark Operations for managing event recognition and validation
pub struct PssEventStatusOperations;

impl PssEventStatusOperations {
    /// Store a PSS event with status mark
    pub fn store_pss_event_with_status(
        conn: &mut Connection, 
        event: &PssEventV2
    ) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;
        
        let event_id = tx.execute(
            "INSERT INTO pss_events_v2 (
                session_id, match_id, round_id, event_type_id, timestamp, raw_data, 
                parsed_data, event_sequence, processing_time_ms, is_valid, error_message,
                recognition_status, protocol_version, parser_confidence, validation_errors, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
                event.recognition_status,
                event.protocol_version,
                event.parser_confidence,
                event.validation_errors,
                event.created_at.to_rfc3339()
            ]
        )?;
        
        tx.commit()?;
        Ok(event_id as i64)
    }

    /// Update event recognition status and record history
    pub fn update_event_recognition_status(
        conn: &mut Connection,
        event_id: i64,
        new_status: &str,
        changed_by: &str,
        change_reason: Option<&str>,
    ) -> DatabaseResult<()> {
        let tx = conn.transaction()?;
        
        // Get current status
        let current_status: String = tx.query_row(
            "SELECT recognition_status FROM pss_events_v2 WHERE id = ?",
            params![event_id],
            |row| row.get(0)
        )?;
        
        // Update event status
        tx.execute(
            "UPDATE pss_events_v2 SET recognition_status = ? WHERE id = ?",
            params![new_status, event_id]
        )?;
        
        // Record status change in history
        let history = PssEventRecognitionHistory::new(
            event_id,
            current_status,
            new_status.to_string(),
            changed_by.to_string(),
            "".to_string(), // We'll get raw_data separately if needed
        );
        
        tx.execute(
            "INSERT INTO pss_event_recognition_history (
                event_id, old_status, new_status, changed_by, change_reason, 
                protocol_version, raw_data, parsed_data, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                history.event_id,
                history.old_status,
                history.new_status,
                history.changed_by,
                change_reason,
                history.protocol_version,
                history.raw_data,
                history.parsed_data,
                history.created_at.to_rfc3339()
            ]
        )?;
        
        tx.commit()?;
        Ok(())
    }

    /// Store unknown event
    pub fn store_unknown_event(
        conn: &mut Connection,
        unknown_event: &PssUnknownEvent,
    ) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;
        
        // Check if this pattern already exists
        let existing_id: Option<i64> = tx.query_row(
            "SELECT id FROM pss_unknown_events WHERE session_id = ? AND raw_data = ?",
            params![unknown_event.session_id, unknown_event.raw_data],
            |row| row.get(0)
        ).optional()?;
        
        if let Some(existing_id) = existing_id {
            // Update existing record
            tx.execute(
                "UPDATE pss_unknown_events SET 
                    last_seen = ?, occurrence_count = occurrence_count + 1, updated_at = ?
                WHERE id = ?",
                params![
                    unknown_event.last_seen.to_rfc3339(),
                    unknown_event.updated_at.to_rfc3339(),
                    existing_id
                ]
            )?;
            tx.commit()?;
            Ok(existing_id)
        } else {
            // Insert new record
            let unknown_event_id = tx.execute(
                "INSERT INTO pss_unknown_events (
                    session_id, raw_data, first_seen, last_seen, occurrence_count,
                    pattern_hash, suggested_event_type, notes, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    unknown_event.session_id,
                    unknown_event.raw_data,
                    unknown_event.first_seen.to_rfc3339(),
                    unknown_event.last_seen.to_rfc3339(),
                    unknown_event.occurrence_count,
                    unknown_event.pattern_hash,
                    unknown_event.suggested_event_type,
                    unknown_event.notes,
                    unknown_event.created_at.to_rfc3339(),
                    unknown_event.updated_at.to_rfc3339()
                ]
            )?;
            
            tx.commit()?;
            Ok(unknown_event_id as i64)
        }
    }

    /// Get validation rules for an event type
    pub fn get_validation_rules(
        conn: &Connection,
        event_code: &str,
        protocol_version: &str,
    ) -> DatabaseResult<Vec<PssEventValidationRule>> {
        let mut stmt = conn.prepare(
            "SELECT id, event_code, protocol_version, rule_name, rule_type, rule_definition, 
                    error_message, is_active, created_at, updated_at 
             FROM pss_event_validation_rules 
             WHERE event_code = ? AND protocol_version = ? AND is_active = 1
             ORDER BY rule_name"
        )?;
        
        let rows = stmt.query_map(params![event_code, protocol_version], |row| {
            PssEventValidationRule::from_row(row)
        })?;
        
        let mut rules = Vec::new();
        for row in rows {
            rules.push(row?);
        }
        
        Ok(rules)
    }

    /// Store validation result
    pub fn store_validation_result(
        conn: &mut Connection,
        validation_result: &PssEventValidationResult,
    ) -> DatabaseResult<i64> {
        let validation_result_id = conn.execute(
            "INSERT INTO pss_event_validation_results (
                event_id, rule_id, validation_passed, error_message, validation_time_ms, created_at
            ) VALUES (?, ?, ?, ?, ?, ?)",
            params![
                validation_result.event_id,
                validation_result.rule_id,
                validation_result.validation_passed,
                validation_result.error_message,
                validation_result.validation_time_ms,
                validation_result.created_at.to_rfc3339()
            ]
        )?;
        
        Ok(validation_result_id as i64)
    }

    /// Update event statistics
    pub fn update_event_statistics(
        conn: &mut Connection,
        session_id: i64,
        event_type_id: Option<i64>,
        recognition_status: &str,
        processing_time_ms: Option<i32>,
    ) -> DatabaseResult<()> {
        let tx = conn.transaction()?;
        
        // Get or create statistics record
        let stats_id: Option<i64> = tx.query_row(
            "SELECT id FROM pss_event_statistics WHERE session_id = ? AND event_type_id IS ?",
            params![session_id, event_type_id],
            |row| row.get(0)
        ).optional()?;
        
        if let Some(stats_id) = stats_id {
            // Update existing statistics
            let update_sql = match recognition_status {
                "recognized" => "recognized_events = recognized_events + 1",
                "unknown" => "unknown_events = unknown_events + 1",
                "partial" => "partial_events = partial_events + 1",
                "deprecated" => "deprecated_events = deprecated_events + 1",
                _ => "total_events = total_events + 1",
            };
            
            tx.execute(
                &format!("UPDATE pss_event_statistics SET 
                    total_events = total_events + 1, 
                    {}, 
                    updated_at = ? 
                    WHERE id = ?", update_sql),
                params![chrono::Utc::now().to_rfc3339(), stats_id]
            )?;
            
            // Update processing time statistics if available
            if let Some(processing_time) = processing_time_ms {
                tx.execute(
                    "UPDATE pss_event_statistics SET 
                        average_processing_time_ms = (
                            (average_processing_time_ms * total_events + ?) / (total_events + 1)
                        ),
                        min_processing_time_ms = CASE 
                            WHEN min_processing_time_ms IS NULL OR ? < min_processing_time_ms 
                            THEN ? ELSE min_processing_time_ms END,
                        max_processing_time_ms = CASE 
                            WHEN max_processing_time_ms IS NULL OR ? > max_processing_time_ms 
                            THEN ? ELSE max_processing_time_ms END
                    WHERE id = ?",
                    params![processing_time, processing_time, processing_time, processing_time, processing_time, stats_id]
                )?;
            }
        } else {
            // Create new statistics record
            let stats = PssEventStatistics::new(session_id, event_type_id);
            let total_events = 1;
            let mut recognized_events = 0;
            let mut unknown_events = 0;
            let mut partial_events = 0;
            let mut deprecated_events = 0;
            
            match recognition_status {
                "recognized" => recognized_events = 1,
                "unknown" => unknown_events = 1,
                "partial" => partial_events = 1,
                "deprecated" => deprecated_events = 1,
                _ => {}
            }
            
            tx.execute(
                "INSERT INTO pss_event_statistics (
                    session_id, event_type_id, total_events, recognized_events, unknown_events,
                    partial_events, deprecated_events, validation_errors, parsing_errors,
                    average_processing_time_ms, min_processing_time_ms, max_processing_time_ms,
                    created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    stats.session_id,
                    stats.event_type_id,
                    total_events,
                    recognized_events,
                    unknown_events,
                    partial_events,
                    deprecated_events,
                    stats.validation_errors,
                    stats.parsing_errors,
                    processing_time_ms.unwrap_or(0) as f64,
                    processing_time_ms,
                    processing_time_ms,
                    stats.created_at.to_rfc3339(),
                    stats.updated_at.to_rfc3339()
                ]
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }

    /// Get event statistics for a session
    pub fn get_session_statistics(
        conn: &Connection,
        session_id: i64,
    ) -> DatabaseResult<Vec<PssEventStatistics>> {
        let mut stmt = conn.prepare(
            "SELECT id, session_id, event_type_id, total_events, recognized_events, unknown_events,
                    partial_events, deprecated_events, validation_errors, parsing_errors,
                    average_processing_time_ms, min_processing_time_ms, max_processing_time_ms,
                    created_at, updated_at
             FROM pss_event_statistics 
             WHERE session_id = ?
             ORDER BY total_events DESC"
        )?;
        
        let rows = stmt.query_map(params![session_id], |row| {
            PssEventStatistics::from_row(row)
        })?;
        
        let mut statistics = Vec::new();
        for row in rows {
            statistics.push(row?);
        }
        
        Ok(statistics)
    }

    /// Get unknown events for analysis
    pub fn get_unknown_events(
        conn: &Connection,
        session_id: Option<i64>,
        limit: Option<i64>,
    ) -> DatabaseResult<Vec<PssUnknownEvent>> {
        let limit = limit.unwrap_or(100);
        
        let sql = if let Some(_session_id) = session_id {
            "SELECT id, session_id, raw_data, first_seen, last_seen, occurrence_count,
                    pattern_hash, suggested_event_type, notes, created_at, updated_at
             FROM pss_unknown_events 
             WHERE session_id = ?
             ORDER BY occurrence_count DESC, last_seen DESC
             LIMIT ?"
        } else {
            "SELECT id, session_id, raw_data, first_seen, last_seen, occurrence_count,
                    pattern_hash, suggested_event_type, notes, created_at, updated_at
             FROM pss_unknown_events 
             ORDER BY occurrence_count DESC, last_seen DESC
             LIMIT ?"
        };
        
        let mut stmt = conn.prepare(sql)?;
        
        let rows = if let Some(session_id) = session_id {
            stmt.query_map(params![session_id, limit], PssUnknownEvent::from_row)?
        } else {
            stmt.query_map(params![limit], PssUnknownEvent::from_row)?
        };
        
        let mut unknown_events = Vec::new();
        for row in rows {
            unknown_events.push(row?);
        }
        
        Ok(unknown_events)
    }

    /// Get recognition history for an event
    pub fn get_event_recognition_history(
        conn: &Connection,
        event_id: i64,
    ) -> DatabaseResult<Vec<PssEventRecognitionHistory>> {
        let mut stmt = conn.prepare(
            "SELECT id, event_id, old_status, new_status, changed_by, change_reason,
                    protocol_version, raw_data, parsed_data, created_at
             FROM pss_event_recognition_history 
             WHERE event_id = ?
             ORDER BY created_at DESC"
        )?;
        
        let rows = stmt.query_map(params![event_id], |row| {
            PssEventRecognitionHistory::from_row(row)
        })?;
        
        let mut history = Vec::new();
        for row in rows {
            history.push(row?);
        }
        
        Ok(history)
    }

    /// Get events by recognition status
    pub fn get_events_by_status(
        conn: &Connection,
        session_id: i64,
        recognition_status: &str,
        limit: Option<i64>,
    ) -> DatabaseResult<Vec<PssEventV2>> {
        let limit = limit.unwrap_or(100);
        
        let mut stmt = conn.prepare(
            "SELECT id, session_id, match_id, round_id, event_type_id, timestamp, raw_data,
                    parsed_data, event_sequence, processing_time_ms, is_valid, error_message,
                    recognition_status, protocol_version, parser_confidence, validation_errors, created_at
             FROM pss_events_v2 
             WHERE session_id = ? AND recognition_status = ?
             ORDER BY created_at DESC
             LIMIT ?"
        )?;
        
        let rows = stmt.query_map(params![session_id, recognition_status, limit], |row| {
            PssEventV2::from_row(row)
        })?;
        
        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }
        
        Ok(events)
    }

    /// Get comprehensive event statistics with status breakdown
    pub fn get_comprehensive_event_statistics(
        conn: &Connection,
        session_id: i64,
    ) -> DatabaseResult<serde_json::Value> {
        // Get overall statistics
        let overall_stats = conn.query_row(
            "SELECT 
                COUNT(*) as total_events,
                SUM(CASE WHEN recognition_status = 'recognized' THEN 1 ELSE 0 END) as recognized_events,
                SUM(CASE WHEN recognition_status = 'unknown' THEN 1 ELSE 0 END) as unknown_events,
                SUM(CASE WHEN recognition_status = 'partial' THEN 1 ELSE 0 END) as partial_events,
                SUM(CASE WHEN recognition_status = 'deprecated' THEN 1 ELSE 0 END) as deprecated_events,
                AVG(parser_confidence) as avg_confidence,
                AVG(processing_time_ms) as avg_processing_time,
                MIN(processing_time_ms) as min_processing_time,
                MAX(processing_time_ms) as max_processing_time
            FROM pss_events_v2 
            WHERE session_id = ?",
            params![session_id],
            |row| {
                Ok(serde_json::json!({
                    "total_events": row.get::<_, i64>(0)?,
                    "recognized_events": row.get::<_, i64>(1)?,
                    "unknown_events": row.get::<_, i64>(2)?,
                    "partial_events": row.get::<_, i64>(3)?,
                    "deprecated_events": row.get::<_, i64>(4)?,
                    "avg_confidence": row.get::<_, Option<f64>>(5)?,
                    "avg_processing_time": row.get::<_, Option<f64>>(6)?,
                    "min_processing_time": row.get::<_, Option<i32>>(7)?,
                    "max_processing_time": row.get::<_, Option<i32>>(8)?
                }))
            }
        )?;

        // Get statistics by event type
        let mut event_type_stats = conn.prepare(
            "SELECT 
                et.event_code,
                et.event_name,
                COUNT(*) as total,
                SUM(CASE WHEN e.recognition_status = 'recognized' THEN 1 ELSE 0 END) as recognized,
                SUM(CASE WHEN e.recognition_status = 'unknown' THEN 1 ELSE 0 END) as unknown,
                SUM(CASE WHEN e.recognition_status = 'partial' THEN 1 ELSE 0 END) as partial,
                AVG(e.parser_confidence) as avg_confidence,
                AVG(e.processing_time_ms) as avg_processing_time
            FROM pss_events_v2 e
            JOIN pss_event_types et ON e.event_type_id = et.id
            WHERE e.session_id = ?
            GROUP BY et.id, et.event_code, et.event_name
            ORDER BY total DESC"
        )?;

        let event_type_rows = event_type_stats.query_map(params![session_id], |row| {
            Ok(serde_json::json!({
                "event_code": row.get::<_, String>(0)?,
                "event_name": row.get::<_, String>(1)?,
                "total": row.get::<_, i64>(2)?,
                "recognized": row.get::<_, i64>(3)?,
                "unknown": row.get::<_, i64>(4)?,
                "partial": row.get::<_, i64>(5)?,
                "avg_confidence": row.get::<_, Option<f64>>(6)?,
                "avg_processing_time": row.get::<_, Option<f64>>(7)?
            }))
        })?;

        let mut event_type_stats_vec = Vec::new();
        for row in event_type_rows {
            event_type_stats_vec.push(row?);
        }

        // Get validation error breakdown
        let mut validation_errors = conn.prepare(
            "SELECT 
                validation_errors,
                COUNT(*) as count
            FROM pss_events_v2 
            WHERE session_id = ? AND validation_errors IS NOT NULL
            GROUP BY validation_errors
            ORDER BY count DESC
            LIMIT 10"
        )?;

        let validation_rows = validation_errors.query_map(params![session_id], |row| {
            Ok(serde_json::json!({
                "error": row.get::<_, String>(0)?,
                "count": row.get::<_, i64>(1)?
            }))
        })?;

        let mut validation_errors_vec = Vec::new();
        for row in validation_rows {
            validation_errors_vec.push(row?);
        }

        // Get unknown events summary
        let unknown_events_summary = conn.query_row(
            "SELECT 
                COUNT(*) as total_unknown,
                COUNT(DISTINCT pattern_hash) as unique_patterns,
                MAX(occurrence_count) as max_occurrences
            FROM pss_unknown_events 
            WHERE session_id = ?",
            params![session_id],
            |row| {
                Ok(serde_json::json!({
                    "total_unknown": row.get::<_, i64>(0)?,
                    "unique_patterns": row.get::<_, i64>(1)?,
                    "max_occurrences": row.get::<_, i64>(2)?
                }))
            }
        ).unwrap_or_else(|_| serde_json::json!({
            "total_unknown": 0,
            "unique_patterns": 0,
            "max_occurrences": 0
        }));

        Ok(serde_json::json!({
            "overall": overall_stats,
            "by_event_type": event_type_stats_vec,
            "validation_errors": validation_errors_vec,
            "unknown_events": unknown_events_summary
        }))
    }
} 

/// PSS Event Operations for managing event types and basic event operations
pub struct PssEventOperations;

impl PssEventOperations {
    /// Get PSS event type by code
    pub fn get_pss_event_type_by_code(conn: &Connection, event_code: &str) -> DatabaseResult<Option<PssEventType>> {
        let mut stmt = conn.prepare(
            "SELECT id, event_code, event_name, description, category, is_active, created_at 
             FROM pss_event_types 
             WHERE event_code = ?"
        )?;
        
        let mut rows = stmt.query_map(params![event_code], |row| {
            PssEventType::from_row(row)
        })?;
        
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    /// Upsert PSS event type
    pub fn upsert_pss_event_type(conn: &mut Connection, event_type: &PssEventType) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;
        
        // Check if event type already exists
        let existing_id: Option<i64> = tx.query_row(
            "SELECT id FROM pss_event_types WHERE event_code = ?",
            params![event_type.event_code],
            |row| row.get(0)
        ).optional()?;
        
        let event_type_id = if let Some(id) = existing_id {
            // Update existing event type - note: pss_event_types table doesn't have updated_at
            tx.execute(
                "UPDATE pss_event_types SET 
                    event_name = ?, description = ?, category = ?, is_active = ?
                WHERE id = ?",
                params![
                    event_type.event_name,
                    event_type.description,
                    event_type.category,
                    event_type.is_active,
                    id
                ]
            )?;
            id
        } else {
            // Insert new event type
            tx.execute(
                "INSERT INTO pss_event_types (event_code, event_name, description, category, is_active, created_at) 
                 VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    event_type.event_code,
                    event_type.event_name,
                    event_type.description,
                    event_type.category,
                    event_type.is_active,
                    event_type.created_at.to_rfc3339()
                ]
            )?;
            tx.last_insert_rowid()
        };
        
        tx.commit()?;
        Ok(event_type_id)
    }

    /// Get all PSS event types
    pub fn get_all_pss_event_types(conn: &Connection) -> DatabaseResult<Vec<PssEventType>> {
        let mut stmt = conn.prepare(
            "SELECT id, event_code, event_name, description, category, is_active, created_at 
             FROM pss_event_types 
             ORDER BY event_code"
        )?;
        
        let rows = stmt.query_map([], |row| {
            PssEventType::from_row(row)
        })?;
        
        let mut event_types = Vec::new();
        for row in rows {
            event_types.push(row?);
        }
        
        Ok(event_types)
    }

    /// Delete PSS event type
    pub fn delete_pss_event_type(conn: &mut Connection, event_type_id: i64) -> DatabaseResult<()> {
        conn.execute(
            "DELETE FROM pss_event_types WHERE id = ?",
            params![event_type_id]
        )?;
        Ok(())
    }
} 

/// Phase 2 Optimization: Data Archival Strategy
/// Manages automatic archival of old events to improve performance
pub struct DataArchivalOperations;

impl DataArchivalOperations {
    /// Archive events older than specified days
    pub fn archive_old_events(conn: &mut rusqlite::Connection, days_old: i64) -> DatabaseResult<usize> {
        let start_time = std::time::Instant::now();
        
        // Create archive table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pss_events_v2_archive (
                id INTEGER PRIMARY KEY,
                session_id INTEGER NOT NULL,
                match_id INTEGER,
                event_type_id INTEGER NOT NULL,
                event_code TEXT NOT NULL,
                event_data TEXT,
                raw_data TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                tournament_id INTEGER,
                tournament_day_id INTEGER,
                recognition_status TEXT DEFAULT 'recognized',
                protocol_version TEXT DEFAULT '2.3',
                parser_confidence INTEGER DEFAULT 100,
                validation_errors TEXT,
                processing_time_ms INTEGER
            )",
            [],
        )?;

        // Create indices for archive table
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_archive_session_id ON pss_events_v2_archive(session_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_archive_created_at ON pss_events_v2_archive(created_at)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_archive_tournament ON pss_events_v2_archive(tournament_id, tournament_day_id)",
            [],
        )?;

        // Archive events older than specified days
        let archived_count = conn.execute(
            "INSERT INTO pss_events_v2_archive 
             SELECT * FROM pss_events_v2 
             WHERE created_at < datetime('now', '-{} days')",
            [days_old],
        )?;

        // Delete archived events from main table
        let deleted_count = conn.execute(
            "DELETE FROM pss_events_v2 
             WHERE created_at < datetime('now', '-{} days')",
            [days_old],
        )?;

        // Archive related event details
        let archived_details = conn.execute(
            "INSERT INTO pss_event_details_archive 
             SELECT * FROM pss_event_details 
             WHERE event_id IN (
                 SELECT id FROM pss_events_v2_archive 
                 WHERE created_at < datetime('now', '-{} days')
             )",
            [days_old],
        )?;

        // Delete archived event details from main table
        let deleted_details = conn.execute(
            "DELETE FROM pss_event_details 
             WHERE event_id IN (
                 SELECT id FROM pss_events_v2_archive 
                 WHERE created_at < datetime('now', '-{} days')
             )",
            [days_old],
        )?;

        let duration = start_time.elapsed();
        log::info!(
            "📦 Archived {} events and {} details in {:?} (deleted {} events and {} details)",
            archived_count,
            archived_details,
            duration,
            deleted_count,
            deleted_details
        );

        Ok(archived_count)
    }

    /// Get archive statistics
    pub fn get_archive_statistics(conn: &rusqlite::Connection) -> DatabaseResult<ArchiveStatistics> {
        let archived_events = conn.query_row(
            "SELECT COUNT(*) FROM pss_events_v2_archive",
            [],
            |row| row.get(0),
        )?;

        let archived_details = conn.query_row(
            "SELECT COUNT(*) FROM pss_event_details_archive",
            [],
            |row| row.get(0),
        )?;

        let oldest_archived = conn.query_row(
            "SELECT MIN(created_at) FROM pss_events_v2_archive",
            [],
            |row| row.get::<_, Option<String>>(0),
        )?;

        let newest_archived = conn.query_row(
            "SELECT MAX(created_at) FROM pss_events_v2_archive",
            [],
            |row| row.get::<_, Option<String>>(0),
        )?;

        let archive_size = conn.query_row(
            "SELECT SUM(length(raw_data)) FROM pss_events_v2_archive",
            [],
            |row| row.get::<_, Option<i64>>(0),
        )?;

        Ok(ArchiveStatistics {
            archived_events,
            archived_details,
            oldest_archived,
            newest_archived,
            archive_size_bytes: archive_size.unwrap_or(0),
        })
    }

    /// Restore events from archive (for data recovery)
    pub fn restore_from_archive(
        conn: &mut rusqlite::Connection,
        start_date: &str,
        end_date: &str,
    ) -> DatabaseResult<usize> {
        let start_time = std::time::Instant::now();

        // Restore events from archive
        let restored_events = conn.execute(
            "INSERT INTO pss_events_v2 
             SELECT * FROM pss_events_v2_archive 
             WHERE created_at BETWEEN ? AND ?",
            [start_date, end_date],
        )?;

        // Restore event details
        let restored_details = conn.execute(
            "INSERT INTO pss_event_details 
             SELECT * FROM pss_event_details_archive 
             WHERE event_id IN (
                 SELECT id FROM pss_events_v2 
                 WHERE created_at BETWEEN ? AND ?
             )",
            [start_date, end_date],
        )?;

        // Remove restored events from archive
        let _removed_from_archive = conn.execute(
            "DELETE FROM pss_events_v2_archive 
             WHERE created_at BETWEEN ? AND ?",
            [start_date, end_date],
        )?;

        let duration = start_time.elapsed();
        log::info!(
            "🔄 Restored {} events and {} details from archive in {:?}",
            restored_events,
            restored_details,
            duration
        );

        Ok(restored_events)
    }

    /// Clean up old archive data (permanent deletion)
    pub fn cleanup_old_archive_data(conn: &mut rusqlite::Connection, days_old: i64) -> DatabaseResult<usize> {
        let start_time = std::time::Instant::now();

        // Delete old archived events
        let deleted_events = conn.execute(
            "DELETE FROM pss_events_v2_archive 
             WHERE created_at < datetime('now', '-{} days')",
            [days_old],
        )?;

        // Delete old archived event details
        let deleted_details = conn.execute(
            "DELETE FROM pss_event_details_archive 
             WHERE event_id NOT IN (SELECT id FROM pss_events_v2_archive)",
            [],
        )?;

        let duration = start_time.elapsed();
        log::info!(
            "🗑️ Cleaned up {} archived events and {} details in {:?}",
            deleted_events,
            deleted_details,
            duration
        );

        Ok(deleted_events)
    }

    /// Optimize archive tables
    pub fn optimize_archive_tables(conn: &mut rusqlite::Connection) -> DatabaseResult<()> {
        log::info!("🔧 Optimizing archive tables...");

        // VACUUM archive tables
        conn.execute("VACUUM pss_events_v2_archive", [])?;
        conn.execute("VACUUM pss_event_details_archive", [])?;

        // Analyze tables for better query planning
        conn.execute("ANALYZE pss_events_v2_archive", [])?;
        conn.execute("ANALYZE pss_event_details_archive", [])?;

        // Optimize indices
        conn.execute("REINDEX pss_events_v2_archive", [])?;
        conn.execute("REINDEX pss_event_details_archive", [])?;

        log::info!("✅ Archive tables optimized successfully");
        Ok(())
    }
}

/// Archive statistics for monitoring
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchiveStatistics {
    pub archived_events: i64,
    pub archived_details: i64,
    pub oldest_archived: Option<String>,
    pub newest_archived: Option<String>,
    pub archive_size_bytes: i64,
}

// ============================================================================
// TRIGGER SYSTEM OPERATIONS
// ============================================================================

impl DatabaseConnection {
    // ========================================================================
    // OBS SCENE OPERATIONS
    // ========================================================================
    
    /// Get all OBS scenes
    pub async fn get_obs_scenes(&self) -> DatabaseResult<Vec<ObsScene>> {
        let conn = self.get_connection().await?;
        let mut stmt = conn.prepare("SELECT * FROM obs_scenes ORDER BY scene_name")?;
        
        let scenes = stmt.query_map([], |row| ObsScene::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(scenes)
    }
    
    /// Get active OBS scenes only
    pub async fn get_active_obs_scenes(&self) -> DatabaseResult<Vec<ObsScene>> {
        let conn = self.get_connection().await?;
        let mut stmt = conn.prepare("SELECT * FROM obs_scenes WHERE is_active = 1 ORDER BY scene_name")?;
        
        let scenes = stmt.query_map([], |row| ObsScene::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(scenes)
    }
    
    /// Get OBS scene by name
    pub async fn get_obs_scene_by_name(&self, scene_name: &str) -> DatabaseResult<Option<ObsScene>> {
        let conn = self.get_connection().await?;
        let mut stmt = conn.prepare("SELECT * FROM obs_scenes WHERE scene_name = ?")?;
        
        let scene = stmt.query_row([scene_name], |row| ObsScene::from_row(row))
            .optional()?;
        
        Ok(scene)
    }
    
    /// Insert or update OBS scene
    pub async fn upsert_obs_scene(&self, scene: &ObsScene) -> DatabaseResult<i64> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();
        
        let id = conn.execute(
            "INSERT OR REPLACE INTO obs_scenes (scene_name, scene_id, is_active, last_seen_at, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?)",
            [
                &scene.scene_name,
                &scene.scene_id,
                &(scene.is_active as i32).to_string(),
                &scene.last_seen_at.to_rfc3339(),
                &scene.created_at.to_rfc3339(),
                &now,
            ],
        )?;
        
        Ok(id as i64)
    }
    
    /// Mark OBS scene as inactive
    pub async fn mark_obs_scene_inactive(&self, scene_name: &str) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();
        
        conn.execute(
            "UPDATE obs_scenes SET is_active = 0, updated_at = ? WHERE scene_name = ?",
            [&now, scene_name],
        )?;
        
        Ok(())
    }
    
    /// Update OBS scene last seen timestamp
    pub async fn update_obs_scene_last_seen(&self, scene_name: &str) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();
        
        conn.execute(
            "UPDATE obs_scenes SET last_seen_at = ?, updated_at = ? WHERE scene_name = ?",
            [&now, &now, scene_name],
        )?;
        
        Ok(())
    }
    
    /// Sync OBS scenes from WebSocket (mark unseen scenes as inactive)
    pub async fn sync_obs_scenes(&self, active_scene_names: &[String]) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();
        
        // Mark all scenes as inactive first
        conn.execute(
            "UPDATE obs_scenes SET is_active = 0, updated_at = ?",
            [&now],
        )?;
        
        // Mark active scenes as active
        for scene_name in active_scene_names {
            conn.execute(
                "UPDATE obs_scenes SET is_active = 1, last_seen_at = ?, updated_at = ? WHERE scene_name = ?",
                [&now, &now, scene_name],
            )?;
        }
        
        Ok(())
    }
    
    // ========================================================================
    // OVERLAY TEMPLATE OPERATIONS
    // ========================================================================
    
    /// Get all overlay templates
    pub async fn get_overlay_templates(&self) -> DatabaseResult<Vec<OverlayTemplate>> {
        let conn = self.get_connection().await?;
        let mut stmt = conn.prepare("SELECT * FROM overlay_templates ORDER BY name")?;
        
        let templates = stmt.query_map([], |row| OverlayTemplate::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(templates)
    }
    
    /// Get active overlay templates only
    pub async fn get_active_overlay_templates(&self) -> DatabaseResult<Vec<OverlayTemplate>> {
        let conn = self.get_connection().await?;
        let mut stmt = conn.prepare("SELECT * FROM overlay_templates WHERE is_active = 1 ORDER BY name")?;
        
        let templates = stmt.query_map([], |row| OverlayTemplate::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(templates)
    }
    
    /// Get overlay template by name
    pub async fn get_overlay_template_by_name(&self, name: &str) -> DatabaseResult<Option<OverlayTemplate>> {
        let conn = self.get_connection().await?;
        let mut stmt = conn.prepare("SELECT * FROM overlay_templates WHERE name = ?")?;
        
        let template = stmt.query_row([name], |row| OverlayTemplate::from_row(row))
            .optional()?;
        
        Ok(template)
    }
    
    /// Insert overlay template
    pub async fn insert_overlay_template(&self, template: &OverlayTemplate) -> DatabaseResult<i64> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();
        
        let id = conn.execute(
            "INSERT INTO overlay_templates (name, description, theme, colors, animation_type, duration_ms, is_active, url, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            [
                &template.name,
                &template.description.as_deref().unwrap_or("").to_string(),
                &template.theme,
                &template.colors.as_deref().unwrap_or("").to_string(),
                &template.animation_type,
                &template.duration_ms.to_string(),
                &(template.is_active as i32).to_string(),
                &template.url.as_deref().unwrap_or("").to_string(),
                &template.created_at.to_rfc3339(),
                &now,
            ],
        )?;
        
        Ok(id as i64)
    }
    
    /// Update overlay template
    pub async fn update_overlay_template(&self, template: &OverlayTemplate) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();
        
        conn.execute(
            "UPDATE overlay_templates SET description = ?, theme = ?, colors = ?, animation_type = ?, duration_ms = ?, is_active = ?, url = ?, updated_at = ? 
             WHERE id = ?",
            [
                &template.description.as_deref().unwrap_or("").to_string(),
                &template.theme,
                &template.colors.as_deref().unwrap_or("").to_string(),
                &template.animation_type,
                &template.duration_ms.to_string(),
                &(template.is_active as i32).to_string(),
                &template.url.as_deref().unwrap_or("").to_string(),
                &now,
                &template.id.unwrap_or(0).to_string(),
            ],
        )?;
        
        Ok(())
    }
    
    /// Delete overlay template
    pub async fn delete_overlay_template(&self, id: i64) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        
        conn.execute("DELETE FROM overlay_templates WHERE id = ?", [id])?;
        
        Ok(())
    }
    
    // ========================================================================
    // EVENT TRIGGER OPERATIONS
    // ========================================================================
    
    /// Get all event triggers
    pub async fn get_event_triggers(&self) -> DatabaseResult<Vec<EventTrigger>> {
        let conn = self.get_connection().await?;
        let mut stmt = conn.prepare("SELECT * FROM event_triggers ORDER BY priority DESC, event_type")?;
        
        let triggers = stmt.query_map([], |row| EventTrigger::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(triggers)
    }
    
    /// Get event triggers for a specific tournament
    pub async fn get_event_triggers_for_tournament(&self, tournament_id: i64) -> DatabaseResult<Vec<EventTrigger>> {
        let conn = self.get_connection().await?;
        let mut stmt = conn.prepare(
            "SELECT * FROM event_triggers WHERE tournament_id = ? ORDER BY priority DESC, event_type"
        )?;
        
        let triggers = stmt.query_map([tournament_id], |row| EventTrigger::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(triggers)
    }
    
    /// Get event triggers for a specific tournament day
    pub async fn get_event_triggers_for_tournament_day(&self, tournament_day_id: i64) -> DatabaseResult<Vec<EventTrigger>> {
        let conn = self.get_connection().await?;
        let mut stmt = conn.prepare(
            "SELECT * FROM event_triggers WHERE tournament_day_id = ? ORDER BY priority DESC, event_type"
        )?;
        
        let triggers = stmt.query_map([tournament_day_id], |row| EventTrigger::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(triggers)
    }
    
    /// Get global event triggers (no tournament/day specified)
    pub async fn get_global_event_triggers(&self) -> DatabaseResult<Vec<EventTrigger>> {
        let conn = self.get_connection().await?;
        let mut stmt = conn.prepare(
            "SELECT * FROM event_triggers WHERE tournament_id IS NULL AND tournament_day_id IS NULL ORDER BY priority DESC, event_type"
        )?;
        
        let triggers = stmt.query_map([], |row| EventTrigger::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(triggers)
    }
    
    /// Get enabled event triggers for a specific event type
    pub async fn get_enabled_triggers_for_event(&self, event_type: &str, tournament_id: Option<i64>, tournament_day_id: Option<i64>) -> DatabaseResult<Vec<EventTrigger>> {
        let conn = self.get_connection().await?;
        
        let mut query = String::from(
            "SELECT * FROM event_triggers WHERE event_type = ? AND is_enabled = 1"
        );
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(event_type.to_string())];
        
        if let Some(tid) = tournament_id {
            query.push_str(" AND (tournament_id = ? OR tournament_id IS NULL)");
            params.push(Box::new(tid));
        }
        
        if let Some(tdid) = tournament_day_id {
            query.push_str(" AND (tournament_day_id = ? OR tournament_day_id IS NULL)");
            params.push(Box::new(tdid));
        }
        
        query.push_str(" ORDER BY priority DESC");
        
        let mut stmt = conn.prepare(&query)?;
        let triggers = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| EventTrigger::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(triggers)
    }
    
    /// Insert event trigger
    pub async fn insert_event_trigger(&self, trigger: &EventTrigger) -> DatabaseResult<i64> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();
        
        use rusqlite::params;
        let id = conn.execute(
            "INSERT INTO event_triggers (tournament_id, tournament_day_id, event_type, trigger_type, obs_scene_id, overlay_template_id, is_enabled, priority, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                trigger.tournament_id,
                trigger.tournament_day_id,
                trigger.event_type,
                trigger.trigger_type,
                trigger.obs_scene_id,
                trigger.overlay_template_id,
                trigger.is_enabled,
                trigger.priority,
                trigger.created_at.to_rfc3339(),
                now,
            ],
        )?;
        
        Ok(id as i64)
    }
    
    /// Update event trigger
    pub async fn update_event_trigger(&self, trigger: &EventTrigger) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();
        
        use rusqlite::params;
        conn.execute(
            "UPDATE event_triggers SET tournament_id = ?, tournament_day_id = ?, event_type = ?, trigger_type = ?, obs_scene_id = ?, overlay_template_id = ?, is_enabled = ?, priority = ?, updated_at = ? 
             WHERE id = ?",
            params![
                trigger.tournament_id,
                trigger.tournament_day_id,
                trigger.event_type,
                trigger.trigger_type,
                trigger.obs_scene_id,
                trigger.overlay_template_id,
                trigger.is_enabled,
                trigger.priority,
                now,
                trigger.id,
            ],
        )?;
        
        Ok(())
    }
    
    /// Delete event trigger
    pub async fn delete_event_trigger(&self, id: i64) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        
        conn.execute("DELETE FROM event_triggers WHERE id = ?", [id])?;
        
        Ok(())
    }
    
    /// Enable/disable event trigger
    pub async fn set_event_trigger_enabled(&self, id: i64, enabled: bool) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();
        
        conn.execute(
            "UPDATE event_triggers SET is_enabled = ?, updated_at = ? WHERE id = ?",
            [&(enabled as i32).to_string(), &now, &id.to_string()],
        )?;
        
        Ok(())
    }
    
    /// Copy triggers from one tournament to another
    pub async fn copy_triggers_to_tournament(&self, source_tournament_id: i64, target_tournament_id: i64) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO event_triggers (tournament_id, tournament_day_id, event_type, trigger_type, obs_scene_id, overlay_template_id, is_enabled, priority, created_at, updated_at)
             SELECT ?, tournament_day_id, event_type, trigger_type, obs_scene_id, overlay_template_id, is_enabled, priority, ?, ?
             FROM event_triggers WHERE tournament_id = ?",
            [&target_tournament_id.to_string(), &now, &now, &source_tournament_id.to_string()],
        )?;
        
        Ok(())
    }
    
    /// Save triggers as template (global triggers)
    pub async fn save_triggers_as_template(&self, tournament_id: i64, _template_name: &str) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();
        
        // Create a special template trigger with the template name
        conn.execute(
            "INSERT INTO event_triggers (tournament_id, tournament_day_id, event_type, trigger_type, obs_scene_id, overlay_template_id, is_enabled, priority, created_at, updated_at)
             SELECT NULL, NULL, event_type, trigger_type, obs_scene_id, overlay_template_id, is_enabled, priority, ?, ?
             FROM event_triggers WHERE tournament_id = ?",
            [&now, &now, &tournament_id.to_string()],
        )?;
        
        Ok(())
    }

    // ========================================================================
    // OBS CONNECTION OPERATIONS
    // ========================================================================
    
    /// Get all OBS connections
    pub async fn get_obs_connections(&self) -> DatabaseResult<Vec<ObsConnection>> {
        let conn = self.get_connection().await?;
        let mut stmt = conn.prepare("SELECT * FROM obs_connections ORDER BY name")?;
        
        let connections = stmt.query_map([], |row| ObsConnection::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(connections)
    }
    
    /// Get active OBS connections only
    pub async fn get_active_obs_connections(&self) -> DatabaseResult<Vec<ObsConnection>> {
        let conn = self.get_connection().await?;
        let mut stmt = conn.prepare("SELECT * FROM obs_connections WHERE is_active = 1 ORDER BY name")?;
        
        let connections = stmt.query_map([], |row| ObsConnection::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(connections)
    }
    
    /// Get OBS connection by name
    pub async fn get_obs_connection_by_name(&self, name: &str) -> DatabaseResult<Option<ObsConnection>> {
        let conn = self.get_connection().await?;
        let mut stmt = conn.prepare("SELECT * FROM obs_connections WHERE name = ?")?;
        
        let connection = stmt.query_row([name], |row| ObsConnection::from_row(row))
            .optional()?;
        
        Ok(connection)
    }
    
    /// Insert or update OBS connection
    pub async fn upsert_obs_connection(&self, connection: &ObsConnection) -> DatabaseResult<i64> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();
        
        let id = conn.execute(
            "INSERT OR REPLACE INTO obs_connections (name, host, port, password, is_active, status, error, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            [
                &connection.name,
                &connection.host,
                &connection.port.to_string(),
                &connection.password.as_deref().unwrap_or("").to_string(),
                &(connection.is_active as i32).to_string(),
                &connection.status,
                &connection.error.as_deref().unwrap_or("").to_string(),
                &connection.created_at.to_rfc3339(),
                &now,
            ],
        )?;
        
        Ok(id as i64)
    }
    
    /// Update OBS connection status
    pub async fn update_obs_connection_status(&self, name: &str, status: &str, error: Option<&str>) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();
        
        conn.execute(
            "UPDATE obs_connections SET status = ?, error = ?, updated_at = ? WHERE name = ?",
            [
                status,
                &error.unwrap_or("").to_string(),
                &now,
                name,
            ],
        )?;
        
        Ok(())
    }
    
    /// Delete OBS connection
    pub async fn delete_obs_connection(&self, name: &str) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        
        conn.execute("DELETE FROM obs_connections WHERE name = ?", [name])?;
        
        Ok(())
    }
    
    /// Clear all OBS connections
    pub async fn clear_obs_connections(&self) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        
        conn.execute("DELETE FROM obs_connections", [])?;
        
        Ok(())
    }
}

/// OBS Recording Operations for managing recording configuration and sessions
pub struct ObsRecordingOperations;

impl ObsRecordingOperations {
    /// Get all OBS recording configurations
    pub fn get_recording_configs(conn: &Connection) -> DatabaseResult<Vec<ObsRecordingConfig>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM obs_recording_config ORDER BY obs_connection_name"
        )?;
        
        let configs = stmt.query_map([], |row| ObsRecordingConfig::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(configs)
    }
    
    /// Get recording configuration for a specific OBS connection
    pub fn get_recording_config(conn: &Connection, obs_connection_name: &str) -> DatabaseResult<Option<ObsRecordingConfig>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM obs_recording_config WHERE obs_connection_name = ?"
        )?;
        
        let config = stmt.query_row([obs_connection_name], |row| ObsRecordingConfig::from_row(row))
            .optional()?;
        
        Ok(config)
    }
    
    /// Create or update recording configuration
    pub fn upsert_recording_config(conn: &mut Connection, config: &ObsRecordingConfig) -> DatabaseResult<i64> {
        let config_id = conn.execute(
            "INSERT OR REPLACE INTO obs_recording_config (
                obs_connection_name, recording_root_path, recording_format, recording_quality,
                recording_bitrate, recording_resolution, replay_buffer_enabled, replay_buffer_duration,
                auto_start_recording, auto_start_replay_buffer, filename_template, is_active,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            [
                &config.obs_connection_name,
                &config.recording_root_path,
                &config.recording_format,
                &config.recording_quality,
                &config.recording_bitrate.map(|b| b.to_string()).unwrap_or_default(),
                &config.recording_resolution.as_deref().unwrap_or("").to_string(),
                &config.replay_buffer_enabled.to_string(),
                &config.replay_buffer_duration.map(|d| d.to_string()).unwrap_or_default(),
                &config.auto_start_recording.to_string(),
                &config.auto_start_replay_buffer.to_string(),
                &config.filename_template,
                &config.is_active.to_string(),
                &config.created_at.to_rfc3339(),
                &Utc::now().to_rfc3339(),
            ],
        )?;
        
        Ok(config_id as i64)
    }
    
    /// Delete recording configuration
    pub fn delete_recording_config(conn: &mut Connection, obs_connection_name: &str) -> DatabaseResult<()> {
        conn.execute(
            "DELETE FROM obs_recording_config WHERE obs_connection_name = ?",
            [obs_connection_name],
        )?;
        Ok(())
    }
    
    /// Get active recording sessions
    pub fn get_active_recording_sessions(conn: &Connection) -> DatabaseResult<Vec<ObsRecordingSession>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM obs_recording_sessions WHERE status IN ('pending', 'recording') ORDER BY created_at DESC"
        )?;
        
        let sessions = stmt.query_map([], |row| ObsRecordingSession::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(sessions)
    }
    
    /// Get recording sessions for a specific OBS connection
    pub fn get_recording_sessions_for_connection(conn: &Connection, obs_connection_name: &str) -> DatabaseResult<Vec<ObsRecordingSession>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM obs_recording_sessions WHERE obs_connection_name = ? ORDER BY created_at DESC"
        )?;
        
        let sessions = stmt.query_map([obs_connection_name], |row| ObsRecordingSession::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(sessions)
    }
    
    /// Get recording sessions for a specific match
    pub fn get_recording_sessions_for_match(conn: &Connection, match_id: &str) -> DatabaseResult<Vec<ObsRecordingSession>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM obs_recording_sessions WHERE match_id = ? ORDER BY created_at DESC"
        )?;
        
        let sessions = stmt.query_map([match_id], |row| ObsRecordingSession::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(sessions)
    }
    
    /// Create new recording session
    pub fn create_recording_session(conn: &mut Connection, session: &ObsRecordingSession) -> DatabaseResult<i64> {
        let session_id = conn.execute(
            "INSERT INTO obs_recording_sessions (
                obs_connection_name, tournament_id, tournament_day_id, match_id, match_number,
                player1_name, player1_flag, player2_name, player2_flag, recording_path,
                recording_filename, recording_start_time, recording_end_time, recording_duration,
                recording_size_bytes, replay_buffer_start_time, replay_buffer_end_time,
                replay_buffer_saved, replay_buffer_filename, status, error_message,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            [
                &session.obs_connection_name,
                &session.tournament_id.map(|id| id.to_string()).unwrap_or_default(),
                &session.tournament_day_id.map(|id| id.to_string()).unwrap_or_default(),
                &session.match_id.as_deref().unwrap_or("").to_string(),
                &session.match_number.as_deref().unwrap_or("").to_string(),
                &session.player1_name.as_deref().unwrap_or("").to_string(),
                &session.player1_flag.as_deref().unwrap_or("").to_string(),
                &session.player2_name.as_deref().unwrap_or("").to_string(),
                &session.player2_flag.as_deref().unwrap_or("").to_string(),
                &session.recording_path,
                &session.recording_filename,
                &session.recording_start_time.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                &session.recording_end_time.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                &session.recording_duration.map(|d| d.to_string()).unwrap_or_default(),
                &session.recording_size_bytes.map(|s| s.to_string()).unwrap_or_default(),
                &session.replay_buffer_start_time.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                &session.replay_buffer_end_time.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                &session.replay_buffer_saved.to_string(),
                &session.replay_buffer_filename.as_deref().unwrap_or("").to_string(),
                &session.status,
                &session.error_message.as_deref().unwrap_or("").to_string(),
                &session.created_at.to_rfc3339(),
                &Utc::now().to_rfc3339(),
            ],
        )?;
        
        Ok(session_id as i64)
    }
    
    /// Update recording session
    pub fn update_recording_session(conn: &mut Connection, session_id: i64, session: &ObsRecordingSession) -> DatabaseResult<()> {
        conn.execute(
            "UPDATE obs_recording_sessions SET
                obs_connection_name = ?, tournament_id = ?, tournament_day_id = ?, match_id = ?, match_number = ?,
                player1_name = ?, player1_flag = ?, player2_name = ?, player2_flag = ?, recording_path = ?,
                recording_filename = ?, recording_start_time = ?, recording_end_time = ?, recording_duration = ?,
                recording_size_bytes = ?, replay_buffer_start_time = ?, replay_buffer_end_time = ?,
                replay_buffer_saved = ?, replay_buffer_filename = ?, status = ?, error_message = ?, updated_at = ?
            WHERE id = ?",
            [
                &session.obs_connection_name,
                &session.tournament_id.map(|id| id.to_string()).unwrap_or_default(),
                &session.tournament_day_id.map(|id| id.to_string()).unwrap_or_default(),
                &session.match_id.as_deref().unwrap_or("").to_string(),
                &session.match_number.as_deref().unwrap_or("").to_string(),
                &session.player1_name.as_deref().unwrap_or("").to_string(),
                &session.player1_flag.as_deref().unwrap_or("").to_string(),
                &session.player2_name.as_deref().unwrap_or("").to_string(),
                &session.player2_flag.as_deref().unwrap_or("").to_string(),
                &session.recording_path,
                &session.recording_filename,
                &session.recording_start_time.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                &session.recording_end_time.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                &session.recording_duration.map(|d| d.to_string()).unwrap_or_default(),
                &session.recording_size_bytes.map(|s| s.to_string()).unwrap_or_default(),
                &session.replay_buffer_start_time.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                &session.replay_buffer_end_time.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                &session.replay_buffer_saved.to_string(),
                &session.replay_buffer_filename.as_deref().unwrap_or("").to_string(),
                &session.status,
                &session.error_message.as_deref().unwrap_or("").to_string(),
                &Utc::now().to_rfc3339(),
                &session_id.to_string(),
            ],
        )?;
        
        Ok(())
    }
    
    /// Get recording session by ID
    pub fn get_recording_session(conn: &Connection, session_id: i64) -> DatabaseResult<Option<ObsRecordingSession>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM obs_recording_sessions WHERE id = ?"
        )?;
        
        let session = stmt.query_row([session_id], |row| ObsRecordingSession::from_row(row))
            .optional()?;
        
        Ok(session)
    }
    
    /// Update recording session status
    pub fn update_recording_session_status(conn: &mut Connection, session_id: i64, status: &str, error_message: Option<&str>) -> DatabaseResult<()> {
        conn.execute(
            "UPDATE obs_recording_sessions SET status = ?, error_message = ?, updated_at = ? WHERE id = ?",
            [
                status,
                &error_message.unwrap_or(""),
                &Utc::now().to_rfc3339(),
                &session_id.to_string(),
            ],
        )?;
        
        Ok(())
    }
    
    /// Get recent recording sessions
    pub fn get_recent_recording_sessions(conn: &Connection, limit: i64) -> DatabaseResult<Vec<ObsRecordingSession>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM obs_recording_sessions ORDER BY created_at DESC LIMIT ?"
        )?;
        
        let sessions = stmt.query_map([limit], |row| ObsRecordingSession::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(sessions)
    }
} 