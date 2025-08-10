use rusqlite::{Connection, Result as SqliteResult};
use crate::database::{DatabaseError, DatabaseResult, CURRENT_SCHEMA_VERSION, SchemaVersion};

/// Migration trait for database schema updates
pub trait Migration: Send + Sync {
    fn version(&self) -> u32;
    fn description(&self) -> &str;
    fn up(&self, conn: &Connection) -> SqliteResult<()>;
    fn down(&self, conn: &Connection) -> SqliteResult<()>;
}

/// Migration 1: Initial schema
pub struct Migration1;

impl Migration for Migration1 {
    fn version(&self) -> u32 {
        1
    }
    
    fn description(&self) -> &str {
        "Initial schema with PSS events, OBS connections, app config, and flag mappings"
    }
    
    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // Create schema_version table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                version INTEGER NOT NULL,
                applied_at TEXT NOT NULL,
                description TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create pss_events table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pss_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                match_id TEXT,
                athlete1_code TEXT,
                athlete2_code TEXT,
                score1 INTEGER,
                score2 INTEGER,
                round TEXT,
                weight_class TEXT,
                category TEXT,
                raw_data TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create index on timestamp for efficient querying
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_events_timestamp ON pss_events(timestamp)",
            [],
        )?;
        
        // Create index on match_id for match-based queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_events_match_id ON pss_events(match_id)",
            [],
        )?;
        
        // Create obs_connections table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS obs_connections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                host TEXT NOT NULL,
                port INTEGER NOT NULL,
                password TEXT,
                is_active BOOLEAN NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create app_config table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_config (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL UNIQUE,
                value TEXT NOT NULL,
                category TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create index on category for efficient config queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_app_config_category ON app_config(category)",
            [],
        )?;
        
        // Create flag_mappings table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS flag_mappings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pss_code TEXT NOT NULL UNIQUE,
                ioc_code TEXT NOT NULL,
                country_name TEXT NOT NULL,
                is_custom BOOLEAN NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create index on pss_code for efficient lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_flag_mappings_pss_code ON flag_mappings(pss_code)",
            [],
        )?;
        
        // Create index on ioc_code for reverse lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_flag_mappings_ioc_code ON flag_mappings(ioc_code)",
            [],
        )?;
        
        Ok(())
    }
    
    fn down(&self, conn: &Connection) -> SqliteResult<()> {
        conn.execute("DROP TABLE IF EXISTS flag_mappings", [])?;
        conn.execute("DROP TABLE IF EXISTS app_config", [])?;
        conn.execute("DROP TABLE IF EXISTS obs_connections", [])?;
        conn.execute("DROP TABLE IF EXISTS pss_events", [])?;
        conn.execute("DROP TABLE IF EXISTS schema_version", [])?;
        Ok(())
    }
}

/// Migration 2: Normalized settings schema
pub struct Migration2;

impl Migration for Migration2 {
    fn version(&self) -> u32 {
        2
    }
    
    fn description(&self) -> &str {
        "Normalized settings schema with categories, keys, values, and history"
    }
    
    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // Create settings_categories table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings_categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                display_order INTEGER DEFAULT 0,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create settings_keys table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings_keys (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category_id INTEGER NOT NULL,
                key_name TEXT NOT NULL UNIQUE,
                display_name TEXT NOT NULL,
                description TEXT,
                data_type TEXT NOT NULL,
                default_value TEXT,
                validation_rules TEXT,
                is_required BOOLEAN DEFAULT 0,
                is_sensitive BOOLEAN DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY (category_id) REFERENCES settings_categories(id)
            )",
            [],
        )?;
        
        // Create settings_values table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings_values (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key_id INTEGER NOT NULL,
                value TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (key_id) REFERENCES settings_keys(id)
            )",
            [],
        )?;
        
        // Create settings_history table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key_id INTEGER NOT NULL,
                old_value TEXT,
                new_value TEXT,
                changed_by TEXT NOT NULL,
                change_reason TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (key_id) REFERENCES settings_keys(id)
            )",
            [],
        )?;
        
        // Create indices for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_settings_keys_category ON settings_keys(category_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_settings_keys_name ON settings_keys(key_name)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_settings_values_key ON settings_values(key_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_settings_history_key ON settings_history(key_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_settings_history_created ON settings_history(created_at)",
            [],
        )?;
        
        // Insert default categories
        let default_categories = vec![
            ("app", "Application Core Settings", 1),
            ("obs", "OBS WebSocket Settings", 2),
            ("udp", "UDP/PSS Protocol Settings", 3),
            ("logging", "Logging and Diagnostics", 4),
            ("ui", "User Interface Settings", 5),
            ("video", "Video Playback Settings", 6),
            ("license", "License and Activation", 7),
            ("flags", "Flag Management Settings", 8),
            ("advanced", "Advanced Features", 9),
        ];
        
        for (name, description, order) in default_categories {
            conn.execute(
                "INSERT OR IGNORE INTO settings_categories (name, description, display_order, created_at) VALUES (?, ?, ?, ?)",
                [name, description, &order.to_string(), &chrono::Utc::now().to_rfc3339()],
            )?;
        }
        
        Ok(())
    }
    
    fn down(&self, conn: &Connection) -> SqliteResult<()> {
        conn.execute("DROP TABLE IF EXISTS settings_history", [])?;
        conn.execute("DROP TABLE IF EXISTS settings_values", [])?;
        conn.execute("DROP TABLE IF EXISTS settings_keys", [])?;
        conn.execute("DROP TABLE IF EXISTS settings_categories", [])?;
        Ok(())
    }
}

/// Migration 3: Comprehensive flag management system
pub struct Migration3;

impl Migration for Migration3 {
    fn version(&self) -> u32 {
        3
    }
    
