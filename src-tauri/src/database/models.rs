use rusqlite::Row;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Helper function to parse DateTime from database string with fallback formats
fn parse_datetime_from_db(date_str: &str, field_name: &str) -> rusqlite::Result<DateTime<Utc>> {
    // Try to parse as RFC3339 first, then fallback to other formats
    DateTime::parse_from_rfc3339(date_str)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            // Try ISO 8601 format without timezone
            chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S")
                .map(|ndt| ndt.and_utc())
        })
        .or_else(|_| {
            // Try simple date format
            chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
                .map(|ndt| ndt.and_utc())
        })
        .or_else(|_| {
            // Try date only format
            chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map(|nd| nd.and_hms_opt(0, 0, 0).unwrap().and_utc())
        })
        .map_err(|_| rusqlite::Error::InvalidColumnType(0, field_name.to_string(), rusqlite::types::Type::Text))
}

/// PSS Event model for storing raw PSS events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssEvent {
    pub id: Option<i64>,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub match_id: Option<String>,
    pub athlete1_code: Option<String>,
    pub athlete2_code: Option<String>,
    pub score1: Option<i32>,
    pub score2: Option<i32>,
    pub round: Option<String>,
    pub weight_class: Option<String>,
    pub category: Option<String>,
    pub raw_data: String,
    pub created_at: DateTime<Utc>,
}

impl PssEvent {
    /// Create a new PSS event
    pub fn new(
        event_type: String,
        timestamp: DateTime<Utc>,
        raw_data: String,
    ) -> Self {
        Self {
            id: None,
            event_type,
            timestamp,
            match_id: None,
            athlete1_code: None,
            athlete2_code: None,
            score1: None,
            score2: None,
            round: None,
            weight_class: None,
            category: None,
            raw_data,
            created_at: Utc::now(),
        }
    }
    
    /// Create from database row
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            event_type: row.get("event_type")?,
            timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>("timestamp")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "timestamp".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            match_id: row.get("match_id")?,
            athlete1_code: row.get("athlete1_code")?,
            athlete2_code: row.get("athlete2_code")?,
            score1: row.get("score1")?,
            score2: row.get("score2")?,
            round: row.get("round")?,
            weight_class: row.get("weight_class")?,
            category: row.get("category")?,
            raw_data: row.get("raw_data")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
}

/// OBS Connection configuration model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsConnection {
    pub id: Option<i64>,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ObsConnection {
    /// Create a new OBS connection
    pub fn new(name: String, host: String, port: u16, password: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            name,
            host,
            port,
            password,
            is_active: false,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Create from database row
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            host: row.get("host")?,
            port: row.get("port")?,
            password: row.get("password")?,
            is_active: row.get("is_active")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("updated_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
}

/// Application configuration model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub id: Option<i64>,
    pub key: String,
    pub value: String,
    pub category: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AppConfig {
    /// Create a new app config entry
    pub fn new(key: String, value: String, category: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            key,
            value,
            category,
            description,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Create from database row
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            key: row.get("key")?,
            value: row.get("value")?,
            category: row.get("category")?,
            description: row.get("description")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("updated_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
}

/// Flag mapping model for PSS to IOC code mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagMapping {
    pub id: Option<i64>,
    pub pss_code: String,
    pub ioc_code: String,
    pub country_name: String,
    pub is_custom: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FlagMapping {
    /// Create a new flag mapping
    pub fn new(pss_code: String, ioc_code: String, country_name: String, is_custom: bool) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            pss_code,
            ioc_code,
            country_name,
            is_custom,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Create from database row
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            pss_code: row.get("pss_code")?,
            ioc_code: row.get("ioc_code")?,
            country_name: row.get("country_name")?,
            is_custom: row.get("is_custom")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("updated_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
}

/// Database schema version tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub id: Option<i64>,
    pub version: u32,
    pub applied_at: DateTime<Utc>,
    pub description: String,
}

impl SchemaVersion {
    /// Create a new schema version entry
    pub fn new(version: u32, description: String) -> Self {
        Self {
            id: None,
            version,
            applied_at: Utc::now(),
            description,
        }
    }
    
    /// Create from database row
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            version: row.get("version")?,
            applied_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("applied_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "applied_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            description: row.get("description")?,
        })
    }
} 

/// Settings category model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsCategory {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
}

impl SettingsCategory {
    /// Create a new settings category
    pub fn new(name: String, description: Option<String>, display_order: i32) -> Self {
        Self {
            id: None,
            name,
            description,
            display_order,
            created_at: Utc::now(),
        }
    }
    
    /// Create from database row
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            description: row.get("description")?,
            display_order: row.get("display_order")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
}

/// Settings key model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsKey {
    pub id: Option<i64>,
    pub category_id: i64,
    pub key_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub data_type: String, // 'string', 'integer', 'boolean', 'float', 'json'
    pub default_value: Option<String>,
    pub validation_rules: Option<String>, // JSON validation rules
    pub is_required: bool,
    pub is_sensitive: bool,
    pub created_at: DateTime<Utc>,
}

