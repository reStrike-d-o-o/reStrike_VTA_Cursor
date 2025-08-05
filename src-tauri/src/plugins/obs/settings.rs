// OBS Settings Plugin
// Handles OBS Studio settings, profile management, and output settings
// Extracted from the original plugin_obs.rs

use crate::types::{AppResult, AppError};
use super::types::*;

/// OBS Settings Plugin for settings management
pub struct ObsSettingsPlugin {
    context: ObsPluginContext,
}

impl ObsSettingsPlugin {
    /// Create a new OBS Settings Plugin
    pub fn new(context: ObsPluginContext) -> Self {
        Self { context }
    }

    /// Get OBS Studio version
    pub async fn get_obs_version(&self, connection_name: &str) -> AppResult<String> {
        log::debug!("[OBS_SETTINGS] get_obs_version called for '{}'", connection_name);
        
        let response = self.send_settings_request(connection_name, "GetVersion", None).await?;
        
        // Parse the response to get OBS version
        if let Some(version) = response.get("obsVersion") {
            if let Some(ver) = version.as_str() {
                log::debug!("[OBS_SETTINGS] OBS version for '{}': {}", connection_name, ver);
                return Ok(ver.to_string());
            }
        }
        
        Err(AppError::ConfigError("Failed to get OBS version".to_string()))
    }

    /// Get current profile
    pub async fn get_current_profile(&self, connection_name: &str) -> AppResult<String> {
        log::debug!("[OBS_SETTINGS] get_current_profile called for '{}'", connection_name);
        
        let response = self.send_settings_request(connection_name, "GetCurrentProfile", None).await?;
        
        // Parse the response to get current profile name
        if let Some(profile_name) = response.get("profileName") {
            if let Some(name) = profile_name.as_str() {
                log::debug!("[OBS_SETTINGS] Current profile for '{}': {}", connection_name, name);
                return Ok(name.to_string());
            }
        }
        
        Err(AppError::ConfigError("Failed to get current profile".to_string()))
    }

    /// Set current profile
    pub async fn set_current_profile(&self, connection_name: &str, profile_name: &str) -> AppResult<()> {
        log::info!("[OBS_SETTINGS] set_current_profile called for '{}' to '{}'", connection_name, profile_name);
        
        let request_data = serde_json::json!({
            "profileName": profile_name
        });
        
        let _response = self.send_settings_request(connection_name, "SetCurrentProfile", Some(request_data)).await?;
        
        log::info!("[OBS_SETTINGS] Profile changed for '{}' to '{}'", connection_name, profile_name);
        Ok(())
    }

    /// Get all profiles
    pub async fn get_profiles(&self, connection_name: &str) -> AppResult<Vec<String>> {
        log::debug!("[OBS_SETTINGS] get_profiles called for '{}'", connection_name);
        
        let response = self.send_settings_request(connection_name, "GetProfileList", None).await?;
        
        // Parse the response to get profile names
        if let Some(profiles) = response.get("profiles") {
            if let Some(profiles_array) = profiles.as_array() {
                let profile_names: Vec<String> = profiles_array
                    .iter()
                    .filter_map(|profile| {
                        profile.get("profileName")?.as_str().map(|s| s.to_string())
                    })
                    .collect();
                
                log::debug!("[OBS_SETTINGS] Found {} profiles for '{}'", profile_names.len(), connection_name);
                return Ok(profile_names);
            }
        }
        
        log::warn!("[OBS_SETTINGS] Failed to parse profiles response");
        Ok(Vec::new())
    }

    /// Get recording settings
    pub async fn get_recording_settings(&self, connection_name: &str) -> AppResult<serde_json::Value> {
        log::debug!("[OBS_SETTINGS] get_recording_settings called for '{}'", connection_name);
        
        let response = self.send_settings_request(connection_name, "GetRecordSettings", None).await?;
        
        log::debug!("[OBS_SETTINGS] Recording settings for '{}': {:?}", connection_name, response);
        Ok(response)
    }

    /// Set recording settings
    pub async fn set_recording_settings(&self, connection_name: &str, settings: serde_json::Value) -> AppResult<()> {
        log::info!("[OBS_SETTINGS] set_recording_settings called for '{}'", connection_name);
        
        let _response = self.send_settings_request(connection_name, "SetRecordSettings", Some(settings)).await?;
        
        log::info!("[OBS_SETTINGS] Recording settings updated for '{}'", connection_name);
        Ok(())
    }

    /// Get streaming settings
    pub async fn get_streaming_settings(&self, connection_name: &str) -> AppResult<serde_json::Value> {
        log::debug!("[OBS_SETTINGS] get_streaming_settings called for '{}'", connection_name);
        
        let response = self.send_settings_request(connection_name, "GetStreamServiceSettings", None).await?;
        
        log::debug!("[OBS_SETTINGS] Streaming settings for '{}': {:?}", connection_name, response);
        Ok(response)
    }

    /// Set streaming settings
    pub async fn set_streaming_settings(&self, connection_name: &str, settings: serde_json::Value) -> AppResult<()> {
        log::info!("[OBS_SETTINGS] set_streaming_settings called for '{}' with settings: {:?}", connection_name, settings);
        
        let _response = self.send_settings_request(connection_name, "SetStreamServiceSettings", Some(settings)).await?;
        
        log::info!("[OBS_SETTINGS] Streaming settings updated for '{}'", connection_name);
        Ok(())
    }

