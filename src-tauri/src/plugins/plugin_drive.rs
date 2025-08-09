use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use reqwest::Client;
use zip::{ZipArchive, write::FileOptions};
use dirs;

use crate::types::{AppError, AppResult};

const REDIRECT_URI: &str = "urn:ietf:wg:oauth:2.0:oob";
const GOOGLE_DRIVE_API_BASE: &str = "https://www.googleapis.com/drive/v3";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes

#[derive(Debug, Serialize, Deserialize)]
struct StoredCreds {
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredToken {
    refresh_token: String,
    access_token: String,
    expires_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleDriveFile {
    id: String,
    name: String,
    #[serde(rename = "mimeType")]
    mime_type: Option<String>,
    size: Option<String>,
    #[serde(rename = "createdTime")]
    created_time: String,
    #[serde(rename = "modifiedTime")]
    modified_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleDriveFileList {
    files: Vec<GoogleDriveFile>,
    next_page_token: Option<String>,
}

pub struct DrivePlugin {
    client: Arc<Mutex<Option<BasicClient>>>,
    http_client: Client,
    access_token: Arc<Mutex<Option<String>>>,
}

impl DrivePlugin {
    pub fn new() -> Self {
        // Create HTTP client with proper timeouts and retry configuration
        let http_client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { 
            client: Arc::new(Mutex::new(None)),
            http_client,
            access_token: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn get_quota(&self) -> AppResult<(u64,u64,u64)> {
        // Use the 'about' endpoint to retrieve quota information
        let token = self.get_access_token().await?;
        let url = "https://www.googleapis.com/drive/v3/about?fields=storageQuota";
        let response = self.http_client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::NetworkError(format!("Drive about failed: {} - {}", status, text)));
        }
        let v: serde_json::Value = response.json().await.map_err(|e| AppError::NetworkError(e.to_string()))?;
        let quota = &v["storageQuota"];
        let limit = quota["limit"].as_str().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        let usage = quota["usage"].as_str().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        let usage_in_drive = quota["usageInDrive"].as_str().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        Ok((limit, usage, usage_in_drive))
    }

    // Enhanced error logging with crash detection
    fn log_error_comprehensively(&self, context: &str, error: &str, details: Option<&str>) {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let mut log_entry = format!("[{}] GOOGLE DRIVE CRITICAL ERROR - {}: {}\n", timestamp, context, error);
        
        if let Some(detail_info) = details {
            log_entry.push_str(&format!("Details: {}\n", detail_info));
        }
        
        // Add stack trace indicator
        log_entry.push_str("Stack Trace: Check Rust panic logs\n");
        log_entry.push_str("Memory Status: Check system memory usage\n");
        log_entry.push_str("=== CRITICAL ERROR END ===\n");
        
        // Log to multiple locations
        log::error!("🚨 CRITICAL: {} - {}", context, error);
        if let Some(detail_info) = details {
            log::error!("🔍 Details: {}", detail_info);
        }
        
        // Force write to multiple log files
        let log_locations = [
            "app_errors.log",
            "google_drive_errors.log", 
            "src-tauri/logs/drive_critical.log"
        ];
        
        for log_path in &log_locations {
            let full_path = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join(log_path);
            
            // Ensure directory exists
            if let Some(parent) = full_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&full_path)
                .and_then(|mut file| {
                    use std::io::Write;
                    file.write_all(log_entry.as_bytes())?;
                    file.sync_all()
                });
        }
    }

    pub async fn auth_url(&self) -> AppResult<(String, String)> {
        // Avoid potential deadlock by scoping mutex guards
        let client_exists = {
            let guard = self.client.lock().await;
            guard.is_some()
        };
        
        if !client_exists {
            if let Err(e) = self.initialize_client().await {
                self.log_error_comprehensively("auth_url", "Failed to initialize client", Some(&e.to_string()));
                return Err(e);
            }
        }
        
        let guard = self.client.lock().await;
        let client = guard.as_ref().ok_or_else(|| {
            AppError::ConfigError("Client not initialized after setup".to_string())
        })?;
        
        let (url, csrf) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/drive".into(),
            ))
            .url();
        Ok((url.to_string(), csrf.secret().to_string()))
    }

    async fn initialize_client(&self) -> AppResult<()> {
        let mut client_guard = self.client.lock().await;
        let (client_id, client_secret) = Self::load_credentials()?;
        let new_client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into()).unwrap(),
            Some(TokenUrl::new("https://oauth2.googleapis.com/token".into()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(REDIRECT_URI.into()).unwrap());
        *client_guard = Some(new_client);
        Ok(())
    }

    pub async fn exchange_code(&self, code: String) -> AppResult<()> {
        let client_guard = self.client.lock().await;
        
        let client = client_guard.as_ref().ok_or_else(|| {
            AppError::ConfigError("Client not initialized".to_string())
        })?;
        
        let token_res = client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .map_err(|e| AppError::ConfigError(e.to_string()))?;

        if let Some(refresh) = token_res.refresh_token() {
            let access_token = token_res.access_token().secret().to_string();
            let expires_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + token_res.expires_in().unwrap().as_secs();
            
            drop(client_guard); // Release before other async operations
            
            self.store_tokens(refresh.secret(), &access_token, expires_at)?;
            
            // Update access token
            let mut token_guard = self.access_token.lock().await;
            *token_guard = Some(access_token);
        }
        Ok(())
    }

