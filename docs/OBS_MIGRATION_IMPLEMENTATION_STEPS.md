# OBS WebSocket Migration: Implementation Steps

## Phase 1: Foundation Setup

### Step 1.1: Add obws Dependency

**File**: `src-tauri/Cargo.toml`

```toml
[dependencies]
# Add obws crate with events feature
obws = { version = "0.14.0", features = ["events"] }
```

### Step 1.2: Create New Plugin Structure

**File**: `src-tauri/src/plugins/obs_obws/mod.rs`

```rust
// New OBS plugin using obws crate
pub mod client;
pub mod manager;
pub mod types;
pub mod operations;

pub use client::ObsClient;
pub use manager::ObsManager;
pub use types::*;

// Re-export for backward compatibility
pub use manager::ObsManager as ObsPluginManager;
```

**File**: `src-tauri/src/plugins/obs_obws/types.rs`

```rust
use serde::{Deserialize, Serialize};
use crate::types::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsConnectionConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObsConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Authenticated,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsStatus {
    pub is_recording: bool,
    pub is_streaming: bool,
    pub is_replay_buffer_active: bool,
    pub studio_mode_enabled: bool,
    pub current_scene: Option<String>,
    pub obs_version: Option<String>,
}

// Convert obws errors to AppError
impl From<obws::Error> for AppError {
    fn from(err: obws::Error) -> Self {
        AppError::ConfigError(format!("OBS WebSocket error: {}", err))
    }
}
```

### Step 1.3: Implement Core Client

**File**: `src-tauri/src/plugins/obs_obws/client.rs`