    fn description(&self) -> &str {
        "Comprehensive flag management system with flags, recognition history, and IOC data population"
    }
    
    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // Create flags table for individual flag files and metadata
        conn.execute(
            "CREATE TABLE IF NOT EXISTS flags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL,
                ioc_code TEXT,
                country_name TEXT,
                recognition_status TEXT DEFAULT 'pending',
                recognition_confidence REAL,
                upload_date DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_modified DATETIME DEFAULT CURRENT_TIMESTAMP,
                file_size INTEGER,
                file_path TEXT NOT NULL,
                is_recognized BOOLEAN DEFAULT FALSE
            )",
            [],
        )?;
        
        // Create recognition_history table for tracking flag recognition attempts
        conn.execute(
            "CREATE TABLE IF NOT EXISTS recognition_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                flag_id INTEGER,
                recognition_method TEXT,
                confidence REAL,
                recognized_as TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (flag_id) REFERENCES flags(id)
            )",
            [],
        )?;
        
        // Create indices for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_flags_ioc_code ON flags(ioc_code)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_flags_recognition_status ON flags(recognition_status)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_recognition_history_flag_id ON recognition_history(flag_id)",
            [],
        )?;
        
        // Populate flag_mappings table with IOC data if empty
        let mapping_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM flag_mappings",
            [],
            |row| row.get(0),
        ).unwrap_or(0);
        
        if mapping_count == 0 {
            log::info!("Populating flag_mappings table with IOC data");
            
            // IOC flag data from flagUtils.tsx
            let ioc_flags = vec![
                // Current NOCs (Table 1) - Main Olympic countries
                ("AFG", "AFG", "Afghanistan"),
                ("ALB", "ALB", "Albania"),
                ("ALG", "ALG", "Algeria"),
                ("AND", "AND", "Andorra"),
                ("ANG", "ANG", "Angola"),
                ("ANT", "ANT", "Antigua and Barbuda"),
                ("ARG", "ARG", "Argentina"),
                ("ARM", "ARM", "Armenia"),
                ("ARU", "ARU", "Aruba"),
                ("ASA", "ASA", "American Samoa"),
                ("AUS", "AUS", "Australia"),
                ("AUT", "AUT", "Austria"),
                ("AZE", "AZE", "Azerbaijan"),
                ("BAH", "BAH", "Bahamas"),
                ("BAN", "BAN", "Bangladesh"),
                ("BAR", "BAR", "Barbados"),
                ("BDI", "BDI", "Burundi"),
                ("BEL", "BEL", "Belgium"),
                ("BEN", "BEN", "Benin"),
                ("BER", "BER", "Bermuda"),
                ("BHU", "BHU", "Bhutan"),
                ("BIH", "BIH", "Bosnia and Herzegovina"),
                ("BIZ", "BIZ", "Belize"),
                ("BLR", "BLR", "Belarus"),
                ("BOL", "BOL", "Bolivia"),
                ("BOT", "BOT", "Botswana"),
                ("BRA", "BRA", "Brazil"),
                ("BRN", "BRN", "Bahrain"),
                ("BRU", "BRU", "Brunei"),
                ("BUL", "BUL", "Bulgaria"),
                ("BUR", "BUR", "Burkina Faso"),
                ("CAF", "CAF", "Central African Republic"),
                ("CAM", "CAM", "Cambodia"),
                ("CAN", "CAN", "Canada"),
                ("CAY", "CAY", "Cayman Islands"),
                ("CGO", "CGO", "Republic of the Congo"),
                ("CHA", "CHA", "Chad"),
                ("CHI", "CHI", "Chile"),
                ("CHN", "CHN", "China"),
                ("CIV", "CIV", "Ivory Coast"),
                ("CMR", "CMR", "Cameroon"),
                ("COD", "COD", "Democratic Republic of the Congo"),
                ("COK", "COK", "Cook Islands"),
                ("COL", "COL", "Colombia"),
                ("COM", "COM", "Comoros"),
                ("CPV", "CPV", "Cape Verde"),
                ("CRC", "CRC", "Costa Rica"),
                ("CRO", "CRO", "Croatia"),
                ("CUB", "CUB", "Cuba"),
                ("CYP", "CYP", "Cyprus"),
                ("CZE", "CZE", "Czech Republic"),
                ("DEN", "DEN", "Denmark"),
                ("DJI", "DJI", "Djibouti"),
                ("DMA", "DMA", "Dominica"),
                ("DOM", "DOM", "Dominican Republic"),
                ("ECU", "ECU", "Ecuador"),
                ("EGY", "EGY", "Egypt"),
                ("ERI", "ERI", "Eritrea"),
                ("ESP", "ESP", "Spain"),
                ("EST", "EST", "Estonia"),
                ("ETH", "ETH", "Ethiopia"),
                ("FIJ", "FIJ", "Fiji"),
                ("FIN", "FIN", "Finland"),
                ("FRA", "FRA", "France"),
                ("FSM", "FSM", "Federated States of Micronesia"),
                ("GAB", "GAB", "Gabon"),
                ("GAM", "GAM", "Gambia"),
                ("GBR", "GBR", "Great Britain"),
                ("GEO", "GEO", "Georgia"),
                ("GEQ", "GEQ", "Equatorial Guinea"),
                ("GER", "GER", "Germany"),
                ("GHA", "GHA", "Ghana"),
                ("GRE", "GRE", "Greece"),
                ("GRN", "GRN", "Grenada"),
                ("GUA", "GUA", "Guatemala"),
                ("GUI", "GUI", "Guinea"),
                ("GUM", "GUM", "Guam"),
                ("GUY", "GUY", "Guyana"),
                ("HAI", "HAI", "Haiti"),
                ("HKG", "HKG", "Hong Kong"),
                ("HON", "HON", "Honduras"),
                ("HUN", "HUN", "Hungary"),
                ("INA", "INA", "Indonesia"),
                ("IND", "IND", "India"),
                ("IRI", "IRI", "Iran"),
                ("IRL", "IRL", "Ireland"),
                ("IRQ", "IRQ", "Iraq"),
                ("ISL", "ISL", "Iceland"),
                ("ISR", "ISR", "Israel"),
                ("ISV", "ISV", "U.S. Virgin Islands"),
                ("ITA", "ITA", "Italy"),
                ("IVB", "IVB", "British Virgin Islands"),
                ("JAM", "JAM", "Jamaica"),
                ("JOR", "JOR", "Jordan"),
                ("JPN", "JPN", "Japan"),
                ("KAZ", "KAZ", "Kazakhstan"),
                ("KEN", "KEN", "Kenya"),
                ("KGZ", "KGZ", "Kyrgyzstan"),
                ("KHM", "KHM", "Cambodia"),
                ("KIR", "KIR", "Kiribati"),
                ("KOR", "KOR", "South Korea"),
                ("KOS", "KOS", "Kosovo"),
                ("KSA", "KSA", "Saudi Arabia"),
                ("KUW", "KUW", "Kuwait"),
                ("LAO", "LAO", "Laos"),
                ("LAT", "LAT", "Latvia"),
                ("LBA", "LBA", "Libya"),
                ("LBN", "LBN", "Lebanon"),
                ("LBR", "LBR", "Liberia"),
                ("LCA", "LCA", "Saint Lucia"),
                ("LES", "LES", "Lesotho"),
                ("LIE", "LIE", "Liechtenstein"),
                ("LTU", "LTU", "Lithuania"),
                ("LUX", "LUX", "Luxembourg"),
                ("MAC", "MAC", "Macau"),
                ("MAD", "MAD", "Madagascar"),
                ("MAL", "MAL", "Malaysia"),
                ("MAR", "MAR", "Morocco"),
                ("MAS", "MAS", "Malaysia"),
                ("MAW", "MAW", "Malawi"),
                ("MDA", "MDA", "Moldova"),
                ("MDV", "MDV", "Maldives"),
                ("MEX", "MEX", "Mexico"),
                ("MGL", "MGL", "Mongolia"),
                ("MHL", "MHL", "Marshall Islands"),
                ("MKD", "MKD", "North Macedonia"),
                ("MLI", "MLI", "Mali"),
                ("MLT", "MLT", "Malta"),
                ("MNE", "MNE", "Montenegro"),
                ("MON", "MON", "Monaco"),
                ("MOZ", "MOZ", "Mozambique"),
                ("MRI", "MRI", "Mauritius"),
                ("MTN", "MTN", "Mauritania"),
                ("MYA", "MYA", "Myanmar"),
                ("NAM", "NAM", "Namibia"),
                ("NCA", "NCA", "Nicaragua"),
                ("NED", "NED", "Netherlands"),
                ("NEP", "NEP", "Nepal"),
                ("NGR", "NGR", "Nigeria"),
                ("NIG", "NIG", "Niger"),
                ("NIU", "NIU", "Niue"),
                ("NOR", "NOR", "Norway"),
                ("NRU", "NRU", "Nauru"),
                ("NZL", "NZL", "New Zealand"),
                ("OMA", "OMA", "Oman"),
                ("PAK", "PAK", "Pakistan"),
                ("PAN", "PAN", "Panama"),
                ("PAR", "PAR", "Paraguay"),
                ("PER", "PER", "Peru"),
                ("PHI", "PHI", "Philippines"),
                ("PLE", "PLE", "Palestine"),
                ("PLW", "PLW", "Palau"),
                ("PNG", "PNG", "Papua New Guinea"),
                ("POL", "POL", "Poland"),
                ("POR", "POR", "Portugal"),
                ("PRK", "PRK", "North Korea"),
                ("PUR", "PUR", "Puerto Rico"),
                ("QAT", "QAT", "Qatar"),
                ("ROU", "ROU", "Romania"),
                ("RSA", "RSA", "South Africa"),
                ("RUS", "RUS", "Russia"),
                ("RWA", "RWA", "Rwanda"),
                ("SAM", "SAM", "Samoa"),
                ("SEN", "SEN", "Senegal"),
                ("SEY", "SEY", "Seychelles"),
                ("SGP", "SGP", "Singapore"),
                ("SKN", "SKN", "Saint Kitts and Nevis"),
                ("SLE", "SLE", "Sierra Leone"),
                ("SLO", "SLO", "Slovenia"),
                ("SMR", "SMR", "San Marino"),
                ("SOL", "SOL", "Solomon Islands"),
                ("SOM", "SOM", "Somalia"),
                ("SRB", "SRB", "Serbia"),
                ("SRI", "SRI", "Sri Lanka"),
                ("SSD", "SSD", "South Sudan"),
                ("STP", "STP", "São Tomé and Príncipe"),
                ("SUD", "SUD", "Sudan"),
                ("SUI", "SUI", "Switzerland"),
                ("SUR", "SUR", "Suriname"),
                ("SVK", "SVK", "Slovakia"),
                ("SWE", "SWE", "Sweden"),
                ("SWZ", "SWZ", "Eswatini"),
                ("TAN", "TAN", "Tanzania"),
                ("TGA", "TGA", "Tonga"),
                ("THA", "THA", "Thailand"),
                ("TJK", "TJK", "Tajikistan"),
                ("TKM", "TKM", "Turkmenistan"),
                ("TLS", "TLS", "East Timor"),
                ("TOG", "TOG", "Togo"),
                ("TPE", "TPE", "Chinese Taipei"),
                ("TTO", "TTO", "Trinidad and Tobago"),
                ("TUN", "TUN", "Tunisia"),
                ("TUR", "TUR", "Turkey"),
                ("TUV", "TUV", "Tuvalu"),
                ("UAE", "UAE", "United Arab Emirates"),
                ("UGA", "UGA", "Uganda"),
                ("UKR", "UKR", "Ukraine"),
                ("URU", "URU", "Uruguay"),
                ("USA", "USA", "United States"),
                ("UZB", "UZB", "Uzbekistan"),
                ("VAN", "VAN", "Vanuatu"),
                ("VEN", "VEN", "Venezuela"),
                ("VIE", "VIE", "Vietnam"),
                ("VIN", "VIN", "Saint Vincent and the Grenadines"),
                ("YEM", "YEM", "Yemen"),
                ("ZAM", "ZAM", "Zambia"),
                ("ZIM", "ZIM", "Zimbabwe"),
                
                // Historic NOCs (Table 3)
                ("URS", "URS", "Soviet Union"),
                ("YUG", "YUG", "Yugoslavia"),
                ("GDR", "GDR", "East Germany"),
                ("FRG", "FRG", "West Germany"),
                ("TCH", "TCH", "Czechoslovakia"),
                ("SCG", "SCG", "Serbia and Montenegro"),
                ("ANZ", "ANZ", "Australasia"),
                ("BWI", "BWI", "British West Indies"),
                ("EUA", "EUA", "United Team of Germany"),
                ("EUN", "EUN", "Unified Team"),
                ("RHO", "RHO", "Rhodesia"),
                ("SAA", "SAA", "Saar"),
                
                // Historic Country Names (Table 4)
                ("BIR", "BIR", "Burma"),
                ("CEY", "CEY", "Ceylon"),
                ("DAH", "DAH", "Dahomey"),
                ("RVN", "RVN", "South Vietnam"),
                ("VOL", "VOL", "Upper Volta"),
                ("YAR", "YAR", "North Yemen"),
                ("YMD", "YMD", "South Yemen"),
                ("ZAI", "ZAI", "Zaire"),
                
                // Special Olympic Codes (Table 5)
                ("EOR", "EOR", "Refugee Olympic Team"),
                ("IOP", "IOP", "Independent Olympic Participants"),
                ("OAR", "OAR", "Olympic Athletes from Russia"),
                ("ROC", "ROC", "Russian Olympic Committee"),
                ("ANA", "ANA", "Authorized Neutral Athletes"),
                ("IOA", "IOA", "Independent Olympic Athletes"),
                ("IPA", "IPA", "Independent Paralympic Athletes"),
                ("NPA", "NPA", "Neutral Paralympic Athletes"),
                ("RPC", "RPC", "Russian Paralympic Committee"),
                ("MIX", "MIX", "Mixed Team"),
                
                // Special Paralympic Codes (Table 6)
                ("IPP", "IPP", "Independent Paralympic Participants"),
                ("NRH", "NRH", "Neutral Paralympic Team"),
                ("AIN", "AIN", "Individual Neutral Athletes"),
                ("COR", "COR", "Unified Korea"),
                ("HBR", "HBR", "New Hebrides"),
                
                // Additional territories and special codes
                ("FRO", "FRO", "Faroe Islands"),
                ("GBS", "GBS", "Guinea-Bissau"),
                ("NFK", "NFK", "Norfolk Island"),
                ("NMI", "NMI", "Northern Mariana Islands"),
                ("AHO", "AHO", "Netherlands Antilles"),
                ("BOH", "BOH", "Bohemia"),
                ("IOC", "IOC", "International Olympic Committee"),
            ];
            
            let current_time = chrono::Utc::now().to_rfc3339();
            let ioc_flags_count = ioc_flags.len();
            
            for (pss_code, ioc_code, country_name) in ioc_flags {
                conn.execute(
                    "INSERT OR IGNORE INTO flag_mappings (pss_code, ioc_code, country_name, is_custom, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                    [pss_code, ioc_code, country_name, "0", &current_time, &current_time],
                )?;
            }
            
            log::info!("Successfully populated flag_mappings table with {} IOC entries", ioc_flags_count);
        } else {
            log::info!("flag_mappings table already contains {} entries, skipping population", mapping_count);
        }
        
        Ok(())
    }
    
    fn down(&self, conn: &Connection) -> SqliteResult<()> {
        conn.execute("DROP TABLE IF EXISTS recognition_history", [])?;
        conn.execute("DROP TABLE IF EXISTS flags", [])?;
        // Note: We don't drop flag_mappings as it was created in Migration 1
        Ok(())
    }
}