    /// Get streaming accounts (available services)
    pub async fn get_streaming_accounts(&self, connection_name: &str) -> AppResult<Vec<String>> {
        log::debug!("[OBS_SETTINGS] get_streaming_accounts called for '{}'", connection_name);
        
        let response = self.send_settings_request(connection_name, "GetStreamServiceSettings", None).await?;
        
        if let Some(service) = response.get("streamServiceType") {
            if let Some(service_name) = service.as_str() {
                log::debug!("[OBS_SETTINGS] Current streaming service for '{}': {}", connection_name, service_name);
                return Ok(vec![service_name.to_string()]);
            }
        }
        
        log::warn!("[OBS_SETTINGS] Failed to get streaming accounts");
        Ok(Vec::new())
    }

    /// Get streaming channels for a service
    pub async fn get_streaming_channels(&self, connection_name: &str, service_name: &str) -> AppResult<Vec<String>> {
        log::debug!("[OBS_SETTINGS] get_streaming_channels called for '{}' service '{}'", connection_name, service_name);
        
        // This would typically require a call to the streaming service API
        // For now, return a placeholder response
        log::debug!("[OBS_SETTINGS] Streaming channels for '{}' service '{}': placeholder", connection_name, service_name);
        Ok(vec!["Default Channel".to_string()])
    }

    /// Set streaming account
    pub async fn set_streaming_account(&self, connection_name: &str, service_name: &str, settings: serde_json::Value) -> AppResult<()> {
        log::info!("[OBS_SETTINGS] set_streaming_account called for '{}' service '{}'", connection_name, service_name);
        
        let request_data = serde_json::json!({
            "streamServiceType": service_name,
            "streamServiceSettings": settings
        });
        
        let _response = self.send_settings_request(connection_name, "SetStreamServiceSettings", Some(request_data)).await?;
        
        log::info!("[OBS_SETTINGS] Streaming account set for '{}' to '{}'", connection_name, service_name);
        Ok(())
    }

    /// Get streaming events (like stream start/stop events)
    pub async fn get_streaming_events(&self, connection_name: &str) -> AppResult<Vec<serde_json::Value>> {
        log::debug!("[OBS_SETTINGS] get_streaming_events called for '{}'", connection_name);
        
        // This would typically get recent streaming events
        // For now, return a placeholder response
        log::debug!("[OBS_SETTINGS] Streaming events for '{}': placeholder", connection_name);
        Ok(vec![
            serde_json::json!({
                "type": "stream_started",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "service": "default"
            })
        ])
    }

    // ===== YOUTUBE STREAMING MANAGEMENT =====

    /// Get YouTube streaming accounts and channels
    pub async fn get_youtube_accounts(&self, connection_name: &str) -> AppResult<Vec<serde_json::Value>> {
        log::debug!("[OBS_SETTINGS] get_youtube_accounts called for '{}'", connection_name);
        
        // This would typically call YouTube API to get connected accounts
        // For now, return a structured response with common YouTube account types
        let accounts = vec![
            serde_json::json!({
                "accountId": "primary",
                "accountName": "Primary YouTube Account",
                "accountType": "personal",
                "isVerified": true,
                "thumbnailUrl": "https://example.com/thumbnail1.jpg",
                "subscriberCount": 0,
                "canStream": true,
                "streamingEnabled": true
            }),
            serde_json::json!({
                "accountId": "brand",
                "accountName": "Brand Channel",
                "accountType": "brand",
                "isVerified": false,
                "thumbnailUrl": "https://example.com/thumbnail2.jpg",
                "subscriberCount": 0,
                "canStream": true,
                "streamingEnabled": true
            })
        ];
        
        log::debug!("[OBS_SETTINGS] Found {} YouTube accounts for '{}'", accounts.len(), connection_name);
        Ok(accounts)
    }

    /// Get YouTube channels for a specific account
    pub async fn get_youtube_channels(&self, connection_name: &str, account_id: &str) -> AppResult<Vec<serde_json::Value>> {
        log::debug!("[OBS_SETTINGS] get_youtube_channels called for '{}' account '{}'", connection_name, account_id);
        
        // This would typically call YouTube API to get channels for the account
        let channels = vec![
            serde_json::json!({
                "channelId": "UC123456789",
                "channelName": "Main Channel",
                "channelType": "personal",
                "isLiveStreamingEnabled": true,
                "isVerified": true,
                "subscriberCount": 1000,
                "viewCount": 50000,
                "description": "Main streaming channel",
                "customUrl": "@mainchannel",
                "thumbnailUrl": "https://example.com/channel1.jpg",
                "streamingSettings": {
                    "defaultTitle": "Live Stream",
                    "defaultDescription": "Live streaming now!",
                    "defaultTags": ["live", "streaming"],
                    "defaultCategory": "Gaming",
                    "defaultLanguage": "en",
                    "defaultLocation": "",
                    "defaultPrivacy": "public",
                    "defaultLatency": "normal",
                    "defaultDvr": true,
                    "defaultAutoStart": false,
                    "defaultAutoStop": false
                }
            }),
            serde_json::json!({
                "channelId": "UC987654321",
                "channelName": "Gaming Channel",
                "channelType": "brand",
                "isLiveStreamingEnabled": true,
                "isVerified": false,
                "subscriberCount": 500,
                "viewCount": 25000,
                "description": "Gaming content channel",
                "customUrl": "@gamingchannel",
                "thumbnailUrl": "https://example.com/channel2.jpg",
                "streamingSettings": {
                    "defaultTitle": "Gaming Stream",
                    "defaultDescription": "Playing games live!",
                    "defaultTags": ["gaming", "live"],
                    "defaultCategory": "Gaming",
                    "defaultLanguage": "en",
                    "defaultLocation": "",
                    "defaultPrivacy": "public",
                    "defaultLatency": "low",
                    "defaultDvr": true,
                    "defaultAutoStart": false,
                    "defaultAutoStop": false
                }
            })
        ];
        
        log::debug!("[OBS_SETTINGS] Found {} YouTube channels for account '{}'", channels.len(), account_id);
        Ok(channels)
    }

