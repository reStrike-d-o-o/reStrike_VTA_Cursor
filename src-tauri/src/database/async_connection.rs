use sqlx::{SqlitePool, Sqlite};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use std::path::PathBuf;
use std::time::Duration;
use std::fs;
use std::str::FromStr;
use crate::database::{DatabaseError, DatabaseResult, DATABASE_FILE};

/// Async database connection using sqlx for thread-safe operations
/// This is specifically designed for use with Tauri commands and async operations
#[derive(Debug, Clone)]
pub struct AsyncDatabaseConnection {
    pool: SqlitePool,
    db_path: PathBuf,
}

impl AsyncDatabaseConnection {
    /// Create a new async database connection
    pub async fn new(data_dir: &PathBuf) -> DatabaseResult<Self> {
        let db_path = data_dir.join(DATABASE_FILE);
        
        // Ensure the directory exists
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).map_err(DatabaseError::Io)?;
        }

        // Create connection options with optimizations
        let connection_options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))
            .map_err(|e| DatabaseError::Connection(e.to_string()))?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .busy_timeout(Duration::from_secs(30))
            .pragma("cache_size", "-65536") // 64MB cache
            .pragma("temp_store", "memory")
            .pragma("recursive_triggers", "on");

        // Create connection pool
        let pool = SqlitePool::connect_with(connection_options)
            .await
            .map_err(DatabaseError::AsyncSqlite)?;

        Ok(Self {
            pool,
            db_path,
        })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Get the database file path
    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// Execute a statement with no return value
    pub async fn execute(&self, sql: &str) -> DatabaseResult<u64> {
        let result = sqlx::query(sql)
            .execute(&self.pool)
            .await
            .map_err(DatabaseError::AsyncSqlite)?;
        Ok(result.rows_affected())
    }

    /// Execute a statement with parameters
    pub async fn execute_with_params(&self, sql: &str, params: &[&str]) -> DatabaseResult<u64> {
        let mut query = sqlx::query(sql);
        for param in params {
            query = query.bind(param);
        }
        let result = query.execute(&self.pool)
            .await
            .map_err(DatabaseError::AsyncSqlite)?;
        Ok(result.rows_affected())
    }

    /// Query for a single row
    pub async fn query_row<F, T>(&self, sql: &str, f: F) -> DatabaseResult<Option<T>>
    where
        F: FnOnce(&sqlx::sqlite::SqliteRow) -> Result<T, sqlx::Error> + Send,
        T: Send,
    {
        let row = sqlx::query(sql)
            .fetch_optional(&self.pool)
            .await
            .map_err(DatabaseError::AsyncSqlite)?;
            
        match row {
            Some(row) => Ok(Some(f(&row).map_err(DatabaseError::AsyncSqlite)?)),
            None => Ok(None),
        }
    }

    /// Query for multiple rows
    pub async fn query_rows<F, T>(&self, sql: &str, f: F) -> DatabaseResult<Vec<T>>
    where
        F: Fn(&sqlx::sqlite::SqliteRow) -> Result<T, sqlx::Error> + Send,
        T: Send,
    {
        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::AsyncSqlite)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(f(&row).map_err(DatabaseError::AsyncSqlite)?);
        }
        Ok(results)
    }

    /// Begin a transaction
    pub async fn begin_transaction(&self) -> DatabaseResult<sqlx::Transaction<'_, Sqlite>> {
        let tx = self.pool.begin().await.map_err(DatabaseError::AsyncSqlite)?;
        Ok(tx)
    }

    /// Close the connection pool
    pub async fn close(self) {
        self.pool.close().await;
    }

    /// Execute a prepared statement with string parameters
    pub async fn execute_with_string_params(&self, sql: &str, params: Vec<String>) -> DatabaseResult<u64> {
        let mut query = sqlx::query(sql);
        for param in params {
            query = query.bind(param);
        }
        let result = query.execute(&self.pool)
            .await
            .map_err(DatabaseError::AsyncSqlite)?;
        Ok(result.rows_affected())
    }

    /// Query that returns a scalar value
    pub async fn query_scalar<T>(&self, sql: &str) -> DatabaseResult<Option<T>>
    where
        T: for<'r> sqlx::Decode<'r, Sqlite> + sqlx::Type<Sqlite> + Send + Unpin,
    {
        let result = sqlx::query_scalar(sql)
            .fetch_optional(&self.pool)
            .await
            .map_err(DatabaseError::AsyncSqlite)?;
        Ok(result)
    }
}

/// Extension trait to add async database functionality to the main DatabaseConnection
pub trait AsyncDatabaseExt {
    fn get_async_connection(&self) -> Option<&AsyncDatabaseConnection>;
}