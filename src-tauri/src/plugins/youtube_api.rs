// YouTube API Integration Plugin
// Provides comprehensive YouTube Data API v3 integration for playlist and stream management
// Uses existing oauth2 and reqwest dependencies for authentication and HTTP requests

use crate::types::{AppResult, AppError};
use oauth2::{ClientId, ClientSecret, RedirectUrl, Scope, TokenResponse};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// YouTube API configuration
#[derive(Debug, Clone)]
pub struct YouTubeApiConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub api_key: Option<String>,
}

/// YouTube API client
pub struct YouTubeApiClient {
    
    oauth_client: BasicClient,
    http_client: Client,
    access_token: Arc<Mutex<Option<String>>>,
}

/// YouTube Playlist
#[derive(Debug, Serialize, Deserialize)]
pub struct YouTubePlaylist {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub privacy_status: String,
    pub item_count: u32,
    pub created_at: String,
}

/// YouTube Stream
#[derive(Debug, Serialize, Deserialize)]
pub struct YouTubeStream {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub scheduled_start_time: Option<String>,
    pub actual_start_time: Option<String>,
    pub actual_end_time: Option<String>,
    pub concurrent_viewers: Option<u32>,
    pub stream_key: Option<String>,
}

/// YouTube Video
#[derive(Debug, Serialize, Deserialize)]
pub struct YouTubeVideo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub published_at: String,
    pub duration: Option<String>,
    pub view_count: Option<u32>,
    pub like_count: Option<u32>,
}

/// YouTube API Response wrapper
#[derive(Debug, Serialize, Deserialize)]
struct YouTubeApiResponse<T> {
    items: Vec<T>,
    next_page_token: Option<String>,
    page_info: Option<PageInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PageInfo {
    total_results: u32,
    results_per_page: u32,
}

impl YouTubeApiClient {
    /// Create a new YouTube API client
    pub fn new(config: YouTubeApiConfig) -> Self {
        let oauth_client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            oauth2::AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap(),
            Some(oauth2::TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_url.clone()).unwrap());

        let http_client = Client::new();

        Self {
            config,
            oauth_client,
            http_client,
            access_token: Arc::new(Mutex::new(None)),
        }
    }

    /// Get OAuth authorization URL
    pub fn get_auth_url(&self) -> AppResult<String> {
        let (auth_url, _csrf_token) = self.oauth_client
            .authorize_url(|| oauth2::CsrfToken::new_random())
            .add_scope(Scope::new("https://www.googleapis.com/auth/youtube".to_string()))
            .add_scope(Scope::new("https://www.googleapis.com/auth/youtube.force-ssl".to_string()))
            .add_scope(Scope::new("https://www.googleapis.com/auth/youtube.readonly".to_string()))
            .url();

        Ok(auth_url.to_string())
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(&self, code: &str) -> AppResult<()> {
        let token_result = self.oauth_client
            .exchange_code(oauth2::AuthorizationCode::new(code.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| AppError::ConfigError(format!("OAuth token exchange failed: {}", e)))?;

        let access_token = token_result.access_token().secret().clone();
        *self.access_token.lock().await = Some(access_token);

        log::info!("YouTube API access token obtained successfully");
        Ok(())
    }

    /// Set access token directly (for testing or manual token management)
    pub async fn set_access_token(&self, token: String) {
        *self.access_token.lock().await = Some(token);
    }

    /// Get access token
    async fn get_access_token(&self) -> AppResult<String> {
        self.access_token.lock().await
            .clone()
            .ok_or_else(|| AppError::ConfigError("No access token available. Please authenticate first.".to_string()))
    }

    /// Make authenticated request to YouTube API
    async fn make_request<T>(&self, endpoint: &str, params: Option<HashMap<String, String>>) -> AppResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let access_token = self.get_access_token().await?;
        
        let mut url = format!("https://www.googleapis.com/youtube/v3/{}", endpoint);
        if let Some(mut params) = params {
            params.insert("access_token".to_string(), access_token);
            let query_string: String = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            url.push_str(&format!("?{}", query_string));
        } else {
            url.push_str(&format!("?access_token={}", access_token));
        }

        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::ConfigError(format!("YouTube API request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ConfigError(format!("YouTube API error: {}", error_text)));
        }

        let data: T = response.json().await
            .map_err(|e| AppError::ConfigError(format!("Failed to parse YouTube API response: {}", e)))?;

        Ok(data)
    }

    /// Create a new YouTube playlist
    pub async fn create_playlist(&self, title: &str, description: Option<&str>, privacy: &str) -> AppResult<YouTubePlaylist> {
        let access_token = self.get_access_token().await?;
        
        let playlist_data = serde_json::json!({
            "snippet": {
                "title": title,
                "description": description.unwrap_or(""),
                "tags": ["reStrike VTA", "Taekwondo", "Tournament"]
            },
            "status": {
                "privacyStatus": privacy
            }
        });

        let response = self.http_client
            .post("https://www.googleapis.com/youtube/v3/playlists?part=snippet,status")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&playlist_data)
            .send()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to create playlist: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ConfigError(format!("Failed to create playlist: {}", error_text)));
        }