/// Migration 4: PSS and UDP subsystem integration with normalization
pub struct Migration4;

impl Migration for Migration4 {
    fn version(&self) -> u32 {
        4
    }
    
    fn description(&self) -> &str {
        "PSS and UDP subsystem integration with network interfaces, server statistics, enhanced events, and normalized relationships"
    }
    
    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // Create network_interfaces table for UDP server configuration
        conn.execute(
            "CREATE TABLE IF NOT EXISTS network_interfaces (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                address TEXT NOT NULL,
                netmask TEXT,
                broadcast TEXT,
                is_loopback BOOLEAN NOT NULL DEFAULT 0,
                is_active BOOLEAN NOT NULL DEFAULT 0,
                is_recommended BOOLEAN NOT NULL DEFAULT 0,
                speed_mbps INTEGER,
                mtu INTEGER,
                mac_address TEXT,
                interface_type TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(name, address)
            )",
            [],
        )?;
        
        // Create udp_server_configs table for UDP server settings
        conn.execute(
            "CREATE TABLE IF NOT EXISTS udp_server_configs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                port INTEGER NOT NULL,
                bind_address TEXT NOT NULL,
                network_interface_id INTEGER,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                auto_start BOOLEAN NOT NULL DEFAULT 0,
                max_packet_size INTEGER DEFAULT 1024,
                buffer_size INTEGER DEFAULT 8192,
                timeout_ms INTEGER DEFAULT 1000,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (network_interface_id) REFERENCES network_interfaces(id)
            )",
            [],
        )?;
        
        // Create udp_server_sessions table for tracking server runtime sessions
        conn.execute(
            "CREATE TABLE IF NOT EXISTS udp_server_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                server_config_id INTEGER NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT,
                status TEXT NOT NULL DEFAULT 'running',
                packets_received INTEGER DEFAULT 0,
                packets_parsed INTEGER DEFAULT 0,
                parse_errors INTEGER DEFAULT 0,
                total_bytes_received INTEGER DEFAULT 0,
                average_packet_size REAL DEFAULT 0.0,
                max_packet_size_seen INTEGER DEFAULT 0,
                min_packet_size_seen INTEGER DEFAULT 0,
                unique_clients_count INTEGER DEFAULT 0,
                error_message TEXT,
                FOREIGN KEY (server_config_id) REFERENCES udp_server_configs(id)
            )",
            [],
        )?;
        
        // Create udp_client_connections table for tracking client connections
        conn.execute(
            "CREATE TABLE IF NOT EXISTS udp_client_connections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                client_address TEXT NOT NULL,
                client_port INTEGER NOT NULL,
                first_seen TEXT NOT NULL,
                last_seen TEXT NOT NULL,
                packets_received INTEGER DEFAULT 0,
                total_bytes_received INTEGER DEFAULT 0,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                FOREIGN KEY (session_id) REFERENCES udp_server_sessions(id),
                UNIQUE(session_id, client_address, client_port)
            )",
            [],
        )?;
        
        // Create pss_event_types table for normalized event type definitions
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pss_event_types (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_code TEXT NOT NULL UNIQUE,
                event_name TEXT NOT NULL,
                description TEXT,
                category TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create pss_matches table for match information
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pss_matches (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id TEXT NOT NULL UNIQUE,
                match_number INTEGER,
                category TEXT,
                weight_class TEXT,
                division TEXT,
                total_rounds INTEGER DEFAULT 3,
                round_duration INTEGER, -- in seconds
                countdown_type TEXT,
                format_type INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create pss_athletes table for athlete information
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pss_athletes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                athlete_code TEXT NOT NULL UNIQUE,
                short_name TEXT NOT NULL,
                long_name TEXT,
                country_code TEXT,
                flag_id INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (flag_id) REFERENCES flags(id)
            )",
            [],
        )?;
        
        // Create pss_match_athletes table for match-athlete relationships
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pss_match_athletes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id INTEGER NOT NULL,
                athlete_id INTEGER NOT NULL,
                athlete_position INTEGER NOT NULL, -- 1 or 2
                bg_color TEXT,
                fg_color TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (match_id) REFERENCES pss_matches(id),
                FOREIGN KEY (athlete_id) REFERENCES pss_athletes(id),
                UNIQUE(match_id, athlete_position)
            )",
            [],
        )?;
        
        // Create pss_rounds table for round information
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pss_rounds (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id INTEGER NOT NULL,
                round_number INTEGER NOT NULL,
                start_time TEXT,
                end_time TEXT,
                duration INTEGER, -- in seconds
                winner_athlete_position INTEGER, -- 1, 2, or NULL for draw
                created_at TEXT NOT NULL,
                FOREIGN KEY (match_id) REFERENCES pss_matches(id),
                UNIQUE(match_id, round_number)
            )",
            [],
        )?;
        
        // Enhanced pss_events table with normalized relationships
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pss_events_v2 (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                match_id INTEGER,
                round_id INTEGER,
                event_type_id INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                raw_data TEXT NOT NULL,
                parsed_data TEXT, -- JSON serialized parsed event data
                event_sequence INTEGER, -- sequence number within session
                processing_time_ms INTEGER, -- time taken to parse event
                is_valid BOOLEAN NOT NULL DEFAULT 1,
                error_message TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES udp_server_sessions(id),
                FOREIGN KEY (match_id) REFERENCES pss_matches(id),
                FOREIGN KEY (round_id) REFERENCES pss_rounds(id),
                FOREIGN KEY (event_type_id) REFERENCES pss_event_types(id)
            )",
            [],
        )?;
        
        // Create pss_event_details table for event-specific data
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pss_event_details (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id INTEGER NOT NULL,
                detail_key TEXT NOT NULL,
                detail_value TEXT,
                detail_type TEXT NOT NULL, -- string, integer, float, boolean, json
                created_at TEXT NOT NULL,
                FOREIGN KEY (event_id) REFERENCES pss_events_v2(id),
                UNIQUE(event_id, detail_key)
            )",
            [],
        )?;
        
        // Create pss_scores table for score tracking
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pss_scores (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id INTEGER NOT NULL,
                round_id INTEGER,
                athlete_position INTEGER NOT NULL, -- 1 or 2
                score_type TEXT NOT NULL, -- current, round1, round2, round3, total
                score_value INTEGER NOT NULL DEFAULT 0,
                timestamp TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (match_id) REFERENCES pss_matches(id),
                FOREIGN KEY (round_id) REFERENCES pss_rounds(id)
            )",
            [],
        )?;
        
        // Create pss_warnings table for warning/gam-jeom tracking
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pss_warnings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id INTEGER NOT NULL,
                round_id INTEGER,
                athlete_position INTEGER NOT NULL, -- 1 or 2
                warning_type TEXT NOT NULL, -- warning, gam_jeom
                warning_count INTEGER NOT NULL DEFAULT 0,
                timestamp TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (match_id) REFERENCES pss_matches(id),
                FOREIGN KEY (round_id) REFERENCES pss_rounds(id)
            )",
            [],
        )?;
        
        // Create indices for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_network_interfaces_active ON network_interfaces(is_active, is_recommended)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_udp_server_configs_enabled ON udp_server_configs(enabled)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_udp_server_sessions_status ON udp_server_sessions(status, start_time)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_udp_client_connections_session ON udp_client_connections(session_id, is_active)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_events_v2_timestamp ON pss_events_v2(timestamp)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_events_v2_match ON pss_events_v2(match_id, round_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_events_v2_session ON pss_events_v2(session_id, event_sequence)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_event_details_event ON pss_event_details(event_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_scores_match ON pss_scores(match_id, round_id, athlete_position)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_warnings_match ON pss_warnings(match_id, round_id, athlete_position)",
            [],
        )?;
        
        // Populate pss_event_types with standard PSS event types
        let event_types = vec![
            // Points events
            ("pt1", "Points Athlete 1", "Points scored by athlete 1", "points"),
            ("pt2", "Points Athlete 2", "Points scored by athlete 2", "points"),
            
            // Hit level events
            ("hl1", "Hit Level Athlete 1", "Hit level for athlete 1", "hit_level"),
            ("hl2", "Hit Level Athlete 2", "Hit level for athlete 2", "hit_level"),
            
            // Warnings/Gam-jeom events
            ("wg1", "Warnings Athlete 1", "Warnings for athlete 1", "warnings"),
            ("wg2", "Warnings Athlete 2", "Warnings for athlete 2", "warnings"),
            
            // Injury events
            ("ij0", "Injury Unidentified", "Injury time for unidentified athlete", "injury"),
            ("ij1", "Injury Athlete 1", "Injury time for athlete 1", "injury"),
            ("ij2", "Injury Athlete 2", "Injury time for athlete 2", "injury"),
            
            // Challenge/IVR events
            ("ch0", "Challenge Referee", "Challenge initiated by referee", "challenge"),
            ("ch1", "Challenge Athlete 1", "Challenge initiated by athlete 1", "challenge"),
            ("ch2", "Challenge Athlete 2", "Challenge initiated by athlete 2", "challenge"),
            
            // Break events
            ("br", "Break", "Match break time", "break"),
            
            // Winner events
            ("wr", "Winner Rounds", "Round winners", "winner"),
            ("wn", "Winner", "Match winner", "winner"),
            
            // Athlete events
            ("at", "Athletes", "Athlete information", "athletes"),
            
            // Match configuration
            ("mc", "Match Config", "Match configuration", "match_config"),
            
            // Scores
            ("sc", "Scores", "Current scores", "scores"),
            
            // Clock events
            ("cl", "Clock", "Match clock", "clock"),
            
            // Round events
            ("rd", "Round", "Round information", "round"),
            
            // System events
            ("fl", "Fight Loaded", "Fight loaded event", "system"),
            ("fr", "Fight Ready", "Fight ready event", "system"),
        ];
        
        for (code, name, description, category) in event_types {
            conn.execute(
                "INSERT OR IGNORE INTO pss_event_types (event_code, event_name, description, category, created_at) VALUES (?, ?, ?, ?, datetime('now'))",
                [code, name, description, category],
            )?;
        }
        
        // Insert default UDP server configuration
        conn.execute(
            "INSERT OR IGNORE INTO udp_server_configs (name, port, bind_address, enabled, auto_start, created_at, updated_at) VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
            ["Default PSS Server", &6000.to_string(), "0.0.0.0", &1.to_string(), &0.to_string()],
        )?;
        
        Ok(())
    }
    
    fn down(&self, conn: &Connection) -> SqliteResult<()> {
        // Drop tables in reverse order
        conn.execute("DROP TABLE IF EXISTS pss_warnings", [])?;
        conn.execute("DROP TABLE IF EXISTS pss_scores", [])?;
        conn.execute("DROP TABLE IF EXISTS pss_event_details", [])?;
        conn.execute("DROP TABLE IF EXISTS pss_events_v2", [])?;
        conn.execute("DROP TABLE IF EXISTS pss_rounds", [])?;
        conn.execute("DROP TABLE IF EXISTS pss_match_athletes", [])?;
        conn.execute("DROP TABLE IF EXISTS pss_athletes", [])?;
        conn.execute("DROP TABLE IF EXISTS pss_matches", [])?;
        conn.execute("DROP TABLE IF EXISTS pss_event_types", [])?;
        conn.execute("DROP TABLE IF EXISTS udp_client_connections", [])?;
        conn.execute("DROP TABLE IF EXISTS udp_server_sessions", [])?;
        conn.execute("DROP TABLE IF EXISTS udp_server_configs", [])?;
        conn.execute("DROP TABLE IF EXISTS network_interfaces", [])?;
        
        Ok(())
    }
}

