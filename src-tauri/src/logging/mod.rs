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
use archival::{LogArchiver, AutoArchiveConfig, ArchiveInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub max_file_size: u64,      // 10MB in bytes
    pub retention_days: u32,     // 30 days
    pub log_dir: String,         // "log"
    pub archive_dir: String,     // "log/archives"
    pub enabled_subsystems: Vec<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            retention_days: 30,
            log_dir: "logs".to_string(),
            archive_dir: "logs/archives".to_string(),
            enabled_subsystems: vec!["app".to_string(), "pss".to_string(), "obs".to_string(), "udp".to_string(), "websocket".to_string(), "db".to_string()],
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

impl Clone for LogManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            loggers: self.loggers.clone(),
            rotator: self.rotator.clone(),
            archiver: self.archiver.clone(),
        }
    }
}

impl LogManager {
    pub fn new(config: LogConfig) -> io::Result<Self> {
        // Create log directory if it doesn't exist
        fs::create_dir_all(&config.log_dir)?;
        
        let rotator = LogRotator::new(config.max_file_size);
        let archiver = LogArchiver::new_with_archive_dir(config.retention_days, config.archive_dir.clone());
        
        let manager = Self {
            config: Arc::new(Mutex::new(config)),
            loggers: Arc::new(Mutex::new(std::collections::HashMap::new())),
            rotator,
            archiver,
        };
        
        // Initialize all subsystem loggers immediately
        manager.initialize_all_subsystems()?;
        
        Ok(manager)
    }
    
    fn initialize_all_subsystems(&self) -> io::Result<()> {
        let config = self.config.lock().unwrap();
        let subsystems = config.enabled_subsystems.clone();
        drop(config);
        
        for subsystem in subsystems {
            if let Err(e) = self.log(&subsystem, "INFO", &format!("{} subsystem logging initialized - ready to receive data", subsystem)) {
                log::error!("Failed to initialize {} subsystem logging: {}", subsystem, e);
            }
        }
        
        Ok(())
    }
    
    pub fn log(&self, subsystem: &str, level: &str, message: &str) -> io::Result<()> {
        // All subsystems are always enabled, no need to check
        let config = self.config.lock().unwrap();
        
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        let entry = LogEntry {
            timestamp,
            level: level.to_string(),
            subsystem: subsystem.to_string(),
            message: message.to_string(),
        };
        
        // Get or create logger for this subsystem
        let mut loggers = self.loggers.lock().unwrap();
        let logger = loggers.entry(subsystem.to_string()).or_insert_with(|| {
            Logger::new(&config.log_dir, subsystem).unwrap_or_else(|e| {
                log::error!("Failed to create logger for subsystem {}: {}", subsystem, e);
                eprintln!("Failed to create logger for subsystem: {}", subsystem);
                Logger::new("log", "fallback").unwrap()
            })
        });
        
        // Write log entry
        logger.write_entry(&entry)?;
        
        // Check if rotation is needed
        if let Ok(true) = self.rotator.should_rotate(&logger.get_current_file_path()) {
            self.rotate_log(subsystem)?;
        }
        
        Ok(())
    }
    
    // Removed set_subsystem_enabled and is_subsystem_enabled methods since all subsystems are always enabled
    
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
    
    pub fn list_archives(&self) -> io::Result<Vec<String>> {
        self.archiver.list_archives()
    }
    
    pub fn extract_archive(&self, archive_name: &str) -> io::Result<()> {
        self.archiver.extract_archive(archive_name)
    }
    
    pub fn download_archive(&self, archive_name: &str) -> io::Result<Vec<u8>> {
        self.archiver.download_archive(archive_name)
    }
    
    pub fn get_config(&self) -> LogConfig {
        let config = self.config.lock().unwrap();
        config.clone()
    }
    
    /// Create a complete archive of all current logs
    pub fn create_complete_archive(&self) -> io::Result<ArchiveInfo> {
        let config = self.config.lock().unwrap();
        let log_dir = config.log_dir.clone();
        drop(config);
        
        let archive_path = self.archiver.create_complete_log_archive(&log_dir)?;
        
        // Get archive info
        let archive_name = archive_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid archive filename"))?;
        
        self.archiver.get_archive_info(archive_name)
    }
    