    pub async fn save_credentials(&self, id: String, secret: String) -> AppResult<()> {
        let creds = StoredCreds {
            client_id: id.clone(),
            client_secret: secret.clone(),
        };
        let dir = Self::config_dir();
        fs::create_dir_all(&dir).map_err(AppError::IoError)?;
        fs::write(
            dir.join("google.json"),
            serde_json::to_vec_pretty(&creds).unwrap(),
        )
        .map_err(AppError::IoError)?;
        
        // Create new client with saved credentials
        let new_client = BasicClient::new(
            ClientId::new(id),
            Some(ClientSecret::new(secret)),
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into()).unwrap(),
            Some(TokenUrl::new("https://oauth2.googleapis.com/token".into()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(REDIRECT_URI.into()).unwrap());
        
        // Update the client
        let mut client_guard = self.client.lock().await;
        *client_guard = Some(new_client);
        
        Ok(())
    }

    pub async fn list_all_files(&self) -> AppResult<Vec<GoogleDriveFile>> {
        let token = self.get_access_token().await?;
        
        let response = self.http_client
            .get(&format!("{}/files", GOOGLE_DRIVE_API_BASE))
            .header("Authorization", format!("Bearer {}", token))
            .query(&[
                ("fields", "files(id,name,mimeType,size,createdTime,modifiedTime)"),
                ("orderBy", "modifiedTime desc"),
                ("pageSize", "10"),
            ])
            .send()
            .await
            .map_err(|e| {
                self.log_error_comprehensively("list_all_files", "HTTP request failed", Some(&e.to_string()));
                AppError::NetworkError(e.to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            self.log_error_comprehensively("list_all_files", &format!("API error: {} - {}", status, error_text), Some(&error_text));
            return Err(AppError::NetworkError(format!("API error: {} - {}", status, error_text)));
        }

        let file_list: GoogleDriveFileList = response.json().await
            .map_err(|e| {
                self.log_error_comprehensively(
                    "list_all_files", 
                    &format!("Failed to parse response JSON: {}", e), 
                    Some("This might be due to unexpected field names from Google Drive API")
                );
                AppError::NetworkError(e.to_string())
            })?;

        Ok(file_list.files)
    }

    pub async fn list_files(&self) -> AppResult<Vec<GoogleDriveFile>> {
        let token = self.get_access_token().await?;
        
        let response = self.http_client
            .get(&format!("{}/files", GOOGLE_DRIVE_API_BASE))
            .header("Authorization", format!("Bearer {}", token))
            .query(&[
                ("q", "mimeType='application/zip' and name contains 'reStrikeVTA_backup'"),
                ("fields", "files(id,name,mimeType,size,createdTime,modifiedTime)"),
                ("orderBy", "modifiedTime desc"),
            ])
            .send()
            .await
            .map_err(|e| {
                self.log_error_comprehensively("list_files", "HTTP request failed", Some(&e.to_string()));
                AppError::NetworkError(e.to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            self.log_error_comprehensively("list_files", &format!("API error: {} - {}", status, error_text), Some(&error_text));
            return Err(AppError::NetworkError(format!("API error: {} - {}", status, error_text)));
        }

        let file_list: GoogleDriveFileList = response.json().await
            .map_err(|e| {
                self.log_error_comprehensively(
                    "list_files", 
                    &format!("Failed to parse response JSON: {}", e), 
                    Some("This might be due to unexpected field names from Google Drive API")
                );
                AppError::NetworkError(e.to_string())
            })?;

        Ok(file_list.files)
    }

    pub async fn upload_backup_archive(&self) -> AppResult<String> {
        // Log error to file immediately
        let log_error_to_file = |error_msg: &str| {
            let error_log = format!(
                "[{}] DrivePlugin Upload Error:\nError: {}\nMethod: upload_backup_archive\n",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                error_msg
            );
            if let Err(write_err) = std::fs::write("logs/app.log", error_log) {
                log::error!("Failed to write error log: {}", write_err);
            }
        };
        
        // Step 1: Get current directory and create backup directory outside project
        log::info!("Step 1: Getting current directory and setting up backup directory...");
        let current_dir = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(e) => {
                let error_msg = format!("Failed to get current directory: {}", e);
                log::error!("{}", error_msg);
                log_error_to_file(&error_msg);
                return Err(AppError::IoError(e));
            }
        };
        
        // Step 2: Create backup directory path OUTSIDE the project directory
        log::info!("Step 2: Creating backup directory path outside project...");
        let backup_dir = match dirs::data_dir() {
            Some(data_dir) => data_dir.join("reStrikeVTA").join("backups"),
            None => {
                // Fallback to current directory if data_dir is not available
                let error_msg = "Failed to get app data directory, using current directory";
                log::warn!("{}", error_msg);
                current_dir.join("backups")
            }
        };
        log::info!("Backup directory: {}", backup_dir.display());
        
        // Step 3: Ensure backup directory exists
        log::info!("Step 3: Ensuring backup directory exists...");
        if !backup_dir.exists() {
            log::info!("Creating backup directory...");
            match std::fs::create_dir_all(&backup_dir) {
                Ok(_) => log::info!("Backup directory created successfully"),
                Err(e) => {
                    let error_msg = format!("Failed to create backup directory: {}", e);
                    log::error!("{}", error_msg);
                    log_error_to_file(&error_msg);
                    self.log_error_comprehensively(
                        "upload_backup_archive", 
                        &error_msg, 
                        Some(&format!("Directory: {}", backup_dir.display()))
                    );
                    return Err(AppError::IoError(e));
                }
            }
        }
        
        // Step 4: Create archive name and path
        log::info!("Step 4: Creating archive name and path...");
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let archive_name = format!("reStrikeVTA_backup_{}.zip", timestamp);
        let archive_path = backup_dir.join(&archive_name);
        log::info!("Archive name: {}", archive_name);
        log::info!("Archive path: {}", archive_path.display());
        
        // Step 5: Create zip archive
        log::info!("Step 5: Creating zip archive...");
        match self.create_backup_archive(&current_dir, &archive_path).await {
            Ok(_) => log::info!("Backup archive created successfully"),
            Err(e) => {
                let error_msg = format!("Failed to create backup archive: {}", e);
                log::error!("{}", error_msg);
                log_error_to_file(&error_msg);
                return Err(e);
            }
        }
        
        // Step 6: Verify archive was created
        log::info!("Step 6: Verifying archive was created...");
        if !archive_path.exists() {
            let error_msg = format!("Archive file was not created: {}", archive_path.display());
            log::error!("{}", error_msg);
            log_error_to_file(&error_msg);
            return Err(AppError::ConfigError(error_msg));
        }
        
        // Get archive size
        let archive_size = match archive_path.metadata() {
            Ok(metadata) => metadata.len(),
            Err(e) => {
                let error_msg = format!("Failed to get archive metadata: {}", e);
                log::error!("{}", error_msg);
                log_error_to_file(&error_msg);
                return Err(AppError::IoError(e));
            }
        };
        log::info!("Archive size: {} bytes", archive_size);
        
        // Step 7: Upload to Google Drive
        log::info!("Step 7: Uploading to Google Drive...");
        let _file_id = match self.upload_file_streaming(&archive_path, &archive_name).await {
            Ok(file_id) => {
                log::info!("Upload completed successfully, file ID: {}", file_id);
                file_id
            },
            Err(e) => {
                let error_msg = format!("Backup upload failed: {}", e);
                log::error!("{}", error_msg);
                log_error_to_file(&error_msg);
                
                // Use comprehensive error logging
                let details = format!(
                    "Archive Path: {}\nArchive Name: {}\nArchive Size: {} bytes",
                    archive_path.display(),
                    archive_name,
                    archive_size
                );
                
                self.log_error_comprehensively(
                    "upload_backup_archive", 
                    &error_msg, 
                    Some(&details)
                );
                
                return Err(e);
            }
        };
        
        // Step 8: Clean up local archive file
        log::info!("Step 8: Cleaning up local archive file...");
        match std::fs::remove_file(&archive_path) {
            Ok(_) => log::info!("Local archive file cleaned up successfully"),
            Err(e) => {
                let error_msg = format!("Failed to clean up local archive file: {}", e);
                log::warn!("{}", error_msg);
                log_error_to_file(&error_msg);
                // Don't fail the operation if cleanup fails
            }
        }
        
        log::info!("=== UPLOAD_BACKUP_ARCHIVE COMPLETED SUCCESSFULLY ===");
        Ok(format!("Backup archive uploaded successfully: {}", archive_name))
    }
    
    pub async fn download_backup_archive(&self, file_id: &str) -> AppResult<String> {
        // Download file from Google Drive
        let download_path = self.download_file(file_id).await?;
        
        // Extract archive to backups directory outside project
        let backup_dir = match dirs::data_dir() {
            Some(data_dir) => data_dir.join("reStrikeVTA").join("backups"),
            None => std::env::current_dir()?.join("backups"),
        };
        self.extract_backup_archive(&download_path, &backup_dir).await?;
        
        // Clean up downloaded archive
        let _ = std::fs::remove_file(&download_path);
        
        Ok("Backup archive downloaded and extracted successfully".to_string())
    }
    
    pub async fn restore_from_archive(&self, file_id: &str) -> AppResult<String> {
        // Download and extract archive
        let download_path = self.download_file(file_id).await?;
        let backup_dir = match dirs::data_dir() {
            Some(data_dir) => data_dir.join("reStrikeVTA").join("backups"),
            None => std::env::current_dir()?.join("backups"),
        };
        self.extract_backup_archive(&download_path, &backup_dir).await?;
        
        // Clean up downloaded archive
        let _ = std::fs::remove_file(&download_path);
        
        Ok("Backup restored successfully".to_string())
    }
    
    pub async fn delete_backup_archive(&self, file_id: &str) -> AppResult<()> {
        let token = self.get_access_token().await?;
        
        // Delete file from Google Drive
        let url = format!("{}/files/{}", GOOGLE_DRIVE_API_BASE, file_id);
        let response = self.http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(AppError::NetworkError(format!(
                "Failed to delete file: {}",
                response.status()
            )));
        }
        
        Ok(())
    }

    pub async fn is_connected(&self) -> AppResult<bool> {
        log::info!("is_connected: Starting connection check");
        
        // Ensure client is initialized first with proper error handling
        let client_exists = {
            let guard = self.client.lock().await;
            guard.is_some()
        };
        
        if !client_exists {
            log::info!("is_connected: Client not initialized, attempting to initialize");
            if let Err(e) = self.initialize_client().await {
                log::warn!("is_connected: Failed to initialize client: {:?}", e);
                return Ok(false);
            }
        }
        
        log::info!("is_connected: About to call get_access_token");
        // Try to get an access token - if successful, we're connected
        match self.get_access_token().await {
            Ok(_) => {
                log::info!("is_connected: Successfully got access token, returning true");
                Ok(true)
            },
            Err(e) => {
                log::warn!("is_connected: Failed to get access token: {:?}", e);
                Ok(false)
            },
        }
    }
    
    async fn create_backup_archive(&self, source_dir: &std::path::Path, archive_path: &std::path::Path) -> AppResult<()> {
        log::info!("=== CREATE_BACKUP_ARCHIVE START ===");
        log::info!("Creating backup archive from: {}", source_dir.display());
        log::info!("Archive path: {}", archive_path.display());
        
        // Log error to file immediately
        let log_error_to_file = |error_msg: &str| {
            let error_log = format!(
                "[{}] DrivePlugin Create Archive Error:\nError: {}\nMethod: create_backup_archive\nSource: {}\nTarget: {}\n",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                error_msg,
                source_dir.display(),
                archive_path.display()
            );
            if let Err(write_err) = std::fs::write("logs/app.log", error_log) {
                log::error!("Failed to write error log: {}", write_err);
            }
        };
        
        // Step 1: Ensure source directory exists
        log::info!("Step 1: Checking source directory exists...");
        if !source_dir.exists() {
            let error_msg = format!("Source directory does not exist: {}", source_dir.display());
            log::error!("{}", error_msg);
            log_error_to_file(&error_msg);
            return Err(AppError::ConfigError(error_msg));
        }
        log::info!("Source directory exists: {}", source_dir.display());
        
        // Step 2: Create the archive file
        log::info!("Step 2: Creating archive file...");
        let file = match std::fs::File::create(archive_path) {
            Ok(file) => {
                log::info!("Archive file created successfully");
                file
            },
            Err(e) => {
                let error_msg = format!("Failed to create archive file: {}", e);
                log::error!("{}", error_msg);
                log_error_to_file(&error_msg);
                return Err(AppError::IoError(e));
            }
        };
        
        // Step 3: Create zip writer
        log::info!("Step 3: Creating zip writer...");
        let mut zip = zip::ZipWriter::new(file);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);
        log::info!("Zip writer created with compression options");
        
        // Step 4: Read source directory
        log::info!("Step 4: Reading source directory...");
        let entries = match std::fs::read_dir(source_dir) {
            Ok(entries) => {
                log::info!("Source directory read successfully");
                entries
            },
            Err(e) => {
                let error_msg = format!("Failed to read source directory: {}", e);
                log::error!("{}", error_msg);
                log_error_to_file(&error_msg);
                return Err(AppError::IoError(e));
            }
        };
        
        // Step 5: Process each entry
        log::info!("Step 5: Processing directory entries...");
        let mut file_count = 0;
        for entry_result in entries {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => {
                    let error_msg = format!("Failed to read directory entry: {}", e);
                    log::error!("{}", error_msg);
                    log_error_to_file(&error_msg);
                    return Err(AppError::IoError(e));
                }
            };
            
            let path = entry.path();
            log::debug!("Processing entry: {}", path.display());
            
            // Skip the archive file itself if it exists
            if path == archive_path {
                log::debug!("Skipping archive file itself");
                continue;
            }
            
            // Get relative name
            let name = match path.strip_prefix(source_dir) {
                Ok(relative_path) => relative_path.to_string_lossy(),
                Err(e) => {
                    let error_msg = format!("Failed to get relative path: {}", e);
                    log::error!("{}", error_msg);
                    log_error_to_file(&error_msg);
                    return Err(AppError::ConfigError(e.to_string()));
                }
            };
            
            if path.is_file() {
                log::debug!("Adding file to archive: {}", name);
                
                // Start file in zip
                match zip.start_file(name.clone(), options) {
                    Ok(_) => log::debug!("Started file in zip: {}", name),
                    Err(e) => {
                        let error_msg = format!("Failed to start file in zip '{}': {}", name, e);
                        log::error!("{}", error_msg);
                        log_error_to_file(&error_msg);
                        return Err(AppError::ConfigError(e.to_string()));
                    }
                }
                
                // Open source file
                let mut f = match std::fs::File::open(&path) {
                    Ok(file) => file,
                    Err(e) => {
                        let error_msg = format!("Failed to open source file '{}': {}", path.display(), e);
                        log::error!("{}", error_msg);
                        log_error_to_file(&error_msg);
                        return Err(AppError::IoError(e));
                    }
                };
                
                // Copy file content to zip
                match std::io::copy(&mut f, &mut zip) {
                    Ok(bytes_copied) => {
                        log::debug!("Copied {} bytes from file: {}", bytes_copied, name);
                        file_count += 1;
                    },
                    Err(e) => {
                        let error_msg = format!("Failed to copy file '{}' to zip: {}", name, e);
                        log::error!("{}", error_msg);
                        log_error_to_file(&error_msg);
                        return Err(AppError::IoError(e));
                    }
                }
            }
        }
        
        // Step 6: Finish zip file
        log::info!("Step 6: Finishing zip file...");
        match zip.finish() {
            Ok(_) => log::info!("Zip file finished successfully"),
            Err(e) => {
                let error_msg = format!("Failed to finish zip file: {}", e);
                log::error!("{}", error_msg);
                log_error_to_file(&error_msg);
                return Err(AppError::ConfigError(e.to_string()));
            }
        }
        
        log::info!("=== CREATE_BACKUP_ARCHIVE SUCCESS ===");
        log::info!("Backup archive created with {} files", file_count);
        Ok(())
    }
    
    async fn extract_backup_archive(&self, archive_path: &std::path::Path, extract_dir: &std::path::Path) -> AppResult<()> {
        let file = std::fs::File::open(archive_path)
            .map_err(|e| AppError::IoError(e))?;
        
        let mut archive = ZipArchive::new(file)
            .map_err(|e| AppError::ConfigError(e.to_string()))?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| AppError::ConfigError(e.to_string()))?;
            let outpath = extract_dir.join(file.name());
            
            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)
                    .map_err(|e| AppError::IoError(e))?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)
                            .map_err(|e| AppError::IoError(e))?;
                    }
                }
                let mut outfile = std::fs::File::create(&outpath)
                    .map_err(|e| AppError::IoError(e))?;
                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| AppError::IoError(e))?;
            }
        }
        
        Ok(())
    }
    
    // FIXED IMPLEMENTATION - Proper Google Drive resumable upload with comprehensive error handling
    pub async fn upload_file_streaming(&self, file_path: &std::path::Path, file_name: &str) -> AppResult<String> {
        log::info!("=== FIXED RESUMABLE UPLOAD START ===");
        log::info!("File path: {}", file_path.display());
        log::info!("File name: {}", file_name);
        
        // Log error to file immediately
        let log_error_to_file = |error_msg: &str| {
            let error_log = format!(
                "[{}] DrivePlugin Upload Streaming Error:\nError: {}\nMethod: upload_file_streaming\nFile: {}\nPath: {}\n",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                error_msg,
                file_name,
                file_path.display()
            );
            if let Err(write_err) = std::fs::write("logs/app.log", error_log) {
                log::error!("Failed to write error log: {}", write_err);
            }
        };
        
        // Step 1: Get access token
        log::info!("Step 1: Getting access token...");
        let token = match self.get_access_token().await {
            Ok(token) => {
                log::info!("Got access token successfully (length: {})", token.len());
                token
            },
            Err(e) => {
                let error_msg = format!("Failed to get access token: {}", e);
                log::error!("{}", error_msg);
                log_error_to_file(&error_msg);
                return Err(e);
            }
        };
        
        // Step 2: Verify file exists and get size
        log::info!("Step 2: Verifying file exists and getting size...");
        if !file_path.exists() {
            let error_msg = format!("File does not exist: {}", file_path.display());
            log::error!("{}", error_msg);
            log_error_to_file(&error_msg);
            self.log_error_comprehensively("upload_file_streaming", &error_msg, None);
            return Err(AppError::ConfigError(error_msg));
        }
        log::info!("File exists: {}", file_path.display());
        
        let file_size = match std::fs::metadata(file_path) {
            Ok(metadata) => {
                let size = metadata.len();
                log::info!("File size: {} bytes ({:.2} MB)", size, size as f64 / (1024.0 * 1024.0));
                size
            },
            Err(e) => {
                let error_msg = format!("Failed to get file metadata: {}", e);
                log::error!("{}", error_msg);
                log_error_to_file(&error_msg);
                self.log_error_comprehensively("upload_file_streaming", &error_msg, Some(&format!("File: {}", file_path.display())));
                return Err(AppError::IoError(e));
            }
        };
        
        log::info!("File size: {} bytes ({:.2} MB)", file_size, file_size as f64 / (1024.0 * 1024.0));
        
        // STEP 1: Initiate resumable upload session with strict Google API compliance
        let metadata = serde_json::json!({
            "name": file_name,
            "mimeType": "application/zip"
        });
        
        let initiate_url = "https://www.googleapis.com/upload/drive/v3/files?uploadType=resumable";
        log::info!("Initiating resumable upload session at: {}", initiate_url);
        log::info!("Metadata: {}", metadata);
        
        // Build request with exact Google Drive API specifications
        let request_builder = self.http_client
            .post(initiate_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json; charset=UTF-8")
            .header("X-Upload-Content-Type", "application/zip")
            .header("X-Upload-Content-Length", file_size.to_string())
            .json(&metadata)
            .timeout(Duration::from_secs(120)); // Increase timeout for initiation
        
        // Log all request headers for debugging
        log::info!("Request headers:");
        log::info!("  Authorization: Bearer [{}...{}]", &token[..10.min(token.len())], &token[token.len().saturating_sub(10)..]);
        log::info!("  Content-Type: application/json; charset=UTF-8");
        log::info!("  X-Upload-Content-Type: application/zip");
        log::info!("  X-Upload-Content-Length: {}", file_size);
        
        // Step 3: Send initiation request
        log::info!("Step 3: Sending resumable upload initiation request...");
        let initiate_response = match request_builder.send().await {
            Ok(response) => {
                log::info!("Initiation request sent successfully");
                response
            },
            Err(e) => {
                let error_msg = format!("Failed to send resumable upload initiation request: {}", e);
                log::error!("{}", error_msg);
                log_error_to_file(&error_msg);
                self.log_error_comprehensively("upload_file_streaming", &error_msg, Some(&format!("URL: {}, File: {}, Size: {} bytes", initiate_url, file_name, file_size)));
                return Err(AppError::NetworkError(e.to_string()));
            }
        };
        
        let status = initiate_response.status();
        log::info!("Initiation response status: {}", status);
        
        // Log all response headers for debugging
        log::info!("Response headers:");
        for (name, value) in initiate_response.headers() {
            log::info!("  {}: {:?}", name, value);
        }
        
        // Step 4: Check response status
        log::info!("Step 4: Checking response status...");
        if !status.is_success() {
            let error_text = match initiate_response.text().await {
                Ok(text) => text,
                Err(e) => {
                    let error_msg = format!("Failed to read error response text: {}", e);
                    log::error!("{}", error_msg);
                    log_error_to_file(&error_msg);
                    format!("Unknown error (failed to read response: {})", e)
                }
            };
            
            let error_msg = format!("Failed to initiate resumable upload: {} - {}", status, error_text);
            log::error!("{}", error_msg);
            log_error_to_file(&error_msg);
            
            let details = format!("File: {}\nSize: {} bytes\nStatus: {}\nURL: {}\nResponse: {}", file_name, file_size, status, initiate_url, error_text);
            self.log_error_comprehensively("upload_file_streaming", &error_msg, Some(&details));
            
            // Specific handling for common errors
            match status.as_u16() {
                401 => return Err(AppError::ConfigError("Authentication failed - access token may be expired".to_string())),
                403 => return Err(AppError::ConfigError("Permission denied - check Google Drive API access".to_string())),
                429 => return Err(AppError::NetworkError("Rate limit exceeded - please try again later".to_string())),
                _ => return Err(AppError::NetworkError(format!("Failed to initiate upload: {} - {}", status, error_text))),
            }
        }
        log::info!("Initiation response status is successful: {}", status);
        
        // Step 5: Extract Location header
        log::info!("Step 5: Extracting Location header...");
        let session_uri = match initiate_response
            .headers()
            .get("location")
            .or_else(|| initiate_response.headers().get("Location"))
            .and_then(|v| v.to_str().ok()) {
            Some(uri) => {
                log::info!("Location header found: {}", uri);
                uri
            },
            None => {
                // Get all headers for debugging
                let all_headers: Vec<String> = initiate_response.headers()
                    .iter()
                    .map(|(name, value)| format!("{}: {:?}", name, value))
                    .collect();
                
                let error_msg = "CRITICAL: No Location header in resumable upload response";
                log::error!("{}", error_msg);
                log_error_to_file(error_msg);
                
                let debug_info = format!(
                    "Status: {}\nAll headers: [\n  {}\n]\nFile: {}\nSize: {} bytes", 
                    status, 
                    all_headers.join("\n  "),
                    file_name, 
                    file_size
                );
                
                self.log_error_comprehensively(
                    "upload_file_streaming", 
                    error_msg, 
                    Some(&debug_info)
                );
                
                return Err(AppError::ConfigError("No Location header in resumable upload response - this indicates a server-side issue or API change".to_string()));
            }
        };
        
        log::info!("✅ Got resumable session URI: {}", session_uri);
        log::info!("Session URI length: {} characters", session_uri.len());
        
        // Validate session URI format
        if !session_uri.starts_with("https://") || !session_uri.contains("upload_id=") {
            let error_msg = format!("Invalid session URI format: {}", session_uri);
            self.log_error_comprehensively("upload_file_streaming", &error_msg, Some(&format!("Expected format: https://...upload_id=...")));
            return Err(AppError::ConfigError(error_msg));
        }
        
        // STEP 2: Upload file data to session URI using proper chunking
        log::info!("Starting chunked upload to session URI...");
        
        // Step 6: Open file for reading
        log::info!("Step 6: Opening file for reading...");
        let mut file = match std::fs::File::open(file_path) {
            Ok(file) => {
                log::info!("File opened successfully for reading");
                file
            },
            Err(e) => {
                let error_msg = format!("Failed to open file for reading: {}", e);
                log::error!("{}", error_msg);
                log_error_to_file(&error_msg);
                self.log_error_comprehensively("upload_file_streaming", &error_msg, Some(&format!("File: {}", file_path.display())));
                return Err(AppError::IoError(e));
            }
        };
        
        // Use optimal chunk size (multiple of 256KB, but larger for better performance)
        const CHUNK_SIZE: usize = 8 * 1024 * 1024; // 8MB chunks for better performance
        let mut uploaded = 0u64;
        let mut chunk_count = 0u32;
        
        log::info!("Using chunk size: {} bytes ({:.1} MB)", CHUNK_SIZE, CHUNK_SIZE as f64 / (1024.0 * 1024.0));
        
        loop {
            // Read chunk from file
            let mut chunk = vec![0u8; CHUNK_SIZE];
            let bytes_read = {
                use std::io::Read;
                match file.read(&mut chunk) {
                    Ok(bytes) => {
                        log::debug!("Read {} bytes from file chunk {}", bytes, chunk_count + 1);
                        bytes
                    },
                    Err(e) => {
                        let error_msg = format!("Failed to read file chunk {}: {}", chunk_count + 1, e);
                        log::error!("{}", error_msg);
                        log_error_to_file(&error_msg);
                        self.log_error_comprehensively("upload_file_streaming", &error_msg, Some(&format!("File: {}, Uploaded: {} bytes", file_name, uploaded)));
                        return Err(AppError::IoError(e));
                    }
                }
            };
            
            if bytes_read == 0 {
                log::info!("✅ File reading complete - uploaded {} chunks totaling {} bytes", chunk_count, uploaded);
                break; // End of file
            }
            
            chunk.truncate(bytes_read);
            chunk_count += 1;
            let chunk_end = uploaded + bytes_read as u64 - 1;
            
            // Prepare Content-Range header
            let content_range = format!("bytes {}-{}/{}", uploaded, chunk_end, file_size);
            log::info!("📤 Uploading chunk {}: {} ({:.1}%)", chunk_count, content_range, (uploaded as f64 / file_size as f64) * 100.0);
            
            // Upload this chunk with retries
            let mut retry_count = 0;
            let max_retries = 3;
            
            loop {
                let upload_response = self.http_client
                    .put(session_uri)
                    .header("Content-Length", bytes_read.to_string())
                    .header("Content-Range", content_range.clone())
                    .body(chunk.clone())
                    .timeout(Duration::from_secs(300)) // 5 minutes per chunk
                    .send()
                    .await;
                
                match upload_response {
                    Ok(response) => {
                        let status = response.status();
                        log::info!("Upload chunk {} response status: {}", chunk_count, status);
                        
                        match status.as_u16() {
                            200 | 201 => {
                                // Upload complete!
                                log::info!("🎉 Upload completed successfully after {} chunks!", chunk_count);
                                let file_data: serde_json::Value = response.json().await
                                    .map_err(|e| {
                                        let error_msg = format!("Failed to parse final upload response: {}", e);
                                        self.log_error_comprehensively("upload_file_streaming", &error_msg, Some(&format!("File: {}", file_name)));
                                        AppError::NetworkError(e.to_string())
                                    })?;
                                
                                log::info!("Upload response JSON: {}", file_data);
                                
                                let file_id = file_data["id"].as_str()
                                    .map(|s| s.to_string())
                                    .ok_or_else(|| {
                                        self.log_error_comprehensively("upload_file_streaming", "No file ID in upload response", Some(&format!("File: {}, Response: {}", file_name, file_data)));
                                        AppError::ConfigError("No file ID in upload response".to_string())
                                    })?;
                                
                                log::info!("=== RESUMABLE UPLOAD SUCCESS ===");
                                log::info!("File uploaded successfully with ID: {}", file_id);
                                log::info!("Total chunks: {}, Total bytes: {}", chunk_count, uploaded + bytes_read as u64);
                                return Ok(file_id);
                            },
                            308 => {
                                // Continue upload - this is expected for chunks
                                log::info!("✅ Chunk {} uploaded successfully, continuing...", chunk_count);
                                uploaded += bytes_read as u64;
                                
                                // Log progress periodically
                                if chunk_count % 10 == 0 || uploaded == file_size {
                                    let progress_percent = (uploaded as f64 / file_size as f64) * 100.0;
                                    log::info!("📊 Upload progress: {}/{} bytes ({:.1}%) - {} chunks completed", uploaded, file_size, progress_percent, chunk_count);
                                }
                                break; // Exit retry loop, continue to next chunk
                            },
                            429 => {
                                // Rate limited - retry with backoff
                                retry_count += 1;
                                if retry_count > max_retries {
                                    let error_msg = "Rate limit exceeded - max retries reached";
                                    self.log_error_comprehensively("upload_file_streaming", error_msg, Some(&format!("Chunk: {}, File: {}", chunk_count, file_name)));
                                    return Err(AppError::NetworkError(error_msg.to_string()));
                                }
                                
                                let backoff_seconds = 2u64.pow(retry_count as u32);
                                log::warn!("⚠️  Rate limited - retrying chunk {} in {} seconds (attempt {}/{})", chunk_count, backoff_seconds, retry_count + 1, max_retries + 1);
                                tokio::time::sleep(Duration::from_secs(backoff_seconds)).await;
                                continue; // Retry this chunk
                            },
                            _ => {
                                // Other error
                                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                                let details = format!(
                                    "Chunk: {}\nFile: {}\nSize: {} bytes\nSession URI: {}\nStatus: {}\nResponse: {}\nUploaded: {} bytes", 
                                    chunk_count, file_name, file_size, session_uri, status, error_text, uploaded
                                );
                                self.log_error_comprehensively("upload_file_streaming", &format!("Failed to upload chunk {}: {} - {}", chunk_count, status, error_text), Some(&details));
                                return Err(AppError::NetworkError(format!("Failed to upload chunk {}: {} - {}", chunk_count, status, error_text)));
                            }
                        }
                    },
                    Err(e) => {
                        retry_count += 1;
                        if retry_count > max_retries {
                            let error_msg = format!("Failed to upload chunk {} after {} retries: {}", chunk_count, max_retries, e);
                            self.log_error_comprehensively("upload_file_streaming", &error_msg, Some(&format!("File: {}, Session URI: {}, Bytes: {}-{}", file_name, session_uri, uploaded, chunk_end)));
                            return Err(AppError::NetworkError(error_msg));
                        }
                        
                        let backoff_seconds = 2u64.pow(retry_count as u32);
                        log::warn!("⚠️  Network error uploading chunk {} - retrying in {} seconds (attempt {}/{}): {}", chunk_count, backoff_seconds, retry_count + 1, max_retries + 1, e);
                        tokio::time::sleep(Duration::from_secs(backoff_seconds)).await;
                        continue; // Retry this chunk
                    }
                }
            }
        }
        
        // Should not reach here
        let error_msg = "Upload loop completed without success or error response";
        self.log_error_comprehensively("upload_file_streaming", error_msg, Some(&format!("File: {}, Uploaded: {} bytes, Chunks: {}", file_name, uploaded, chunk_count)));
        Err(AppError::ConfigError(error_msg.to_string()))
    }
    
    async fn download_file(&self, file_id: &str) -> AppResult<std::path::PathBuf> {
        let token = self.get_access_token().await?;
        
        let url = format!("{}/files/{}?alt=media", GOOGLE_DRIVE_API_BASE, file_id);
        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(AppError::NetworkError(format!(
                "Failed to download file: {}",
                response.status()
            )));
        }
        
        let content = response.bytes().await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;
        
        let temp_path = std::env::temp_dir().join(format!("drive_download_{}.zip", file_id));
        std::fs::write(&temp_path, content)
            .map_err(|e| AppError::IoError(e))?;
        
        Ok(temp_path)
    }

    async fn get_access_token(&self) -> AppResult<String> {
        // First check if we already have a token
        {
            let token_guard = self.access_token.lock().await;
            if let Some(token) = token_guard.as_ref() {
                return Ok(token.clone());
            }
        }

        // Ensure client is initialized first (without holding token lock)
        {
            let client_guard = self.client.lock().await;
            if client_guard.is_none() {
                drop(client_guard);
                self.initialize_client().await?;
            }
        }

        // Try to load stored token
        if let Ok(stored_token) = Self::load_tokens() {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if stored_token.expires_at > current_time {
                // Store the token and return it
                {
                    let mut token_guard = self.access_token.lock().await;
                    *token_guard = Some(stored_token.access_token.clone());
                }
                return Ok(stored_token.access_token);
            } else {
                // Token expired, try to refresh it
                log::info!("Access token expired, attempting to refresh...");
                if let Ok(new_token) = self.refresh_access_token(&stored_token.refresh_token).await {
                    // Store the new token and return it
                    {
                        let mut token_guard = self.access_token.lock().await;
                        *token_guard = Some(new_token.clone());
                    }
                    return Ok(new_token);
                }
            }
        }

        Err(AppError::ConfigError("No valid access token available".to_string()))
    }

    async fn refresh_access_token(&self, refresh_token: &str) -> AppResult<String> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            AppError::ConfigError("Client not initialized".to_string())
        })?;

        let token_res = client
            .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to refresh token: {}", e)))?;

        let access_token = token_res.access_token().secret().to_string();
        let refresh_token = token_res.refresh_token()
            .map(|rt| rt.secret().to_string())
            .unwrap_or_else(|| refresh_token.to_string());
        
        let expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() + token_res.expires_in().unwrap().as_secs();

        drop(client_guard); // Release lock before async operation

        // Store the new tokens
        self.store_tokens(&refresh_token, &access_token, expires_at)?;
        
        Ok(access_token)
    }

    fn store_tokens(&self, refresh_token: &str, access_token: &str, expires_at: u64) -> AppResult<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir).map_err(AppError::IoError)?;
        fs::write(
            dir.join("drive_token.json"),
            serde_json::to_vec_pretty(&StoredToken {
                refresh_token: refresh_token.to_string(),
                access_token: access_token.to_string(),
                expires_at,
            })
            .unwrap(),
        )
        .map_err(AppError::IoError)?;
        Ok(())
    }

    fn load_tokens() -> AppResult<StoredToken> {
        let path = Self::config_dir().join("drive_token.json");
        if path.exists() {
            let bytes = fs::read(path).map_err(AppError::IoError)?;
            let token: StoredToken =
                serde_json::from_slice(&bytes).map_err(AppError::SerializationError)?;
            return Ok(token);
        }
        Err(AppError::ConfigError("No stored tokens found".to_string()))
    }

    fn load_credentials() -> AppResult<(String, String)> {
        /* 1. ENV vars --------------------------------------------------- */
        if let (Ok(id), Ok(secret)) = (
            std::env::var("GOOGLE_CLIENT_ID"),
            std::env::var("GOOGLE_CLIENT_SECRET"),
        ) {
            if !id.is_empty() && !secret.is_empty() {
                return Ok((id, secret));
            }
        }

        /* 2. Config file ------------------------------------------------ */
        let path = Self::config_dir().join("google.json");
        if path.exists() {
            let bytes = fs::read(path).map_err(AppError::IoError)?;
            let creds: StoredCreds =
                serde_json::from_slice(&bytes).map_err(AppError::SerializationError)?;
            return Ok((creds.client_id, creds.client_secret));
        }

        Err(AppError::ConfigError(
            "Google credentials not found".to_string(),
        ))
    }

    fn config_dir() -> PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata).join("reStrikeVTA")
        } else {
            PathBuf::from(".")
        }
    }
}

static DRIVE_PLUGIN: Lazy<DrivePlugin> = Lazy::new(DrivePlugin::new);
pub fn drive_plugin() -> &'static DrivePlugin {
    &DRIVE_PLUGIN
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the drive plugin
    let _plugin = drive_plugin();
    println!("✅ Google Drive plugin initialized");
    Ok(())
} 