        let playlist: YouTubePlaylist = response.json().await
            .map_err(|e| AppError::ConfigError(format!("Failed to parse playlist response: {}", e)))?;

        log::info!("Created YouTube playlist: {} ({})", playlist.title, playlist.id);
        Ok(playlist)
    }

    /// Get user's playlists
    pub async fn get_playlists(&self, max_results: Option<u32>) -> AppResult<Vec<YouTubePlaylist>> {
        let mut params = HashMap::new();
        params.insert("part".to_string(), "snippet,status,contentDetails".to_string());
        params.insert("mine".to_string(), "true".to_string());
        if let Some(max) = max_results {
            params.insert("maxResults".to_string(), max.to_string());
        }

        let response: YouTubeApiResponse<YouTubePlaylist> = self.make_request("playlists", Some(params)).await?;
        Ok(response.items)
    }

    /// Get playlist by ID
    pub async fn get_playlist(&self, playlist_id: &str) -> AppResult<YouTubePlaylist> {
        let mut params = HashMap::new();
        params.insert("part".to_string(), "snippet,status,contentDetails".to_string());
        params.insert("id".to_string(), playlist_id.to_string());

        let response: YouTubeApiResponse<YouTubePlaylist> = self.make_request("playlists", Some(params)).await?;
        
        response.items.into_iter().next()
            .ok_or_else(|| AppError::ConfigError("Playlist not found".to_string()))
    }

    /// Add video to playlist
    pub async fn add_video_to_playlist(&self, playlist_id: &str, video_id: &str) -> AppResult<()> {
        let access_token = self.get_access_token().await?;
        
        let playlist_item_data = serde_json::json!({
            "snippet": {
                "playlistId": playlist_id,
                "resourceId": {
                    "kind": "youtube#video",
                    "videoId": video_id
                }
            }
        });

        let response = self.http_client
            .post("https://www.googleapis.com/youtube/v3/playlistItems?part=snippet")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&playlist_item_data)
            .send()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to add video to playlist: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ConfigError(format!("Failed to add video to playlist: {}", error_text)));
        }

        log::info!("Added video {} to playlist {}", video_id, playlist_id);
        Ok(())
    }

    /// Remove video from playlist
    pub async fn remove_video_from_playlist(&self, playlist_item_id: &str) -> AppResult<()> {
        let access_token = self.get_access_token().await?;
        
        let response = self.http_client
            .delete(format!("https://www.googleapis.com/youtube/v3/playlistItems?id={}", playlist_item_id))
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to remove video from playlist: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ConfigError(format!("Failed to remove video from playlist: {}", error_text)));
        }

        log::info!("Removed playlist item {}", playlist_item_id);
        Ok(())
    }

    /// Get live streams
    pub async fn get_live_streams(&self, max_results: Option<u32>) -> AppResult<Vec<YouTubeStream>> {
        let mut params = HashMap::new();
        params.insert("part".to_string(), "snippet,status,contentDetails".to_string());
        params.insert("eventType".to_string(), "live".to_string());
        params.insert("type".to_string(), "video".to_string());
        if let Some(max) = max_results {
            params.insert("maxResults".to_string(), max.to_string());
        }

        let response: YouTubeApiResponse<YouTubeStream> = self.make_request("search", Some(params)).await?;
        Ok(response.items)
    }

    /// Get scheduled streams
    pub async fn get_scheduled_streams(&self, max_results: Option<u32>) -> AppResult<Vec<YouTubeStream>> {
        let mut params = HashMap::new();
        params.insert("part".to_string(), "snippet,status,contentDetails".to_string());
        params.insert("eventType".to_string(), "upcoming".to_string());
        params.insert("type".to_string(), "video".to_string());
        if let Some(max) = max_results {
            params.insert("maxResults".to_string(), max.to_string());
        }

        let response: YouTubeApiResponse<YouTubeStream> = self.make_request("search", Some(params)).await?;
        Ok(response.items)
    }

    /// Get completed streams
    pub async fn get_completed_streams(&self, max_results: Option<u32>) -> AppResult<Vec<YouTubeStream>> {
        let mut params = HashMap::new();
        params.insert("part".to_string(), "snippet,status,contentDetails".to_string());
        params.insert("eventType".to_string(), "completed".to_string());
        params.insert("type".to_string(), "video".to_string());
        if let Some(max) = max_results {
            params.insert("maxResults".to_string(), max.to_string());
        }

        let response: YouTubeApiResponse<YouTubeStream> = self.make_request("search", Some(params)).await?;
        Ok(response.items)
    }

    /// Get stream details by ID
    pub async fn get_stream(&self, stream_id: &str) -> AppResult<YouTubeStream> {
        let mut params = HashMap::new();
        params.insert("part".to_string(), "snippet,status,contentDetails,statistics".to_string());
        params.insert("id".to_string(), stream_id.to_string());

        let response: YouTubeApiResponse<YouTubeStream> = self.make_request("videos", Some(params)).await?;
        
        response.items.into_iter().next()
            .ok_or_else(|| AppError::ConfigError("Stream not found".to_string()))
    }

    /// End a live stream
    pub async fn end_stream(&self, stream_id: &str) -> AppResult<()> {
        let access_token = self.get_access_token().await?;
        
        // To end a stream, we need to update its status
        let update_data = serde_json::json!({
            "id": stream_id,
            "status": {
                "privacyStatus": "private"
            }
        });

        let response = self.http_client
            .put("https://www.googleapis.com/youtube/v3/videos?part=status")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&update_data)
            .send()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to end stream: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ConfigError(format!("Failed to end stream: {}", error_text)));
        }

        log::info!("Ended stream {}", stream_id);
        Ok(())
    }

    /// Create a new scheduled stream
    pub async fn create_scheduled_stream(&self, title: &str, description: Option<&str>, scheduled_time: &str) -> AppResult<YouTubeStream> {
        let access_token = self.get_access_token().await?;
        
        let stream_data = serde_json::json!({
            "snippet": {
                "title": title,
                "description": description.unwrap_or(""),
                "scheduledStartTime": scheduled_time,
                "tags": ["reStrike VTA", "Taekwondo", "Tournament", "Live"]
            },
            "status": {
                "privacyStatus": "private",
                "selfDeclaredMadeForKids": false
            }
        });

        let response = self.http_client
            .post("https://www.googleapis.com/youtube/v3/liveBroadcasts?part=snippet,status")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&stream_data)
            .send()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to create scheduled stream: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ConfigError(format!("Failed to create scheduled stream: {}", error_text)));
        }

        let stream: YouTubeStream = response.json().await
            .map_err(|e| AppError::ConfigError(format!("Failed to parse stream response: {}", e)))?;

        log::info!("Created scheduled stream: {} ({})", stream.title, stream.id);
        Ok(stream)
    }

    /// Get videos in a playlist
    pub async fn get_playlist_videos(&self, playlist_id: &str, max_results: Option<u32>) -> AppResult<Vec<YouTubeVideo>> {
        let mut params = HashMap::new();
        params.insert("part".to_string(), "snippet,contentDetails".to_string());
        params.insert("playlistId".to_string(), playlist_id.to_string());
        if let Some(max) = max_results {
            params.insert("maxResults".to_string(), max.to_string());
        }

        let response: YouTubeApiResponse<YouTubeVideo> = self.make_request("playlistItems", Some(params)).await?;
        Ok(response.items)
    }

    /// Update playlist details
    pub async fn update_playlist(&self, playlist_id: &str, title: Option<&str>, description: Option<&str>, privacy: Option<&str>) -> AppResult<YouTubePlaylist> {
        let access_token = self.get_access_token().await?;
        
        let mut update_data = serde_json::Map::new();
        
        if let Some(title) = title {
            update_data.insert("title".to_string(), serde_json::Value::String(title.to_string()));
        }
        if let Some(description) = description {
            update_data.insert("description".to_string(), serde_json::Value::String(description.to_string()));
        }
        if let Some(privacy) = privacy {
            update_data.insert("privacyStatus".to_string(), serde_json::Value::String(privacy.to_string()));
        }

        let playlist_data = serde_json::json!({
            "id": playlist_id,
            "snippet": update_data
        });

        let response = self.http_client
            .put("https://www.googleapis.com/youtube/v3/playlists?part=snippet,status")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&playlist_data)
            .send()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to update playlist: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ConfigError(format!("Failed to update playlist: {}", error_text)));
        }

        let playlist: YouTubePlaylist = response.json().await
            .map_err(|e| AppError::ConfigError(format!("Failed to parse playlist response: {}", e)))?;

        log::info!("Updated playlist: {} ({})", playlist.title, playlist.id);
        Ok(playlist)
    }

    /// Delete playlist
    pub async fn delete_playlist(&self, playlist_id: &str) -> AppResult<()> {
        let access_token = self.get_access_token().await?;
        
        let response = self.http_client
            .delete(format!("https://www.googleapis.com/youtube/v3/playlists?id={}", playlist_id))
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to delete playlist: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ConfigError(format!("Failed to delete playlist: {}", error_text)));
        }

        log::info!("Deleted playlist {}", playlist_id);
        Ok(())
    }

    /// Get channel information
    pub async fn get_channel_info(&self) -> AppResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("part".to_string(), "snippet,statistics,contentDetails".to_string());
        params.insert("mine".to_string(), "true".to_string());

        let response: serde_json::Value = self.make_request("channels", Some(params)).await?;
        Ok(response)
    }

    /// Get video analytics
    pub async fn get_video_analytics(&self, video_id: &str) -> AppResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("part".to_string(), "statistics,snippet".to_string());
        params.insert("id".to_string(), video_id.to_string());

        let response: serde_json::Value = self.make_request("videos", Some(params)).await?;
        Ok(response)
    }
}

