//! Test implementation for the OBS obws plugin

use crate::types::AppResult;
use super::{ObsManager, ObsConnectionConfig};

/// Test basic OBS obws plugin functionality
pub async fn test_obs_obws_plugin() -> AppResult<()> {
    log::info!("🧪 Testing OBS obws plugin...");
    
    // Create a new manager
    let manager = ObsManager::new();
    
    // Test adding a connection
    let config = ObsConnectionConfig {
        name: "test_connection".to_string(),
        host: "localhost".to_string(),
        port: 4455,
        password: None,
        timeout_seconds: 10,
    };
    
    manager.add_connection(config).await?;
    log::info!("✅ Added test connection");
    
    // Test getting connections
    let connections = manager.get_connections().await?;
    log::info!("✅ Found {} connections", connections.len());
    
    // Test connection count
    let count = manager.connection_count().await;
    log::info!("✅ Connection count: {}", count);
    
    log::info!("✅ OBS obws plugin test completed successfully");
    Ok(())
}

/// Test OBS obws plugin with real OBS Studio
pub async fn test_obs_obws_with_real_obs() -> AppResult<()> {
    log::info!("🧪 Testing OBS obws plugin with real OBS Studio...");
    
    // Create a new manager
    let manager = ObsManager::new();
    
    // Add a connection to local OBS
    let config = ObsConnectionConfig {
        name: "local_obs".to_string(),
        host: "localhost".to_string(),
        port: 4455,
        password: None,
        timeout_seconds: 10,
    };
    
    manager.add_connection(config).await?;
    log::info!("✅ Added local OBS connection");
    
    // Try to connect
    match manager.connect("local_obs").await {
        Ok(_) => {
            log::info!("✅ Successfully connected to OBS Studio");
            
            // Test basic operations
            let status = manager.get_status(Some("local_obs")).await?;
            log::info!("✅ OBS Status: {:?}", status);
            
            // Disconnect
            manager.disconnect("local_obs").await?;
            log::info!("✅ Disconnected from OBS Studio");
        }
        Err(e) => {
            log::warn!("⚠️ Could not connect to OBS Studio: {}", e);
            log::info!("ℹ️ Make sure OBS Studio is running with WebSocket enabled");
        }
    }
    
    log::info!("✅ OBS obws plugin real OBS test completed");
    Ok(())
}