impl SettingsKey {
    /// Create a new settings key
    pub fn new(
        category_id: i64,
        key_name: String,
        display_name: String,
        description: Option<String>,
        data_type: String,
        default_value: Option<String>,
        validation_rules: Option<String>,
        is_required: bool,
        is_sensitive: bool,
    ) -> Self {
        Self {
            id: None,
            category_id,
            key_name,
            display_name,
            description,
            data_type,
            default_value,
            validation_rules,
            is_required,
            is_sensitive,
            created_at: Utc::now(),
        }
    }
    
    /// Create from database row
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            category_id: row.get("category_id")?,
            key_name: row.get("key_name")?,
            display_name: row.get("display_name")?,
            description: row.get("description")?,
            data_type: row.get("data_type")?,
            default_value: row.get("default_value")?,
            validation_rules: row.get("validation_rules")?,
            is_required: row.get("is_required")?,
            is_sensitive: row.get("is_sensitive")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
}

/// Settings value model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsValue {
    pub id: Option<i64>,
    pub key_id: i64,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SettingsValue {
    /// Create a new settings value
    pub fn new(key_id: i64, value: String) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            key_id,
            value,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Create from database row
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            key_id: row.get("key_id")?,
            value: row.get("value")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("updated_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
}

/// Settings history model for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsHistory {
    pub id: Option<i64>,
    pub key_id: i64,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: String, // 'user', 'system', 'migration'
    pub change_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl SettingsHistory {
    /// Create a new settings history entry
    pub fn new(
        key_id: i64,
        old_value: Option<String>,
        new_value: Option<String>,
        changed_by: String,
        change_reason: Option<String>,
    ) -> Self {
        Self {
            id: None,
            key_id,
            old_value,
            new_value,
            changed_by,
            change_reason,
            created_at: Utc::now(),
        }
    }
    
    /// Create from database row
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            key_id: row.get("key_id")?,
            old_value: row.get("old_value")?,
            new_value: row.get("new_value")?,
            changed_by: row.get("changed_by")?,
            change_reason: row.get("change_reason")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
} 

/// Network Interface model for UDP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub id: Option<i64>,
    pub name: String,
    pub address: String,
    pub netmask: Option<String>,
    pub broadcast: Option<String>,
    pub is_loopback: bool,
    pub is_active: bool,
    pub is_recommended: bool,
    pub speed_mbps: Option<i32>,
    pub mtu: Option<i32>,
    pub mac_address: Option<String>,
    pub interface_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NetworkInterface {
    pub fn new(
        name: String,
        address: String,
        is_loopback: bool,
        is_active: bool,
        is_recommended: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            name,
            address,
            netmask: None,
            broadcast: None,
            is_loopback,
            is_active,
            is_recommended,
            speed_mbps: None,
            mtu: None,
            mac_address: None,
            interface_type: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            address: row.get("address")?,
            netmask: row.get("netmask")?,
            broadcast: row.get("broadcast")?,
            is_loopback: row.get("is_loopback")?,
            is_active: row.get("is_active")?,
            is_recommended: row.get("is_recommended")?,
            speed_mbps: row.get("speed_mbps")?,
            mtu: row.get("mtu")?,
            mac_address: row.get("mac_address")?,
            interface_type: row.get("interface_type")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("updated_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
}

/// UDP Server Configuration model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpServerConfig {
    pub id: Option<i64>,
    pub name: String,
    pub port: u16,
    pub bind_address: String,
    pub network_interface_id: Option<i64>,
    pub enabled: bool,
    pub auto_start: bool,
    pub max_packet_size: i32,
    pub buffer_size: i32,
    pub timeout_ms: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UdpServerConfig {
    pub fn new(name: String, port: u16, bind_address: String) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            name,
            port,
            bind_address,
            network_interface_id: None,
            enabled: true,
            auto_start: false,
            max_packet_size: 1024,
            buffer_size: 8192,
            timeout_ms: 1000,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            port: row.get("port")?,
            bind_address: row.get("bind_address")?,
            network_interface_id: row.get("network_interface_id")?,
            enabled: row.get("enabled")?,
            auto_start: row.get("auto_start")?,
            max_packet_size: row.get("max_packet_size")?,
            buffer_size: row.get("buffer_size")?,
            timeout_ms: row.get("timeout_ms")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("updated_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
}

/// UDP Server Session model for tracking runtime sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpServerSession {
    pub id: Option<i64>,
    pub server_config_id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: String,
    pub packets_received: i32,
    pub packets_parsed: i32,
    pub parse_errors: i32,
    pub total_bytes_received: i32,
    pub average_packet_size: f64,
    pub max_packet_size_seen: i32,
    pub min_packet_size_seen: i32,
    pub unique_clients_count: i32,
    pub error_message: Option<String>,
}

impl UdpServerSession {
    pub fn new(server_config_id: i64) -> Self {
        Self {
            id: None,
            server_config_id,
            start_time: Utc::now(),
            end_time: None,
            status: "running".to_string(),
            packets_received: 0,
            packets_parsed: 0,
            parse_errors: 0,
            total_bytes_received: 0,
            average_packet_size: 0.0,
            max_packet_size_seen: 0,
            min_packet_size_seen: 0,
            unique_clients_count: 0,
            error_message: None,
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            server_config_id: row.get("server_config_id")?,
            start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>("start_time")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "start_time".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            end_time: row.get::<_, Option<String>>("end_time")?
                .map(|s| DateTime::parse_from_rfc3339(&s)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(0, "end_time".to_string(), rusqlite::types::Type::Text))
                    .map(|dt| dt.with_timezone(&Utc)))
                .transpose()?,
            status: row.get("status")?,
            packets_received: row.get("packets_received")?,
            packets_parsed: row.get("packets_parsed")?,
            parse_errors: row.get("parse_errors")?,
            total_bytes_received: row.get("total_bytes_received")?,
            average_packet_size: row.get("average_packet_size")?,
            max_packet_size_seen: row.get("max_packet_size_seen")?,
            min_packet_size_seen: row.get("min_packet_size_seen")?,
            unique_clients_count: row.get("unique_clients_count")?,
            error_message: row.get("error_message")?,
        })
    }
}

/// UDP Client Connection model for tracking client connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpClientConnection {
    pub id: Option<i64>,
    pub session_id: i64,
    pub client_address: String,
    pub client_port: u16,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub packets_received: i32,
    pub total_bytes_received: i32,
    pub is_active: bool,
}