```rust
use obws::Client;
use crate::types::{AppError, AppResult};
use super::types::{ObsConnectionConfig, ObsConnectionStatus, ObsStatus};

pub struct ObsClient {
    client: Option<Client>,
    config: ObsConnectionConfig,
    status: ObsConnectionStatus,
}

impl ObsClient {
    pub fn new(config: ObsConnectionConfig) -> Self {
        Self {
            client: None,
            config,
            status: ObsConnectionStatus::Disconnected,
        }
    }

    pub async fn connect(&mut self) -> AppResult<()> {
        self.status = ObsConnectionStatus::Connecting;
        
        let client = Client::connect(
            &self.config.host,
            self.config.port,
            self.config.password.as_deref(),
        ).await?;
        
        self.client = Some(client);
        self.status = ObsConnectionStatus::Authenticated;
        
        log::info!("✅ Connected to OBS at {}:{}", self.config.host, self.config.port);
        Ok(())
    }
    
    pub async fn disconnect(&mut self) -> AppResult<()> {
        self.client = None;
        self.status = ObsConnectionStatus::Disconnected;
        
        log::info!("✅ Disconnected from OBS at {}:{}", self.config.host, self.config.port);
        Ok(())
    }
    
    pub fn get_status(&self) -> ObsConnectionStatus {
        self.status.clone()
    }
    
    pub fn get_config(&self) -> &ObsConnectionConfig {
        &self.config
    }
    
    // Recording operations
    pub async fn start_recording(&self) -> AppResult<()> {
        if let Some(client) = &self.client {
            client.recording().start().await?;
            log::info!("✅ Recording started");
            Ok(())
        } else {
            Err(AppError::ConfigError("No active OBS connection".to_string()))
        }
    }
    
    pub async fn stop_recording(&self) -> AppResult<()> {
        if let Some(client) = &self.client {
            client.recording().stop().await?;
            log::info!("✅ Recording stopped");
            Ok(())
        } else {
            Err(AppError::ConfigError("No active OBS connection".to_string()))
        }
    }
    
    pub async fn is_recording(&self) -> AppResult<bool> {
        if let Some(client) = &self.client {
            let status = client.recording().status().await?;
            Ok(status.output_active)
        } else {
            Err(AppError::ConfigError("No active OBS connection".to_string()))
        }
    }
    
    // Streaming operations
    pub async fn start_streaming(&self) -> AppResult<()> {
        if let Some(client) = &self.client {
            client.streaming().start().await?;
            log::info!("✅ Streaming started");
            Ok(())
        } else {
            Err(AppError::ConfigError("No active OBS connection".to_string()))
        }
    }
    
    pub async fn stop_streaming(&self) -> AppResult<()> {
        if let Some(client) = &self.client {
            client.streaming().stop().await?;
            log::info!("✅ Streaming stopped");
            Ok(())
        } else {
            Err(AppError::ConfigError("No active OBS connection".to_string()))
        }
    }
    
    pub async fn is_streaming(&self) -> AppResult<bool> {
        if let Some(client) = &self.client {
            let status = client.streaming().status().await?;
            Ok(status.output_active)
        } else {
            Err(AppError::ConfigError("No active OBS connection".to_string()))
        }
    }
    
    // Scene operations
    pub async fn get_current_scene(&self) -> AppResult<String> {
        if let Some(client) = &self.client {
            let scene = client.scenes().current().await?;
            Ok(scene.name)
        } else {
            Err(AppError::ConfigError("No active OBS connection".to_string()))
        }
    }
    
    pub async fn set_current_scene(&self, scene_name: &str) -> AppResult<()> {
        if let Some(client) = &self.client {
            client.scenes().set_current(scene_name).await?;
            log::info!("✅ Scene changed to: {}", scene_name);
            Ok(())
        } else {
            Err(AppError::ConfigError("No active OBS connection".to_string()))
        }
    }
    
    pub async fn get_scenes(&self) -> AppResult<Vec<String>> {
        if let Some(client) = &self.client {
            let scenes = client.scenes().list().await?;
            Ok(scenes.scenes.into_iter().map(|s| s.name).collect())
        } else {
            Err(AppError::ConfigError("No active OBS connection".to_string()))
        }
    }
    
    // Version and status
    pub async fn get_obs_version(&self) -> AppResult<String> {
        if let Some(client) = &self.client {
            let version = client.general().version().await?;
            Ok(format!("{}.{}.{}", 
                version.obs_version.major, 
                version.obs_version.minor, 
                version.obs_version.patch))
        } else {
            Err(AppError::ConfigError("No active OBS connection".to_string()))
        }
    }
    
    pub async fn get_full_status(&self) -> AppResult<ObsStatus> {
        if let Some(client) = &self.client {
            let recording_status = client.recording().status().await?;
            let streaming_status = client.streaming().status().await?;
            let studio_mode = client.general().studio_mode_enabled().await?;
            let current_scene = client.scenes().current().await.ok().map(|s| s.name);
            let version = self.get_obs_version().await.ok();
            
            Ok(ObsStatus {
                is_recording: recording_status.output_active,
                is_streaming: streaming_status.output_active,
                is_replay_buffer_active: false, // TODO: Implement replay buffer
                studio_mode_enabled: studio_mode,
                current_scene,
                obs_version: version,
            })
        } else {
            Err(AppError::ConfigError("No active OBS connection".to_string()))
        }
    }
}
```

### Step 1.4: Implement Manager

**File**: `src-tauri/src/plugins/obs_obws/manager.rs`

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::types::{AppError, AppResult};
use super::client::ObsClient;
use super::types::{ObsConnectionConfig, ObsConnectionStatus, ObsStatus};

pub struct ObsManager {
    clients: Arc<Mutex<HashMap<String, ObsClient>>>,
}

impl ObsManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub async fn add_connection(&self, config: ObsConnectionConfig) -> AppResult<()> {
        let mut clients = self.clients.lock().await;
        
        if clients.contains_key(&config.name) {
            return Err(AppError::ConfigError(format!("Connection '{}' already exists", config.name)));
        }
        
        let client = ObsClient::new(config);
        clients.insert(client.get_config().name.clone(), client);
        
