use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Migration error: {0}")]
    Migration(String),
    
    #[error("Schema version mismatch: expected {expected}, got {actual}")]
    SchemaVersion { expected: u32, actual: u32 },
    
    #[error("Table not found: {table_name}")]
    TableNotFound { table_name: String },
    
    #[error("Column not found: {column_name} in table {table_name}")]
    ColumnNotFound { column_name: String, table_name: String },
    
    #[error("Constraint violation: {message}")]
    ConstraintViolation { message: String },
    
    #[error("Transaction error: {0}")]
    Transaction(String),
    
    #[error("Initialization error: {0}")]
    Initialization(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
}

 