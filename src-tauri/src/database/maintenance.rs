use crate::database::connection::DatabaseConnection;
use std::time::{Duration, Instant};
use crate::database::{DatabaseError, DatabaseResult};
use serde::{Serialize, Deserialize};

/// Database maintenance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceConfig {
    pub vacuum_interval: Duration,
    pub integrity_check_interval: Duration,
    pub analyze_interval: Duration,
    pub optimize_interval: Duration,
    pub max_vacuum_time: Duration,
    pub backup_before_maintenance: bool,
}

impl Default for MaintenanceConfig {
    fn default() -> Self {
        Self {
            vacuum_interval: Duration::from_secs(86400), // 24 hours
            integrity_check_interval: Duration::from_secs(3600), // 1 hour
            analyze_interval: Duration::from_secs(7200), // 2 hours
            optimize_interval: Duration::from_secs(604800), // 1 week
            max_vacuum_time: Duration::from_secs(300), // 5 minutes
            backup_before_maintenance: true,
        }
    }
}

/// Database maintenance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceStatistics {
    pub last_vacuum: Option<String>,
    pub last_integrity_check: Option<String>,
    pub last_analyze: Option<String>,
    pub last_optimize: Option<String>,
    pub vacuum_count: u64,
    pub integrity_check_count: u64,
    pub analyze_count: u64,
    pub optimize_count: u64,
    pub total_maintenance_time_secs: u64,
}

impl Default for MaintenanceStatistics {
    fn default() -> Self {
        Self {
            last_vacuum: None,
            last_integrity_check: None,
            last_analyze: None,
            last_optimize: None,
            vacuum_count: 0,
            integrity_check_count: 0,
            analyze_count: 0,
            optimize_count: 0,
            total_maintenance_time_secs: 0,
        }
    }
}

/// Database maintenance manager
pub struct DatabaseMaintenance {
    config: MaintenanceConfig,
    stats: MaintenanceStatistics,
    // Internal tracking with Instant (not serialized)
    last_vacuum: Option<Instant>,
    last_integrity_check: Option<Instant>,
    last_analyze: Option<Instant>,
    last_optimize: Option<Instant>,
    total_maintenance_time: Duration,
}

impl DatabaseMaintenance {
    /// Create a new database maintenance manager
    pub fn new(config: MaintenanceConfig) -> Self {
        Self {
            config,
            stats: MaintenanceStatistics::default(),
            last_vacuum: None,
            last_integrity_check: None,
            last_analyze: None,
            last_optimize: None,
            total_maintenance_time: Duration::ZERO,
        }
    }
    
    /// Create a new database maintenance manager with default configuration
    pub fn new_default() -> Self {
        Self::new(MaintenanceConfig::default())
    }
    
    /// Run VACUUM operation to reclaim space and optimize the database
    pub async fn run_vacuum(&mut self, db_conn: &DatabaseConnection) -> DatabaseResult<()> {
        let start_time = Instant::now();
        
        log::info!("🧹 Starting database VACUUM operation...");
        
        // Check if VACUUM is needed
        let page_count: i64 = db_conn.read_transaction(|tx| {
            tx.query_row("PRAGMA page_count", [], |row| row.get(0))
                .map_err(|e| DatabaseError::Sqlite(e))
        }).await?;
        
        let freelist_count: i64 = db_conn.read_transaction(|tx| {
            tx.query_row("PRAGMA freelist_count", [], |row| row.get(0))
                .map_err(|e| DatabaseError::Sqlite(e))
        }).await?;
        
        if freelist_count == 0 {
            log::info!("📊 No fragmentation detected, VACUUM not needed");
            return Ok(());
        }
        
        let fragmentation_percentage = (freelist_count as f64 / page_count as f64) * 100.0;
        log::info!("📊 Fragmentation detected: {:.2}% ({} free pages out of {} total)", 
                  fragmentation_percentage, freelist_count, page_count);
        
        // Run VACUUM operation
        db_conn.transaction(|tx| {
            tx.execute("VACUUM", [])
                .map_err(|e| DatabaseError::Sqlite(e))
        }).await?;
        
        // Update statistics
        let duration = start_time.elapsed();
        self.last_vacuum = Some(Instant::now());
        self.stats.last_vacuum = Some(chrono::Utc::now().to_rfc3339());
        self.stats.vacuum_count += 1;
        self.total_maintenance_time += duration;
        self.stats.total_maintenance_time_secs = self.total_maintenance_time.as_secs();
        
        log::info!("✅ Database VACUUM completed successfully in {:.2?}", duration);
        Ok(())
    }
    