/// Migration 5: Tournament management system
pub struct Migration5;

impl Migration for Migration5 {
    fn version(&self) -> u32 {
        5
    }
    
    fn description(&self) -> &str {
        "Tournament management system with tournaments, tournament days, and PSS integration"
    }
    
    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // Create tournaments table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tournaments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                duration_days INTEGER NOT NULL DEFAULT 1,
                city TEXT NOT NULL,
                country TEXT NOT NULL,
                country_code TEXT,
                logo_path TEXT,
                status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'active', 'ended')),
                start_date TEXT,
                end_date TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create tournament_days table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tournament_days (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tournament_id INTEGER NOT NULL,
                day_number INTEGER NOT NULL,
                date DATE NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'active', 'completed')),
                start_time TEXT,
                end_time TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE,
                UNIQUE(tournament_id, day_number)
            )",
            [],
        )?;
        
        // Add tournament_id to existing PSS tables
        conn.execute(
            "ALTER TABLE pss_matches ADD COLUMN tournament_id INTEGER REFERENCES tournaments(id)",
            [],
        )?;
        
        conn.execute(
            "ALTER TABLE pss_events_v2 ADD COLUMN tournament_id INTEGER REFERENCES tournaments(id)",
            [],
        )?;
        
        conn.execute(
            "ALTER TABLE pss_scores ADD COLUMN tournament_id INTEGER REFERENCES tournaments(id)",
            [],
        )?;
        
        conn.execute(
            "ALTER TABLE pss_warnings ADD COLUMN tournament_id INTEGER REFERENCES tournaments(id)",
            [],
        )?;
        
        // Add tournament_day_id to PSS tables
        conn.execute(
            "ALTER TABLE pss_matches ADD COLUMN tournament_day_id INTEGER REFERENCES tournament_days(id)",
            [],
        )?;
        
        conn.execute(
            "ALTER TABLE pss_events_v2 ADD COLUMN tournament_day_id INTEGER REFERENCES tournament_days(id)",
            [],
        )?;
        
        conn.execute(
            "ALTER TABLE pss_scores ADD COLUMN tournament_day_id INTEGER REFERENCES tournament_days(id)",
            [],
        )?;
        
        conn.execute(
            "ALTER TABLE pss_warnings ADD COLUMN tournament_day_id INTEGER REFERENCES tournament_days(id)",
            [],
        )?;
        
        // Create indices for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tournaments_status ON tournaments(status)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tournaments_city_country ON tournaments(city, country)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tournament_days_tournament ON tournament_days(tournament_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tournament_days_status ON tournament_days(status)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_matches_tournament ON pss_matches(tournament_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_events_v2_tournament ON pss_events_v2(tournament_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_scores_tournament ON pss_scores(tournament_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_warnings_tournament ON pss_warnings(tournament_id)",
            [],
        )?;
        
        // Insert default tournament for testing
        conn.execute(
            "INSERT OR IGNORE INTO tournaments (name, duration_days, city, country, country_code, status, start_date, end_date, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
            ["Sample Tournament", &3.to_string(), "Seoul", "South Korea", "KOR", "pending", "2025-02-01 09:00:00", "2025-02-03 18:00:00"],
        )?;
        
        Ok(())
    }
    
    fn down(&self, conn: &Connection) -> SqliteResult<()> {
        // Remove indices
        conn.execute("DROP INDEX IF EXISTS idx_tournaments_status", [])?;
        conn.execute("DROP INDEX IF EXISTS idx_tournaments_city_country", [])?;
        conn.execute("DROP INDEX IF EXISTS idx_tournament_days_tournament", [])?;
        conn.execute("DROP INDEX IF EXISTS idx_tournament_days_status", [])?;
        conn.execute("DROP INDEX IF EXISTS idx_pss_matches_tournament", [])?;
        conn.execute("DROP INDEX IF EXISTS idx_pss_events_v2_tournament", [])?;
        conn.execute("DROP INDEX IF EXISTS idx_pss_scores_tournament", [])?;
        conn.execute("DROP INDEX IF EXISTS idx_pss_warnings_tournament", [])?;
        
        // Note: SQLite doesn't support DROP COLUMN, so we'll need to recreate tables
        // For now, we'll just drop the tournament tables
        conn.execute("DROP TABLE IF EXISTS tournament_days", [])?;
        conn.execute("DROP TABLE IF EXISTS tournaments", [])?;
        
        Ok(())
    }
}

/// Migration 6: Fix date column types for tournament tables
pub struct Migration6;

pub struct Migration7;

impl Migration for Migration6 {
    fn version(&self) -> u32 {
        6
    }
    
    fn description(&self) -> &str {
        "Fix date column types from DATETIME to TEXT for tournament tables"
    }
    
    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // SQLite doesn't support ALTER COLUMN TYPE, so we need to recreate the tables
        // First, create new tables with correct column types
        
        // Create new tournaments table with TEXT date columns
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tournaments_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                duration_days INTEGER NOT NULL DEFAULT 1,
                city TEXT NOT NULL,
                country TEXT NOT NULL,
                country_code TEXT,
                logo_path TEXT,
                status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'active', 'ended')),
                start_date TEXT,
                end_date TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create new tournament_days table with TEXT date columns
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tournament_days_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tournament_id INTEGER NOT NULL,
                day_number INTEGER NOT NULL,
                date TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'active', 'completed')),
                start_time TEXT,
                end_time TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (tournament_id) REFERENCES tournaments_new(id) ON DELETE CASCADE,
                UNIQUE(tournament_id, day_number)
            )",
            [],
        )?;
        
        // Copy data from old tables to new tables (if they exist)
        conn.execute(
            "INSERT INTO tournaments_new SELECT * FROM tournaments",
            [],
        ).ok(); // Ignore error if old table doesn't exist
        
        conn.execute(
            "INSERT INTO tournament_days_new SELECT * FROM tournament_days",
            [],
        ).ok(); // Ignore error if old table doesn't exist
        
        // Drop old tables
        conn.execute("DROP TABLE IF EXISTS tournament_days", []).ok();
        conn.execute("DROP TABLE IF EXISTS tournaments", []).ok();
        
        // Rename new tables to original names
        conn.execute("ALTER TABLE tournaments_new RENAME TO tournaments", []).ok();
        conn.execute("ALTER TABLE tournament_days_new RENAME TO tournament_days", []).ok();
        
        // Recreate indices
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tournaments_status ON tournaments(status)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tournaments_city_country ON tournaments(city, country)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tournament_days_tournament ON tournament_days(tournament_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tournament_days_status ON tournament_days(status)",
            [],
        )?;
        
        Ok(())
    }
    
    fn down(&self, _conn: &Connection) -> SqliteResult<()> {
        // This migration is mostly about fixing column types
        // The down migration would be complex and not really needed
        // Just log that this is a one-way migration
        println!("Migration 6 is a one-way migration to fix column types");
        Ok(())
    }
}

impl Migration for Migration7 {
    fn version(&self) -> u32 {
        7
    }

    fn description(&self) -> &str {
        "Add database indexes for performance optimization"
    }

    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // PSS Events indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_events_v2_session_id ON pss_events_v2(session_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_events_v2_match_id ON pss_events_v2(match_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_events_v2_event_type_id ON pss_events_v2(event_type_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_events_v2_timestamp ON pss_events_v2(timestamp)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_events_v2_created_at ON pss_events_v2(created_at)", [])?;

        // PSS Event Types indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_event_types_event_code ON pss_event_types(event_code)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_event_types_category ON pss_event_types(category)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_event_types_is_active ON pss_event_types(is_active)", [])?;

        // PSS Matches indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_matches_match_id ON pss_matches(match_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_matches_created_at ON pss_matches(created_at)", [])?;

        // PSS Athletes indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_athletes_athlete_code ON pss_athletes(athlete_code)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_athletes_created_at ON pss_athletes(created_at)", [])?;

        // PSS Scores indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_scores_match_id ON pss_scores(match_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_scores_athlete_position ON pss_scores(athlete_position)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_scores_timestamp ON pss_scores(timestamp)", [])?;

        // PSS Warnings indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_warnings_match_id ON pss_warnings(match_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_warnings_athlete_position ON pss_warnings(athlete_position)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pss_warnings_timestamp ON pss_warnings(timestamp)", [])?;

        // UDP Server Configs indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_udp_server_configs_name ON udp_server_configs(name)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_udp_server_configs_enabled ON udp_server_configs(enabled)", [])?;