        log::info!("✅ Added OBS connection: {}", config.name);
        Ok(())
    }
    
    pub async fn connect(&self, connection_name: &str) -> AppResult<()> {
        let mut clients = self.clients.lock().await;
        
        if let Some(client) = clients.get_mut(connection_name) {
            client.connect().await
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", connection_name)))
        }
    }
    
    pub async fn disconnect(&self, connection_name: &str) -> AppResult<()> {
        let mut clients = self.clients.lock().await;
        
        if let Some(client) = clients.get_mut(connection_name) {
            client.disconnect().await
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", connection_name)))
        }
    }
    
    pub async fn remove_connection(&self, connection_name: &str) -> AppResult<()> {
        let mut clients = self.clients.lock().await;
        
        if clients.remove(connection_name).is_some() {
            log::info!("✅ Removed OBS connection: {}", connection_name);
            Ok(())
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", connection_name)))
        }
    }
    
    pub async fn get_connection_status(&self, connection_name: &str) -> Option<ObsConnectionStatus> {
        let clients = self.clients.lock().await;
        
        clients.get(connection_name).map(|client| client.get_status())
    }
    
    pub async fn get_connection_names(&self) -> Vec<String> {
        let clients = self.clients.lock().await;
        
        clients.keys().cloned().collect()
    }
    
    // Recording operations
    pub async fn start_recording(&self, connection_name: &str) -> AppResult<()> {
        let mut clients = self.clients.lock().await;
        
        if let Some(client) = clients.get_mut(connection_name) {
            client.start_recording().await
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", connection_name)))
        }
    }
    
    pub async fn stop_recording(&self, connection_name: &str) -> AppResult<()> {
        let mut clients = self.clients.lock().await;
        
        if let Some(client) = clients.get_mut(connection_name) {
            client.stop_recording().await
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", connection_name)))
        }
    }
    
    // Streaming operations
    pub async fn start_streaming(&self, connection_name: &str) -> AppResult<()> {
        let mut clients = self.clients.lock().await;
        
        if let Some(client) = clients.get_mut(connection_name) {
            client.start_streaming().await
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", connection_name)))
        }
    }
    
    pub async fn stop_streaming(&self, connection_name: &str) -> AppResult<()> {
        let mut clients = self.clients.lock().await;
        
        if let Some(client) = clients.get_mut(connection_name) {
            client.stop_streaming().await
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", connection_name)))
        }
    }
    
    // Scene operations
    pub async fn get_current_scene(&self, connection_name: &str) -> AppResult<String> {
        let clients = self.clients.lock().await;
        
        if let Some(client) = clients.get(connection_name) {
            client.get_current_scene().await
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", connection_name)))
        }
    }
    
    pub async fn set_current_scene(&self, connection_name: &str, scene_name: &str) -> AppResult<()> {
        let clients = self.clients.lock().await;
        
        if let Some(client) = clients.get(connection_name) {
            client.set_current_scene(scene_name).await
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", connection_name)))
        }
    }
    
    // Status operations
    pub async fn get_obs_status(&self, connection_name: &str) -> AppResult<ObsStatus> {
        let clients = self.clients.lock().await;
        
        if let Some(client) = clients.get(connection_name) {
            client.get_full_status().await
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", connection_name)))
        }
    }
    
    pub async fn get_obs_version(&self, connection_name: &str) -> AppResult<String> {
        let clients = self.clients.lock().await;
        
        if let Some(client) = clients.get(connection_name) {
            client.get_obs_version().await
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", connection_name)))
        }
    }
}
```

## Phase 2: Integration

### Step 2.1: Update App Structure

**File**: `src-tauri/src/core/app.rs`

```rust
// Add new import
use crate::plugins::obs_obws::ObsManager;

pub struct App {
    // ... existing fields ...
    obs_manager: Arc<ObsManager>, // Replace obs_plugin_manager
}

impl App {
    pub fn new() -> Result<Self, crate::types::AppError> {
        // ... existing initialization ...
        
        // Initialize new OBS manager
        let obs_manager = Arc::new(ObsManager::new());
        log::info!("✅ OBS manager initialized with obws");
        
        Ok(Self {
            // ... existing fields ...
            obs_manager,
        })
    }
    
    // Update getter method
    pub fn obs_plugin(&self) -> &Arc<ObsManager> {
        &self.obs_manager
    }
}
```

### Step 2.2: Update Tauri Commands

**File**: `src-tauri/src/tauri_commands.rs`

```rust
// Update existing commands to use new API

#[command]
pub async fn obs_connect(url: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    // Extract connection name from URL
    let connection_name = if let Some(name) = url.split('/').last() {
        name.to_string()
    } else {
        "default".to_string()
    };
    
    match app.obs_plugin().connect(&connection_name).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("Connected to OBS via {}", connection_name)
        })),
        Err(e) => {
            log::error!("Failed to connect to OBS: {}", e);
            Err(TauriError::from(anyhow::anyhow!("Failed to connect to OBS: {}", e)))
        }
    }
}

