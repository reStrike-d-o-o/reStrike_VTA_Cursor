use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

use crate::types::{AppError, AppResult};

const REDIRECT_URI: &str = "urn:ietf:wg:oauth:2.0:oob";

#[derive(Debug, Serialize, Deserialize)]
struct StoredCreds {
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredToken {
    refresh_token: String,
}

pub struct DrivePlugin {
    client: Arc<Mutex<Option<BasicClient>>>,
}

impl DrivePlugin {
    pub fn new() -> Self {
        Self { 
            client: Arc::new(Mutex::new(None))
        }
    }

    pub async fn auth_url(&self) -> AppResult<(String, String)> {
        let mut client_guard = self.client.lock().await;
        
        // Try to get existing client or create new one
        if client_guard.is_none() {
            let (client_id, client_secret) = Self::load_credentials()?;
            let new_client = BasicClient::new(
                ClientId::new(client_id),
                Some(ClientSecret::new(client_secret)),
                AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into()).unwrap(),
                Some(TokenUrl::new("https://oauth2.googleapis.com/token".into()).unwrap()),
            )
            .set_redirect_uri(RedirectUrl::new(REDIRECT_URI.into()).unwrap());
            *client_guard = Some(new_client);
        }
        
        let client = client_guard.as_ref().unwrap();
        let (url, csrf) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/drive.file".into(),
            ))
            .url();
        Ok((url.to_string(), csrf.secret().to_string()))
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
            self.store_refresh_token(refresh.secret())?;
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

    fn store_refresh_token(&self, token: &str) -> AppResult<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir).map_err(AppError::IoError)?;
        fs::write(
            dir.join("drive_token.json"),
            serde_json::to_vec_pretty(&StoredToken {
                refresh_token: token.to_string(),
            })
            .unwrap(),
        )
        .map_err(AppError::IoError)?;
        Ok(())
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