impl UdpClientConnection {
    pub fn new(session_id: i64, client_address: String, client_port: u16) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            session_id,
            client_address,
            client_port,
            first_seen: now,
            last_seen: now,
            packets_received: 0,
            total_bytes_received: 0,
            is_active: true,
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            session_id: row.get("session_id")?,
            client_address: row.get("client_address")?,
            client_port: row.get("client_port")?,
            first_seen: DateTime::parse_from_rfc3339(&row.get::<_, String>("first_seen")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "first_seen".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            last_seen: DateTime::parse_from_rfc3339(&row.get::<_, String>("last_seen")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "last_seen".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            packets_received: row.get("packets_received")?,
            total_bytes_received: row.get("total_bytes_received")?,
            is_active: row.get("is_active")?,
        })
    }
}

/// PSS Event Type model for normalized event type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssEventType {
    pub id: Option<i64>,
    pub event_code: String,
    pub event_name: String,
    pub description: Option<String>,
    pub category: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl PssEventType {
    pub fn new(event_code: String, event_name: String, category: String, description: Option<String>) -> Self {
        Self {
            id: None,
            event_code,
            event_name,
            description,
            category,
            is_active: true,
            created_at: Utc::now(),
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            event_code: row.get("event_code")?,
            event_name: row.get("event_name")?,
            description: row.get("description")?,
            category: row.get("category")?,
            is_active: row.get("is_active")?,
            created_at: parse_datetime_from_db(&row.get::<_, String>("created_at")?, "created_at")?,
        })
    }
}