#[command]
pub async fn obs_start_recording(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    // Get first available connection
    let connections = app.obs_plugin().get_connection_names().await;
    let connection_name = connections.first()
        .ok_or_else(|| TauriError::from(anyhow::anyhow!("No OBS connections available")))?;
    
    match app.obs_plugin().start_recording(connection_name).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Recording started"
        })),
        Err(e) => {
            log::error!("Failed to start recording: {}", e);
            Err(TauriError::from(anyhow::anyhow!("Failed to start recording: {}", e)))
        }
    }
}

#[command]
pub async fn obs_stop_recording(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    let connections = app.obs_plugin().get_connection_names().await;
    let connection_name = connections.first()
        .ok_or_else(|| TauriError::from(anyhow::anyhow!("No OBS connections available")))?;
    
    match app.obs_plugin().stop_recording(connection_name).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Recording stopped"
        })),
        Err(e) => {
            log::error!("Failed to stop recording: {}", e);
            Err(TauriError::from(anyhow::anyhow!("Failed to stop recording: {}", e)))
        }
    }
}

#[command]
pub async fn obs_start_streaming(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    let connections = app.obs_plugin().get_connection_names().await;
    let connection_name = connections.first()
        .ok_or_else(|| TauriError::from(anyhow::anyhow!("No OBS connections available")))?;
    
    match app.obs_plugin().start_streaming(connection_name).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Streaming started"
        })),
        Err(e) => {
            log::error!("Failed to start streaming: {}", e);
            Err(TauriError::from(anyhow::anyhow!("Failed to start streaming: {}", e)))
        }
    }
}

#[command]
pub async fn obs_stop_streaming(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    let connections = app.obs_plugin().get_connection_names().await;
    let connection_name = connections.first()
        .ok_or_else(|| TauriError::from(anyhow::anyhow!("No OBS connections available")))?;
    
    match app.obs_plugin().stop_streaming(connection_name).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Streaming stopped"
        })),
        Err(e) => {
            log::error!("Failed to stop streaming: {}", e);
            Err(TauriError::from(anyhow::anyhow!("Failed to stop streaming: {}", e)))
        }
    }
}

#[command]
pub async fn obs_get_current_scene(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    let connections = app.obs_plugin().get_connection_names().await;
    let connection_name = connections.first()
        .ok_or_else(|| TauriError::from(anyhow::anyhow!("No OBS connections available")))?;
    
    match app.obs_plugin().get_current_scene(connection_name).await {
        Ok(scene_name) => Ok(serde_json::json!({
            "success": true,
            "scene_name": scene_name
        })),
        Err(e) => {
            log::error!("Failed to get current scene: {}", e);
            Err(TauriError::from(anyhow::anyhow!("Failed to get current scene: {}", e)))
        }
    }
}

#[command]
pub async fn obs_set_current_scene(scene_name: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    let connections = app.obs_plugin().get_connection_names().await;
    let connection_name = connections.first()
        .ok_or_else(|| TauriError::from(anyhow::anyhow!("No OBS connections available")))?;
    
    match app.obs_plugin().set_current_scene(connection_name, &scene_name).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("Scene changed to: {}", scene_name)
        })),
        Err(e) => {
            log::error!("Failed to set current scene: {}", e);
            Err(TauriError::from(anyhow::anyhow!("Failed to set current scene: {}", e)))
        }
    }
}