    /// Run integrity check to verify database consistency
    pub async fn run_integrity_check(&mut self, db_conn: &DatabaseConnection) -> DatabaseResult<bool> {
        let start_time = Instant::now();
        
        log::info!("🔍 Starting database integrity check...");
        
        let integrity_ok: String = db_conn.read_transaction(|tx| {
            tx.query_row("PRAGMA integrity_check", [], |row| row.get(0))
                .map_err(|e| DatabaseError::Sqlite(e))
        }).await?;
        
        let is_ok = integrity_ok == "ok";
        
        // Update statistics
        let duration = start_time.elapsed();
        self.last_integrity_check = Some(Instant::now());
        self.stats.last_integrity_check = Some(chrono::Utc::now().to_rfc3339());
        self.stats.integrity_check_count += 1;
        self.total_maintenance_time += duration;
        self.stats.total_maintenance_time_secs = self.total_maintenance_time.as_secs();
        
        if is_ok {
            log::info!("✅ Database integrity check passed in {:.2?}", duration);
        } else {
            log::error!("❌ Database integrity check failed: {}", integrity_ok);
        }
        
        Ok(is_ok)
    }
    
    /// Run ANALYZE to update query planner statistics
    pub async fn run_analyze(&mut self, db_conn: &DatabaseConnection) -> DatabaseResult<()> {
        let start_time = Instant::now();
        
        log::info!("📈 Starting database ANALYZE operation...");
        
        db_conn.transaction(|tx| {
            tx.execute("ANALYZE", [])
                .map_err(|e| DatabaseError::Sqlite(e))
        }).await?;
        
        // Update statistics
        let duration = start_time.elapsed();
        self.last_analyze = Some(Instant::now());
        self.stats.last_analyze = Some(chrono::Utc::now().to_rfc3339());
        self.stats.analyze_count += 1;
        self.total_maintenance_time += duration;
        self.stats.total_maintenance_time_secs = self.total_maintenance_time.as_secs();
        
        log::info!("✅ Database ANALYZE completed successfully in {:.2?}", duration);
        Ok(())
    }
    
    /// Run OPTIMIZE to optimize the database
    pub async fn run_optimize(&mut self, db_conn: &DatabaseConnection) -> DatabaseResult<()> {
        let start_time = Instant::now();
        
        log::info!("⚡ Starting database OPTIMIZE operation...");
        
        db_conn.transaction(|tx| {
            tx.execute("PRAGMA optimize", [])
                .map_err(|e| DatabaseError::Sqlite(e))
        }).await?;
        
        // Update statistics
        let duration = start_time.elapsed();
        self.last_optimize = Some(Instant::now());
        self.stats.last_optimize = Some(chrono::Utc::now().to_rfc3339());
        self.stats.optimize_count += 1;
        self.total_maintenance_time += duration;
        self.stats.total_maintenance_time_secs = self.total_maintenance_time.as_secs();
        
        log::info!("✅ Database OPTIMIZE completed successfully in {:.2?}", duration);
        Ok(())
    }
    
    /// Run full maintenance cycle
    pub async fn run_full_maintenance(&mut self, db_conn: &DatabaseConnection) -> DatabaseResult<MaintenanceResult> {
        let start_time = Instant::now();
        
        log::info!("🔧 Starting full database maintenance cycle...");
        
        // Run integrity check first
        let integrity_check_passed = self.run_integrity_check(db_conn).await?;
        
        if !integrity_check_passed {
            log::error!("❌ Integrity check failed, aborting maintenance");
            return Ok(MaintenanceResult {
                integrity_check_passed: false,
                analyze_success: false,
                optimize_success: false,
                vacuum_success: false,
                total_duration: start_time.elapsed(),
            });
        }
        
        // Run ANALYZE
        let analyze_success = self.run_analyze(db_conn).await.is_ok();
        
        // Run OPTIMIZE
        let optimize_success = self.run_optimize(db_conn).await.is_ok();
        
        // Run VACUUM last (most time-consuming)
        let vacuum_success = self.run_vacuum(db_conn).await.is_ok();
        
        let total_duration = start_time.elapsed();
        
        log::info!("🎉 Full database maintenance completed in {:.2?}", total_duration);
        
        Ok(MaintenanceResult {
            integrity_check_passed,
            analyze_success,
            optimize_success,
            vacuum_success,
            total_duration,
        })
    }
    
    /// Check if maintenance operations are needed
    pub fn check_maintenance_needed(&self) -> MaintenanceNeeded {
        let now = Instant::now();
        
        let vacuum_needed = self.last_vacuum
            .map(|last| now.duration_since(last) >= self.config.vacuum_interval)
            .unwrap_or(true);
            
        let integrity_check_needed = self.last_integrity_check
            .map(|last| now.duration_since(last) >= self.config.integrity_check_interval)
            .unwrap_or(true);
            
        let analyze_needed = self.last_analyze
            .map(|last| now.duration_since(last) >= self.config.analyze_interval)
            .unwrap_or(true);
            
        let optimize_needed = self.last_optimize
            .map(|last| now.duration_since(last) >= self.config.optimize_interval)
            .unwrap_or(true);
        
        MaintenanceNeeded {
            vacuum_needed,
            integrity_check_needed,
            analyze_needed,
            optimize_needed,
        }
    }
    