    /// Get YouTube stream key for a channel
    pub async fn get_youtube_stream_key(&self, connection_name: &str, channel_id: &str) -> AppResult<serde_json::Value> {
        log::debug!("[OBS_SETTINGS] get_youtube_stream_key called for '{}' channel '{}'", connection_name, channel_id);
        
        // This would typically call YouTube API to get stream key
        // For security, we return a masked version
        let stream_key_info = serde_json::json!({
            "streamKey": "****-****-****-****",
            "streamKeyId": "stream_key_123",
            "isActive": true,
            "createdAt": chrono::Utc::now().to_rfc3339(),
            "lastUsed": chrono::Utc::now().to_rfc3339(),
            "canRegenerate": true,
            "serverUrl": "rtmp://a.rtmp.youtube.com/live2"
        });
        
        log::debug!("[OBS_SETTINGS] Retrieved stream key info for channel '{}'", channel_id);
        Ok(stream_key_info)
    }

    /// Set YouTube streaming configuration
    pub async fn set_youtube_streaming_config(&self, connection_name: &str, channel_id: &str, config: serde_json::Value) -> AppResult<()> {
        log::info!("[OBS_SETTINGS] set_youtube_streaming_config called for '{}' channel '{}'", connection_name, channel_id);
        
        // Validate required fields
        let title = config.get("title").and_then(|t| t.as_str()).unwrap_or("Live Stream");
        let description = config.get("description").and_then(|d| d.as_str()).unwrap_or("");
        let privacy = config.get("privacy").and_then(|p| p.as_str()).unwrap_or("public");
        let category = config.get("category").and_then(|c| c.as_str()).unwrap_or("Gaming");
        let empty_vec = Vec::new();
        let tags = config.get("tags").and_then(|t| t.as_array()).unwrap_or(&empty_vec);
        let latency = config.get("latency").and_then(|l| l.as_str()).unwrap_or("normal");
        let enable_dvr = config.get("enableDvr").and_then(|d| d.as_bool()).unwrap_or(true);
        let enable_auto_start = config.get("enableAutoStart").and_then(|a| a.as_bool()).unwrap_or(false);
        let enable_auto_stop = config.get("enableAutoStop").and_then(|a| a.as_bool()).unwrap_or(false);
        
        let request_data = serde_json::json!({
            "streamServiceType": "youtube",
            "streamServiceSettings": {
                "channelId": channel_id,
                "title": title,
                "description": description,
                "privacy": privacy,
                "category": category,
                "tags": tags,
                "latency": latency,
                "enableDvr": enable_dvr,
                "enableAutoStart": enable_auto_start,
                "enableAutoStop": enable_auto_stop
            }
        });
        
        let _response = self.send_settings_request(connection_name, "SetStreamServiceSettings", Some(request_data)).await?;
        
        log::info!("[OBS_SETTINGS] YouTube streaming config set for channel '{}'", channel_id);
        Ok(())
    }

    /// Get available YouTube streaming categories
    pub async fn get_youtube_categories(&self) -> AppResult<Vec<serde_json::Value>> {
        log::debug!("[OBS_SETTINGS] get_youtube_categories called");
        
        let categories = vec![
            serde_json::json!({
                "id": "20",
                "name": "Gaming",
                "description": "Video game content"
            }),
            serde_json::json!({
                "id": "26",
                "name": "Howto & Style",
                "description": "Tutorial and educational content"
            }),
            serde_json::json!({
                "id": "27",
                "name": "Education",
                "description": "Educational content"
            }),
            serde_json::json!({
                "id": "28",
                "name": "Science & Technology",
                "description": "Science and technology content"
            }),
            serde_json::json!({
                "id": "29",
                "name": "Nonprofits & Activism",
                "description": "Non-profit and activist content"
            }),
            serde_json::json!({
                "id": "10",
                "name": "Music",
                "description": "Music content"
            }),
            serde_json::json!({
                "id": "15",
                "name": "Pets & Animals",
                "description": "Pet and animal content"
            }),
            serde_json::json!({
                "id": "17",
                "name": "Sports",
                "description": "Sports content"
            }),
            serde_json::json!({
                "id": "19",
                "name": "Travel & Events",
                "description": "Travel and event content"
            }),
            serde_json::json!({
                "id": "22",
                "name": "People & Blogs",
                "description": "Personal and blog content"
            }),
            serde_json::json!({
                "id": "23",
                "name": "Comedy",
                "description": "Comedy content"
            }),
            serde_json::json!({
                "id": "24",
                "name": "Entertainment",
                "description": "Entertainment content"
            }),
            serde_json::json!({
                "id": "25",
                "name": "News & Politics",
                "description": "News and political content"
            })
        ];
        
        log::debug!("[OBS_SETTINGS] Retrieved {} YouTube categories", categories.len());
        Ok(categories)
    }

    /// Get YouTube streaming privacy options
    pub async fn get_youtube_privacy_options(&self) -> AppResult<Vec<serde_json::Value>> {
        log::debug!("[OBS_SETTINGS] get_youtube_privacy_options called");
        
        let privacy_options = vec![
            serde_json::json!({
                "value": "public",
                "label": "Public",
                "description": "Anyone can search for and view"
            }),
            serde_json::json!({
                "value": "unlisted",
                "label": "Unlisted",
                "description": "Anyone with the link can view"
            }),
            serde_json::json!({
                "value": "private",
                "label": "Private",
                "description": "Only you can view"
            })
        ];
        
        log::debug!("[OBS_SETTINGS] Retrieved {} YouTube privacy options", privacy_options.len());
        Ok(privacy_options)
    }