        // UDP Server Sessions indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_udp_server_sessions_config_id ON udp_server_sessions(server_config_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_udp_server_sessions_status ON udp_server_sessions(status)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_udp_server_sessions_start_time ON udp_server_sessions(start_time)", [])?;

        // UDP Client Connections indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_udp_client_connections_session_id ON udp_client_connections(session_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_udp_client_connections_client_address ON udp_client_connections(client_address)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_udp_client_connections_first_seen ON udp_client_connections(first_seen)", [])?;

        // Network Interfaces indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_network_interfaces_name ON network_interfaces(name)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_network_interfaces_is_active ON network_interfaces(is_active)", [])?;

        // Tournaments indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_tournaments_name ON tournaments(name)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_tournaments_status ON tournaments(status)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_tournaments_start_date ON tournaments(start_date)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_tournaments_created_at ON tournaments(created_at)", [])?;

        // Tournament Days indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_tournament_days_tournament_id ON tournament_days(tournament_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_tournament_days_status ON tournament_days(status)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_tournament_days_date ON tournament_days(date)", [])?;

        // Settings indexes (from Migration2)
        conn.execute("CREATE INDEX IF NOT EXISTS idx_settings_keys_name ON settings_keys(key_name)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_settings_keys_category ON settings_keys(category_id)", [])?;

        // Settings Categories indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_settings_categories_name ON settings_categories(name)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_settings_categories_display_order ON settings_categories(display_order)", [])?;

        // Settings Values indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_settings_values_key ON settings_values(key_id)", [])?;

        // Settings History indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_settings_history_key ON settings_history(key_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_settings_history_created ON settings_history(created_at)", [])?;

        // Schema Version indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_schema_version_version ON schema_version(version)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_schema_version_applied_at ON schema_version(applied_at)", [])?;

        log::info!("✅ Database indexes created successfully");
        Ok(())
    }

    fn down(&self, conn: &Connection) -> SqliteResult<()> {
        // Drop all indexes
        let indexes = [
            "idx_pss_events_v2_session_id", "idx_pss_events_v2_match_id", "idx_pss_events_v2_event_type_id",
            "idx_pss_events_v2_timestamp", "idx_pss_events_v2_created_at", "idx_pss_event_types_event_code",
            "idx_pss_event_types_category", "idx_pss_event_types_is_active", "idx_pss_matches_match_id",
            "idx_pss_matches_created_at", "idx_pss_athletes_athlete_code", "idx_pss_athletes_created_at",
            "idx_pss_scores_match_id", "idx_pss_scores_athlete_position", "idx_pss_scores_timestamp",
            "idx_pss_warnings_match_id", "idx_pss_warnings_athlete_position", "idx_pss_warnings_timestamp",
            "idx_udp_server_configs_name", "idx_udp_server_configs_enabled", "idx_udp_server_sessions_config_id",
            "idx_udp_server_sessions_status", "idx_udp_server_sessions_start_time", "idx_udp_client_connections_session_id",
            "idx_udp_client_connections_client_address", "idx_udp_client_connections_first_seen", "idx_network_interfaces_name",
            "idx_network_interfaces_is_active", 
            "idx_tournaments_name", "idx_tournaments_status", "idx_tournaments_start_date", "idx_tournaments_created_at", 
            "idx_tournament_days_tournament_id", "idx_tournament_days_status", "idx_tournament_days_date", 
            "idx_settings_keys_name", "idx_settings_keys_category", "idx_settings_categories_name", 
            "idx_settings_categories_display_order", "idx_settings_values_key", "idx_settings_history_key", "idx_settings_history_created", 
            "idx_schema_version_version", "idx_schema_version_applied_at"
        ];

        for index in &indexes {
            if let Err(e) = conn.execute(&format!("DROP INDEX IF EXISTS {}", index), []) {
                log::warn!("Failed to drop index {}: {}", index, e);
            }
        }

        log::info!("✅ Database indexes dropped successfully");
        Ok(())
    }
}

/// Migration 8: PSS Event Status Mark System and Recognition History
pub struct Migration8;

impl Migration for Migration8 {
    fn version(&self) -> u32 {
        8
    }

    fn description(&self) -> &str {
        "PSS Event Status Mark System with recognition history, unknown event collection, and enhanced validation"
    }

    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // Add recognition_status field to pss_events_v2 table
        conn.execute(
            "ALTER TABLE pss_events_v2 ADD COLUMN recognition_status TEXT NOT NULL DEFAULT 'recognized' CHECK (recognition_status IN ('recognized', 'unknown', 'partial', 'deprecated'))",
            [],
        )?;

        // Add protocol_version field to track which protocol version was used for parsing
        conn.execute(
            "ALTER TABLE pss_events_v2 ADD COLUMN protocol_version TEXT DEFAULT '2.3'",
            [],
        )?;

        // Add parser_confidence field to store confidence scores
        conn.execute(
            "ALTER TABLE pss_events_v2 ADD COLUMN parser_confidence REAL DEFAULT 1.0 CHECK (parser_confidence >= 0.0 AND parser_confidence <= 1.0)",
            [],
        )?;

        // Add validation_errors field to store validation error details
        conn.execute(
            "ALTER TABLE pss_events_v2 ADD COLUMN validation_errors TEXT",
            [],
        )?;

        // Create pss_event_recognition_history table for tracking status changes
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pss_event_recognition_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id INTEGER NOT NULL,
                old_status TEXT NOT NULL,
                new_status TEXT NOT NULL,
                changed_by TEXT NOT NULL DEFAULT 'system',
                change_reason TEXT,
                protocol_version TEXT,
                raw_data TEXT NOT NULL,
                parsed_data TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (event_id) REFERENCES pss_events_v2(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create pss_unknown_events table for collecting unrecognized events
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pss_unknown_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                raw_data TEXT NOT NULL,
                first_seen TEXT NOT NULL,
                last_seen TEXT NOT NULL,
                occurrence_count INTEGER DEFAULT 1,
                pattern_hash TEXT,
                suggested_event_type TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES udp_server_sessions(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create pss_event_validation_rules table for protocol validation
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pss_event_validation_rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_code TEXT NOT NULL,
                protocol_version TEXT NOT NULL,
                rule_name TEXT NOT NULL,
                rule_type TEXT NOT NULL CHECK (rule_type IN ('format', 'data_type', 'range', 'required', 'custom')),
                rule_definition TEXT NOT NULL,
                error_message TEXT,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(event_code, protocol_version, rule_name)
            )",
            [],
        )?;

        // Create pss_event_validation_results table for storing validation results
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pss_event_validation_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id INTEGER NOT NULL,
                rule_id INTEGER NOT NULL,
                validation_passed BOOLEAN NOT NULL,
                error_message TEXT,
                validation_time_ms INTEGER,
                created_at TEXT NOT NULL,
                FOREIGN KEY (event_id) REFERENCES pss_events_v2(id) ON DELETE CASCADE,
                FOREIGN KEY (rule_id) REFERENCES pss_event_validation_rules(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create pss_event_statistics table for tracking event processing metrics
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pss_event_statistics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                event_type_id INTEGER,
                total_events INTEGER DEFAULT 0,
                recognized_events INTEGER DEFAULT 0,
                unknown_events INTEGER DEFAULT 0,
                partial_events INTEGER DEFAULT 0,
                deprecated_events INTEGER DEFAULT 0,
                validation_errors INTEGER DEFAULT 0,
                parsing_errors INTEGER DEFAULT 0,
                average_processing_time_ms REAL DEFAULT 0.0,
                min_processing_time_ms INTEGER,
                max_processing_time_ms INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES udp_server_sessions(id) ON DELETE CASCADE,
                FOREIGN KEY (event_type_id) REFERENCES pss_event_types(id) ON DELETE SET NULL
            )",
            [],
        )?;

        // Create indices for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_events_v2_recognition_status ON pss_events_v2(recognition_status)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_events_v2_protocol_version ON pss_events_v2(protocol_version)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_events_v2_parser_confidence ON pss_events_v2(parser_confidence)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_event_recognition_history_event_id ON pss_event_recognition_history(event_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_event_recognition_history_status_change ON pss_event_recognition_history(old_status, new_status)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_event_recognition_history_created_at ON pss_event_recognition_history(created_at)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_unknown_events_session_id ON pss_unknown_events(session_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_unknown_events_pattern_hash ON pss_unknown_events(pattern_hash)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_unknown_events_first_seen ON pss_unknown_events(first_seen)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_event_validation_rules_event_code ON pss_event_validation_rules(event_code)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_event_validation_rules_protocol_version ON pss_event_validation_rules(protocol_version)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_event_validation_rules_active ON pss_event_validation_rules(is_active)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_event_validation_results_event_id ON pss_event_validation_results(event_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_event_validation_results_rule_id ON pss_event_validation_results(rule_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_event_validation_results_passed ON pss_event_validation_results(validation_passed)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_event_statistics_session_id ON pss_event_statistics(session_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pss_event_statistics_event_type_id ON pss_event_statistics(event_type_id)",
            [],
        )?;

        // Populate validation rules for PSS v2.3 protocol
        let validation_rules = vec![
            // Points events validation
            ("pt1", "2.3", "point_type_range", "range", "1-5", "Point type must be between 1 and 5"),
            ("pt2", "2.3", "point_type_range", "range", "1-5", "Point type must be between 1 and 5"),
            
            // Hit level events validation
            ("hl1", "2.3", "hit_level_range", "range", "1-100", "Hit level must be between 1 and 100"),
            ("hl2", "2.3", "hit_level_range", "range", "1-100", "Hit level must be between 1 and 100"),
            
            // Warnings events validation
            ("wg1", "2.3", "warning_count_range", "range", "0-4", "Warning count must be between 0 and 4"),
            ("wg2", "2.3", "warning_count_range", "range", "0-4", "Warning count must be between 0 and 4"),
            
            // Injury time format validation
            ("ij0", "2.3", "time_format", "format", "m:ss", "Time must be in m:ss format"),
            ("ij1", "2.3", "time_format", "format", "m:ss", "Time must be in m:ss format"),
            ("ij2", "2.3", "time_format", "format", "m:ss", "Time must be in m:ss format"),
            
            // Challenge events validation
            ("ch0", "2.3", "challenge_status", "data_type", "integer", "Challenge status must be integer"),
            ("ch1", "2.3", "challenge_status", "data_type", "integer", "Challenge status must be integer"),
            ("ch2", "2.3", "challenge_status", "data_type", "integer", "Challenge status must be integer"),
            
            // Clock events validation
            ("clk", "2.3", "time_format", "format", "m:ss", "Time must be in m:ss format"),
            
            // Round events validation
            ("rnd", "2.3", "round_number_range", "range", "1-3", "Round number must be between 1 and 3"),
            
            // Match config validation
            ("mch", "2.3", "match_number_positive", "range", "1-9999", "Match number must be positive"),
            ("mch", "2.3", "total_rounds_range", "range", "0-5", "Total rounds must be between 0 and 5"),
            ("mch", "2.3", "round_duration_positive", "range", "1-9999", "Round duration must be positive"),
        ];

        let current_time = chrono::Utc::now().to_rfc3339();
        for (event_code, protocol_version, rule_name, rule_type, rule_definition, error_message) in validation_rules {
            conn.execute(
                "INSERT OR IGNORE INTO pss_event_validation_rules (event_code, protocol_version, rule_name, rule_type, rule_definition, error_message, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                [event_code, protocol_version, rule_name, rule_type, rule_definition, error_message, &current_time, &current_time],
            )?;
        }

        log::info!("✅ Migration 8 completed: PSS Event Status Mark System added");
        Ok(())
    }

    fn down(&self, conn: &Connection) -> SqliteResult<()> {
        // Drop new tables
        conn.execute("DROP TABLE IF EXISTS pss_event_statistics", [])?;
        conn.execute("DROP TABLE IF EXISTS pss_event_validation_results", [])?;
        conn.execute("DROP TABLE IF EXISTS pss_event_validation_rules", [])?;
        conn.execute("DROP TABLE IF EXISTS pss_unknown_events", [])?;
        conn.execute("DROP TABLE IF EXISTS pss_event_recognition_history", [])?;

        // Note: SQLite doesn't support DROP COLUMN, so we can't remove the added columns
        // The columns will remain but won't affect functionality
        log::warn!("⚠️ Migration 8 rollback: New columns in pss_events_v2 table cannot be removed (SQLite limitation)");
        
        Ok(())
    }
}