#[command]
pub async fn obs_get_status(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    let connections = app.obs_plugin().get_connection_names().await;
    let connection_name = connections.first()
        .ok_or_else(|| TauriError::from(anyhow::anyhow!("No OBS connections available")))?;
    
    match app.obs_plugin().get_obs_status(connection_name).await {
        Ok(status) => Ok(serde_json::json!({
            "success": true,
            "status": status
        })),
        Err(e) => {
            log::error!("Failed to get OBS status: {}", e);
            Err(TauriError::from(anyhow::anyhow!("Failed to get OBS status: {}", e)))
        }
    }
}
```

## Phase 3: Testing

### Step 3.1: Create Test Implementation

**File**: `src-tauri/src/plugins/obs_obws/test.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_obs_client_creation() {
        let config = ObsConnectionConfig {
            name: "test".to_string(),
            host: "localhost".to_string(),
            port: 4455,
            password: None,
            enabled: true,
        };
        
        let client = ObsClient::new(config);
        assert_eq!(client.get_status(), ObsConnectionStatus::Disconnected);
    }
    
    #[tokio::test]
    async fn test_obs_manager_creation() {
        let manager = ObsManager::new();
        let connections = manager.get_connection_names().await;
        assert_eq!(connections.len(), 0);
    }
}
```

### Step 3.2: Manual Testing Steps

1. **Add obws dependency** to Cargo.toml
2. **Create new plugin structure** (obs_obws/)
3. **Implement basic client and manager**
4. **Test with OBS Studio**:
   - Start OBS Studio
   - Enable WebSocket server (Tools → WebSocket Server Settings)
   - Set port to 4455
   - Test connection from reStrike VTA
5. **Verify basic operations**:
   - Connect/disconnect
   - Start/stop recording
   - Start/stop streaming
   - Get current scene
   - Set current scene

## Phase 4: Migration

### Step 4.1: Gradual Replacement

1. **Keep old implementation** during testing
2. **Add feature flag** to switch between implementations
3. **Test thoroughly** with new implementation
4. **Remove old implementation** once verified

### Step 4.2: Feature Flag Implementation

**File**: `src-tauri/Cargo.toml`

```toml
[features]
default = ["obs-legacy"]
obs-legacy = []  # Current implementation
obs-obws = []    # New obws implementation
```

**File**: `src-tauri/src/core/app.rs`

```rust
#[cfg(feature = "obs-legacy")]
use crate::plugins::obs::ObsPluginManager;

#[cfg(feature = "obs-obws")]
use crate::plugins::obs_obws::ObsManager;

pub struct App {
    #[cfg(feature = "obs-legacy")]
    obs_plugin_manager: Arc<ObsPluginManager>,
    
    #[cfg(feature = "obs-obws")]
    obs_manager: Arc<ObsManager>,
}

impl App {
    pub fn new() -> Result<Self, crate::types::AppError> {
        #[cfg(feature = "obs-legacy")]
        let obs_plugin_manager = Arc::new(ObsPluginManager::new()?);
        
        #[cfg(feature = "obs-obws")]
        let obs_manager = Arc::new(ObsManager::new());
        
        Ok(Self {
            #[cfg(feature = "obs-legacy")]
            obs_plugin_manager,
            
            #[cfg(feature = "obs-obws")]
            obs_manager,
        })
    }
    
    #[cfg(feature = "obs-legacy")]
    pub fn obs_plugin(&self) -> &Arc<ObsPluginManager> {
        &self.obs_plugin_manager
    }
    
    #[cfg(feature = "obs-obws")]
    pub fn obs_plugin(&self) -> &Arc<ObsManager> {
        &self.obs_manager
    }
}
```

## Phase 5: Cleanup

### Step 5.1: Remove Old Implementation

Once the new implementation is thoroughly tested:

1. **Remove feature flags**
2. **Delete old obs plugin** (`src-tauri/src/plugins/obs/`)
3. **Update all imports** to use new implementation
4. **Clean up unused dependencies**

### Step 5.2: Update Documentation

1. **Update architecture documentation**
2. **Update API documentation**
3. **Update user guides**
4. **Remove references to old implementation**

---

## Testing Checklist

### Basic Functionality
- [ ] Connection to OBS Studio
- [ ] Disconnection from OBS Studio
- [ ] Start recording
- [ ] Stop recording
- [ ] Start streaming
- [ ] Stop streaming
- [ ] Get current scene
- [ ] Set current scene
- [ ] Get OBS version
- [ ] Get connection status

### Error Handling
- [ ] Invalid connection parameters
- [ ] OBS not running
- [ ] Wrong password
- [ ] Network issues
- [ ] OBS WebSocket disabled

### Performance
- [ ] Connection speed
- [ ] Response time for operations
- [ ] Memory usage
- [ ] CPU usage

### Integration
- [ ] Frontend integration
- [ ] Database integration
- [ ] Event system integration
- [ ] Configuration system integration

---

## Rollback Plan

If issues arise during migration:

1. **Keep old implementation** as backup
2. **Use feature flags** to switch back
3. **Document issues** for future resolution
4. **Plan incremental migration** approach

---

*Implementation Status: Ready to Begin*
*Next Step: Add obws dependency and create test implementation*