    /// Create archive and upload to Google Drive
    pub async fn create_and_upload_archive(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Create the archive
        let archive_info = self.create_complete_archive()
            .map_err(|e| format!("Failed to create archive: {}", e))?;
        
        log::info!("Created archive: {} ({} bytes)", archive_info.name, archive_info.size);
        
        // Upload to Google Drive
        let drive_plugin = crate::plugins::plugin_drive::drive_plugin();
        let file_id = drive_plugin.upload_file_streaming(&archive_info.file_path, &archive_info.name)
            .await
            .map_err(|e| format!("Failed to upload to Google Drive: {}", e))?;
        
        log::info!("Successfully uploaded archive {} to Google Drive with ID: {}", archive_info.name, file_id);
        
        Ok(format!("Archive '{}' uploaded successfully to Google Drive", archive_info.name))
    }
    
    /// Create archive, upload to Google Drive, and delete local file
    pub async fn create_upload_and_cleanup_archive(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Create the archive
        let archive_info = self.create_complete_archive()
            .map_err(|e| format!("Failed to create archive: {}", e))?;
        
        log::info!("Created archive: {} ({} bytes)", archive_info.name, archive_info.size);
        
        // Upload to Google Drive
        let drive_plugin = crate::plugins::plugin_drive::drive_plugin();
        let file_id = drive_plugin.upload_file_streaming(&archive_info.file_path, &archive_info.name)
            .await
            .map_err(|e| format!("Failed to upload to Google Drive: {}", e))?;
        
        log::info!("Successfully uploaded archive {} to Google Drive with ID: {}", archive_info.name, file_id);
        
        // Delete local archive file after successful upload
        if let Err(e) = self.archiver.delete_archive(&archive_info.name) {
            log::warn!("Failed to delete local archive after upload: {}", e);
            return Ok(format!("Archive '{}' uploaded successfully to Google Drive but local cleanup failed", archive_info.name));
        }
        
        log::info!("Deleted local archive after successful upload: {}", archive_info.name);
        
        Ok(format!("Archive '{}' uploaded to Google Drive and cleaned up locally", archive_info.name))
    }
    
    /// Check if auto-archiving should be performed
    pub fn should_auto_archive(&self, config: &AutoArchiveConfig) -> bool {
        self.archiver.should_auto_archive(config)
    }
    
    /// Get next scheduled archive time
    pub fn get_next_archive_time(&self, config: &AutoArchiveConfig) -> Option<String> {
        self.archiver.get_next_archive_time(config)
    }
    
    /// Perform auto-archive based on configuration
    pub async fn perform_auto_archive(&self, config: &mut AutoArchiveConfig) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if !config.enabled || !self.should_auto_archive(config) {
            return Ok("Auto-archive not needed at this time".to_string());
        }
        
        let result = if config.upload_to_drive {
            if config.delete_after_upload {
                self.create_upload_and_cleanup_archive().await?
            } else {
                self.create_and_upload_archive().await?
            }
        } else {
            let archive_info = self.create_complete_archive()
                .map_err(|e| format!("Failed to create archive: {}", e))?;
            format!("Archive '{}' created successfully", archive_info.name)
        };
        
        // Update last archive time
        config.last_archive_time = Some(chrono::Utc::now().to_rfc3339());
        
        log::info!("Auto-archive completed: {}", result);
        Ok(result)
    }
    
    /// Get archive information
    pub fn get_archive_info(&self, archive_name: &str) -> io::Result<ArchiveInfo> {
        self.archiver.get_archive_info(archive_name)
    }
    
    /// Delete an archive
    pub fn delete_archive(&self, archive_name: &str) -> io::Result<()> {
        self.archiver.delete_archive(archive_name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFileInfo {
    pub name: String,
    pub size: u64,
    pub modified: String,
    pub subsystem: String,
} 