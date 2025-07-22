use thiserror::Error;

/// Database-specific error types
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Database connection failed: {0}")]
    Connection(String),
    
    #[error("Migration error: {0}")]
    Migration(String),
    
    #[error("Schema version mismatch: expected {expected}, got {actual}")]
    SchemaVersion { expected: u32, actual: u32 },
    
    #[error("Table not found: {table}")]
    TableNotFound { table: String },
    
    #[error("Column not found: {column} in table {table}")]
    ColumnNotFound { table: String, column: String },
    
    #[error("Constraint violation: {message}")]
    ConstraintViolation { message: String },
    
    #[error("Transaction error: {0}")]
    Transaction(String),
    
    #[error("Database initialization failed: {0}")]
    Initialization(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

impl From<DatabaseError> for crate::types::AppError {
    fn from(err: DatabaseError) -> Self {
        crate::types::AppError::ConfigError(err.to_string())
    }
} 