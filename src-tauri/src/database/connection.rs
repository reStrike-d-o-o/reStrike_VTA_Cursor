use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use crate::database::{DatabaseError, DatabaseResult, DATABASE_FILE};

/// Database connection wrapper with thread-safe access
#[derive(Clone)]
pub struct DatabaseConnection {
    connection: Arc<Mutex<Connection>>,
}

impl DatabaseConnection {
    /// Create a new database connection
    pub fn new() -> DatabaseResult<Self> {
        let db_path = Self::get_database_path()?;
        
        // Ensure the directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DatabaseError::Initialization(format!("Failed to create database directory: {}", e)))?;
        }
        
        let connection = Connection::open(&db_path)
            .map_err(|e| DatabaseError::Connection(format!("Failed to open database: {}", e)))?;
        
        // Enable foreign keys
        connection.execute("PRAGMA foreign_keys = ON", [])
            .map_err(|e| DatabaseError::Initialization(format!("Failed to enable foreign keys: {}", e)))?;
        
        // Set UTF-8 encoding for international text support
        connection.execute("PRAGMA encoding = 'UTF-8'", [])
            .map_err(|e| DatabaseError::Initialization(format!("Failed to set UTF-8 encoding: {}", e)))?;
        
        // Enable WAL mode for better concurrency - use query_row to handle the returned result
        let _: String = connection.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))
            .map_err(|e| DatabaseError::Initialization(format!("Failed to enable WAL mode: {}", e)))?;
        
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }
    
    /// Get the database file path
    pub fn get_database_path() -> DatabaseResult<PathBuf> {
        let mut path = std::env::current_exe()
            .map_err(|e| DatabaseError::Initialization(format!("Failed to get executable path: {}", e)))?
            .parent()
            .ok_or_else(|| DatabaseError::Initialization("Failed to get executable directory".to_string()))?
            .to_path_buf();
        
        path.push("data");
        path.push(DATABASE_FILE);
        
        Ok(path)
    }
    
    /// Get a reference to the underlying connection
    pub fn get_connection(&self) -> DatabaseResult<MutexGuard<Connection>> {
        self.connection.lock()
            .map_err(|e| DatabaseError::Connection(format!("Failed to acquire database lock: {}", e)))
    }
    
    /// Execute a transaction
    pub fn transaction<F, T>(&self, f: F) -> DatabaseResult<T>
    where
        F: FnOnce(&rusqlite::Transaction) -> DatabaseResult<T>,
    {
        let mut conn = self.get_connection()?;
        let transaction = conn.transaction()
            .map_err(|e| DatabaseError::Transaction(format!("Failed to start transaction: {}", e)))?;
        
        let result = f(&transaction)?;
        
        transaction.commit()
            .map_err(|e| DatabaseError::Transaction(format!("Failed to commit transaction: {}", e)))?;
        
        Ok(result)
    }
    
    /// Execute a read-only transaction
    pub fn read_transaction<F, T>(&self, f: F) -> DatabaseResult<T>
    where
        F: FnOnce(&rusqlite::Transaction) -> DatabaseResult<T>,
    {
        let mut conn = self.get_connection()?;
        let transaction = conn.transaction()
            .map_err(|e| DatabaseError::Transaction(format!("Failed to start read transaction: {}", e)))?;
        
        let result = f(&transaction)?;
        
        transaction.commit()
            .map_err(|e| DatabaseError::Transaction(format!("Failed to commit read transaction: {}", e)))?;
        
        Ok(result)
    }
    
    /// Check if the database is accessible
    pub fn is_accessible(&self) -> bool {
        self.get_connection().is_ok()
    }
    
    /// Get database file size
    pub fn get_file_size(&self) -> DatabaseResult<u64> {
        let path = Self::get_database_path()?;
        let metadata = std::fs::metadata(&path)
            .map_err(|e| DatabaseError::Connection(format!("Failed to get database metadata: {}", e)))?;
        Ok(metadata.len())
    }
    
    /// Get the current database encoding
    pub fn get_encoding(&self) -> DatabaseResult<String> {
        let conn = self.get_connection()?;
        let encoding: String = conn.query_row("PRAGMA encoding", [], |row| row.get(0))
            .map_err(|e| DatabaseError::Connection(format!("Failed to get database encoding: {}", e)))?;
        Ok(encoding)
    }
}

impl Default for DatabaseConnection {
    fn default() -> Self {
        Self::new().expect("Failed to create default database connection")
    }
} 