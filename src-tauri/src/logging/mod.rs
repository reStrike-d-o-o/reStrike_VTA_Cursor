use std::fs;
use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod logger;
pub mod rotation;
pub mod archival;

use logger::Logger;
use rotation::LogRotator;
use archival::LogArchiver;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub max_file_size: u64,      // 10MB in bytes
    pub retention_days: u32,     // 30 days
    pub log_dir: String,         // "log"
    pub enabled_subsystems: Vec<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            retention_days: 30,
            log_dir: "log".to_string(),
            enabled_subsystems: vec!["app".to_string(), "pss".to_string(), "obs".to_string(), "udp".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub subsystem: String,
    pub message: String,
}

pub struct LogManager {
    config: Arc<Mutex<LogConfig>>,
    loggers: Arc<Mutex<std::collections::HashMap<String, Logger>>>,
    rotator: LogRotator,
    archiver: LogArchiver,
}

impl LogManager {
    pub fn new(config: LogConfig) -> io::Result<Self> {
        // Create log directory if it doesn't exist
        fs::create_dir_all(&config.log_dir)?;
        
        let rotator = LogRotator::new(config.max_file_size);
        let archiver = LogArchiver::new(config.retention_days);
        
        Ok(Self {
            config: Arc::new(Mutex::new(config)),
            loggers: Arc::new(Mutex::new(std::collections::HashMap::new())),
            rotator,
            archiver,
        })
    }
    
    pub fn log(&self, subsystem: &str, level: &str, message: &str) -> io::Result<()> {
        log::info!("🔧 LogManager.log called with subsystem: {}, level: {}, message: {}", subsystem, level, message);
        
        // Check if logging is enabled for this subsystem
        let config = self.config.lock().unwrap();
        log::info!("🔍 Checking if subsystem '{}' is enabled...", subsystem);
        log::info!("🔍 Enabled subsystems: {:?}", config.enabled_subsystems);
        
        if !config.enabled_subsystems.contains(&subsystem.to_string()) {
            log::info!("❌ Subsystem '{}' is not enabled, skipping log", subsystem);
            return Ok(());
        }
        
        log::info!("✅ Subsystem '{}' is enabled, proceeding with logging", subsystem);
        
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        let entry = LogEntry {
            timestamp,
            level: level.to_string(),
            subsystem: subsystem.to_string(),
            message: message.to_string(),
        };
        
        log::info!("📝 Created log entry, getting logger for subsystem: {}", subsystem);
        
        // Get or create logger for this subsystem
        let mut loggers = self.loggers.lock().unwrap();
        let logger = loggers.entry(subsystem.to_string()).or_insert_with(|| {
            log::info!("🆕 Creating new logger for subsystem: {}", subsystem);
            Logger::new(&config.log_dir, subsystem).unwrap_or_else(|e| {
                log::error!("❌ Failed to create logger for subsystem {}: {}", subsystem, e);
                eprintln!("Failed to create logger for subsystem: {}", subsystem);
                Logger::new("log", "fallback").unwrap()
            })
        });
        
        log::info!("✅ Got logger, writing entry...");
        
        // Write log entry
        match logger.write_entry(&entry) {
            Ok(_) => log::info!("✅ Successfully wrote log entry"),
            Err(e) => {
                log::error!("❌ Failed to write log entry: {}", e);
                return Err(e);
            }
        }
        
        log::info!("🔍 Checking if rotation is needed...");
        
        // Check if rotation is needed
        match self.rotator.should_rotate(&logger.get_current_file_path()) {
            Ok(true) => {
                log::info!("🔄 Rotation needed, rotating log...");
                if let Err(e) = self.rotate_log(subsystem) {
                    log::error!("❌ Failed to rotate log: {}", e);
                    return Err(e);
                }
                log::info!("✅ Log rotation completed");
            }
            Ok(false) => log::info!("✅ No rotation needed"),
            Err(e) => {
                log::error!("❌ Error checking rotation: {}", e);
                return Err(e);
            }
        }
        
        log::info!("✅ LogManager.log completed successfully");
        Ok(())
    }
    
    pub fn set_subsystem_enabled(&self, subsystem: &str, enabled: bool) {
        log::info!("🔧 set_subsystem_enabled called with subsystem: {}, enabled: {}", subsystem, enabled);
        
        let mut config = self.config.lock().unwrap();
        log::info!("🔍 Current enabled subsystems: {:?}", config.enabled_subsystems);
        
        if enabled {
            if !config.enabled_subsystems.contains(&subsystem.to_string()) {
                log::info!("➕ Adding subsystem '{}' to enabled list", subsystem);
                config.enabled_subsystems.push(subsystem.to_string());
            } else {
                log::info!("ℹ️ Subsystem '{}' already enabled", subsystem);
            }
        } else {
            log::info!("➖ Removing subsystem '{}' from enabled list", subsystem);
            config.enabled_subsystems.retain(|s| s != subsystem);
        }
        
        log::info!("✅ Updated enabled subsystems: {:?}", config.enabled_subsystems);
    }
    
    pub fn is_subsystem_enabled(&self, subsystem: &str) -> bool {
        let config = self.config.lock().unwrap();
        config.enabled_subsystems.contains(&subsystem.to_string())
    }
    
    pub fn list_log_files(&self, subsystem: Option<&str>) -> io::Result<Vec<LogFileInfo>> {
        let config = self.config.lock().unwrap();
        let log_dir = Path::new(&config.log_dir);
        let mut files = Vec::new();
        
        if !log_dir.exists() {
            return Ok(files);
        }
        
        for entry in fs::read_dir(log_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let filename = path.file_name().unwrap().to_string_lossy();
                
                // Filter by subsystem if specified
                if let Some(sub) = subsystem {
                    if !filename.contains(sub) {
                        continue;
                    }
                }
                
                let metadata = fs::metadata(&path)?;
                let modified = metadata.modified()?
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                    .as_secs();
                
                let modified_iso = DateTime::from_timestamp(modified as i64, 0)
                    .unwrap_or_else(|| Utc::now())
                    .to_rfc3339();
                
                files.push(LogFileInfo {
                    name: filename.to_string(),
                    size: metadata.len(),
                    modified: modified_iso,
                    subsystem: subsystem.unwrap_or("general").to_string(),
                });
            }
        }
        
        // Sort by modification time (newest first)
        files.sort_by(|a, b| b.modified.cmp(&a.modified));
        
        Ok(files)
    }
    
    pub fn read_log_file(&self, filename: &str) -> io::Result<Vec<u8>> {
        let config = self.config.lock().unwrap();
        let file_path = Path::new(&config.log_dir).join(filename);
        
        if !file_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Log file not found"));
        }
        
        fs::read(file_path)
    }
    
    fn rotate_log(&self, subsystem: &str) -> io::Result<()> {
        let mut loggers = self.loggers.lock().unwrap();
        if let Some(logger) = loggers.get_mut(subsystem) {
            logger.rotate()?;
        }
        Ok(())
    }
    
    pub fn cleanup_old_logs(&self) -> io::Result<()> {
        let config = self.config.lock().unwrap();
        self.archiver.cleanup_old_logs(&config.log_dir)
    }
    
    pub fn get_config(&self) -> LogConfig {
        let config = self.config.lock().unwrap();
        config.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFileInfo {
    pub name: String,
    pub size: u64,
    pub modified: String,
    pub subsystem: String,
} 