/// PSS Match model for match information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssMatch {
    pub id: Option<i64>,
    pub match_id: String,
    pub match_number: Option<i32>,
    pub category: Option<String>,
    pub weight_class: Option<String>,
    pub division: Option<String>,
    pub total_rounds: i32,
    pub round_duration: Option<i32>,
    pub countdown_type: Option<String>,
    pub format_type: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PssMatch {
    pub fn new(match_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            match_id,
            match_number: None,
            category: None,
            weight_class: None,
            division: None,
            total_rounds: 3,
            round_duration: None,
            countdown_type: None,
            format_type: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            match_id: row.get("match_id")?,
            match_number: row.get("match_number")?,
            category: row.get("category")?,
            weight_class: row.get("weight_class")?,
            division: row.get("division")?,
            total_rounds: row.get("total_rounds")?,
            round_duration: row.get("round_duration")?,
            countdown_type: row.get("countdown_type")?,
            format_type: row.get("format_type")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("updated_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
}

/// PSS Athlete model for athlete information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssAthlete {
    pub id: Option<i64>,
    pub athlete_code: String,
    pub short_name: String,
    pub long_name: Option<String>,
    pub country_code: Option<String>,
    pub flag_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PssAthlete {
    pub fn new(athlete_code: String, short_name: String) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            athlete_code,
            short_name,
            long_name: None,
            country_code: None,
            flag_id: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            athlete_code: row.get("athlete_code")?,
            short_name: row.get("short_name")?,
            long_name: row.get("long_name")?,
            country_code: row.get("country_code")?,
            flag_id: row.get("flag_id")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("updated_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
}

/// PSS Match Athlete relationship model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssMatchAthlete {
    pub id: Option<i64>,
    pub match_id: i64,
    pub athlete_id: i64,
    pub athlete_position: i32, // 1 or 2
    pub bg_color: Option<String>,
    pub fg_color: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl PssMatchAthlete {
    pub fn new(match_id: i64, athlete_id: i64, athlete_position: i32) -> Self {
        Self {
            id: None,
            match_id,
            athlete_id,
            athlete_position,
            bg_color: None,
            fg_color: None,
            created_at: Utc::now(),
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            match_id: row.get("match_id")?,
            athlete_id: row.get("athlete_id")?,
            athlete_position: row.get("athlete_position")?,
            bg_color: row.get("bg_color")?,
            fg_color: row.get("fg_color")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
}

/// PSS Round model for round information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssRound {
    pub id: Option<i64>,
    pub match_id: i64,
    pub round_number: i32,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<i32>, // in seconds
    pub winner_athlete_position: Option<i32>, // 1, 2, or None for draw
    pub created_at: DateTime<Utc>,
}

impl PssRound {
    pub fn new(match_id: i64, round_number: i32) -> Self {
        Self {
            id: None,
            match_id,
            round_number,
            start_time: None,
            end_time: None,
            duration: None,
            winner_athlete_position: None,
            created_at: Utc::now(),
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            match_id: row.get("match_id")?,
            round_number: row.get("round_number")?,
            start_time: row.get::<_, Option<String>>("start_time")?
                .map(|s| DateTime::parse_from_rfc3339(&s)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(0, "start_time".to_string(), rusqlite::types::Type::Text))
                    .map(|dt| dt.with_timezone(&Utc)))
                .transpose()?,
            end_time: row.get::<_, Option<String>>("end_time")?
                .map(|s| DateTime::parse_from_rfc3339(&s)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(0, "end_time".to_string(), rusqlite::types::Type::Text))
                    .map(|dt| dt.with_timezone(&Utc)))
                .transpose()?,
            duration: row.get("duration")?,
            winner_athlete_position: row.get("winner_athlete_position")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
}

/// Enhanced PSS Event model with normalized relationships and status marks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssEventV2 {
    pub id: Option<i64>,
    pub session_id: i64,
    pub match_id: Option<i64>,
    pub round_id: Option<i64>,
    pub event_type_id: i64,
    pub timestamp: DateTime<Utc>,
    pub raw_data: String,
    pub parsed_data: Option<String>, // JSON serialized parsed event data
    pub event_sequence: i32,
    pub processing_time_ms: Option<i32>,
    pub is_valid: bool,
    pub error_message: Option<String>,
    // New fields for status mark system
    pub recognition_status: String, // 'recognized', 'unknown', 'partial', 'deprecated'
    pub protocol_version: Option<String>,
    pub parser_confidence: Option<f64>,
    pub validation_errors: Option<String>,
    // Tournament context fields
    pub tournament_id: Option<i64>,
    pub tournament_day_id: Option<i64>,
    pub created_at: DateTime<Utc>,
}

impl PssEventV2 {
    pub fn new(
        session_id: i64,
        event_type_id: i64,
        timestamp: DateTime<Utc>,
        raw_data: String,
        event_sequence: i32,
    ) -> Self {
        Self {
            id: None,
            session_id,
            match_id: None,
            round_id: None,
            event_type_id,
            timestamp,
            raw_data,
            parsed_data: None,
            event_sequence,
            processing_time_ms: None,
            is_valid: true,
            error_message: None,
            recognition_status: "recognized".to_string(),
            protocol_version: Some("2.3".to_string()),
            parser_confidence: Some(1.0),
            validation_errors: None,
            tournament_id: None,
            tournament_day_id: None,
            created_at: Utc::now(),
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            session_id: row.get("session_id")?,
            match_id: row.get("match_id")?,
            round_id: row.get("round_id")?,
            event_type_id: row.get("event_type_id")?,
            timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>("timestamp")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "timestamp".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            raw_data: row.get("raw_data")?,
            parsed_data: row.get("parsed_data")?,
            event_sequence: row.get("event_sequence")?,
            processing_time_ms: row.get("processing_time_ms")?,
            is_valid: row.get("is_valid")?,
            error_message: row.get("error_message")?,
            recognition_status: row.get("recognition_status")?,
            protocol_version: row.get("protocol_version")?,
            parser_confidence: row.get("parser_confidence")?,
            validation_errors: row.get("validation_errors")?,
            tournament_id: row.get("tournament_id")?,
            tournament_day_id: row.get("tournament_day_id")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
}

/// PSS Event Recognition History model for tracking status changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssEventRecognitionHistory {
    pub id: Option<i64>,
    pub event_id: i64,
    pub old_status: String,
    pub new_status: String,
    pub changed_by: String,
    pub change_reason: Option<String>,
    pub protocol_version: Option<String>,
    pub raw_data: String,
    pub parsed_data: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl PssEventRecognitionHistory {
    pub fn new(
        event_id: i64,
        old_status: String,
        new_status: String,
        changed_by: String,
        raw_data: String,
    ) -> Self {
        Self {
            id: None,
            event_id,
            old_status,
            new_status,
            changed_by,
            change_reason: None,
            protocol_version: Some("2.3".to_string()),
            raw_data,
            parsed_data: None,
            created_at: Utc::now(),
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            event_id: row.get("event_id")?,
            old_status: row.get("old_status")?,
            new_status: row.get("new_status")?,
            changed_by: row.get("changed_by")?,
            change_reason: row.get("change_reason")?,
            protocol_version: row.get("protocol_version")?,
            raw_data: row.get("raw_data")?,
            parsed_data: row.get("parsed_data")?,
            created_at: parse_datetime_from_db(&row.get::<_, String>("created_at")?, "created_at")?,
        })
    }
}

/// PSS Unknown Events model for collecting unrecognized events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssUnknownEvent {
    pub id: Option<i64>,
    pub session_id: i64,
    pub raw_data: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub occurrence_count: i32,
    pub pattern_hash: Option<String>,
    pub suggested_event_type: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PssUnknownEvent {
    pub fn new(session_id: i64, raw_data: String) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            session_id,
            raw_data,
            first_seen: now,
            last_seen: now,
            occurrence_count: 1,
            pattern_hash: None,
            suggested_event_type: None,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            session_id: row.get("session_id")?,
            raw_data: row.get("raw_data")?,
            first_seen: parse_datetime_from_db(&row.get::<_, String>("first_seen")?, "first_seen")?,
            last_seen: parse_datetime_from_db(&row.get::<_, String>("last_seen")?, "last_seen")?,
            occurrence_count: row.get("occurrence_count")?,
            pattern_hash: row.get("pattern_hash")?,
            suggested_event_type: row.get("suggested_event_type")?,
            notes: row.get("notes")?,
            created_at: parse_datetime_from_db(&row.get::<_, String>("created_at")?, "created_at")?,
            updated_at: parse_datetime_from_db(&row.get::<_, String>("updated_at")?, "updated_at")?,
        })
    }
}

/// PSS Event Validation Rule model for protocol validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssEventValidationRule {
    pub id: Option<i64>,
    pub event_code: String,
    pub protocol_version: String,
    pub rule_name: String,
    pub rule_type: String, // 'format', 'data_type', 'range', 'required', 'custom'
    pub rule_definition: String,
    pub error_message: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PssEventValidationRule {
    pub fn new(
        event_code: String,
        protocol_version: String,
        rule_name: String,
        rule_type: String,
        rule_definition: String,
        error_message: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            event_code,
            protocol_version,
            rule_name,
            rule_type,
            rule_definition,
            error_message,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            event_code: row.get("event_code")?,
            protocol_version: row.get("protocol_version")?,
            rule_name: row.get("rule_name")?,
            rule_type: row.get("rule_type")?,
            rule_definition: row.get("rule_definition")?,
            error_message: row.get("error_message")?,
            is_active: row.get("is_active")?,
            created_at: parse_datetime_from_db(&row.get::<_, String>("created_at")?, "created_at")?,
            updated_at: parse_datetime_from_db(&row.get::<_, String>("updated_at")?, "updated_at")?,
        })
    }
}