pub struct Migration9;

// ---------------- Migration10 ------------------------
pub struct Migration10;

impl Migration for Migration9 {
    fn version(&self) -> u32 {
        9
    }

    fn description(&self) -> &str {
        "Add trigger system tables: obs_scenes, overlay_templates, event_triggers"
    }

    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // Create obs_scenes table for OBS scene management
        conn.execute(
            "CREATE TABLE IF NOT EXISTS obs_scenes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                scene_name TEXT NOT NULL UNIQUE,
                scene_id TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                last_seen_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create overlay_templates table for available overlays
        conn.execute(
            "CREATE TABLE IF NOT EXISTS overlay_templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                theme TEXT NOT NULL DEFAULT 'default',
                colors TEXT, -- JSON string for color configuration
                animation_type TEXT NOT NULL DEFAULT 'fade',
                duration_ms INTEGER NOT NULL DEFAULT 3000,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create event_triggers table for PSS event triggers
        conn.execute(
            "CREATE TABLE IF NOT EXISTS event_triggers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tournament_id INTEGER,
                tournament_day_id INTEGER,
                event_type TEXT NOT NULL, -- PSS event type (e.g., 'pt1', 'wg1', 'mch', etc.)
                trigger_type TEXT NOT NULL CHECK (trigger_type IN ('scene', 'overlay', 'both')),
                obs_scene_id INTEGER,
                overlay_template_id INTEGER,
                is_enabled BOOLEAN NOT NULL DEFAULT 1,
                priority INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE,
                FOREIGN KEY (tournament_day_id) REFERENCES tournament_days(id) ON DELETE CASCADE,
                FOREIGN KEY (obs_scene_id) REFERENCES obs_scenes(id) ON DELETE SET NULL,
                FOREIGN KEY (overlay_template_id) REFERENCES overlay_templates(id) ON DELETE SET NULL,
                UNIQUE(tournament_id, tournament_day_id, event_type)
            )",
            [],
        )?;

        // Create indices for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_obs_scenes_active ON obs_scenes(is_active)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_obs_scenes_name ON obs_scenes(scene_name)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_overlay_templates_active ON overlay_templates(is_active)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_overlay_templates_theme ON overlay_templates(theme)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_event_triggers_tournament ON event_triggers(tournament_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_event_triggers_tournament_day ON event_triggers(tournament_day_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_event_triggers_event_type ON event_triggers(event_type)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_event_triggers_enabled ON event_triggers(is_enabled)",
            [],
        )?;

        // Insert default overlay templates
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR IGNORE INTO overlay_templates (name, description, theme, colors, animation_type, duration_ms, is_active, created_at, updated_at) VALUES 
            ('Point Scored', 'Overlay for when a point is scored', 'default', '{\"primary\": \"#00ff00\", \"secondary\": \"#ffffff\"}', 'slide', 2000, 1, ?, ?),
            ('Warning Issued', 'Overlay for when a warning is issued', 'default', '{\"primary\": \"#ff0000\", \"secondary\": \"#ffffff\"}', 'fade', 3000, 1, ?, ?),
            ('Match Start', 'Overlay for match start', 'default', '{\"primary\": \"#0000ff\", \"secondary\": \"#ffffff\"}', 'zoom', 4000, 1, ?, ?),
            ('Round End', 'Overlay for round end', 'default', '{\"primary\": \"#ffff00\", \"secondary\": \"#000000\"}', 'slide', 2500, 1, ?, ?),
            ('Winner', 'Overlay for match winner', 'default', '{\"primary\": \"#ffd700\", \"secondary\": \"#000000\"}', 'bounce', 5000, 1, ?, ?)",
            [&now, &now, &now, &now, &now, &now, &now, &now, &now, &now],
        )?;

        log::info!("✅ Migration 9: Trigger system tables created successfully");
        Ok(())
    }

    fn down(&self, conn: &Connection) -> SqliteResult<()> {
        // Drop tables in reverse order
        conn.execute("DROP TABLE IF EXISTS event_triggers", [])?;
        conn.execute("DROP TABLE IF EXISTS overlay_templates", [])?;
        conn.execute("DROP TABLE IF EXISTS obs_scenes", [])?;

        log::warn!("⚠️ Migration 9 rollback: Trigger system tables dropped");
        Ok(())
    }
}

// ---------------- Migration10 ------------------------
impl Migration for Migration10 {
    fn version(&self) -> u32 {
        10
    }

    fn description(&self) -> &str {
        "Add action, target_type, delay_ms columns to event_triggers"
    }

    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // Add new columns with default values if they do not already exist
        // Note: SQLite's ALTER TABLE ADD COLUMN only adds if the column does not exist
        conn.execute("ALTER TABLE event_triggers ADD COLUMN action TEXT NOT NULL DEFAULT 'show'", [])?;
        conn.execute("ALTER TABLE event_triggers ADD COLUMN target_type TEXT NOT NULL DEFAULT 'scene'", [])?;
        conn.execute("ALTER TABLE event_triggers ADD COLUMN delay_ms INTEGER NOT NULL DEFAULT 0", [])?;

        log::info!("✅ Migration 10: Added action, target_type, delay_ms columns to event_triggers");
        Ok(())
    }

    fn down(&self, _conn: &Connection) -> SqliteResult<()> {
        // SQLite does not support DROP COLUMN; no-op but log warning
        log::warn!("⚠️  Migration 10 rollback: Cannot drop columns action, target_type, delay_ms due to SQLite limitations");
        Ok(())
    }
}

pub struct Migration11;

impl Migration for Migration11 {
    fn version(&self) -> u32 {
        11
    }

    fn description(&self) -> &str {
        "Add url column to overlay_templates table and clear existing data"
    }

    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // Clear existing overlay_templates data
        conn.execute("DELETE FROM overlay_templates", [])?;
        
        // Add url column to overlay_templates table
        conn.execute("ALTER TABLE overlay_templates ADD COLUMN url TEXT", [])?;
        
        log::info!("✅ Migration 11: Added url column to overlay_templates and cleared existing data");
        Ok(())
    }

    fn down(&self, _conn: &Connection) -> SqliteResult<()> {
        // SQLite does not support DROP COLUMN; no-op but log warning
        log::warn!("⚠️  Migration 11 rollback: Cannot drop url column due to SQLite limitations");
        Ok(())
    }
}

pub struct Migration12;

impl Migration for Migration12 {
    fn version(&self) -> u32 {
        12
    }

    fn description(&self) -> &str {
        "Add status and error fields to obs_connections table"
    }

    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // Add status column to obs_connections table
        conn.execute(
            "ALTER TABLE obs_connections ADD COLUMN status TEXT NOT NULL DEFAULT 'disconnected'",
            [],
        )?;
        
        // Add error column to obs_connections table
        conn.execute(
            "ALTER TABLE obs_connections ADD COLUMN error TEXT",
            [],
        )?;
        
        log::info!("✅ Migration 12: Added status and error columns to obs_connections table");
        Ok(())
    }

    fn down(&self, _conn: &Connection) -> SqliteResult<()> {
        // SQLite does not support DROP COLUMN; no-op but log warning
        log::warn!("⚠️  Migration 12 rollback: Cannot drop columns status, error due to SQLite limitations");
        Ok(())
    }
}

pub struct Migration13;

impl Migration for Migration13 {
    fn version(&self) -> u32 {
        13
    }
    
    fn description(&self) -> &str {
        "Add creation_mode field to pss_matches and update match_number to string"
    }
    
    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // Add creation_mode column to pss_matches table
        conn.execute(
            "ALTER TABLE pss_matches ADD COLUMN creation_mode TEXT NOT NULL DEFAULT 'Automatic'",
            [],
        )?;
        
        // Update existing records to have 'Automatic' creation_mode
        conn.execute(
            "UPDATE pss_matches SET creation_mode = 'Automatic' WHERE creation_mode IS NULL",
            [],
        )?;
        
        log::info!("Successfully added creation_mode field to pss_matches table");
        Ok(())
    }
    
    fn down(&self, _conn: &Connection) -> SqliteResult<()> {
        // Note: SQLite doesn't support dropping columns, so we can't easily rollback
        // This is a limitation of SQLite
        log::warn!("Cannot rollback creation_mode column addition (SQLite limitation)");
        Ok(())
    }
}