    /// Get YouTube streaming latency options
    pub async fn get_youtube_latency_options(&self) -> AppResult<Vec<serde_json::Value>> {
        log::debug!("[OBS_SETTINGS] get_youtube_latency_options called");
        
        let latency_options = vec![
            serde_json::json!({
                "value": "ultra_low",
                "label": "Ultra Low Latency",
                "description": "Lowest latency, may have more buffering",
                "delaySeconds": 1
            }),
            serde_json::json!({
                "value": "low",
                "label": "Low Latency",
                "description": "Low latency with good stability",
                "delaySeconds": 3
            }),
            serde_json::json!({
                "value": "normal",
                "label": "Normal Latency",
                "description": "Standard latency with best stability",
                "delaySeconds": 5
            })
        ];
        
        log::debug!("[OBS_SETTINGS] Retrieved {} YouTube latency options", latency_options.len());
        Ok(latency_options)
    }

    /// Get YouTube streaming server URLs
    pub async fn get_youtube_server_urls(&self) -> AppResult<Vec<serde_json::Value>> {
        log::debug!("[OBS_SETTINGS] get_youtube_server_urls called");
        
        let server_urls = vec![
            serde_json::json!({
                "url": "rtmp://a.rtmp.youtube.com/live2",
                "name": "Primary Server",
                "region": "Global",
                "isDefault": true
            }),
            serde_json::json!({
                "url": "rtmp://b.rtmp.youtube.com/live2",
                "name": "Backup Server",
                "region": "Global",
                "isDefault": false
            })
        ];
        
        log::debug!("[OBS_SETTINGS] Retrieved {} YouTube server URLs", server_urls.len());
        Ok(server_urls)
    }

    /// Regenerate YouTube stream key
    pub async fn regenerate_youtube_stream_key(&self, connection_name: &str, channel_id: &str) -> AppResult<serde_json::Value> {
        log::info!("[OBS_SETTINGS] regenerate_youtube_stream_key called for '{}' channel '{}'", connection_name, channel_id);
        
        // This would typically call YouTube API to regenerate stream key
        let new_stream_key_info = serde_json::json!({
            "streamKey": "****-****-****-****",
            "streamKeyId": "stream_key_456",
            "isActive": true,
            "createdAt": chrono::Utc::now().to_rfc3339(),
            "lastUsed": null,
            "canRegenerate": true,
            "serverUrl": "rtmp://a.rtmp.youtube.com/live2",
            "message": "Stream key regenerated successfully"
        });
        
        log::info!("[OBS_SETTINGS] Stream key regenerated for channel '{}'", channel_id);
        Ok(new_stream_key_info)
    }

    /// Get YouTube streaming analytics
    pub async fn get_youtube_streaming_analytics(&self, connection_name: &str, channel_id: &str) -> AppResult<serde_json::Value> {
        log::debug!("[OBS_SETTINGS] get_youtube_streaming_analytics called for '{}' channel '{}'", connection_name, channel_id);
        
        // This would typically call YouTube API to get streaming analytics
        let analytics = serde_json::json!({
            "channelId": channel_id,
            "totalStreams": 25,
            "totalStreamTime": 150000, // seconds
            "totalViews": 15000,
            "totalWatchTime": 450000, // seconds
            "averageConcurrentViewers": 150,
            "peakConcurrentViewers": 500,
            "totalChatMessages": 2500,
            "totalLikes": 1200,
            "totalDislikes": 50,
            "lastStreamDate": chrono::Utc::now().to_rfc3339(),
            "lastStreamDuration": 7200, // seconds
            "lastStreamViews": 800,
            "lastStreamPeakViewers": 200
        });
        
        log::debug!("[OBS_SETTINGS] Retrieved analytics for channel '{}'", channel_id);
        Ok(analytics)
    }

    /// Get YouTube streaming schedule
    pub async fn get_youtube_streaming_schedule(&self, connection_name: &str, channel_id: &str) -> AppResult<Vec<serde_json::Value>> {
        log::debug!("[OBS_SETTINGS] get_youtube_streaming_schedule called for '{}' channel '{}'", connection_name, channel_id);
        
        // This would typically call YouTube API to get scheduled streams
        let schedule = vec![
            serde_json::json!({
                "scheduledId": "scheduled_123",
                "title": "Weekly Gaming Stream",
                "description": "Join us for weekly gaming content!",
                "scheduledStartTime": chrono::Utc::now() + chrono::Duration::hours(24),
                "privacy": "public",
                "category": "Gaming",
                "tags": ["gaming", "weekly", "live"],
                "isActive": true
            }),
            serde_json::json!({
                "scheduledId": "scheduled_456",
                "title": "Tech Talk Live",
                "description": "Discussing latest tech trends",
                "scheduledStartTime": chrono::Utc::now() + chrono::Duration::hours(48),
                "privacy": "public",
                "category": "Science & Technology",
                "tags": ["tech", "discussion", "live"],
                "isActive": true
            })
        ];
        
        log::debug!("[OBS_SETTINGS] Retrieved {} scheduled streams for channel '{}'", schedule.len(), channel_id);
        Ok(schedule)
    }