/// PSS Event Validation Result model for storing validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssEventValidationResult {
    pub id: Option<i64>,
    pub event_id: i64,
    pub rule_id: i64,
    pub validation_passed: bool,
    pub error_message: Option<String>,
    pub validation_time_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

impl PssEventValidationResult {
    pub fn new(event_id: i64, rule_id: i64, validation_passed: bool) -> Self {
        Self {
            id: None,
            event_id,
            rule_id,
            validation_passed,
            error_message: None,
            validation_time_ms: None,
            created_at: Utc::now(),
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            event_id: row.get("event_id")?,
            rule_id: row.get("rule_id")?,
            validation_passed: row.get("validation_passed")?,
            error_message: row.get("error_message")?,
            validation_time_ms: row.get("validation_time_ms")?,
            created_at: parse_datetime_from_db(&row.get::<_, String>("created_at")?, "created_at")?,
        })
    }
}

/// PSS Event Statistics model for tracking event processing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssEventStatistics {
    pub id: Option<i64>,
    pub session_id: i64,
    pub event_type_id: Option<i64>,
    pub total_events: i32,
    pub recognized_events: i32,
    pub unknown_events: i32,
    pub partial_events: i32,
    pub deprecated_events: i32,
    pub validation_errors: i32,
    pub parsing_errors: i32,
    pub average_processing_time_ms: f64,
    pub min_processing_time_ms: Option<i32>,
    pub max_processing_time_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PssEventStatistics {
    pub fn new(session_id: i64, event_type_id: Option<i64>) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            session_id,
            event_type_id,
            total_events: 0,
            recognized_events: 0,
            unknown_events: 0,
            partial_events: 0,
            deprecated_events: 0,
            validation_errors: 0,
            parsing_errors: 0,
            average_processing_time_ms: 0.0,
            min_processing_time_ms: None,
            max_processing_time_ms: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            session_id: row.get("session_id")?,
            event_type_id: row.get("event_type_id")?,
            total_events: row.get("total_events")?,
            recognized_events: row.get("recognized_events")?,
            unknown_events: row.get("unknown_events")?,
            partial_events: row.get("partial_events")?,
            deprecated_events: row.get("deprecated_events")?,
            validation_errors: row.get("validation_errors")?,
            parsing_errors: row.get("parsing_errors")?,
            average_processing_time_ms: row.get("average_processing_time_ms")?,
            min_processing_time_ms: row.get("min_processing_time_ms")?,
            max_processing_time_ms: row.get("max_processing_time_ms")?,
            created_at: parse_datetime_from_db(&row.get::<_, String>("created_at")?, "created_at")?,
            updated_at: parse_datetime_from_db(&row.get::<_, String>("updated_at")?, "updated_at")?,
        })
    }
}

