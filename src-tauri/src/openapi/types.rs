use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Supported serialization formats for OpenAPI schemas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaFormat {
    Json,
    Yaml,
}

impl SchemaFormat {
    pub fn extension(self) -> &'static str {
        match self {
            SchemaFormat::Json => "json",
            SchemaFormat::Yaml => "yaml",
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            SchemaFormat::Json => "application/json",
            SchemaFormat::Yaml => "application/yaml",
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "json" => Some(SchemaFormat::Json),
            "yaml" | "yml" => Some(SchemaFormat::Yaml),
            _ => None,
        }
    }
}

impl Display for SchemaFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaFormat::Json => write!(f, "json"),
            SchemaFormat::Yaml => write!(f, "yaml"),
        }
    }
}

/// In-memory representation of the active OpenAPI document.
#[derive(Debug, Clone)]
pub struct OpenApiDocument {
    pub raw: String,
    pub format: SchemaFormat,
    pub json: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

/// Metadata persisted alongside the schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMetadata {
    pub format: SchemaFormat,
    pub updated_at: DateTime<Utc>,
}

/// Detailed validation error for OpenAPI schemas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaValidationError {
    pub message: String,
    pub pointer: Option<String>,
    pub line: Option<u64>,
    pub column: Option<u64>,
}

impl SchemaValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            pointer: None,
            line: None,
            column: None,
        }
    }

    pub fn with_pointer(mut self, pointer: Option<String>) -> Self {
        self.pointer = pointer;
        self
    }

    pub fn with_location(mut self, line: Option<u64>, column: Option<u64>) -> Self {
        self.line = line;
        self.column = column;
        self
    }
}

/// Result of validating an OpenAPI schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaValidationOutcome {
    pub format: SchemaFormat,
    pub valid: bool,
    pub errors: Vec<SchemaValidationError>,
}

impl SchemaValidationOutcome {
    pub fn success(format: SchemaFormat) -> Self {
        Self {
            format,
            valid: true,
            errors: Vec::new(),
        }
    }

    pub fn with_errors(format: SchemaFormat, errors: Vec<SchemaValidationError>) -> Self {
        Self {
            format,
            valid: errors.is_empty(),
            errors,
        }
    }
}