/// Migration 14: Change match_number from INTEGER to TEXT
pub struct Migration14;

impl Migration for Migration14 {
    fn version(&self) -> u32 {
        14
    }
    
    fn description(&self) -> &str {
        "Change match_number column from INTEGER to TEXT to support non-integer match numbers"
    }
    
    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // Disable foreign key constraints temporarily
        conn.execute("PRAGMA foreign_keys = OFF", [])?;
        
        // Drop the temporary table if it exists from a previous failed migration
        let _ = conn.execute("DROP TABLE IF EXISTS pss_matches_new", []);
        
        // SQLite doesn't support ALTER COLUMN TYPE, so we need to recreate the table
        // First, create a temporary table with the new schema
        conn.execute(
            "CREATE TABLE pss_matches_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id TEXT NOT NULL UNIQUE,
                match_number TEXT,
                category TEXT,
                weight_class TEXT,
                division TEXT,
                total_rounds INTEGER DEFAULT 3,
                round_duration INTEGER,
                countdown_type TEXT,
                format_type INTEGER,
                creation_mode TEXT DEFAULT 'Automatic',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Copy data from old table to new table, converting match_number to TEXT
        conn.execute(
            "INSERT INTO pss_matches_new 
             SELECT id, match_id, 
                    CASE 
                        WHEN match_number IS NULL THEN NULL 
                        ELSE CAST(match_number AS TEXT) 
                    END as match_number,
                    category, weight_class, division, total_rounds, 
                    round_duration, countdown_type, format_type, 
                    COALESCE(creation_mode, 'Automatic') as creation_mode,
                    created_at, updated_at
             FROM pss_matches",
            [],
        )?;
        
        // Drop the old table
        conn.execute("DROP TABLE pss_matches", [])?;
        
        // Rename the new table to the original name
        conn.execute("ALTER TABLE pss_matches_new RENAME TO pss_matches", [])?;
        
        // Re-enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        
        log::info!("Successfully changed match_number column from INTEGER to TEXT");
        Ok(())
    }
    
    fn down(&self, conn: &Connection) -> SqliteResult<()> {
        // Disable foreign key constraints temporarily
        conn.execute("PRAGMA foreign_keys = OFF", [])?;
        
        // Drop the temporary table if it exists from a previous failed migration
        let _ = conn.execute("DROP TABLE IF EXISTS pss_matches_old", []);
        
        // Revert back to INTEGER (this might lose data if match_number contains non-numeric values)
        conn.execute(
            "CREATE TABLE pss_matches_old (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id TEXT NOT NULL UNIQUE,
                match_number INTEGER,
                category TEXT,
                weight_class TEXT,
                division TEXT,
                total_rounds INTEGER DEFAULT 3,
                round_duration INTEGER,
                countdown_type TEXT,
                format_type INTEGER,
                creation_mode TEXT DEFAULT 'Automatic',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Copy data back, converting TEXT to INTEGER where possible
        conn.execute(
            "INSERT INTO pss_matches_old 
             SELECT id, match_id, 
                    CASE 
                        WHEN match_number IS NULL THEN NULL 
                        WHEN match_number GLOB '*[^0-9]*' THEN NULL  -- Contains non-numeric chars
                        ELSE CAST(match_number AS INTEGER) 
                    END as match_number,
                    category, weight_class, division, total_rounds, 
                    round_duration, countdown_type, format_type, 
                    COALESCE(creation_mode, 'Automatic') as creation_mode,
                    created_at, updated_at
             FROM pss_matches",
            [],
        )?;
        
        // Drop the new table
        conn.execute("DROP TABLE pss_matches", [])?;
        
        // Rename the old table back
        conn.execute("ALTER TABLE pss_matches_old RENAME TO pss_matches", [])?;
        
        // Re-enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        
        log::info!("Successfully reverted match_number column back to INTEGER");
        Ok(())
    }
}

/// Migration manager for handling database schema updates
pub struct MigrationManager {
    migrations: Vec<Box<dyn Migration>>,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new() -> Self {
        let mut migrations: Vec<Box<dyn Migration>> = Vec::new();
        migrations.push(Box::new(Migration1));
        migrations.push(Box::new(Migration2));
        migrations.push(Box::new(Migration3));
        migrations.push(Box::new(Migration4));
        migrations.push(Box::new(Migration5));
        migrations.push(Box::new(Migration6));
        migrations.push(Box::new(Migration7));
        migrations.push(Box::new(Migration8));
        migrations.push(Box::new(Migration9)); // Trigger system migration
        migrations.push(Box::new(Migration10)); // Add columns action, target_type, delay_ms
        migrations.push(Box::new(Migration11)); // Add url column to overlay_templates
        migrations.push(Box::new(Migration12)); // Add status and error columns to obs_connections
        migrations.push(Box::new(Migration13)); // Add creation_mode field to pss_matches
        migrations.push(Box::new(Migration14)); // Change match_number from INTEGER to TEXT
        migrations.push(Box::new(Migration15)); // Secure configuration storage with SHA256 encryption
        migrations.push(Box::new(Migration16)); // OBS recording configuration and session management
        migrations.push(Box::new(Migration17)); // Ensure folder_pattern column exists on obs_recording_config
        migrations.push(Box::new(Migration18)); // Triggers v2: conditions, action_kind, connection targeting
        
        Self { migrations }
    }
    
    /// Get the current schema version from the database
    pub fn get_current_version(&self, conn: &Connection) -> DatabaseResult<u32> {
        // Check if schema_version table exists
        let table_exists: i32 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |row| row.get(0),
        ).unwrap_or(0);
        
        if table_exists == 0 {
            return Ok(0);
        }
        
        // Get the highest version number
        let version: u32 = conn.query_row(
            "SELECT MAX(version) FROM schema_version",
            [],
            |row| row.get(0),
        ).unwrap_or(0);
        
        Ok(version)
    }
    
    /// Apply all pending migrations
    pub fn migrate(&self, conn: &Connection) -> DatabaseResult<()> {
        let current_version = self.get_current_version(conn)?;
        let target_version = CURRENT_SCHEMA_VERSION;
        
        if current_version == target_version {
            log::info!("Database schema is up to date (version {})", current_version);
            return Ok(());
        }
        
        if current_version > target_version {
            return Err(DatabaseError::SchemaVersion(format!(
                "Schema version mismatch: expected {}, actual {}",
                target_version, current_version
            )));
        }
        
        log::info!("Migrating database from version {} to {}", current_version, target_version);
        
        // Apply migrations in order
        for migration in &self.migrations {
            if migration.version() > current_version && migration.version() <= target_version {
                log::info!("Applying migration {}: {}", migration.version(), migration.description());
                
                // Apply the migration
                migration.up(conn)
                    .map_err(|e| DatabaseError::Migration(format!("Failed to apply migration {}: {}", migration.version(), e)))?;
                
                // Record the migration
                let schema_version = SchemaVersion::new(migration.version(), migration.description().to_string());
                conn.execute(
                    "INSERT INTO schema_version (version, applied_at, description) VALUES (?, ?, ?)",
                    [
                        &schema_version.version.to_string(),
                        &schema_version.applied_at.to_rfc3339(),
                        &schema_version.description,
                    ],
                ).map_err(|e| DatabaseError::Migration(format!("Failed to record migration {}: {}", migration.version(), e)))?;
                
                log::info!("Successfully applied migration {}", migration.version());
            }
        }
        
        log::info!("Database migration completed successfully");
        Ok(())
    }
    
    /// Rollback to a specific version
    pub fn rollback(&self, conn: &Connection, target_version: u32) -> DatabaseResult<()> {
        let current_version = self.get_current_version(conn)?;
        
        if current_version <= target_version {
            log::info!("Database is already at or below target version {}", target_version);
            return Ok(());
        }
        
        log::info!("Rolling back database from version {} to {}", current_version, target_version);
        
        // Rollback migrations in reverse order
        for migration in self.migrations.iter().rev() {
            if migration.version() <= current_version && migration.version() > target_version {
                log::info!("Rolling back migration {}: {}", migration.version(), migration.description());
                
                // Rollback the migration
                migration.down(conn)
                    .map_err(|e| DatabaseError::Migration(format!("Failed to rollback migration {}: {}", migration.version(), e)))?;
                
                // Remove the migration record
                conn.execute(
                    "DELETE FROM schema_version WHERE version = ?",
                    [migration.version()],
                ).map_err(|e| DatabaseError::Migration(format!("Failed to remove migration record {}: {}", migration.version(), e)))?;
                
                log::info!("Successfully rolled back migration {}", migration.version());
            }
        }
        
        log::info!("Database rollback completed successfully");
        Ok(())
    }
    
    /// Get migration history
    pub fn get_migration_history(&self, conn: &Connection) -> DatabaseResult<Vec<SchemaVersion>> {
        let mut stmt = conn.prepare("SELECT id, version, applied_at, description FROM schema_version ORDER BY version")?;
        let rows = stmt.query_map([], |row| SchemaVersion::from_row(row))?;
        
        let mut history = Vec::new();
        for row in rows {
            history.push(row?);
        }
        
        Ok(history)
    }
}

/// Migration 15: Secure Configuration Storage
pub struct Migration15;

impl Migration for Migration15 {
    fn version(&self) -> u32 {
        15
    }
    