/// PSS Event Detail model for event-specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssEventDetail {
    pub id: Option<i64>,
    pub event_id: i64,
    pub detail_key: String,
    pub detail_value: Option<String>,
    pub detail_type: String, // string, integer, float, boolean, json
    pub created_at: DateTime<Utc>,
}

impl PssEventDetail {
    pub fn new(event_id: i64, detail_key: String, detail_value: Option<String>, detail_type: String) -> Self {
        Self {
            id: None,
            event_id,
            detail_key,
            detail_value,
            detail_type,
            created_at: Utc::now(),
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            event_id: row.get("event_id")?,
            detail_key: row.get("detail_key")?,
            detail_value: row.get("detail_value")?,
            detail_type: row.get("detail_type")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
}

/// PSS Score model for score tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssScore {
    pub id: Option<i64>,
    pub match_id: i64,
    pub round_id: Option<i64>,
    pub athlete_position: i32, // 1 or 2
    pub score_type: String, // current, round1, round2, round3, total
    pub score_value: i32,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl PssScore {
    pub fn new(match_id: i64, athlete_position: i32, score_type: String, score_value: i32) -> Self {
        Self {
            id: None,
            match_id,
            round_id: None,
            athlete_position,
            score_type,
            score_value,
            timestamp: Utc::now(),
            created_at: Utc::now(),
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            match_id: row.get("match_id")?,
            round_id: row.get("round_id")?,
            athlete_position: row.get("athlete_position")?,
            score_type: row.get("score_type")?,
            score_value: row.get("score_value")?,
            timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>("timestamp")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "timestamp".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
}

/// PSS Warning model for warning/gam-jeom tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssWarning {
    pub id: Option<i64>,
    pub match_id: i64,
    pub round_id: Option<i64>,
    pub athlete_position: i32, // 1 or 2
    pub warning_type: String, // warning, gam_jeom
    pub warning_count: i32,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl PssWarning {
    pub fn new(match_id: i64, athlete_position: i32, warning_type: String, warning_count: i32) -> Self {
        Self {
            id: None,
            match_id,
            round_id: None,
            athlete_position,
            warning_type,
            warning_count,
            timestamp: Utc::now(),
            created_at: Utc::now(),
        }
    }
    
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            match_id: row.get("match_id")?,
            round_id: row.get("round_id")?,
            athlete_position: row.get("athlete_position")?,
            warning_type: row.get("warning_type")?,
            warning_count: row.get("warning_count")?,
            timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>("timestamp")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "timestamp".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
} 

/// Tournament model for database storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tournament {
    pub id: Option<i64>,
    pub name: String,
    pub duration_days: i32,
    pub city: String,
    pub country: String,
    pub country_code: Option<String>,
    pub logo_path: Option<String>,
    pub status: String, // 'pending', 'active', 'ended'
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tournament {
    /// Create a new tournament
    pub fn new(
        name: String,
        duration_days: i32,
        city: String,
        country: String,
        country_code: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            name,
            duration_days,
            city,
            country,
            country_code,
            logo_path: None,
            status: "pending".to_string(),
            start_date: None,
            end_date: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Create from database row
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            duration_days: row.get("duration_days")?,
            city: row.get("city")?,
            country: row.get("country")?,
            country_code: row.get("country_code")?,
            logo_path: row.get("logo_path")?,
            status: row.get("status")?,
            start_date: row.get::<_, Option<String>>("start_date")?
                .map(|s| DateTime::parse_from_rfc3339(&s)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(0, "start_date".to_string(), rusqlite::types::Type::Text))
                    .map(|dt| dt.with_timezone(&Utc)))
                .transpose()?,
            end_date: row.get::<_, Option<String>>("end_date")?
                .map(|s| DateTime::parse_from_rfc3339(&s)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(0, "end_date".to_string(), rusqlite::types::Type::Text))
                    .map(|dt| dt.with_timezone(&Utc)))
                .transpose()?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("updated_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
}

/// Tournament Day model for database storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentDay {
    pub id: Option<i64>,
    pub tournament_id: i64,
    pub day_number: i32,
    pub date: DateTime<Utc>,
    pub status: String, // 'pending', 'active', 'completed'
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TournamentDay {
    /// Create a new tournament day
    pub fn new(tournament_id: i64, day_number: i32, date: DateTime<Utc>) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            tournament_id,
            day_number,
            date,
            status: "pending".to_string(),
            start_time: None,
            end_time: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Create from database row
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            tournament_id: row.get("tournament_id")?,
            day_number: row.get("day_number")?,
            date: DateTime::parse_from_rfc3339(&row.get::<_, String>("date")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "date".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            status: row.get("status")?,
            start_time: row.get::<_, Option<String>>("start_time")?
                .map(|s| DateTime::parse_from_rfc3339(&s)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(0, "start_time".to_string(), rusqlite::types::Type::Text))
                    .map(|dt| dt.with_timezone(&Utc)))
                .transpose()?,
            end_time: row.get::<_, Option<String>>("end_time")?
                .map(|s| DateTime::parse_from_rfc3339(&s)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(0, "end_time".to_string(), rusqlite::types::Type::Text))
                    .map(|dt| dt.with_timezone(&Utc)))
                .transpose()?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("updated_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }
} 

/// OBS Scene model for managing OBS scenes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsScene {
    pub id: Option<i64>,
    pub scene_name: String,
    pub scene_id: String,
    pub is_active: bool,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ObsScene {
    pub fn new(scene_name: String, scene_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            scene_name,
            scene_id,
            is_active: true,
            last_seen_at: now,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            scene_name: row.get("scene_name")?,
            scene_id: row.get("scene_id")?,
            is_active: row.get("is_active")?,
            last_seen_at: parse_datetime_from_db(&row.get::<_, String>("last_seen_at")?, "last_seen_at")?,
            created_at: parse_datetime_from_db(&row.get::<_, String>("created_at")?, "created_at")?,
            updated_at: parse_datetime_from_db(&row.get::<_, String>("updated_at")?, "updated_at")?,
        })
    }
}

/// Overlay Template model for managing overlay animations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayTemplate {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub theme: String,
    pub colors: Option<String>, // JSON string for color configuration
    pub animation_type: String,
    pub duration_ms: i32,
    pub is_active: bool,
    pub url: Option<String>, // URL/path to the overlay file
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OverlayTemplate {
    pub fn new(
        name: String,
        description: Option<String>,
        theme: String,
        colors: Option<String>,
        animation_type: String,
        duration_ms: i32,
        url: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            name,
            description,
            theme,
            colors,
            animation_type,
            duration_ms,
            is_active: true,
            url,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            description: row.get("description")?,
            theme: row.get("theme")?,
            colors: row.get("colors")?,
            animation_type: row.get("animation_type")?,
            duration_ms: row.get("duration_ms")?,
            is_active: row.get("is_active")?,
            url: row.get("url")?,
            created_at: parse_datetime_from_db(&row.get::<_, String>("created_at")?, "created_at")?,
            updated_at: parse_datetime_from_db(&row.get::<_, String>("updated_at")?, "updated_at")?,
        })
    }
}

