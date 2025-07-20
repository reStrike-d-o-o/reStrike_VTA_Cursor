use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::types::{AppError, AppResult};

/// Protocol version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub version: String,
    pub filename: String,
    pub file_path: String,
    pub description: String,
    pub created_date: String,
    pub last_modified: String,
    pub is_active: bool,
    pub file_size: u64,
    pub checksum: Option<String>,
}

/// Protocol file content structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolFile {
    pub version: String,
    pub description: String,
    pub year: u32,
    pub streams: HashMap<String, StreamDefinition>,
    pub examples: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Stream definition for protocol parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDefinition {
    pub name: String,
    pub description: String,
    pub required_arguments: Vec<String>,
    pub optional_arguments: Option<Vec<String>>,
    pub examples: Vec<String>,
}

/// Protocol Manager Plugin
#[derive(Clone)]
pub struct ProtocolManager {
    protocols_dir: PathBuf,
    active_version: Arc<Mutex<Option<String>>>,
    versions: Arc<Mutex<HashMap<String, ProtocolVersion>>>,
    current_protocol: Arc<Mutex<Option<ProtocolFile>>>,
}

impl ProtocolManager {
    pub fn new() -> AppResult<Self> {
        let app_dir = std::env::current_dir()
            .map_err(|e| AppError::ConfigError(format!("Failed to get current directory: {}", e)))?;
        
        let protocols_dir = app_dir.join("protocol");
        
        // Create protocols directory if it doesn't exist
        if !protocols_dir.exists() {
            fs::create_dir_all(&protocols_dir)
                .map_err(|e| AppError::ConfigError(format!("Failed to create protocols directory: {}", e)))?;
        }

        Ok(Self {
            protocols_dir,
            active_version: Arc::new(Mutex::new(None)),
            versions: Arc::new(Mutex::new(HashMap::new())),
            current_protocol: Arc::new(Mutex::new(None)),
        })
    }

    /// Initialize the protocol manager
    pub async fn init(&self) -> AppResult<()> {
        log::info!("🔧 Initializing Protocol Manager...");
        
        // Scan for existing protocol files
        self.scan_protocol_files().await?;
        
        // Load the active protocol
        self.load_active_protocol().await?;
        
        log::info!("✅ Protocol Manager initialized successfully");
        Ok(())
    }

