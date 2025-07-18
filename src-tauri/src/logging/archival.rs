use std::fs;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};


pub struct LogArchiver {
    retention_days: u32,
}

impl LogArchiver {
    pub fn new(retention_days: u32) -> Self {
        Self { retention_days }
    }
    
    pub fn cleanup_old_logs(&self, log_dir: &str) -> io::Result<()> {
        let log_path = Path::new(log_dir);
        
        if !log_path.exists() {
            return Ok(());
        }
        
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - (self.retention_days as u64 * 24 * 60 * 60);
        
        for entry in fs::read_dir(log_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(modified_secs) = modified.duration_since(UNIX_EPOCH) {
                            if modified_secs.as_secs() < cutoff_time {
                                // File is older than retention period, delete it
                                if let Err(e) = fs::remove_file(&path) {
                                    eprintln!("Failed to delete old log file {:?}: {}", path, e);
                                } else {
                                    println!("Deleted old log file: {:?}", path);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    pub fn get_retention_days(&self) -> u32 {
        self.retention_days
    }
    
    pub fn set_retention_days(&mut self, days: u32) {
        self.retention_days = days;
    }
} 