use rusqlite::{Connection, Result as SqliteResult};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::Mutex as TokioMutex;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::database::{DatabaseError, DatabaseResult, DATABASE_FILE};

/// Phase 2 Optimization: Database Connection Pool
/// Manages a pool of database connections for high-volume operations
pub struct DatabaseConnectionPool {
    connections: Arc<Mutex<VecDeque<rusqlite::Connection>>>,
    max_connections: usize,
    connection_timeout: Duration,
    last_cleanup: Arc<Mutex<Instant>>,
}

impl DatabaseConnectionPool {
    /// Create a new connection pool
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: Arc::new(Mutex::new(VecDeque::new())),
            max_connections,
            connection_timeout: Duration::from_secs(300), // 5 minutes
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Get a connection from the pool or create a new one
    pub fn get_connection(&self) -> SqliteResult<PooledConnection> {
        let start_time = Instant::now();
        
        loop {
            let mut connections = self.connections.lock().unwrap();
            
            // Try to get an existing connection
            if let Some(conn) = connections.pop_front() {
                // Check if connection is still valid
                if let Ok(_) = conn.execute("SELECT 1", []) {
                    return Ok(PooledConnection {
                        connection: Some(conn),
                        pool: self.connections.clone(),
                        max_connections: self.max_connections,
                    });
                }
            }

            // Create a new connection if pool is empty or connection was invalid
            if connections.len() < self.max_connections {
                let conn = rusqlite::Connection::open(crate::database::DATABASE_FILE)?;
                self.configure_connection(&conn)?;
                
                return Ok(PooledConnection {
                    connection: Some(conn),
                    pool: self.connections.clone(),
                    max_connections: self.max_connections,
                });
            }
            
            // Check if we've exceeded the timeout
            if start_time.elapsed() > self.connection_timeout {
                return Err(rusqlite::Error::InvalidPath("Connection timeout reached".to_string().into()));
            }
            
            // Release lock and wait a bit before retrying
            drop(connections);
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Configure a connection with performance optimizations
    fn configure_connection(&self, conn: &rusqlite::Connection) -> SqliteResult<()> {
        // Phase 1 optimizations (already implemented)
        conn.execute("PRAGMA journal_mode = WAL", [])?;
        conn.execute("PRAGMA synchronous = NORMAL", [])?;
        conn.execute("PRAGMA cache_size = -65536", [])?; // 64MB cache
        conn.execute("PRAGMA temp_store = MEMORY", [])?;
        
        // Optional mmap size setting (may not be supported in all SQLite builds)
        if let Err(e) = conn.execute("PRAGMA mmap_size = 134217728", []) { // 128MB mmap
            log::warn!("Failed to set mmap size (this is optional): {}", e);
        }
        
        conn.execute("PRAGMA recursive_triggers = ON", [])?;
        conn.execute("PRAGMA busy_timeout = 30000", [])?;
        conn.execute("PRAGMA optimize", [])?;
        conn.execute("PRAGMA page_size = 4096", [])?;

        // Phase 2 optimizations
        conn.execute("PRAGMA auto_vacuum = INCREMENTAL", [])?; // Better space management
        
        // Optional WAL autocheckpoint setting
        if let Err(e) = conn.execute("PRAGMA wal_autocheckpoint = 1000", []) { // Checkpoint every 1000 pages
            log::warn!("Failed to set WAL autocheckpoint (this is optional): {}", e);
        }
        
        conn.execute("PRAGMA checkpoint_fullfsync = OFF", [])?; // Faster checkpoints
        conn.execute("PRAGMA locking_mode = NORMAL", [])?; // Balance between concurrency and safety

        Ok(())
    }

    /// Clean up old connections periodically
    pub fn cleanup_old_connections(&self) {
        let mut last_cleanup = self.last_cleanup.lock().unwrap();
        if last_cleanup.elapsed() > Duration::from_secs(60) { // Cleanup every minute
            let mut connections = self.connections.lock().unwrap();
            
            // Remove connections that are too old
            let now = Instant::now();
            connections.retain(|_conn| {
                // For now, we'll keep all connections as SQLite doesn't expose connection age
                // In a more sophisticated implementation, we could track connection creation time
                true
            });

            // Limit pool size
            while connections.len() > self.max_connections {
                connections.pop_back();
            }

            *last_cleanup = now;
        }
    }

    /// Get pool statistics
    pub fn get_pool_stats(&self) -> PoolStats {
        let connections = self.connections.lock().unwrap();
        PoolStats {
            available_connections: connections.len(),
            max_connections: self.max_connections,
            pool_utilization: connections.len() as f64 / self.max_connections as f64,
        }
    }
}

/// A pooled database connection that returns to the pool when dropped
pub struct PooledConnection {
    connection: Option<rusqlite::Connection>,
    pool: Arc<Mutex<VecDeque<rusqlite::Connection>>>,
    max_connections: usize,
}

impl PooledConnection {
    /// Get a reference to the underlying connection
    pub fn connection(&self) -> &rusqlite::Connection {
        self.connection.as_ref().unwrap()
    }

    /// Get a mutable reference to the underlying connection
    pub fn connection_mut(&mut self) -> &mut rusqlite::Connection {
        self.connection.as_mut().unwrap()
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            let mut pool = self.pool.lock().unwrap();
            
            // Only return to pool if it's not full
            if pool.len() < self.max_connections {
                pool.push_back(conn);
            }
        }
    }
}

/// Pool statistics for monitoring
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PoolStats {
    pub available_connections: usize,
    pub max_connections: usize,
    pub pool_utilization: f64,
}

/// Database connection wrapper with thread-safe access and safety measures
#[derive(Clone)]
pub struct DatabaseConnection {
    connection: Arc<TokioMutex<Connection>>,
}

impl DatabaseConnection {
    /// Create a new database connection with safety measures
    pub fn new() -> DatabaseResult<Self> {
        let db_path = Self::get_database_path()?;
        
        // Ensure the directory exists
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| DatabaseError::Initialization(format!("Failed to create database directory: {}", e)))?;
        }
        
