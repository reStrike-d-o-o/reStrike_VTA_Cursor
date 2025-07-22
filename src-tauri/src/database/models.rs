use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use rusqlite::Row;

/// PSS Event model for database storage
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