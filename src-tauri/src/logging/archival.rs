use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use zip::{write::FileOptions, ZipWriter};
use chrono::Utc;


pub struct LogArchiver {
    retention_days: u32,
    archive_dir: String,
}

impl LogArchiver {
    pub fn new(retention_days: u32) -> Self {
        Self { 
            retention_days,
            archive_dir: "log/archives".to_string(),
        }
    }
    
    pub fn new_with_archive_dir(retention_days: u32, archive_dir: String) -> Self {
        Self { 
            retention_days,
            archive_dir,
        }
    }
    
    pub fn cleanup_old_logs(&self, log_dir: &str) -> io::Result<()> {
        let log_path = Path::new(log_dir);
        
        if !log_path.exists() {
            return Ok(());
        }
        
        // Create archive directory if it doesn't exist
        let archive_path = Path::new(&self.archive_dir);
        fs::create_dir_all(archive_path)?;
        
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - (self.retention_days as u64 * 24 * 60 * 60);
        
        // Group files by subsystem for archiving
        let mut files_to_archive: HashMap<String, Vec<PathBuf>> = HashMap::new();
        
        for entry in fs::read_dir(log_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(modified_secs) = modified.duration_since(UNIX_EPOCH) {
                            if modified_secs.as_secs() < cutoff_time {
                                // File is older than retention period, archive it
                                if let Some(file_stem) = path.file_stem() {
                                    if let Some(subsystem) = file_stem.to_str() {
                                        // Extract subsystem name (remove timestamp and counter)
                                        let subsystem_name = self.extract_subsystem_name(subsystem);
                                        files_to_archive
                                            .entry(subsystem_name)
                                            .or_insert_with(Vec::new)
                                            .push(path.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Archive files by subsystem
        for (subsystem, files) in files_to_archive {
            if !files.is_empty() {
                self.archive_subsystem_files(&subsystem, &files)?;
                
                // Delete original files after successful archiving
                for file_path in files {
                    if let Err(e) = fs::remove_file(&file_path) {
                        eprintln!("Failed to delete archived log file {:?}: {}", file_path, e);
                    } else {
                        println!("Archived and deleted old log file: {:?}", file_path);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn extract_subsystem_name(&self, filename: &str) -> String {
        // Extract subsystem name from filenames like:
        // "pss_20241219_143022_0" -> "pss"
        // "obs_20241219_143156_1" -> "obs"
        // "udp_20241219_143245_0" -> "udp"
        if let Some(underscore_pos) = filename.find('_') {
            filename[..underscore_pos].to_string()
        } else {
            filename.to_string()
        }
    }
    
    fn archive_subsystem_files(&self, subsystem: &str, files: &[PathBuf]) -> io::Result<()> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let archive_filename = format!("{}_{}_archive.zip", subsystem, timestamp);
        let archive_path = Path::new(&self.archive_dir).join(&archive_filename);
        
        // Create ZIP archive
        let file = fs::File::create(&archive_path)?;
        let mut zip = ZipWriter::new(file);
        
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(6));
        
        for file_path in files {
            if let Some(file_name) = file_path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    zip.start_file(name_str, options)?;
                    
                    // Read and write file content
                    let content = fs::read(file_path)?;
                    zip.write_all(&content)?;
                }
            }
        }
        
        zip.finish()?;
        println!("Created archive: {:?}", archive_path);
        
        Ok(())
    }
    
    pub fn get_retention_days(&self) -> u32 {
        self.retention_days
    }
    
    pub fn set_retention_days(&mut self, days: u32) {
        self.retention_days = days;
    }
    
    pub fn get_archive_dir(&self) -> &str {
        &self.archive_dir
    }
    
    pub fn set_archive_dir(&mut self, archive_dir: String) {
        self.archive_dir = archive_dir;
    }
    
    /// List all archive files
    pub fn list_archives(&self) -> io::Result<Vec<String>> {
        let archive_path = Path::new(&self.archive_dir);
        if !archive_path.exists() {
            return Ok(Vec::new());
        }
        
        let mut archives = Vec::new();
        for entry in fs::read_dir(archive_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    if let Some(name_str) = file_name.to_str() {
                        if name_str.ends_with(".zip") {
                            archives.push(name_str.to_string());
                        }
                    }
                }
            }
        }
        
        archives.sort();
        Ok(archives)
    }
    
    /// Extract a specific archive
    pub fn extract_archive(&self, archive_name: &str) -> io::Result<()> {
        let archive_path = Path::new(&self.archive_dir).join(archive_name);
        if !archive_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Archive not found"));
        }
        
        let extract_dir = Path::new(&self.archive_dir).join("extracted");
        fs::create_dir_all(&extract_dir)?;
        
        let file = fs::File::open(&archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = extract_dir.join(file.name());
            
            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }
        
        println!("Extracted archive {:?} to {:?}", archive_path, extract_dir);
        Ok(())
    }
    
    /// Download a specific archive (read file contents)
    pub fn download_archive(&self, archive_name: &str) -> io::Result<Vec<u8>> {
        let archive_path = Path::new(&self.archive_dir).join(archive_name);
        if !archive_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Archive not found"));
        }
        
        fs::read(&archive_path)
    }
} 