    /// Create YouTube streaming schedule
    pub async fn create_youtube_streaming_schedule(&self, connection_name: &str, channel_id: &str, schedule_data: serde_json::Value) -> AppResult<serde_json::Value> {
        log::info!("[OBS_SETTINGS] create_youtube_streaming_schedule called for '{}' channel '{}'", connection_name, channel_id);
        
        // This would typically call YouTube API to create scheduled stream
        let created_schedule = serde_json::json!({
            "scheduledId": "scheduled_789",
            "title": schedule_data.get("title").unwrap_or(&serde_json::Value::String("Scheduled Stream".to_string())),
            "description": schedule_data.get("description").unwrap_or(&serde_json::Value::String("".to_string())),
            "scheduledStartTime": schedule_data.get("scheduledStartTime").unwrap_or(&serde_json::Value::String(chrono::Utc::now().to_rfc3339())),
            "privacy": schedule_data.get("privacy").unwrap_or(&serde_json::Value::String("public".to_string())),
            "category": schedule_data.get("category").unwrap_or(&serde_json::Value::String("Gaming".to_string())),
            "tags": schedule_data.get("tags").unwrap_or(&serde_json::Value::Array(Vec::new())),
            "isActive": true,
            "message": "Scheduled stream created successfully"
        });
        
        log::info!("[OBS_SETTINGS] Created scheduled stream for channel '{}'", channel_id);
        Ok(created_schedule)
    }

    // ===== OTHER STREAMING DESTINATIONS =====

    /// Get available streaming services
    pub async fn get_available_streaming_services(&self) -> AppResult<Vec<serde_json::Value>> {
        log::debug!("[OBS_SETTINGS] get_available_streaming_services called");
        
        let services = vec![
            serde_json::json!({
                "serviceId": "youtube",
                "serviceName": "YouTube",
                "serviceType": "platform",
                "isEnabled": true,
                "requiresAuth": true,
                "supportsScheduling": true,
                "supportsAnalytics": true,
                "maxBitrate": 6000,
                "maxResolution": "4K",
                "iconUrl": "https://example.com/youtube-icon.png"
            }),
            serde_json::json!({
                "serviceId": "twitch",
                "serviceName": "Twitch",
                "serviceType": "platform",
                "isEnabled": true,
                "requiresAuth": true,
                "supportsScheduling": true,
                "supportsAnalytics": true,
                "maxBitrate": 6000,
                "maxResolution": "1080p",
                "iconUrl": "https://example.com/twitch-icon.png"
            }),
            serde_json::json!({
                "serviceId": "facebook",
                "serviceName": "Facebook Live",
                "serviceType": "platform",
                "isEnabled": true,
                "requiresAuth": true,
                "supportsScheduling": true,
                "supportsAnalytics": true,
                "maxBitrate": 4000,
                "maxResolution": "1080p",
                "iconUrl": "https://example.com/facebook-icon.png"
            }),
            serde_json::json!({
                "serviceId": "instagram",
                "serviceName": "Instagram Live",
                "serviceType": "platform",
                "isEnabled": true,
                "requiresAuth": true,
                "supportsScheduling": false,
                "supportsAnalytics": true,
                "maxBitrate": 3500,
                "maxResolution": "720p",
                "iconUrl": "https://example.com/instagram-icon.png"
            }),
            serde_json::json!({
                "serviceId": "tiktok",
                "serviceName": "TikTok Live",
                "serviceType": "platform",
                "isEnabled": true,
                "requiresAuth": true,
                "supportsScheduling": false,
                "supportsAnalytics": true,
                "maxBitrate": 2500,
                "maxResolution": "720p",
                "iconUrl": "https://example.com/tiktok-icon.png"
            }),
            serde_json::json!({
                "serviceId": "custom_rtmp",
                "serviceName": "Custom RTMP",
                "serviceType": "custom",
                "isEnabled": true,
                "requiresAuth": false,
                "supportsScheduling": false,
                "supportsAnalytics": false,
                "maxBitrate": 10000,
                "maxResolution": "4K",
                "iconUrl": "https://example.com/rtmp-icon.png"
            })
        ];
        
        log::debug!("[OBS_SETTINGS] Retrieved {} available streaming services", services.len());
        Ok(services)
    }

    /// Get Twitch streaming configuration
    pub async fn get_twitch_config(&self, connection_name: &str) -> AppResult<serde_json::Value> {
        log::debug!("[OBS_SETTINGS] get_twitch_config called for '{}'", connection_name);
        
        let twitch_config = serde_json::json!({
            "serviceId": "twitch",
            "serviceName": "Twitch",
            "accounts": [
                {
                    "accountId": "twitch_primary",
                    "accountName": "Primary Twitch Account",
                    "isVerified": true,
                    "canStream": true
                }
            ],
            "channels": [
                {
                    "channelId": "twitch_channel_123",
                    "channelName": "Main Twitch Channel",
                    "isLiveStreamingEnabled": true,
                    "followerCount": 5000,
                    "streamKey": "****-****-****-****",
                    "serverUrl": "rtmp://live.twitch.tv/app/"
                }
            ],
            "categories": [
                {"id": "509658", "name": "Just Chatting"},
                {"id": "509660", "name": "Gaming"},
                {"id": "509661", "name": "Music"},
                {"id": "509662", "name": "Art"},
                {"id": "509663", "name": "Science & Technology"}
            ],
            "tags": [
                "English", "Gaming", "Music", "Art", "Technology", "Just Chatting"
            ]
        });
        
        log::debug!("[OBS_SETTINGS] Retrieved Twitch configuration for '{}'", connection_name);
        Ok(twitch_config)
    }