/// YouTube API Plugin
pub struct YouTubeApiPlugin {
    client: Option<YouTubeApiClient>,
}

impl YouTubeApiPlugin {
    /// Create a new YouTube API Plugin
    pub fn new() -> Self {
        Self { client: None }
    }

    /// Initialize the YouTube API client
    pub fn initialize(&mut self, config: YouTubeApiConfig) -> AppResult<()> {
        self.client = Some(YouTubeApiClient::new(config));
        log::info!("YouTube API Plugin initialized");
        Ok(())
    }

    /// Get the YouTube API client
    fn get_client(&self) -> AppResult<&YouTubeApiClient> {
        self.client.as_ref()
            .ok_or_else(|| AppError::ConfigError("YouTube API client not initialized".to_string()))
    }

    /// Get OAuth authorization URL
    pub async fn get_auth_url(&self) -> AppResult<String> {
        let client = self.get_client()?;
        client.get_auth_url()
    }

    /// Exchange authorization code for access token
    pub async fn authenticate(&self, code: &str) -> AppResult<()> {
        let client = self.get_client()?;
        client.exchange_code(code).await
    }

    /// Create a new playlist
    pub async fn create_playlist(&self, title: &str, description: Option<&str>, privacy: &str) -> AppResult<YouTubePlaylist> {
        let client = self.get_client()?;
        client.create_playlist(title, description, privacy).await
    }