    /// Get maintenance statistics
    pub fn get_statistics(&self) -> &MaintenanceStatistics {
        &self.stats
    }
    
    /// Get maintenance configuration
    pub fn get_config(&self) -> &MaintenanceConfig {
        &self.config
    }
    
    /// Update maintenance configuration
    pub fn update_config(&mut self, config: MaintenanceConfig) {
        self.config = config;
    }
    
    /// Get database information
    pub async fn get_database_info(&self, db_conn: &DatabaseConnection) -> DatabaseResult<DatabaseInfo> {
        let page_count: i64 = db_conn.read_transaction(|tx| {
            tx.query_row("PRAGMA page_count", [], |row| row.get(0))
                .map_err(|e| DatabaseError::Sqlite(e))
        }).await?;
        
        let page_size: i64 = db_conn.read_transaction(|tx| {
            tx.query_row("PRAGMA page_size", [], |row| row.get(0))
                .map_err(|e| DatabaseError::Sqlite(e))
        }).await?;
        
        let freelist_count: i64 = db_conn.read_transaction(|tx| {
            tx.query_row("PRAGMA freelist_count", [], |row| row.get(0))
                .map_err(|e| DatabaseError::Sqlite(e))
        }).await?;
        
        let cache_size: i64 = db_conn.read_transaction(|tx| {
            tx.query_row("PRAGMA cache_size", [], |row| row.get(0))
                .map_err(|e| DatabaseError::Sqlite(e))
        }).await?;
        
        let journal_mode: String = db_conn.read_transaction(|tx| {
            tx.query_row("PRAGMA journal_mode", [], |row| row.get(0))
                .map_err(|e| DatabaseError::Sqlite(e))
        }).await?;
        
        let synchronous: String = db_conn.read_transaction(|tx| {
            tx.query_row("PRAGMA synchronous", [], |row| row.get(0))
                .map_err(|e| DatabaseError::Sqlite(e))
        }).await?;
        
        let total_size = page_count * page_size;
        let used_size = (page_count - freelist_count) * page_size;
        let free_size = freelist_count * page_size;
        let fragmentation_percentage = if page_count > 0 {
            (freelist_count as f64 / page_count as f64) * 100.0
        } else {
            0.0
        };
        
        Ok(DatabaseInfo {
            total_size,
            used_size,
            free_size,
            fragmentation_percentage,
            page_count,
            page_size,
            freelist_count,
            cache_size,
            journal_mode,
            synchronous,
        })
    }
}

/// Result of maintenance operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceResult {
    pub integrity_check_passed: bool,
    pub analyze_success: bool,
    pub optimize_success: bool,
    pub vacuum_success: bool,
    pub total_duration: Duration,
}

/// Indicates which maintenance operations are needed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceNeeded {
    pub vacuum_needed: bool,
    pub integrity_check_needed: bool,
    pub analyze_needed: bool,
    pub optimize_needed: bool,
}

impl MaintenanceNeeded {
    /// Check if any maintenance is needed
    pub fn any_needed(&self) -> bool {
        self.vacuum_needed || self.integrity_check_needed || self.analyze_needed || self.optimize_needed
    }
}

/// Database information for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub total_size: i64,
    pub used_size: i64,
    pub free_size: i64,
    pub fragmentation_percentage: f64,
    pub page_count: i64,
    pub page_size: i64,
    pub freelist_count: i64,
    pub cache_size: i64,
    pub journal_mode: String,
    pub synchronous: String,
}

impl std::fmt::Display for DatabaseInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Database Info:\n")?;
        write!(f, "  Total Size: {} bytes ({:.2} MB)\n", self.total_size, self.total_size as f64 / 1024.0 / 1024.0)?;
        write!(f, "  Used Size: {} bytes ({:.2} MB)\n", self.used_size, self.used_size as f64 / 1024.0 / 1024.0)?;
        write!(f, "  Free Size: {} bytes ({:.2} MB)\n", self.free_size, self.free_size as f64 / 1024.0 / 1024.0)?;
        write!(f, "  Fragmentation: {:.2}%\n", self.fragmentation_percentage)?;
        write!(f, "  Page Count: {}\n", self.page_count)?;
        write!(f, "  Page Size: {} bytes\n", self.page_size)?;
        write!(f, "  Free List Count: {}\n", self.freelist_count)?;
        write!(f, "  Cache Size: {} pages\n", self.cache_size)?;
        write!(f, "  Journal Mode: {}\n", self.journal_mode)?;
        write!(f, "  Synchronous: {}", self.synchronous)?;
        Ok(())
    }
} 