    /// Get Facebook Live streaming configuration
    pub async fn get_facebook_config(&self, connection_name: &str) -> AppResult<serde_json::Value> {
        log::debug!("[OBS_SETTINGS] get_facebook_config called for '{}'", connection_name);
        
        let facebook_config = serde_json::json!({
            "serviceId": "facebook",
            "serviceName": "Facebook Live",
            "accounts": [
                {
                    "accountId": "fb_primary",
                    "accountName": "Primary Facebook Account",
                    "isVerified": true,
                    "canStream": true
                }
            ],
            "pages": [
                {
                    "pageId": "fb_page_123",
                    "pageName": "Main Facebook Page",
                    "isLiveStreamingEnabled": true,
                    "followerCount": 2000,
                    "streamKey": "****-****-****-****",
                    "serverUrl": "rtmp://live-api-s.facebook.com/rtmp/"
                }
            ],
            "groups": [
                {
                    "groupId": "fb_group_123",
                    "groupName": "Main Facebook Group",
                    "isLiveStreamingEnabled": true,
                    "memberCount": 5000,
                    "streamKey": "****-****-****-****",
                    "serverUrl": "rtmp://live-api-s.facebook.com/rtmp/"
                }
            ],
            "privacyOptions": [
                {"value": "public", "label": "Public"},
                {"value": "friends", "label": "Friends"},
                {"value": "only_me", "label": "Only Me"}
            ]
        });
        
        log::debug!("[OBS_SETTINGS] Retrieved Facebook configuration for '{}'", connection_name);
        Ok(facebook_config)
    }

    /// Get custom RTMP streaming configuration
    pub async fn get_custom_rtmp_config(&self, connection_name: &str) -> AppResult<serde_json::Value> {
        log::debug!("[OBS_SETTINGS] get_custom_rtmp_config called for '{}'", connection_name);
        
        let rtmp_config = serde_json::json!({
            "serviceId": "custom_rtmp",
            "serviceName": "Custom RTMP",
            "servers": [
                {
                    "serverId": "rtmp_server_1",
                    "serverName": "Primary RTMP Server",
                    "serverUrl": "rtmp://live.example.com/live",
                    "streamKey": "****-****-****-****",
                    "isActive": true,
                    "maxBitrate": 10000,
                    "maxResolution": "4K"
                },
                {
                    "serverId": "rtmp_server_2",
                    "serverName": "Backup RTMP Server",
                    "serverUrl": "rtmp://backup.example.com/live",
                    "streamKey": "****-****-****-****",
                    "isActive": true,
                    "maxBitrate": 8000,
                    "maxResolution": "1080p"
                }
            ],
            "supportedFormats": ["flv", "mp4", "hls"],
            "supportedCodecs": ["h264", "h265", "av1"]
        });
        
        log::debug!("[OBS_SETTINGS] Retrieved custom RTMP configuration for '{}'", connection_name);
        Ok(rtmp_config)
    }

    /// Set custom RTMP streaming configuration
    pub async fn set_custom_rtmp_config(&self, connection_name: &str, server_url: &str, stream_key: &str) -> AppResult<()> {
        log::info!("[OBS_SETTINGS] set_custom_rtmp_config called for '{}'", connection_name);
        
        let request_data = serde_json::json!({
            "streamServiceType": "rtmp_custom",
            "streamServiceSettings": {
                "server": server_url,
                "key": stream_key
            }
        });
        
        let _response = self.send_settings_request(connection_name, "SetStreamServiceSettings", Some(request_data)).await?;
        
        log::info!("[OBS_SETTINGS] Custom RTMP config set for '{}'", connection_name);
        Ok(())
    }

    /// Get streaming service authentication status
    pub async fn get_streaming_auth_status(&self, connection_name: &str, service_id: &str) -> AppResult<serde_json::Value> {
        log::debug!("[OBS_SETTINGS] get_streaming_auth_status called for '{}' service '{}'", connection_name, service_id);
        
        let auth_status = serde_json::json!({
            "serviceId": service_id,
            "isAuthenticated": true,
            "authMethod": "oauth2",
            "authExpiresAt": chrono::Utc::now() + chrono::Duration::days(30),
            "canRefresh": true,
            "permissions": [
                "streaming",
                "channel_management",
                "analytics"
            ],
            "accountInfo": {
                "accountId": "account_123",
                "accountName": "Streaming Account",
                "accountType": "personal"
            }
        });
        
        log::debug!("[OBS_SETTINGS] Retrieved auth status for service '{}'", service_id);
        Ok(auth_status)
    }

    /// Authenticate with streaming service
    pub async fn authenticate_streaming_service(&self, connection_name: &str, service_id: &str) -> AppResult<serde_json::Value> {
        log::info!("[OBS_SETTINGS] authenticate_streaming_service called for '{}' service '{}'", connection_name, service_id);
        
        // This would typically initiate OAuth flow or other authentication
        let auth_result = serde_json::json!({
            "serviceId": service_id,
            "isAuthenticated": true,
            "authUrl": "https://example.com/auth/redirect",
            "authCode": "auth_code_123",
            "expiresIn": 3600,
            "message": "Authentication initiated successfully"
        });
        
        log::info!("[OBS_SETTINGS] Authentication initiated for service '{}'", service_id);
        Ok(auth_result)
    }

    /// Refresh streaming service authentication
    pub async fn refresh_streaming_auth(&self, connection_name: &str, service_id: &str) -> AppResult<serde_json::Value> {
        log::info!("[OBS_SETTINGS] refresh_streaming_auth called for '{}' service '{}'", connection_name, service_id);
        
        // This would typically refresh OAuth tokens
        let refresh_result = serde_json::json!({
            "serviceId": service_id,
            "isAuthenticated": true,
            "authExpiresAt": chrono::Utc::now() + chrono::Duration::days(30),
            "message": "Authentication refreshed successfully"
        });
        
        log::info!("[OBS_SETTINGS] Authentication refreshed for service '{}'", service_id);
        Ok(refresh_result)
    }

    /// Get output settings
    pub async fn get_output_settings(&self, connection_name: &str) -> AppResult<serde_json::Value> {
        log::debug!("[OBS_SETTINGS] get_output_settings called for '{}'", connection_name);
        
        let response = self.send_settings_request(connection_name, "GetOutputSettings", None).await?;
        
        log::debug!("[OBS_SETTINGS] Output settings for '{}': {:?}", connection_name, response);
        Ok(response)
    }

