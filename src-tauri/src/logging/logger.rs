//! Subsystem logger with rotation support
//!
//! Purpose: Structured log writing per subsystem; pairs with LogRotator and
//! LogArchiver to manage file lifecycles.
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write, BufWriter};
use std::path::{Path, PathBuf};
use super::LogEntry;

pub struct Logger {
    log_dir: String,
    subsystem: String,
    current_file: Option<BufWriter<File>>,
    current_file_path: PathBuf,
    file_counter: u32,
}

impl Logger {
    pub fn new(log_dir: &str, subsystem: &str) -> io::Result<Self> {
        let current_file_path = Path::new(log_dir).join(format!("{}.log", subsystem));
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&current_file_path)?;
        
        let current_file = Some(BufWriter::new(file));
        
        Ok(Self {
            log_dir: log_dir.to_string(),
            subsystem: subsystem.to_string(),
            current_file,
            current_file_path,
            file_counter: 0,
        })
    }
    
    pub fn write_entry(&mut self, entry: &LogEntry) -> io::Result<()> {
        if let Some(writer) = &mut self.current_file {
            let log_line = format!("[{}] [{}] [{}] {}\n", 
                entry.timestamp, 
                entry.level, 
                entry.subsystem, 
                entry.message
            );
            writer.write_all(log_line.as_bytes())?;
            writer.flush()?;
        }
        Ok(())
    }
    
    pub fn rotate(&mut self) -> io::Result<()> {
        // Close current file
        if let Some(mut writer) = self.current_file.take() {
            writer.flush()?;
        }
        
        // Generate new filename with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let new_filename = format!("{}_{}_{}.log", self.subsystem, timestamp, self.file_counter);
        let new_path = Path::new(&self.log_dir).join(&new_filename);
        
        // Rename current file
        if self.current_file_path.exists() {
            fs::rename(&self.current_file_path, &new_path)?;
        }
        
        // Create new file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.current_file_path)?;
        
        self.current_file = Some(BufWriter::new(file));
        self.file_counter += 1;
        
        Ok(())
    }
    
    pub fn get_current_file_path(&self) -> &Path {
        &self.current_file_path
    }
    
    pub fn get_subsystem(&self) -> &str {
        &self.subsystem
    }
} 