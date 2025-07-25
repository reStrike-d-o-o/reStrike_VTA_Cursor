use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use reqwest::Client;
use zip::{ZipArchive, write::FileOptions};


use crate::types::{AppError, AppResult};

const REDIRECT_URI: &str = "urn:ietf:wg:oauth:2.0:oob";
const GOOGLE_DRIVE_API_BASE: &str = "https://www.googleapis.com/drive/v3";

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
        Self { 
            client: Arc::new(Mutex::new(None)),
            http_client: Client::new(),
            access_token: Arc::new(Mutex::new(None)),
        }
    }

    // Helper function for comprehensive error logging
    fn log_error_to_file(&self, context: &str, error: &str, details: Option<&str>) {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let mut log_entry = format!("[{}] Google Drive Error - {}: {}\n", timestamp, context, error);
        
        if let Some(detail_info) = details {
            log_entry.push_str(&format!("Details: {}\n", detail_info));
        }
        
        log_entry.push_str("---\n");
        
        // Log to console as well
        log::error!("{}: {}", context, error);
        if let Some(detail_info) = details {
            log::error!("Details: {}", detail_info);
        }
        
        // Write to main app log file
        let log_file_path = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("app_errors.log");
        
        if let Err(write_err) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)
            .and_then(|mut file| {
                use std::io::Write;
                file.write_all(log_entry.as_bytes())
            }) {
            log::error!("Failed to write to error log file {}: {}", log_file_path.display(), write_err);
        }
    }

    pub async fn auth_url(&self) -> AppResult<(String, String)> {
        let client_guard = self.client.lock().await;
        
        // Try to get existing client or create new one
        if client_guard.is_none() {
            self.initialize_client().await?;
        }
        
        let client = client_guard.as_ref().unwrap();
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
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            self.log_error_to_file("list_all_files", &format!("API error: {} - {}", status, error_text), Some(&error_text));
            return Err(AppError::NetworkError(format!("API error: {} - {}", status, error_text)));
        }

        let file_list: GoogleDriveFileList = response.json().await
            .map_err(|e| {
                self.log_error_to_file(
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
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            self.log_error_to_file("list_files", &format!("API error: {} - {}", status, error_text), Some(&error_text));
            return Err(AppError::NetworkError(format!("API error: {} - {}", status, error_text)));
        }

        let file_list: GoogleDriveFileList = response.json().await
            .map_err(|e| {
                self.log_error_to_file(
                    "list_files", 
                    &format!("Failed to parse response JSON: {}", e), 
                    Some("This might be due to unexpected field names from Google Drive API")
                );
                AppError::NetworkError(e.to_string())
            })?;

        Ok(file_list.files)
    }





    pub async fn upload_backup_archive(&self) -> AppResult<String> {
        log::info!("=== UPLOAD_BACKUP_ARCHIVE START ===");
        
        // Create backup archive
        let backup_dir = std::env::current_dir()?.join("backups");
        log::info!("Backup directory: {}", backup_dir.display());
        
        // Ensure backup directory exists
        if !backup_dir.exists() {
            log::info!("Creating backup directory...");
            std::fs::create_dir_all(&backup_dir)
                .map_err(|e| {
                    self.log_error_to_file(
                        "upload_backup_archive", 
                        &format!("Failed to create backup directory: {}", e), 
                        Some(&format!("Directory: {}", backup_dir.display()))
                    );
                    AppError::IoError(e)
                })?;
        }
        
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let archive_name = format!("reStrikeVTA_backup_{}.zip", timestamp);
        let archive_path = backup_dir.join(&archive_name);
        
        log::info!("Creating backup archive: {}", archive_path.display());
        
        // Create zip archive from backups directory
        log::info!("Calling create_backup_archive...");
        self.create_backup_archive(&backup_dir, &archive_path).await?;
        log::info!("Backup archive created successfully");
        
        log::info!("Backup archive created, uploading to Google Drive...");
        
        // Upload to Google Drive
        log::info!("Calling upload_file...");
        let _file_id = match self.upload_file(&archive_path, &archive_name).await {
            Ok(file_id) => {
                log::info!("Upload completed successfully");
                file_id
            },
            Err(e) => {
                // Use comprehensive error logging
                let details = format!(
                    "Archive Path: {}\nArchive Name: {}\nArchive Size: {} bytes",
                    archive_path.display(),
                    archive_name,
                    archive_path.metadata().map(|m| m.len()).unwrap_or(0)
                );
                
                self.log_error_to_file(
                    "upload_backup_archive", 
                    &format!("Backup upload failed: {}", e), 
                    Some(&details)
                );
                
                return Err(e);
            }
        };
        
        // Clean up local archive
        log::info!("Cleaning up local archive...");
        if let Err(e) = std::fs::remove_file(&archive_path) {
            log::warn!("Failed to clean up local archive: {}", e);
        } else {
            log::info!("Local archive cleaned up successfully");
        }
        
        log::info!("=== UPLOAD_BACKUP_ARCHIVE SUCCESS ===");
        Ok(format!("Backup archive '{}' uploaded successfully", archive_name))
    }
    
    pub async fn download_backup_archive(&self, file_id: &str) -> AppResult<String> {
        // Download file from Google Drive
        let download_path = self.download_file(file_id).await?;
        
        // Extract archive to backups directory
        let backup_dir = std::env::current_dir()?.join("backups");
        self.extract_backup_archive(&download_path, &backup_dir).await?;
        
        // Clean up downloaded archive
        let _ = std::fs::remove_file(&download_path);
        
        Ok("Backup archive downloaded and extracted successfully".to_string())
    }
    
    pub async fn restore_from_archive(&self, file_id: &str) -> AppResult<String> {
        // Download and extract archive
        let download_path = self.download_file(file_id).await?;
        let backup_dir = std::env::current_dir()?.join("backups");
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
        
        // Ensure client is initialized first
        {
            let client_guard = self.client.lock().await;
            if client_guard.is_none() {
                drop(client_guard);
                log::info!("is_connected: Client not initialized, attempting to initialize");
                if let Err(e) = self.initialize_client().await {
                    log::warn!("is_connected: Failed to initialize client: {:?}", e);
                    return Ok(false);
                }
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
        log::info!("Creating backup archive from: {}", source_dir.display());
        
        // Ensure source directory exists
        if !source_dir.exists() {
            return Err(AppError::ConfigError(format!("Source directory does not exist: {}", source_dir.display())));
        }
        
        let file = std::fs::File::create(archive_path)
            .map_err(|e| AppError::IoError(e))?;
        
        let mut zip = zip::ZipWriter::new(file);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);
        
        let mut file_count = 0;
        for entry in std::fs::read_dir(source_dir)
            .map_err(|e| AppError::IoError(e))? {
            let entry = entry.map_err(|e| AppError::IoError(e))?;
            let path = entry.path();
            
            // Skip the archive file itself if it exists
            if path == archive_path {
                continue;
            }
            
            let name = path.strip_prefix(source_dir)
                .map_err(|e| AppError::ConfigError(e.to_string()))?
                .to_string_lossy();
            
            if path.is_file() {
                log::debug!("Adding file to archive: {}", name);
                zip.start_file(name, options)
                    .map_err(|e| AppError::ConfigError(e.to_string()))?;
                let mut f = std::fs::File::open(path)
                    .map_err(|e| AppError::IoError(e))?;
                std::io::copy(&mut f, &mut zip)
                    .map_err(|e| AppError::IoError(e))?;
                file_count += 1;
            }
        }
        
        zip.finish()
            .map_err(|e| AppError::ConfigError(e.to_string()))?;
        
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
    
    async fn upload_file(&self, file_path: &std::path::Path, file_name: &str) -> AppResult<String> {
        log::info!("=== UPLOAD_FILE START ===");
        log::info!("File path: {}", file_path.display());
        log::info!("File name: {}", file_name);
        
        let token = self.get_access_token().await?;
        log::info!("Got access token successfully");
        
        // Check if file exists
        if !file_path.exists() {
            let error_msg = format!("File does not exist: {}", file_path.display());
            self.log_error_to_file("upload_file", &error_msg, None);
            return Err(AppError::ConfigError(error_msg));
        }
        
        // Read file content
        log::info!("Reading file content...");
        let file_content = std::fs::read(file_path)
            .map_err(|e| {
                let error_msg = format!("Failed to read file: {}", e);
                self.log_error_to_file(
                    "upload_file", 
                    &error_msg, 
                    Some(&format!("File path: {}", file_path.display()))
                );
                AppError::IoError(e)
            })?;
        let file_size = file_content.len();
        log::info!("File content read successfully, size: {} bytes", file_size);
        
        // Use resumable upload (recommended for files)
        log::info!("Starting resumable upload session...");
        
        // Step 1: Initiate resumable upload session
        let metadata = serde_json::json!({
            "name": file_name,
            "mimeType": "application/zip"
        });
        
        let initiate_url = format!("{}/files?uploadType=resumable", GOOGLE_DRIVE_API_BASE);
        log::info!("Initiate URL: {}", initiate_url);
        
        let initiate_response = self.http_client
            .post(&initiate_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json; charset=UTF-8")
            .header("X-Upload-Content-Type", "application/zip")
            .header("X-Upload-Content-Length", file_size.to_string())
            .json(&metadata)
            .send()
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to initiate resumable upload: {}", e);
                self.log_error_to_file(
                    "upload_file", 
                    &error_msg, 
                    Some(&format!("File: {}, Size: {} bytes", file_name, file_size))
                );
                AppError::NetworkError(e.to_string())
            })?;
        
        if !initiate_response.status().is_success() {
            let status = initiate_response.status();
            let error_text = initiate_response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            let details = format!(
                "File: {}\nSize: {} bytes\nStatus: {}\nResponse: {}",
                file_name, file_size, status, error_text
            );
            self.log_error_to_file("upload_file", &format!("Failed to initiate resumable upload: {} - {}", status, error_text), Some(&details));
            return Err(AppError::NetworkError(format!(
                "Failed to initiate upload: {} - {}",
                status, error_text
            )));
        }
        
        // Get the resumable session URI
        let session_uri = initiate_response
            .headers()
            .get("Location")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                self.log_error_to_file("upload_file", "No Location header in resumable upload response", Some(&format!("File: {}", file_name)));
                AppError::ConfigError("No Location header in resumable upload response".to_string())
            })?;
        
        log::info!("Got resumable session URI: {}", session_uri);
        
        // Step 2: Upload the file content
        log::info!("Uploading file content to session URI...");
        let upload_response = self.http_client
            .put(session_uri)
            .header("Content-Type", "application/zip")
            .header("Content-Length", file_size.to_string())
            .body(file_content)
            .send()
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to upload file content: {}", e);
                self.log_error_to_file(
                    "upload_file", 
                    &error_msg, 
                    Some(&format!("File: {}, Session URI: {}", file_name, session_uri))
                );
                AppError::NetworkError(e.to_string())
            })?;
        
        log::info!("Upload response status: {}", upload_response.status());
        
        if !upload_response.status().is_success() {
            let status = upload_response.status();
            let error_text = upload_response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            let details = format!(
                "File: {}\nSize: {} bytes\nSession URI: {}\nStatus: {}\nResponse: {}",
                file_name, file_size, session_uri, status, error_text
            );
            self.log_error_to_file("upload_file", &format!("Failed to upload file content: {} - {}", status, error_text), Some(&details));
            return Err(AppError::NetworkError(format!(
                "Failed to upload file: {} - {}",
                status, error_text
            )));
        }
        
        // Parse response to get file ID
        log::info!("Parsing upload response...");
        let file_data: serde_json::Value = upload_response.json().await
            .map_err(|e| {
                let error_msg = format!("Failed to parse upload response: {}", e);
                self.log_error_to_file("upload_file", &error_msg, Some(&format!("File: {}", file_name)));
                AppError::NetworkError(e.to_string())
            })?;
        
        log::info!("Upload response JSON: {}", file_data);
        
        let file_id = file_data["id"].as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                self.log_error_to_file("upload_file", "No file ID in upload response", Some(&format!("File: {}, Response: {}", file_name, file_data)));
                AppError::ConfigError("No file ID in upload response".to_string())
            })?;
        
        log::info!("=== UPLOAD_FILE SUCCESS ===");
        log::info!("File uploaded successfully with ID: {}", file_id);
        Ok(file_id)
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