    fn description(&self) -> &str {
        "Add secure configuration storage with SHA256 encryption and audit logging"
    }
    
    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // Create secure_config table for encrypted configuration storage
        conn.execute(
            "CREATE TABLE IF NOT EXISTS secure_config (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                config_key TEXT NOT NULL UNIQUE,
                encrypted_value BLOB NOT NULL,
                category TEXT NOT NULL,
                is_sensitive BOOLEAN NOT NULL DEFAULT 1,
                salt BLOB NOT NULL,
                algorithm TEXT NOT NULL DEFAULT 'AES-256-GCM',
                kdf_params TEXT NOT NULL, -- JSON with KDF parameters
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_accessed TEXT,
                access_count INTEGER DEFAULT 0,
                description TEXT
            )",
            [],
        )?;
        
        // Create index on config_key for fast lookups
        conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_secure_config_key ON secure_config(config_key)",
            [],
        )?;
        
        // Create index on category for grouped queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_secure_config_category ON secure_config(category)",
            [],
        )?;
        
        // Create index on is_sensitive for filtering
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_secure_config_sensitive ON secure_config(is_sensitive)",
            [],
        )?;
        
        // Create config_audit table for security audit logging
        conn.execute(
            "CREATE TABLE IF NOT EXISTS config_audit (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                config_key TEXT NOT NULL,
                action TEXT NOT NULL, -- 'create', 'read', 'update', 'delete'
                user_context TEXT, -- User or system context
                source_ip TEXT, -- IP address if applicable
                timestamp TEXT NOT NULL,
                details TEXT, -- Additional audit details as JSON
                success BOOLEAN NOT NULL DEFAULT 1,
                error_message TEXT
            )",
            [],
        )?;
        
        // Create index on config_key for audit queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_config_audit_key ON config_audit(config_key)",
            [],
        )?;
        
        // Create index on action for audit filtering
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_config_audit_action ON config_audit(action)",
            [],
        )?;
        
        // Create index on timestamp for time-based queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_config_audit_timestamp ON config_audit(timestamp)",
            [],
        )?;
        
        // Create security_sessions table for session management
        conn.execute(
            "CREATE TABLE IF NOT EXISTS security_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL UNIQUE,
                user_context TEXT NOT NULL,
                access_level TEXT NOT NULL, -- 'read_only', 'configuration', 'administrator'
                created_at TEXT NOT NULL,
                last_accessed TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                source_ip TEXT,
                user_agent TEXT
            )",
            [],
        )?;
        
        // Create index on session_id for session lookups
        conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_security_sessions_id ON security_sessions(session_id)",
            [],
        )?;
        
        // Create index on expires_at for cleanup
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_security_sessions_expires ON security_sessions(expires_at)",
            [],
        )?;
        
        // Create config_categories table for configuration organization
        conn.execute(
            "CREATE TABLE IF NOT EXISTS config_categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category_name TEXT NOT NULL UNIQUE,
                display_name TEXT NOT NULL,
                description TEXT,
                access_level TEXT NOT NULL DEFAULT 'configuration', -- Required access level
                is_system BOOLEAN NOT NULL DEFAULT 0, -- System vs user category
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Insert default configuration categories
        let categories = [
            ("obs_credentials", "OBS Credentials", "OBS WebSocket passwords and authentication", "configuration", true),
            ("api_keys", "API Keys", "Third-party service API keys and tokens", "administrator", true),
            ("database_config", "Database Configuration", "Database connection and settings", "administrator", true),
            ("network_secrets", "Network Secrets", "Network authentication and certificates", "administrator", true),
            ("license_info", "License Information", "License keys and activation data", "administrator", true),
            ("user_preferences", "User Preferences", "User-specific configuration settings", "read_only", false),
            ("system_config", "System Configuration", "System-level configuration settings", "administrator", true),
            ("encryption_keys", "Encryption Keys", "Encryption and security keys", "administrator", true),
        ];
        
        for (name, display, desc, access, is_system) in categories {
            conn.execute(
                "INSERT OR IGNORE INTO config_categories 
                (category_name, display_name, description, access_level, is_system, created_at, updated_at) 
                VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
                [name, display, desc, access, if is_system { "1" } else { "0" }],
            )?;
        }
        
        Ok(())
    }
    
    fn down(&self, conn: &Connection) -> SqliteResult<()> {
        conn.execute("DROP TABLE IF EXISTS config_categories", [])?;
        conn.execute("DROP TABLE IF EXISTS security_sessions", [])?;
        conn.execute("DROP TABLE IF EXISTS config_audit", [])?;
        conn.execute("DROP TABLE IF EXISTS secure_config", [])?;
        Ok(())
    }
}

/// Migration 16: OBS Recording Configuration and Sessions
pub struct Migration16;

impl Migration for Migration16 {
    fn version(&self) -> u32 {
        16
    }
    
    fn description(&self) -> &str {
        "Add OBS recording configuration and session management tables"
    }
    
    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // Create obs_recording_config table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS obs_recording_config (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                obs_connection_name TEXT NOT NULL UNIQUE,
                recording_root_path TEXT NOT NULL,
                recording_format TEXT NOT NULL DEFAULT 'mp4',
                recording_quality TEXT NOT NULL DEFAULT 'high',
                recording_bitrate INTEGER,
                recording_resolution TEXT,
                replay_buffer_enabled BOOLEAN NOT NULL DEFAULT 1,
                replay_buffer_duration INTEGER DEFAULT 30,
                auto_start_recording BOOLEAN NOT NULL DEFAULT 1,
                auto_start_replay_buffer BOOLEAN NOT NULL DEFAULT 1,
                filename_template TEXT NOT NULL DEFAULT '{matchNumber}_{player1}_{player2}_{date}',
                folder_pattern TEXT NOT NULL DEFAULT '{tournament}/{tournamentDay}',
                is_active BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Add folder_pattern column if it doesn't exist (for existing installs)
        let mut stmt = conn.prepare("PRAGMA table_info('obs_recording_config')")?;
        let mut has_folder_pattern = false;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let col_name: String = row.get(1)?;
            if col_name == "folder_pattern" { has_folder_pattern = true; break; }
        }
        if !has_folder_pattern {
            let _ = conn.execute(
                "ALTER TABLE obs_recording_config ADD COLUMN folder_pattern TEXT NOT NULL DEFAULT '{tournament}/{tournamentDay}'",
                [],
            );
        }
        
        // Create obs_recording_sessions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS obs_recording_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                obs_connection_name TEXT NOT NULL,
                tournament_id INTEGER,
                tournament_day_id INTEGER,
                match_id TEXT,
                match_number TEXT,
                player1_name TEXT,
                player1_flag TEXT,
                player2_name TEXT,
                player2_flag TEXT,
                recording_path TEXT NOT NULL,
                recording_filename TEXT NOT NULL,
                recording_start_time TEXT,
                recording_end_time TEXT,
                recording_duration INTEGER,
                recording_size_bytes INTEGER,
                replay_buffer_start_time TEXT,
                replay_buffer_end_time TEXT,
                replay_buffer_saved BOOLEAN NOT NULL DEFAULT 0,
                replay_buffer_filename TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                error_message TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE SET NULL,
                FOREIGN KEY (tournament_day_id) REFERENCES tournament_days(id) ON DELETE SET NULL
            )",
            [],
        )?;
        
        // Create indexes for efficient querying
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_obs_recording_config_connection ON obs_recording_config(obs_connection_name)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_obs_recording_config_active ON obs_recording_config(is_active)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_obs_recording_sessions_connection ON obs_recording_sessions(obs_connection_name)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_obs_recording_sessions_status ON obs_recording_sessions(status)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_obs_recording_sessions_tournament ON obs_recording_sessions(tournament_id, tournament_day_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_obs_recording_sessions_match ON obs_recording_sessions(match_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_obs_recording_sessions_created ON obs_recording_sessions(created_at)",
            [],
        )?;
        
        Ok(())
    }
    
    fn down(&self, conn: &Connection) -> SqliteResult<()> {
        // Drop the tables
        conn.execute("DROP TABLE IF EXISTS obs_recording_sessions", [])?;
        conn.execute("DROP TABLE IF EXISTS obs_recording_config", [])?;
        Ok(())
    }
} 

/// Migration 17: Ensure folder_pattern column exists on obs_recording_config
pub struct Migration17;

impl Migration for Migration17 {
    fn version(&self) -> u32 { 17 }

    fn description(&self) -> &str {
        "Ensure folder_pattern column exists on obs_recording_config"
    }

    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // Guard: table may or may not exist depending on prior installs
        // If table does not exist, create it with the correct schema (delegated to Migration16 originally),
        // but here we only ensure the missing column is present for already-created tables.
        let mut has_table = false;
        {
            let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='obs_recording_config'")?;
            let mut rows = stmt.query([])?;
            if let Some(_) = rows.next()? { has_table = true; }
        }

        if has_table {
            let mut stmt = conn.prepare("PRAGMA table_info('obs_recording_config')")?;
            let mut has_folder_pattern = false;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let col_name: String = row.get(1)?;
                if col_name == "folder_pattern" { has_folder_pattern = true; break; }
            }

            if !has_folder_pattern {
                let _ = conn.execute(
                    "ALTER TABLE obs_recording_config ADD COLUMN folder_pattern TEXT NOT NULL DEFAULT '{tournament}/{tournamentDay}'",
                    [],
                );
            }
        }

        Ok(())
    }

    fn down(&self, _conn: &Connection) -> SqliteResult<()> {
        // SQLite does not support DROP COLUMN; this is a no-op.
        Ok(())
    }
}

/// Migration 18: Triggers v2 additional columns
pub struct Migration18;

impl Migration for Migration18 {
    fn version(&self) -> u32 { 18 }
    fn description(&self) -> &str { "Add condition and action columns to event_triggers (Triggers v2)" }
    fn up(&self, conn: &Connection) -> SqliteResult<()> {
        // Helper to add a column if it does not exist
        fn add_column_if_missing(conn: &Connection, table: &str, col: &str, ddl: &str) -> SqliteResult<()> {
            let mut has_col = false;
            let mut stmt = conn.prepare(&format!("PRAGMA table_info('{}')", table))?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let name: String = row.get(1)?;
                if name == col { has_col = true; break; }
            }
            if !has_col {
                let _ = conn.execute(&format!("ALTER TABLE {} ADD COLUMN {} {}", table, col, ddl), []);
            }
            Ok(())
        }

        // New columns for event_triggers
        add_column_if_missing(conn, "event_triggers", "action_kind", "TEXT")?; // scene|overlay|record_start|record_stop|replay_save|hotkey|stinger
        add_column_if_missing(conn, "event_triggers", "obs_connection_name", "TEXT")?;
        add_column_if_missing(conn, "event_triggers", "condition_round", "INTEGER")?;
        add_column_if_missing(conn, "event_triggers", "condition_once_per", "TEXT")?; // 'round'|'match'
        add_column_if_missing(conn, "event_triggers", "debounce_ms", "INTEGER NOT NULL DEFAULT 0")?;
        add_column_if_missing(conn, "event_triggers", "cooldown_ms", "INTEGER NOT NULL DEFAULT 0")?;

        log::info!("✅ Migration 18: Added Triggers v2 columns to event_triggers");
        Ok(())
    }
    fn down(&self, _conn: &Connection) -> SqliteResult<()> {
        // SQLite cannot drop columns; no-op
        log::warn!("⚠️ Migration 18 rollback: cannot drop added columns due to SQLite limitations");
        Ok(())
    }
}