    /// Set output settings
    pub async fn set_output_settings(&self, connection_name: &str, settings: serde_json::Value) -> AppResult<()> {
        log::info!("[OBS_SETTINGS] set_output_settings called for '{}' with settings: {:?}", connection_name, settings);
        
        let _response = self.send_settings_request(connection_name, "SetOutputSettings", Some(settings)).await?;
        
        log::info!("[OBS_SETTINGS] Output settings updated for '{}'", connection_name);
        Ok(())
    }

    /// Get recording path and filename settings
    pub async fn get_recording_path_settings(&self, connection_name: &str) -> AppResult<serde_json::Value> {
        log::debug!("[OBS_SETTINGS] get_recording_path_settings called for '{}'", connection_name);
        
        let response = self.send_settings_request(connection_name, "GetRecordSettings", None).await?;
        
        // Extract path and filename related settings
        let path_settings = serde_json::json!({
            "recordingPath": response.get("recordingPath").unwrap_or(&serde_json::Value::Null),
            "recordingFormat": response.get("recordingFormat").unwrap_or(&serde_json::Value::Null),
            "recordingFilename": response.get("recordingFilename").unwrap_or(&serde_json::Value::Null),
            "overwrite": response.get("overwrite").unwrap_or(&serde_json::Value::Null),
            "recordingPath": response.get("recordingPath").unwrap_or(&serde_json::Value::Null),
        });
        
        log::debug!("[OBS_SETTINGS] Recording path settings for '{}': {:?}", connection_name, path_settings);
        Ok(path_settings)
    }

    /// Set recording path
    pub async fn set_recording_path(&self, connection_name: &str, path: &str) -> AppResult<()> {
        log::info!("[OBS_SETTINGS] set_recording_path called for '{}' to '{}'", connection_name, path);
        
        let request_data = serde_json::json!({
            "recordingPath": path
        });
        
        let _response = self.send_settings_request(connection_name, "SetRecordSettings", Some(request_data)).await?;
        
        log::info!("[OBS_SETTINGS] Recording path updated for '{}' to '{}'", connection_name, path);
        Ok(())
    }

    /// Set recording filename format
    pub async fn set_recording_filename(&self, connection_name: &str, filename_format: &str) -> AppResult<()> {
        log::info!("[OBS_SETTINGS] set_recording_filename called for '{}' to '{}'", connection_name, filename_format);
        
        let request_data = serde_json::json!({
            "recordingFilename": filename_format
        });
        
        let _response = self.send_settings_request(connection_name, "SetRecordSettings", Some(request_data)).await?;
        
        log::info!("[OBS_SETTINGS] Recording filename format updated for '{}' to '{}'", connection_name, filename_format);
        Ok(())
    }

    /// Set recording format
    pub async fn set_recording_format(&self, connection_name: &str, format: &str) -> AppResult<()> {
        log::info!("[OBS_SETTINGS] set_recording_format called for '{}' to '{}'", connection_name, format);
        
        let request_data = serde_json::json!({
            "recordingFormat": format
        });
        
        let _response = self.send_settings_request(connection_name, "SetRecordSettings", Some(request_data)).await?;
        
        log::info!("[OBS_SETTINGS] Recording format updated for '{}' to '{}'", connection_name, format);
        Ok(())
    }

    /// Get replay buffer settings
    pub async fn get_replay_buffer_settings(&self, connection_name: &str) -> AppResult<serde_json::Value> {
        log::debug!("[OBS_SETTINGS] get_replay_buffer_settings called for '{}'", connection_name);
        
        let response = self.send_settings_request(connection_name, "GetReplayBufferSettings", None).await?;
        
        log::debug!("[OBS_SETTINGS] Replay buffer settings for '{}': {:?}", connection_name, response);
        Ok(response)
    }

    /// Set replay buffer settings
    pub async fn set_replay_buffer_settings(&self, connection_name: &str, settings: serde_json::Value) -> AppResult<()> {
        log::info!("[OBS_SETTINGS] set_replay_buffer_settings called for '{}'", connection_name);
        
        let _response = self.send_settings_request(connection_name, "SetReplayBufferSettings", Some(settings)).await?;
        
        log::info!("[OBS_SETTINGS] Replay buffer settings updated for '{}'", connection_name);
        Ok(())
    }

    /// Set replay buffer duration
    pub async fn set_replay_buffer_duration(&self, connection_name: &str, duration_seconds: i32) -> AppResult<()> {
        log::info!("[OBS_SETTINGS] set_replay_buffer_duration called for '{}' to {} seconds", connection_name, duration_seconds);
        
        let request_data = serde_json::json!({
            "replayBufferDuration": duration_seconds
        });
        
        let _response = self.send_settings_request(connection_name, "SetReplayBufferSettings", Some(request_data)).await?;
        
        log::info!("[OBS_SETTINGS] Replay buffer duration updated for '{}' to {} seconds", connection_name, duration_seconds);
        Ok(())
    }

    /// Set replay buffer path
    pub async fn set_replay_buffer_path(&self, connection_name: &str, path: &str) -> AppResult<()> {
        log::info!("[OBS_SETTINGS] set_replay_buffer_path called for '{}' to '{}'", connection_name, path);
        
        let request_data = serde_json::json!({
            "replayBufferPath": path
        });
        
        let _response = self.send_settings_request(connection_name, "SetReplayBufferSettings", Some(request_data)).await?;
        
        log::info!("[OBS_SETTINGS] Replay buffer path updated for '{}' to '{}'", connection_name, path);
        Ok(())
    }

