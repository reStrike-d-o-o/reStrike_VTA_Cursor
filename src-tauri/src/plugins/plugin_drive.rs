use crate::database::connection::DatabaseConnection;
use crate::types::{AppError, AppResult};
use chrono::{DateTime, Utc};
use drive_v3::objects;
use drive_v3::{AccessToken, ClientSecrets, Credentials as DriveCredentials, Drive};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, TokenResponse, TokenUrl,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex as StdMutex},
    time::SystemTime,
};
use tokio::sync::Mutex;

const REDIRECT_URI: &str = "urn:ietf:wg:oauth:2.0:oob";
const GOOGLE_AUTH_URI: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URI: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_CERT_URI: &str = "https://www.googleapis.com/oauth2/v1/certs";
const DRIVE_SCOPES: &[&str] = &["https://www.googleapis.com/auth/drive"];

#[derive(Debug, Serialize, Deserialize)]
struct StoredCreds {
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleDriveFile {
    pub id: String,
    pub name: String,
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    pub size: Option<String>,
    #[serde(rename = "createdTime")]
    pub created_time: String,
    #[serde(rename = "modifiedTime")]
    pub modified_time: String,
}

pub struct DrivePlugin {
    client: Arc<Mutex<Option<BasicClient>>>,
    credentials_cache: Arc<StdMutex<Option<DriveCredentials>>>,
}

impl Default for DrivePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl DrivePlugin {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            credentials_cache: Arc::new(StdMutex::new(None)),
        }
    }

    fn config_dir() -> PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata).join("reStrikeVTA")
        } else {
            PathBuf::from(".")
        }
    }

    fn stored_creds_path() -> PathBuf {
        Self::config_dir().join("google.json")
    }

    fn credentials_store_path() -> PathBuf {
        Self::config_dir().join("drive_credentials.json")
    }

    fn ensure_backup_directory() -> AppResult<PathBuf> {
        let dir = DatabaseConnection::get_backup_directory().map_err(AppError::from)?;
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(AppError::IoError)?;
        }
        Ok(dir)
    }

    fn latest_local_backup() -> AppResult<Option<(PathBuf, SystemTime)>> {
        let dir = Self::ensure_backup_directory()?;
        let mut latest: Option<(PathBuf, SystemTime)> = None;
        for entry in fs::read_dir(&dir).map_err(AppError::IoError)? {
            let entry = entry.map_err(AppError::IoError)?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path
                .extension()
                .and_then(|e| e.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("db"))
                != Some(true)
            {
                continue;
            }
            let metadata = entry.metadata().map_err(AppError::IoError)?;
            let modified = metadata.modified().map_err(AppError::IoError)?;
            match &latest {
                Some((_, current)) if *current >= modified => {}
                _ => latest = Some((path.clone(), modified)),
            }
        }
        Ok(latest)
    }

    fn load_stored_creds() -> AppResult<StoredCreds> {
        if let (Ok(id), Ok(secret)) = (
            std::env::var("GOOGLE_CLIENT_ID"),
            std::env::var("GOOGLE_CLIENT_SECRET"),
        ) {
            if !id.trim().is_empty() && !secret.trim().is_empty() {
                return Ok(StoredCreds {
                    client_id: id,
                    client_secret: secret,
                });
            }
        }

        let path = Self::stored_creds_path();
        if path.exists() {
            let bytes = fs::read(path).map_err(AppError::IoError)?;
            let creds = serde_json::from_slice::<StoredCreds>(&bytes)
                .map_err(AppError::SerializationError)?;
            return Ok(creds);
        }

        Err(AppError::ConfigError(
            "Google Drive credentials not found. Please save credentials first.".to_string(),
        ))
    }

    fn build_basic_client(creds: &StoredCreds) -> AppResult<BasicClient> {
        let client = BasicClient::new(
            ClientId::new(creds.client_id.clone()),
            Some(ClientSecret::new(creds.client_secret.clone())),
            AuthUrl::new(GOOGLE_AUTH_URI.to_string())
                .map_err(|e| AppError::ConfigError(format!("Invalid auth URL: {e}")))?,
            Some(
                TokenUrl::new(GOOGLE_TOKEN_URI.to_string())
                    .map_err(|e| AppError::ConfigError(format!("Invalid token URL: {e}")))?,
            ),
        )
        .set_redirect_uri(
            RedirectUrl::new(REDIRECT_URI.to_string())
                .map_err(|e| AppError::ConfigError(format!("Invalid redirect URI: {e}")))?,
        );

        Ok(client)
    }

    fn client_secrets_from(creds: &StoredCreds) -> ClientSecrets {
        ClientSecrets {
            client_id: creds.client_id.clone(),
            project_id: "reStrikeVTA".to_string(),
            auth_uri: GOOGLE_AUTH_URI.to_string(),
            token_uri: GOOGLE_TOKEN_URI.to_string(),
            auth_provider_x509_cert_url: GOOGLE_CERT_URI.to_string(),
            client_secret: creds.client_secret.clone(),
            redirect_uris: vec![
                REDIRECT_URI.to_string(),
                "http://localhost".to_string(),
                "http://127.0.0.1".to_string(),
            ],
        }
    }

    fn load_drive_credentials(path: &Path) -> AppResult<DriveCredentials> {
        if !path.exists() {
            return Err(AppError::ConfigError(
                "Google Drive access token not found. Complete the authorization flow.".to_string(),
            ));
        }
        let scopes: Vec<&str> = DRIVE_SCOPES.to_vec();
        DriveCredentials::from_file(path, &scopes).map_err(|e| {
            AppError::ConfigError(format!(
                "Failed to read stored Google Drive credentials: {e}"
            ))
        })
    }

    fn map_drive_error(context: &str, err: drive_v3::Error) -> AppError {
        AppError::NetworkError(format!("Drive API {context} error: {err}"))
    }

    fn parse_quota(field: Option<String>) -> AppResult<u64> {
        let value = field.unwrap_or_else(|| "0".to_string());
        value
            .parse::<u64>()
            .map_err(|e| AppError::ConfigError(format!("Failed to parse quota value: {e}")))
    }

    fn convert_drive_file(file: objects::File) -> Option<GoogleDriveFile> {
        let id = file.id?;
        let name = file.name.unwrap_or_else(|| "Unnamed".to_string());
        let mime_type = file.mime_type;
        let size = file.size;
        let created_time = file.created_time.unwrap_or_default();
        let modified_time = file.modified_time.unwrap_or_default();
        Some(GoogleDriveFile {
            id,
            name,
            mime_type,
            size,
            created_time,
            modified_time,
        })
    }

    fn fetch_files(drive: &Drive, query: Option<&str>) -> AppResult<Vec<objects::File>> {
        let mut collected = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = drive
                .files
                .list()
                .supports_all_drives(true)
                .include_items_from_all_drives(true)
                .page_size(500);

            if let Some(q) = query {
                request = request.q(q);
            }

            if let Some(token) = &next_token {
                request = request.page_token(token);
            }

            let response = request
                .execute()
                .map_err(|e| Self::map_drive_error("files.list", e))?;

            if let Some(mut files) = response.files {
                collected.append(&mut files);
            }

            if let Some(token) = response.next_page_token {
                next_token = Some(token);
            } else {
                break;
            }
        }

        Ok(collected)
    }

    async fn ensure_client(&self) -> AppResult<()> {
        let mut guard = self.client.lock().await;
        if guard.is_some() {
            return Ok(());
        }

        let creds = Self::load_stored_creds()?;
        let client = Self::build_basic_client(&creds)?;
        *guard = Some(client);
        Ok(())
    }

    async fn with_drive<F, T>(&self, context: &'static str, operation: F) -> AppResult<T>
    where
        F: FnOnce(Drive) -> AppResult<T> + Send + 'static,
        T: Send + 'static,
    {
        let credentials_path = Self::credentials_store_path();
        let cache = self.credentials_cache.clone();

        tokio::task::spawn_blocking(move || {
            let mut cache_guard = cache.lock().unwrap();
            let mut credentials = if let Some(existing) = cache_guard.clone() {
                existing
            } else {
                Self::load_drive_credentials(&credentials_path)?
            };

            if !credentials.are_valid() {
                credentials.refresh().map_err(|e| {
                    AppError::NetworkError(format!("Failed to refresh Drive token: {e}"))
                })?;
                credentials.store(&credentials_path).map_err(|e| {
                    AppError::ConfigError(format!("Failed to persist refreshed Drive token: {e}"))
                })?;
            }

            *cache_guard = Some(credentials.clone());
            let drive = Drive::new(&credentials);
            operation(drive)
        })
        .await
        .map_err(|e| AppError::ConfigError(format!("{context} worker join error: {e}")))?
    }

    async fn download_file(&self, file_id: &str) -> AppResult<PathBuf> {
        let file_id = file_id.to_string();
        let (bytes, name) = self
            .with_drive("download_file", move |drive| {
                let metadata = drive
                    .files
                    .get(&file_id)
                    .fields("id,name")
                    .supports_all_drives(true)
                    .execute()
                    .map_err(|e| Self::map_drive_error("files.get", e))?;

                let file_name = metadata
                    .name
                    .unwrap_or_else(|| format!("drive_file_{file_id}"));

                let content = drive
                    .files
                    .get_media(&file_id)
                    .supports_all_drives(true)
                    .execute()
                    .map_err(|e| Self::map_drive_error("files.get_media", e))?;

                Ok::<_, AppError>((content, file_name))
            })
            .await?;

        let temp_path = std::env::temp_dir().join(format!("gdrive_{name}"));
        let mut file = fs::File::create(&temp_path).map_err(AppError::IoError)?;
        file.write_all(&bytes).map_err(AppError::IoError)?;
        Ok(temp_path)
    }

    pub async fn get_quota(&self) -> AppResult<(u64, u64, u64)> {
        self.with_drive("get_quota", |drive| {
            let about = drive
                .about
                .get()
                .fields("storageQuota")
                .execute()
                .map_err(|e| Self::map_drive_error("about.get", e))?;

            let quota = about.storage_quota.ok_or_else(|| {
                AppError::ConfigError("Drive API response missing storage quota".to_string())
            })?;

            let limit = Self::parse_quota(quota.limit)?;
            let usage = Self::parse_quota(quota.usage)?;
            let usage_in_drive = Self::parse_quota(quota.usage_in_drive)?;
            Ok((limit, usage, usage_in_drive))
        })
        .await
    }

    pub async fn create_folder(&self, name: &str, parent_id: Option<&str>) -> AppResult<String> {
        let name = name.to_string();
        let parent = parent_id.map(|p| p.to_string());

        self.with_drive("create_folder", move |drive| {
            let mut metadata = objects::File::default();
            metadata.name = Some(name);
            metadata.mime_type = Some("application/vnd.google-apps.folder".to_string());
            if let Some(parent_id) = parent {
                metadata.parents = Some(vec![parent_id]);
            }

            let file = drive
                .files
                .create()
                .supports_all_drives(true)
                .metadata(&metadata)
                .execute()
                .map_err(|e| Self::map_drive_error("files.create", e))?;

            file.id.ok_or_else(|| {
                AppError::NetworkError("Drive API did not return folder id".to_string())
            })
        })
        .await
    }

    pub async fn list_children(&self, parent_id: Option<&str>) -> AppResult<Vec<GoogleDriveFile>> {
        let query = parent_id.map(|id| format!("'{id}' in parents and trashed=false"));

        self.with_drive("list_children", move |drive| {
            let files = Self::fetch_files(&drive, query.as_deref())?;
            let mapped = files
                .into_iter()
                .filter_map(Self::convert_drive_file)
                .collect();
            Ok(mapped)
        })
        .await
    }

    pub async fn upload_file_streaming_to_folder(
        &self,
        path: &Path,
        file_name: &str,
        mime_type: &str,
        parent_id: Option<&str>,
    ) -> AppResult<String> {
        self.upload_file_streaming(path, file_name, mime_type, parent_id)
            .await
    }

    pub async fn auth_url(&self) -> AppResult<(String, String)> {
        self.ensure_client().await?;
        let guard = self.client.lock().await;
        let client = guard
            .as_ref()
            .ok_or_else(|| AppError::ConfigError("OAuth client not initialized".to_string()))?;

        let mut auth = client.authorize_url(CsrfToken::new_random);
        for scope in DRIVE_SCOPES {
            auth = auth.add_scope(oauth2::Scope::new(scope.to_string()));
        }
        let (url, csrf) = auth.url();
        Ok((url.to_string(), csrf.secret().to_string()))
    }

    pub async fn exchange_code(&self, code: String) -> AppResult<()> {
        self.ensure_client().await?;
        let guard = self.client.lock().await;
        let client = guard
            .as_ref()
            .ok_or_else(|| AppError::ConfigError("OAuth client not initialized".to_string()))?;

        let token_res = client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to exchange auth code: {e}")))?;

        let refresh = token_res.refresh_token().ok_or_else(|| {
            AppError::ConfigError("Google response did not include a refresh token".to_string())
        })?;

        let scopes = token_res
            .scopes()
            .map(|set| {
                set.iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_else(|| DRIVE_SCOPES.join(" "));

        let token_type = token_res.token_type().as_ref().to_string();

        let expires_in = token_res
            .expires_in()
            .map(|d| d.as_secs() as i128)
            .unwrap_or(3600);

        let access_token = AccessToken {
            access_token: token_res.access_token().secret().to_string(),
            expires_in,
            refresh_token: refresh.secret().to_string(),
            scope: scopes,
            token_type,
        };

        let stored = Self::load_stored_creds()?;
        let secrets = Self::client_secrets_from(&stored);
        let credentials = DriveCredentials::new(&secrets, &access_token);

        let path = Self::credentials_store_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(AppError::IoError)?;
        }
        credentials
            .store(&path)
            .map_err(|e| AppError::ConfigError(format!("Failed to store credentials: {e}")))?;

        let mut cache = self.credentials_cache.lock().unwrap();
        *cache = Some(credentials);
        Ok(())
    }

    pub async fn save_credentials(&self, id: String, secret: String) -> AppResult<()> {
        let creds = StoredCreds {
            client_id: id,
            client_secret: secret,
        };

        let path = Self::stored_creds_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(AppError::IoError)?;
        }
        let bytes = serde_json::to_vec_pretty(&creds).map_err(AppError::SerializationError)?;
        fs::write(&path, bytes).map_err(AppError::IoError)?;

        // Invalidate cached credentials so the user must re-authorize.
        let mut cache = self.credentials_cache.lock().unwrap();
        *cache = None;
        let cred_path = Self::credentials_store_path();
        if cred_path.exists() {
            let _ = fs::remove_file(&cred_path);
        }
        Ok(())
    }

    pub async fn list_all_files(&self) -> AppResult<Vec<GoogleDriveFile>> {
        self.with_drive("list_all_files", |drive| {
            let files = Self::fetch_files(&drive, Some("trashed=false"))?;
            let mapped = files
                .into_iter()
                .filter_map(Self::convert_drive_file)
                .collect();
            Ok(mapped)
        })
        .await
    }

    pub async fn list_files(&self) -> AppResult<Vec<GoogleDriveFile>> {
        self.list_all_files().await
    }

    pub async fn upload_backup_archive(&self) -> AppResult<String> {
        {
            let connection = DatabaseConnection::new().map_err(AppError::from)?;
            connection.create_backup(None).map_err(AppError::from)?;
        }

        let (latest_path, latest_modified) = Self::latest_local_backup()?.ok_or_else(|| {
            AppError::ConfigError("No local database backups found to upload".to_string())
        })?;

        let local_timestamp: DateTime<Utc> = DateTime::<Utc>::from(latest_modified);

        let remote_files = match self.list_files().await {
            Ok(files) => files,
            Err(err) => {
                log::warn!(
                    "Failed to list existing Drive backups ({err}). Proceeding with upload."
                );
                Vec::new()
            }
        };

        if let Some(remote_time) = remote_files
            .iter()
            .filter_map(|f| DateTime::parse_from_rfc3339(&f.modified_time).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .max()
        {
            if remote_time >= local_timestamp {
                return Ok(format!(
                    "Remote backup ({}) is newer or equal to local backup ({}). Skipping upload.",
                    remote_time.format("%Y-%m-%d %H:%M:%S"),
                    local_timestamp.format("%Y-%m-%d %H:%M:%S")
                ));
            }
        }

        let file_name = latest_path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                AppError::ConfigError("Failed to determine backup file name".to_string())
            })?
            .to_string();

        let file_id = self
            .upload_file_streaming(&latest_path, &file_name, "application/octet-stream", None)
            .await?;

        Ok(format!(
            "Uploaded backup {file_name} to Google Drive (file id: {file_id})"
        ))
    }

    pub async fn download_backup_archive(&self, file_id: &str) -> AppResult<String> {
        let download_path = self.download_file(file_id).await?;
        let backup_dir = Self::ensure_backup_directory()?;
        let file_name = download_path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| AppError::ConfigError("Downloaded file has no valid name".into()))?;
        let target_path = backup_dir.join(file_name);
        fs::copy(&download_path, &target_path).map_err(AppError::IoError)?;
        let _ = fs::remove_file(&download_path);
        Ok(format!("Backup saved locally: {}", target_path.display()))
    }

    pub async fn restore_from_archive(&self, file_id: &str) -> AppResult<String> {
        let download_path = self.download_file(file_id).await?;
        let file_name = download_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("remote_backup.db")
            .to_string();
        let backup_dir = Self::ensure_backup_directory()?;
        let persisted_path = backup_dir.join(&file_name);
        fs::copy(&download_path, &persisted_path).map_err(AppError::IoError)?;

        let connection = DatabaseConnection::new().map_err(AppError::from)?;
        connection
            .restore_from_backup(&persisted_path)
            .await
            .map_err(AppError::from)?;

        let _ = fs::remove_file(&download_path);
        Ok(format!(
            "Database restored from {}",
            persisted_path.display()
        ))
    }

    pub async fn delete_backup_archive(&self, file_id: &str) -> AppResult<()> {
        let file_id = file_id.to_string();
        self.with_drive("delete_backup_archive", move |drive| {
            drive
                .files
                .delete(&file_id)
                .supports_all_drives(true)
                .execute()
                .map_err(|e| Self::map_drive_error("files.delete", e))?;
            Ok(())
        })
        .await
    }

    pub async fn is_connected(&self) -> AppResult<bool> {
        let credentials_path = Self::credentials_store_path();
        if !credentials_path.exists() {
            return Ok(false);
        }

        let cache = self.credentials_cache.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut cache_guard = cache.lock().unwrap();
            let mut credentials = if let Some(existing) = cache_guard.clone() {
                existing
            } else {
                Self::load_drive_credentials(&credentials_path)?
            };

            if !credentials.are_valid() {
                credentials.refresh().map_err(|e| {
                    AppError::NetworkError(format!("Failed to refresh Drive token: {e}"))
                })?;
                credentials.store(&credentials_path).map_err(|e| {
                    AppError::ConfigError(format!("Failed to persist refreshed Drive token: {e}"))
                })?;
            }

            *cache_guard = Some(credentials);
            Ok::<bool, AppError>(true)
        })
        .await;

        match result {
            Ok(Ok(flag)) => Ok(flag),
            Ok(Err(err)) => {
                log::warn!("Drive connection check failed: {err}");
                Ok(false)
            }
            Err(join_err) => {
                log::warn!("Drive connection task join error: {join_err}");
                Ok(false)
            }
        }
    }

    pub async fn upload_file_streaming(
        &self,
        path: &Path,
        file_name: &str,
        mime_type: &str,
        parent_id: Option<&str>,
    ) -> AppResult<String> {
        let path = path.to_path_buf();
        let file_name = file_name.to_string();
        let mime_type = mime_type.to_string();
        let parent = parent_id.map(|p| p.to_string());

        self.with_drive("upload_file_streaming", move |drive| {
            let mut metadata = objects::File::default();
            metadata.name = Some(file_name.clone());
            metadata.mime_type = Some(mime_type.clone());
            if let Some(parent_id) = parent {
                metadata.parents = Some(vec![parent_id]);
            }

            let uploaded = drive
                .files
                .create()
                .supports_all_drives(true)
                .ignore_default_visibility(true)
                .upload_type(objects::UploadType::Multipart)
                .metadata(&metadata)
                .content_source(path.clone())
                .execute()
                .map_err(|e| Self::map_drive_error("files.create", e))?;

            uploaded.id.ok_or_else(|| {
                AppError::NetworkError("Drive API response missing file id".to_string())
            })
        })
        .await
    }
}

static DRIVE_PLUGIN: Lazy<DrivePlugin> = Lazy::new(DrivePlugin::new);

pub fn drive_plugin() -> &'static DrivePlugin {
    &DRIVE_PLUGIN
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let _ = drive_plugin();
    log::info!("Google Drive plugin initialized");
    Ok(())
}