/// Event Trigger model for PSS event triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTrigger {
    pub action: String,
    pub target_type: String,
    pub delay_ms: i64,
    pub id: Option<i64>,
    pub tournament_id: Option<i64>,
    pub tournament_day_id: Option<i64>,
    pub event_type: String, // PSS event type (e.g., 'pt1', 'wg1', 'mch', etc.)
    pub trigger_type: String, // 'scene', 'overlay', 'both'
    pub obs_scene_id: Option<i64>,
    pub overlay_template_id: Option<i64>,
    pub is_enabled: bool,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl EventTrigger {
    pub fn new(
        tournament_id: Option<i64>,
        tournament_day_id: Option<i64>,
        event_type: String,
        trigger_type: String,
        obs_scene_id: Option<i64>,
        overlay_template_id: Option<i64>,
        priority: i32,
    ) -> Self {
        let now = Utc::now();
        Self {
            action: "show".to_string(),
            target_type: "scene".to_string(),
            delay_ms: 0,
            id: None,
            tournament_id,
            tournament_day_id,
            event_type,
            trigger_type,
            obs_scene_id,
            overlay_template_id,
            is_enabled: true,
            priority,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            tournament_id: row.get("tournament_id")?,
            tournament_day_id: row.get("tournament_day_id")?,
            action: row.get("action")?,
            target_type: row.get("target_type")?,
            delay_ms: row.get("delay_ms")?,
            event_type: row.get("event_type")?,
            trigger_type: row.get("trigger_type")?,
            obs_scene_id: row.get("obs_scene_id")?,
            overlay_template_id: row.get("overlay_template_id")?,
            is_enabled: row.get("is_enabled")?,
            priority: row.get("priority")?,
            created_at: parse_datetime_from_db(&row.get::<_, String>("created_at")?, "created_at")?,
            updated_at: parse_datetime_from_db(&row.get::<_, String>("updated_at")?, "updated_at")?,
        })
    }
} 