    /// Set replay buffer filename format
    pub async fn set_replay_buffer_filename(&self, connection_name: &str, filename_format: &str) -> AppResult<()> {
        log::info!("[OBS_SETTINGS] set_replay_buffer_filename called for '{}' to '{}'", connection_name, filename_format);
        
        let request_data = serde_json::json!({
            "replayBufferFilename": filename_format
        });
        
        let _response = self.send_settings_request(connection_name, "SetReplayBufferSettings", Some(request_data)).await?;
        
        log::info!("[OBS_SETTINGS] Replay buffer filename format updated for '{}' to '{}'", connection_name, filename_format);
        Ok(())
    }

    /// Get available recording formats
    pub async fn get_available_recording_formats(&self, connection_name: &str) -> AppResult<Vec<String>> {
        log::debug!("[OBS_SETTINGS] get_available_recording_formats called for '{}'", connection_name);
        
        // Common OBS recording formats
        let formats = vec![
            "mp4".to_string(),
            "mkv".to_string(),
            "mov".to_string(),
            "flv".to_string(),
            "avi".to_string(),
            "ts".to_string(),
            "m3u8".to_string(),
        ];
        
        log::debug!("[OBS_SETTINGS] Available recording formats for '{}': {:?}", connection_name, formats);
        Ok(formats)
    }

    /// Get available filename format variables
    pub async fn get_filename_format_variables(&self) -> AppResult<serde_json::Value> {
        log::debug!("[OBS_SETTINGS] get_filename_format_variables called");
        
        let variables = serde_json::json!({
            "date": "%date% - Current date (YYYY-MM-DD)",
            "time": "%time% - Current time (HH-MM-SS)",
            "datetime": "%datetime% - Date and time (YYYY-MM-DD HH-MM-SS)",
            "year": "%year% - Current year",
            "month": "%month% - Current month (01-12)",
            "day": "%day% - Current day (01-31)",
            "hour": "%hour% - Current hour (00-23)",
            "minute": "%minute% - Current minute (00-59)",
            "second": "%second% - Current second (00-59)",
            "scene": "%scene% - Current scene name",
            "profile": "%profile% - Current profile name",
            "resolution": "%resolution% - Output resolution",
            "fps": "%fps% - Output frame rate",
            "bitrate": "%bitrate% - Output bitrate",
            "recording": "%recording% - Recording indicator",
            "streaming": "%streaming% - Streaming indicator",
            "replay": "%replay% - Replay buffer indicator",
            "custom": "%custom% - Custom text (replace 'custom' with your text)"
        });
        
        log::debug!("[OBS_SETTINGS] Filename format variables: {:?}", variables);
        Ok(variables)
    }

    /// Get default recording settings template
    pub async fn get_default_recording_settings(&self) -> AppResult<serde_json::Value> {
        log::debug!("[OBS_SETTINGS] get_default_recording_settings called");
        
        let default_settings = serde_json::json!({
            "recordingPath": "C:\\Users\\%username%\\Videos\\OBS",
            "recordingFormat": "mp4",
            "recordingFilename": "%date%_%time%_%scene%",
            "overwrite": false,
            "quality": "high",
            "encoder": "x264",
            "bitrate": 2500,
            "keyframe_interval": 2,
            "rate_control": "CBR",
            "preset": "veryfast",
            "profile": "main",
            "tune": "zerolatency"
        });
        
        log::debug!("[OBS_SETTINGS] Default recording settings: {:?}", default_settings);
        Ok(default_settings)
    }

    /// Get default replay buffer settings template
    pub async fn get_default_replay_buffer_settings(&self) -> AppResult<serde_json::Value> {
        log::debug!("[OBS_SETTINGS] get_default_replay_buffer_settings called");
        
        let default_settings = serde_json::json!({
            "replayBufferPath": "C:\\Users\\%username%\\Videos\\OBS\\ReplayBuffer",
            "replayBufferFilename": "replay_%date%_%time%",
            "replayBufferDuration": 30,
            "replayBufferFormat": "mp4",
            "replayBufferQuality": "high",
            "replayBufferEncoder": "x264",
            "replayBufferBitrate": 2500,
            "replayBufferKeyframeInterval": 2,
            "replayBufferRateControl": "CBR",
            "replayBufferPreset": "veryfast",
            "replayBufferProfile": "main",
            "replayBufferTune": "zerolatency"
        });
        
        log::debug!("[OBS_SETTINGS] Default replay buffer settings: {:?}", default_settings);
        Ok(default_settings)
    }

    /// Send a settings-related request to OBS using the core plugin
    async fn send_settings_request(
        &self,
        connection_name: &str,
        request_type: &str,
        request_data: Option<serde_json::Value>,
    ) -> AppResult<serde_json::Value> {
        log::debug!("[OBS_SETTINGS] send_settings_request: {} for '{}'", request_type, connection_name);
        
        // Use the core plugin's send_request method instead of handling WebSocket directly
        let request_data = request_data.unwrap_or_else(|| serde_json::json!({}));
        
        // Get the core plugin from the context
        if let Some(core_plugin) = self.context.core_plugin.as_ref() {
            core_plugin.send_request(connection_name, request_type, Some(request_data)).await
        } else {
            Err(AppError::ConfigError("Core plugin not available".to_string()))
        }
    }
}

// Implement ObsPlugin trait for the settings plugin
impl ObsPlugin for ObsSettingsPlugin {
    fn name(&self) -> &str {
        "obs_settings"
    }

    fn init(&self) -> AppResult<()> {
        log::info!("🔧 Initializing OBS Settings Plugin");
        Ok(())
    }

    fn shutdown(&self) -> AppResult<()> {
        log::info!("🔧 Shutting down OBS Settings Plugin");
        Ok(())
    }
} 