    /// Scan for protocol files in the protocols directory
    async fn scan_protocol_files(&self) -> AppResult<()> {
        let mut versions = self.versions.lock().unwrap();
        versions.clear();

        if !self.protocols_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.protocols_dir)
            .map_err(|e| AppError::ConfigError(format!("Failed to read protocols directory: {}", e)))? {
            
            let entry = entry
                .map_err(|e| AppError::ConfigError(format!("Failed to read directory entry: {}", e)))?;
            
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "txt" || extension == "json" {
                        if let Some(filename) = path.file_name() {
                            let filename_str = filename.to_string_lossy().to_string();
                            
                            // Extract version from filename (e.g., "pss_v2.3.txt" -> "2.3")
                            let version = self.extract_version_from_filename(&filename_str);
                            
                            let metadata = fs::metadata(&path)
                                .map_err(|e| AppError::ConfigError(format!("Failed to get file metadata: {}", e)))?;
                            
                            let created_date = metadata.created()
                                .unwrap_or_else(|_| std::time::SystemTime::now())
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            
                            let modified_date = metadata.modified()
                                .unwrap_or_else(|_| std::time::SystemTime::now())
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            
                            let protocol_version = ProtocolVersion {
                                version: version.clone(),
                                filename: filename_str,
                                file_path: path.to_string_lossy().to_string(),
                                description: format!("PSS Protocol Version {}", version),
                                created_date: DateTime::from_timestamp(created_date as i64, 0)
                                    .unwrap_or_default()
                                    .to_rfc3339(),
                                last_modified: DateTime::from_timestamp(modified_date as i64, 0)
                                    .unwrap_or_default()
                                    .to_rfc3339(),
                                is_active: false,
                                file_size: metadata.len(),
                                checksum: None,
                            };
                            
                            versions.insert(version, protocol_version);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Extract version from filename
    fn extract_version_from_filename(&self, filename: &str) -> String {
        // Try to extract version from patterns like "pss_v2.3.txt", "WT_UDP_v2.3_2024.txt", etc.
        if let Some(v_start) = filename.find('v') {
            if let Some(extension_start) = filename.rfind('.') {
                let version_part = &filename[v_start + 1..extension_start];
                // Look for the first underscore or end of string to get just the version number
                if let Some(underscore_pos) = version_part.find('_') {
                    let clean_version = &version_part[..underscore_pos];
                    if clean_version.chars().any(|c| c.is_digit(10)) {
                        return clean_version.to_string();
                    }
                } else {
                    // No underscore, use the whole version part
                    if version_part.chars().any(|c| c.is_digit(10)) {
                        return version_part.to_string();
                    }
                }
            }
        }
        
        // Fallback: use filename without extension
        if let Some(extension_start) = filename.rfind('.') {
            filename[..extension_start].to_string()
        } else {
            filename.to_string()
        }
    }

    /// Load the active protocol file
    async fn load_active_protocol(&self) -> AppResult<()> {
        let active_version = {
            let active = self.active_version.lock().unwrap();
            active.clone()
        };

        if let Some(version) = active_version {
            if let Some(protocol_version) = self.get_version(&version).await? {
                let protocol_file = self.parse_protocol_file(&protocol_version.file_path).await?;
                let mut current = self.current_protocol.lock().unwrap();
                *current = Some(protocol_file);
            }
        }

        Ok(())
    }

    /// Get all protocol versions
    pub async fn get_versions(&self) -> AppResult<Vec<ProtocolVersion>> {
        let versions = self.versions.lock().unwrap();
        Ok(versions.values().cloned().collect())
    }

    /// Get a specific protocol version
    pub async fn get_version(&self, version: &str) -> AppResult<Option<ProtocolVersion>> {
        let versions = self.versions.lock().unwrap();
        Ok(versions.get(version).cloned())
    }

    /// Get the currently active protocol
    pub async fn get_current_protocol(&self) -> AppResult<Option<ProtocolFile>> {
        let current = self.current_protocol.lock().unwrap();
        Ok(current.clone())
    }

    /// Set the active protocol version
    pub async fn set_active_version(&self, version: &str) -> AppResult<()> {
        // Verify the version exists
        if !self.versions.lock().unwrap().contains_key(version) {
            return Err(AppError::ConfigError(format!("Protocol version {} not found", version)));
        }

        // Update active version
        {
            let mut active = self.active_version.lock().unwrap();
            *active = Some(version.to_string());
        }

        // Update active status in versions
        {
            let mut versions = self.versions.lock().unwrap();
            for (ver, protocol_version) in versions.iter_mut() {
                protocol_version.is_active = ver == version;
            }
        }

        // Load the new active protocol
        self.load_active_protocol().await?;

        log::info!("✅ Set active protocol version to {}", version);
        Ok(())
    }

    /// Upload a new protocol file
    pub async fn upload_protocol_file(&self, file_content: Vec<u8>, filename: &str) -> AppResult<String> {
        log::info!("📤 Starting upload of protocol file: {}", filename);
        
        // Validate file content
        if file_content.is_empty() {
            log::error!("❌ File content is empty");
            return Err(AppError::ConfigError("File content is empty".to_string()));
        }

        // Extract version from filename
        let version = self.extract_version_from_filename(filename);
        log::info!("📋 Extracted version: {} from filename: {}", version, filename);
        
        // Check if version already exists
        if self.versions.lock().unwrap().contains_key(&version) {
            log::error!("❌ Protocol version {} already exists", version);
            return Err(AppError::ConfigError(format!("Protocol version {} already exists", version)));
        }

        // Create file path
        let file_path = self.protocols_dir.join(filename);
        log::info!("📁 File path: {}", file_path.display());
        
        // Write file
        match fs::write(&file_path, &file_content) {
            Ok(_) => log::info!("✅ File written successfully"),
            Err(e) => {
                log::error!("❌ Failed to write protocol file: {}", e);
                return Err(AppError::ConfigError(format!("Failed to write protocol file: {}", e)));
            }
        }

        // Parse the protocol file to validate it
        log::info!("🔍 Parsing protocol file for validation...");
        let protocol_file = match self.parse_protocol_file(&file_path.to_string_lossy()).await {
            Ok(file) => {
                log::info!("✅ Protocol file parsed successfully");
                file
            }
            Err(e) => {
                log::error!("❌ Failed to parse protocol file: {}", e);
                // Clean up the file we just wrote
                let _ = fs::remove_file(&file_path);
                return Err(e);
            }
        };

        // Create protocol version record
        let metadata = match fs::metadata(&file_path) {
            Ok(meta) => meta,
            Err(e) => {
                log::error!("❌ Failed to get file metadata: {}", e);
                return Err(AppError::ConfigError(format!("Failed to get file metadata: {}", e)));
            }
        };

        let now = Utc::now();
        let protocol_version = ProtocolVersion {
            version: version.clone(),
            filename: filename.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            description: protocol_file.description.clone(),
            created_date: now.to_rfc3339(),
            last_modified: now.to_rfc3339(),
            is_active: false,
            file_size: metadata.len(),
            checksum: None,
        };

        // Add to versions
        {
            let mut versions = self.versions.lock().unwrap();
            versions.insert(version.clone(), protocol_version);
            log::info!("✅ Added protocol version to registry");
        }

        log::info!("✅ Successfully uploaded protocol file: {} (version {})", filename, version);
        Ok(version)
    }

    /// Delete a protocol version
    pub async fn delete_version(&self, version: &str) -> AppResult<()> {
        let protocol_version = {
            let versions = self.versions.lock().unwrap();
            versions.get(version).cloned()
        };

        if let Some(protocol_version) = protocol_version {
            // Don't allow deletion of active version
            if protocol_version.is_active {
                return Err(AppError::ConfigError("Cannot delete active protocol version".to_string()));
            }

            // Delete file
            let file_path = Path::new(&protocol_version.file_path);
            if file_path.exists() {
                fs::remove_file(file_path)
                    .map_err(|e| AppError::ConfigError(format!("Failed to delete protocol file: {}", e)))?;
            }

            // Remove from versions
            {
                let mut versions = self.versions.lock().unwrap();
                versions.remove(version);
            }

            log::info!("✅ Deleted protocol version: {}", version);
            Ok(())
        } else {
            Err(AppError::ConfigError(format!("Protocol version {} not found", version)))
        }
    }

    /// Export a protocol file
    pub async fn export_protocol_file(&self, version: &str) -> AppResult<Vec<u8>> {
        let protocol_version = {
            let versions = self.versions.lock().unwrap();
            versions.get(version).cloned()
        };

        if let Some(protocol_version) = protocol_version {
            let file_path = Path::new(&protocol_version.file_path);
            if file_path.exists() {
                let content = fs::read(file_path)
                    .map_err(|e| AppError::ConfigError(format!("Failed to read protocol file: {}", e)))?;
                Ok(content)
            } else {
                Err(AppError::ConfigError("Protocol file not found on disk".to_string()))
            }
        } else {
            Err(AppError::ConfigError(format!("Protocol version {} not found", version)))
        }
    }

    /// Parse a protocol file (supports both TXT and JSON formats)
    async fn parse_protocol_file(&self, file_path: &str) -> AppResult<ProtocolFile> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| AppError::ConfigError(format!("Failed to read protocol file: {}", e)))?;

        let path = Path::new(file_path);
        if let Some(extension) = path.extension() {
            match extension.to_string_lossy().as_ref() {
                "json" => self.parse_json_protocol(&content),
                "txt" => self.parse_txt_protocol(&content),
                _ => Err(AppError::ConfigError("Unsupported protocol file format".to_string())),
            }
        } else {
            // Default to TXT format
            self.parse_txt_protocol(&content)
        }
    }

    /// Parse JSON protocol file
    fn parse_json_protocol(&self, content: &str) -> AppResult<ProtocolFile> {
        serde_json::from_str(content)
            .map_err(|e| AppError::ConfigError(format!("Failed to parse JSON protocol: {}", e)))
    }

    /// Parse TXT protocol file (enhanced parsing for the WT_UDP format)
    fn parse_txt_protocol(&self, content: &str) -> AppResult<ProtocolFile> {
        let lines: Vec<&str> = content.lines().collect();
        
        // Extract basic information
        let mut version = "1.0".to_string();
        let mut description = "PSS Protocol".to_string();
        let mut year = 2024;
        let mut streams = HashMap::new();
        let mut examples = Vec::new();
        let mut metadata = HashMap::new();

        // Parse header information
        for line in &lines {
            let line = line.trim();
            if line.is_empty() || !line.starts_with('#') {
                continue;
            }

            if line.starts_with("# Version:") {
                version = line.split(':').nth(1).unwrap_or("1.0").trim().to_string();
            } else if line.starts_with("# Year:") {
                year = line.split(':').nth(1).unwrap_or("2024").trim().parse().unwrap_or(2024);
            } else if line.starts_with("# Description:") {
                description = line.split(':').nth(1).unwrap_or("PSS Protocol").trim().to_string();
            }
        }

        // Parse stream sections
        let mut current_stream: Option<String> = None;
        let mut current_required_args = Vec::new();
        let mut current_optional_args = Vec::new();
        let mut current_examples = Vec::new();

        for line in &lines {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("---") {
                continue;
            }

            // Check for section headers (stream types)
            if line.starts_with("# ") && !line.starts_with("# EXAMPLE") && !line.starts_with("# STEP") {
                // Save previous section if exists
                if let Some(stream_name) = current_stream {
                    let stream_def = StreamDefinition {
                        name: stream_name.clone(),
                        description: format!("Stream for {}", stream_name),
                        required_arguments: current_required_args.clone(),
                        optional_arguments: if current_optional_args.is_empty() { None } else { Some(current_optional_args.clone()) },
                        examples: current_examples.clone(),
                    };
                    streams.insert(stream_name, stream_def);
                }

                // Start new section
                current_stream = None;
                current_required_args.clear();
                current_optional_args.clear();
                current_examples.clear();
            }
            // Parse MAIN_STREAMS
            else if line.starts_with("MAIN_STREAMS:") {
                // Continue to next line to get stream names
            }
            else if line.contains(';') && (line.starts_with("  ") || !line.starts_with("REQUIRED_ARGUMENTS:") && !line.starts_with("OPTIONAL_ARGUMENTS:") && !line.starts_with("EXAMPLES:")) {
                // This is a stream name line
                let stream_name = line.split(';').next().unwrap_or("").trim();
                if !stream_name.is_empty() {
                    current_stream = Some(stream_name.to_string());
                }
            }
            // Parse REQUIRED_ARGUMENTS
            else if line.starts_with("REQUIRED_ARGUMENTS:") {
                // Continue to next lines to get arguments
            }
            else if line.starts_with("  ") && line.contains(';') && current_stream.is_some() {
                // This is a required argument line
                let arg = line.split(';').next().unwrap_or("").trim();
                if !arg.is_empty() {
                    current_required_args.push(arg.to_string());
                }
            }
            // Parse OPTIONAL_ARGUMENTS
            else if line.starts_with("OPTIONAL_ARGUMENTS:") {
                // Continue to next lines to get arguments
            }
            else if line.starts_with("  ") && line.contains(';') && current_stream.is_some() {
                // This is an optional argument line
                let arg = line.split(';').next().unwrap_or("").trim();
                if !arg.is_empty() {
                    current_optional_args.push(arg.to_string());
                }
            }
            // Parse EXAMPLES
            else if line.starts_with("EXAMPLES:") {
                // Continue to next lines to get examples
            }
            else if line.starts_with("  ") && line.contains(';') && current_stream.is_some() {
                // This is an example line
                let example = line.trim();
                if !example.is_empty() {
                    current_examples.push(example.to_string());
                    examples.push(example.to_string());
                }
            }
        }

        // Save the last section
        if let Some(stream_name) = current_stream {
            let stream_def = StreamDefinition {
                name: stream_name.clone(),
                description: format!("Stream for {}", stream_name),
                required_arguments: current_required_args.clone(),
                optional_arguments: if current_optional_args.is_empty() { None } else { Some(current_optional_args.clone()) },
                examples: current_examples.clone(),
            };
            streams.insert(stream_name, stream_def);
        }

        // Add metadata
        metadata.insert("format".to_string(), "txt".to_string());
        metadata.insert("parsed_at".to_string(), chrono::Utc::now().to_rfc3339());

        Ok(ProtocolFile {
            version,
            description,
            year,
            streams,
            examples,
            metadata,
        })
    }

    /// Get protocol parsing rules for UDP parser
    pub async fn get_parsing_rules(&self) -> AppResult<HashMap<String, Vec<String>>> {
        let current = self.current_protocol.lock().unwrap();
        
        if let Some(protocol) = current.as_ref() {
            let mut rules = HashMap::new();
            
            for (stream_name, stream_def) in &protocol.streams {
                rules.insert(stream_name.clone(), stream_def.required_arguments.clone());
            }
            
            Ok(rules)
        } else {
            Ok(HashMap::new())
        }
    }
}

/// Initialize the Protocol Manager plugin
pub fn init() -> Result<ProtocolManager, Box<dyn std::error::Error>> {
    log::info!("🔧 Initializing Protocol Manager plugin...");
    ProtocolManager::new().map_err(|e| e.into())
} 