    /// Get user's playlists
    pub async fn get_playlists(&self, max_results: Option<u32>) -> AppResult<Vec<YouTubePlaylist>> {
        let client = self.get_client()?;
        client.get_playlists(max_results).await
    }

    /// Add video to playlist
    pub async fn add_video_to_playlist(&self, playlist_id: &str, video_id: &str) -> AppResult<()> {
        let client = self.get_client()?;
        client.add_video_to_playlist(playlist_id, video_id).await
    }

    /// End a live stream
    pub async fn end_stream(&self, stream_id: &str) -> AppResult<()> {
        let client = self.get_client()?;
        client.end_stream(stream_id).await
    }

    /// Create a new scheduled stream
    pub async fn create_scheduled_stream(&self, title: &str, description: Option<&str>, scheduled_time: &str) -> AppResult<YouTubeStream> {
        let client = self.get_client()?;
        client.create_scheduled_stream(title, description, scheduled_time).await
    }

    /// Get live streams
    pub async fn get_live_streams(&self, max_results: Option<u32>) -> AppResult<Vec<YouTubeStream>> {
        let client = self.get_client()?;
        client.get_live_streams(max_results).await
    }

    /// Get scheduled streams
    pub async fn get_scheduled_streams(&self, max_results: Option<u32>) -> AppResult<Vec<YouTubeStream>> {
        let client = self.get_client()?;
        client.get_scheduled_streams(max_results).await
    }

    /// Get completed streams
    pub async fn get_completed_streams(&self, max_results: Option<u32>) -> AppResult<Vec<YouTubeStream>> {
        let client = self.get_client()?;
        client.get_completed_streams(max_results).await
    }

    /// Get videos in a playlist
    pub async fn get_playlist_videos(&self, playlist_id: &str, max_results: Option<u32>) -> AppResult<Vec<YouTubeVideo>> {
        let client = self.get_client()?;
        client.get_playlist_videos(playlist_id, max_results).await
    }

    /// Update playlist details
    pub async fn update_playlist(&self, playlist_id: &str, title: Option<&str>, description: Option<&str>, privacy: Option<&str>) -> AppResult<YouTubePlaylist> {
        let client = self.get_client()?;
        client.update_playlist(playlist_id, title, description, privacy).await
    }

    /// Delete playlist
    pub async fn delete_playlist(&self, playlist_id: &str) -> AppResult<()> {
        let client = self.get_client()?;
        client.delete_playlist(playlist_id).await
    }

    /// Get channel information
    pub async fn get_channel_info(&self) -> AppResult<serde_json::Value> {
        let client = self.get_client()?;
        client.get_channel_info().await
    }

    /// Get video analytics
    pub async fn get_video_analytics(&self, video_id: &str) -> AppResult<serde_json::Value> {
        let client = self.get_client()?;
        client.get_video_analytics(video_id).await
    }
} 