        let connection = Connection::open(&db_path)
            .map_err(|e| DatabaseError::Connection(format!("Failed to open database: {}", e)))?;
        
        // Apply comprehensive safety and performance settings
        Self::configure_connection(&connection)?;
        
        Ok(Self {
            connection: Arc::new(TokioMutex::new(connection)),
        })
    }
    
    /// Configure SQLite connection with safety and performance optimizations
    fn configure_connection(conn: &Connection) -> DatabaseResult<()> {
        // Enable foreign keys for referential integrity
        conn.execute("PRAGMA foreign_keys = ON", [])
            .map_err(|e| DatabaseError::Initialization(format!("Failed to enable foreign keys: {}", e)))?;
        
        // Set UTF-8 encoding for international text support
        conn.execute("PRAGMA encoding = 'UTF-8'", [])
            .map_err(|e| DatabaseError::Initialization(format!("Failed to set UTF-8 encoding: {}", e)))?;
        
        // Enable WAL mode for better concurrency and crash recovery
        let _: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))
            .map_err(|e| DatabaseError::Initialization(format!("Failed to enable WAL mode: {}", e)))?;
        
        // Set synchronous mode to FULL for maximum durability (slower but safer)
        conn.execute("PRAGMA synchronous = FULL", [])
            .map_err(|e| DatabaseError::Initialization(format!("Failed to set synchronous mode: {}", e)))?;
        
        // Phase 1 Optimization: Enhanced cache size to 64MB for high-volume performance
        conn.execute("PRAGMA cache_size = -65536", []) // Negative value means KB, so -65536 = 64MB
            .map_err(|e| DatabaseError::Initialization(format!("Failed to set cache size: {}", e)))?;
        
        // Set temp store to memory for better performance
        conn.execute("PRAGMA temp_store = MEMORY", [])
            .map_err(|e| DatabaseError::Initialization(format!("Failed to set temp store: {}", e)))?;
        
        // Phase 1 Optimization: Enhanced mmap size to 128MB for high-volume performance (optional)
        if let Err(e) = conn.execute("PRAGMA mmap_size = 134217728", []) { // 128MB in bytes
            log::warn!("Failed to set mmap size (this is optional): {}", e);
        }
        
        // Enable recursive triggers
        conn.execute("PRAGMA recursive_triggers = ON", [])
            .map_err(|e| DatabaseError::Initialization(format!("Failed to enable recursive triggers: {}", e)))?;
        
        // Set busy timeout to 30 seconds to handle concurrent access
        conn.busy_timeout(std::time::Duration::from_secs(30))
            .map_err(|e| DatabaseError::Initialization(format!("Failed to set busy timeout: {}", e)))?;
        
        // Phase 1 Optimization: Additional performance settings for high-volume processing
        // Optimize for bulk operations
        conn.execute("PRAGMA optimize", [])
            .map_err(|e| DatabaseError::Initialization(format!("Failed to optimize database: {}", e)))?;
        
        // Set page size to 4KB for better performance
        conn.execute("PRAGMA page_size = 4096", [])
            .map_err(|e| DatabaseError::Initialization(format!("Failed to set page size: {}", e)))?;
        
        // Set WAL auto-checkpoint to 1000 pages for better performance (optional)
        if let Err(e) = conn.execute("PRAGMA wal_autocheckpoint = 1000", []) {
            log::warn!("Failed to set WAL autocheckpoint (this is optional): {}", e);
        }
        
        Ok(())
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
    
    /// Get the backup directory path
    pub fn get_backup_directory() -> DatabaseResult<PathBuf> {
        let mut path = std::env::current_exe()
            .map_err(|e| DatabaseError::Initialization(format!("Failed to get executable path: {}", e)))?
            .parent()
            .ok_or_else(|| DatabaseError::Initialization("Failed to get executable directory".to_string()))?
            .to_path_buf();
        
        path.push("data");
        path.push("backups");
        
        Ok(path)
    }
    
    /// Get a reference to the underlying connection
    pub async fn get_connection(&self) -> DatabaseResult<tokio::sync::MutexGuard<'_, Connection>> {
        Ok(self.connection.lock().await)
    }
    
    /// Get a mutable reference to the underlying connection
    pub async fn get_connection_mut(&self) -> DatabaseResult<tokio::sync::MutexGuard<'_, Connection>> {
        Ok(self.connection.lock().await)
    }
    
    /// Execute a transaction with automatic rollback on error
    pub async fn transaction<F, T>(&self, f: F) -> DatabaseResult<T>
    where
        F: FnOnce(&rusqlite::Transaction) -> DatabaseResult<T>,
    {
        let mut conn = self.get_connection().await?;
        let transaction = conn.transaction()
            .map_err(|e| DatabaseError::Transaction(format!("Failed to start transaction: {}", e)))?;
        
        let result = f(&transaction);
        
        match result {
            Ok(value) => {
                transaction.commit()
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to commit transaction: {}", e)))?;
                Ok(value)
            }
            Err(e) => {
                // Transaction will be automatically rolled back when dropped
                log::warn!("Transaction failed, rolling back: {}", e);
                Err(e)
            }
        }
    }
    
    /// Execute a read-only transaction
    pub async fn read_transaction<F, T>(&self, f: F) -> DatabaseResult<T>
    where
        F: FnOnce(&rusqlite::Transaction) -> DatabaseResult<T>,
    {
        let mut conn = self.get_connection().await?;
        let transaction = conn.transaction()
            .map_err(|e| DatabaseError::Transaction(format!("Failed to start read transaction: {}", e)))?;
        
        let result = f(&transaction)?;
        
        transaction.commit()
            .map_err(|e| DatabaseError::Transaction(format!("Failed to commit read transaction: {}", e)))?;
        
        Ok(result)
    }
    
    /// Execute a transaction with retry logic for busy database
    pub async fn transaction_with_retry<F, T>(&self, mut f: F, max_retries: u32) -> DatabaseResult<T>
    where
        F: FnMut(&rusqlite::Transaction) -> DatabaseResult<T>,
    {
        let mut last_error = None;
        
        for attempt in 0..max_retries {
            match self.transaction(&mut f).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    let error_msg = e.to_string();
                    last_error = Some(e);
                    if attempt < max_retries - 1 {
                        // Exponential backoff: wait 2^attempt * 100ms
                        let delay = std::time::Duration::from_millis(100 * (1 << attempt));
                        tokio::time::sleep(delay).await;
                        log::warn!("Transaction attempt {} failed, retrying in {:?}: {}", attempt + 1, delay, error_msg);
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| DatabaseError::Transaction("Max retries exceeded".to_string())))
    }
    
    /// Restore database from backup
    pub async fn restore_from_backup(&self, backup_path: &PathBuf) -> DatabaseResult<()> {
        // Verify backup file exists and is accessible
        if !backup_path.exists() {
            return Err(DatabaseError::Connection(format!("Backup file does not exist: {:?}", backup_path)));
        }
        
        // Check backup file integrity
        let backup_conn = Connection::open(backup_path)
            .map_err(|e| DatabaseError::Connection(format!("Failed to open backup file: {}", e)))?;
        
        let integrity: String = backup_conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(|e| DatabaseError::Connection(format!("Failed to check backup integrity: {}", e)))?;
        
        if integrity != "ok" {
            return Err(DatabaseError::Connection(format!("Backup file integrity check failed: {}", integrity)));
        }
        
        // Create a temporary backup of current database before restore
        let current_backup = self.create_backup(Some("pre_restore"))?;
        
        // Close current connection to allow file replacement
        drop(self.get_connection().await?);
        
        // Replace current database with backup
        let db_path = Self::get_database_path()?;
        fs::copy(backup_path, &db_path)
            .map_err(|e| DatabaseError::Connection(format!("Failed to restore database: {}", e)))?;
        
        log::info!("Database restored from backup: {:?}", backup_path);
        log::info!("Previous database backed up to: {:?}", current_backup);
        
        Ok(())
    }
    
    /// Check if the database is accessible
    pub async fn is_accessible(&self) -> bool {
        self.get_connection().await.is_ok()
    }
    
    /// Get database file size
    pub fn get_file_size(&self) -> DatabaseResult<u64> {
        let path = Self::get_database_path()?;
        let metadata = fs::metadata(&path)
            .map_err(|e| DatabaseError::Connection(format!("Failed to get database metadata: {}", e)))?;
        Ok(metadata.len())
    }
    
    /// Get the current database encoding
    pub async fn get_encoding(&self) -> DatabaseResult<String> {
        let conn = self.get_connection().await?;
        let encoding: String = conn.query_row("PRAGMA encoding", [], |row| row.get(0))
            .map_err(|e| DatabaseError::Connection(format!("Failed to get database encoding: {}", e)))?;
        Ok(encoding)
    }
    
    /// Get database integrity status
    pub async fn check_integrity(&self) -> DatabaseResult<bool> {
        let conn = self.get_connection().await?;
        let result: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(|e| DatabaseError::Connection(format!("Failed to check integrity: {}", e)))?;
        
        Ok(result == "ok")
    }
    
    /// Create a backup of the database
    pub fn create_backup(&self, backup_name: Option<&str>) -> DatabaseResult<PathBuf> {
        let backup_dir = Self::get_backup_directory()?;
        
        // Ensure backup directory exists
        fs::create_dir_all(&backup_dir)
            .map_err(|e| DatabaseError::Connection(format!("Failed to create backup directory: {}", e)))?;
        
        // Generate backup filename with timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| DatabaseError::Connection(format!("Failed to get timestamp: {}", e)))?
            .as_secs();
        
        let backup_filename = match backup_name {
            Some(name) => format!("{}_{}.db", name, timestamp),
            None => format!("backup_{}.db", timestamp),
        };
        
        let backup_path = backup_dir.join(backup_filename);
        let db_path = Self::get_database_path()?;
        
        // Create backup by copying the database file
        fs::copy(&db_path, &backup_path)
            .map_err(|e| DatabaseError::Connection(format!("Failed to create backup: {}", e)))?;
        
        log::info!("Database backup created: {:?}", backup_path);
        Ok(backup_path)
    }
    
    /// List available backups
    pub fn list_backups(&self) -> DatabaseResult<Vec<PathBuf>> {
        let backup_dir = Self::get_backup_directory()?;
        
        if !backup_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut backups = Vec::new();
        for entry in fs::read_dir(backup_dir)
            .map_err(|e| DatabaseError::Connection(format!("Failed to read backup directory: {}", e)))? {
            let entry = entry
                .map_err(|e| DatabaseError::Connection(format!("Failed to read backup entry: {}", e)))?;
            
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("db") {
                backups.push(path);
            }
        }
        
        // Sort by modification time (newest first)
        backups.sort_by(|a, b| {
            let a_time = fs::metadata(a).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
            let b_time = fs::metadata(b).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
            b_time.cmp(&a_time)
        });
        
        Ok(backups)
    }
    
    /// Clean up old backups (keep only the most recent N)
    pub fn cleanup_old_backups(&self, keep_count: usize) -> DatabaseResult<usize> {
        let backups = self.list_backups()?;
        
        if backups.len() <= keep_count {
            return Ok(0);
        }
        
        let to_delete = &backups[keep_count..];
        let mut deleted_count = 0;
        
        for backup_path in to_delete {
            if let Err(e) = fs::remove_file(backup_path) {
                log::warn!("Failed to delete old backup {:?}: {}", backup_path, e);
            } else {
                deleted_count += 1;
                log::info!("Deleted old backup: {:?}", backup_path);
            }
        }
        
        Ok(deleted_count)
    }
    
    /// Get database statistics
    pub async fn get_statistics(&self) -> DatabaseResult<DatabaseStatistics> {
        let conn = self.get_connection().await?;
        
        let page_count: i64 = conn.query_row("PRAGMA page_count", [], |row| row.get(0))
            .map_err(|e| DatabaseError::Connection(format!("Failed to get page count: {}", e)))?;
        
        let page_size: i64 = conn.query_row("PRAGMA page_size", [], |row| row.get(0))
            .map_err(|e| DatabaseError::Connection(format!("Failed to get page size: {}", e)))?;
        
        let cache_size: i64 = conn.query_row("PRAGMA cache_size", [], |row| row.get(0))
            .map_err(|e| DatabaseError::Connection(format!("Failed to get cache size: {}", e)))?;
        
        let journal_mode: String = conn.query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .map_err(|e| DatabaseError::Connection(format!("Failed to get journal mode: {}", e)))?;
        
        let synchronous: String = conn.query_row("PRAGMA synchronous", [], |row| row.get(0))
            .map_err(|e| DatabaseError::Connection(format!("Failed to get synchronous mode: {}", e)))?;
        
        Ok(DatabaseStatistics {
            page_count,
            page_size,
            cache_size,
            journal_mode,
            synchronous,
            file_size: self.get_file_size()?,
            integrity_ok: self.check_integrity().await?,
        })
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStatistics {
    pub page_count: i64,
    pub page_size: i64,
    pub cache_size: i64,
    pub journal_mode: String,
    pub synchronous: String,
    pub file_size: u64,
    pub integrity_ok: bool,
}

impl Default for DatabaseConnection {
    fn default() -> Self {
        Self::new().expect("Failed to create default database connection")
    }
} 