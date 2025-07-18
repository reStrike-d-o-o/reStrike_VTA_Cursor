use std::fs;
use std::io;
use std::path::Path;

pub struct LogRotator {
    max_file_size: u64,
}

impl LogRotator {
    pub fn new(max_file_size: u64) -> Self {
        Self { max_file_size }
    }
    
    pub fn should_rotate(&self, file_path: &Path) -> io::Result<bool> {
        if !file_path.exists() {
            return Ok(false);
        }
        
        let metadata = fs::metadata(file_path)?;
        Ok(metadata.len() >= self.max_file_size)
    }
    
    pub fn get_max_file_size(&self) -> u64 {
        self.max_file_size
    }
    
    pub fn set_max_file_size(&mut self, size: u64) {
        self.max_file_size = size;
    }
} 