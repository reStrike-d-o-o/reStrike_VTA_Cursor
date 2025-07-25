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