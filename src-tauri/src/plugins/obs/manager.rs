// OBS Plugin Manager
// Coordinates all individual OBS plugins and provides unified interface

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use crate::types::{AppError, AppResult};
use crate::logging::LogManager;
use super::types::*;

// Individual plugin imports (will be created next)
// use super::core::ObsCorePlugin;
// use super::recording::ObsRecordingPlugin;
// use super::streaming::ObsStreamingPlugin;
// use super::scenes::ObsScenesPlugin;
// use super::settings::ObsSettingsPlugin;
// use super::events::ObsEventsPlugin;
// use super::status::ObsStatusPlugin;

/// Main OBS Plugin Manager that coordinates all individual plugins
pub struct ObsPluginManager {
    context: ObsPluginContext,
    plugins: HashMap<String, Box<dyn ObsPlugin>>,
    // Individual plugins will be added here
    // core_plugin: ObsCorePlugin,
    // recording_plugin: ObsRecordingPlugin,
    // streaming_plugin: ObsStreamingPlugin,
    // scenes_plugin: ObsScenesPlugin,
    // settings_plugin: ObsSettingsPlugin,
    // events_plugin: ObsEventsPlugin,
    // status_plugin: ObsStatusPlugin,
}

impl ObsPluginManager {
    /// Create a new OBS Plugin Manager
    pub fn new(event_tx: mpsc::UnboundedSender<ObsEvent>, log_manager: Arc<Mutex<LogManager>>) -> Self {
        let context = ObsPluginContext {
            connections: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
            debug_ws_messages: Arc::new(Mutex::new(true)),
            show_full_events: Arc::new(Mutex::new(false)),
            recent_events: Arc::new(Mutex::new(Vec::new())),
            log_manager,
        };

        Self {
            context,
            plugins: HashMap::new(),
        }
    }

    /// Initialize all OBS plugins
    pub async fn init_plugins(&mut self) -> AppResult<()> {
        log::info!("🔧 Initializing OBS Plugin Manager...");

        // Initialize individual plugins (will be implemented as we create them)
        // self.core_plugin = ObsCorePlugin::new(self.context.clone());
        // self.recording_plugin = ObsRecordingPlugin::new(self.context.clone());
        // self.streaming_plugin = ObsStreamingPlugin::new(self.context.clone());
        // self.scenes_plugin = ObsScenesPlugin::new(self.context.clone());
        // self.settings_plugin = ObsSettingsPlugin::new(self.context.clone());
        // self.events_plugin = ObsEventsPlugin::new(self.context.clone());
        // self.status_plugin = ObsStatusPlugin::new(self.context.clone());

        log::info!("✅ OBS Plugin Manager initialized successfully");
        Ok(())
    }

    /// Shutdown all OBS plugins
    pub async fn shutdown_plugins(&mut self) -> AppResult<()> {
        log::info!("🔧 Shutting down OBS Plugin Manager...");

        // Shutdown individual plugins
        for (name, plugin) in &mut self.plugins {
            if let Err(e) = plugin.shutdown() {
                log::warn!("⚠️ Failed to shutdown plugin '{}': {}", name, e);
            }
        }

        log::info!("✅ OBS Plugin Manager shutdown complete");
        Ok(())
    }

    /// Get the shared plugin context
    pub fn context(&self) -> &ObsPluginContext {
        &self.context
    }

    /// Get mutable access to the shared plugin context
    pub fn context_mut(&mut self) -> &mut ObsPluginContext {
        &mut self.context
    }

    /// Add a plugin to the manager
    pub fn add_plugin(&mut self, name: String, plugin: Box<dyn ObsPlugin>) -> AppResult<()> {
        if self.plugins.contains_key(&name) {
            return Err(AppError::ConfigError(format!("Plugin '{}' already exists", name)));
        }

        if let Err(e) = plugin.init() {
            return Err(AppError::ConfigError(format!("Failed to initialize plugin '{}': {}", name, e)));
        }

        self.plugins.insert(name.clone(), plugin);
        log::info!("✅ Added plugin '{}' to OBS Plugin Manager", name);
        Ok(())
    }

    /// Remove a plugin from the manager
    pub fn remove_plugin(&mut self, name: &str) -> AppResult<()> {
        if let Some(mut plugin) = self.plugins.remove(name) {
            if let Err(e) = plugin.shutdown() {
                log::warn!("⚠️ Failed to shutdown plugin '{}': {}", name, e);
            }
            log::info!("✅ Removed plugin '{}' from OBS Plugin Manager", name);
        }
        Ok(())
    }

    /// Get a list of all registered plugins
    pub fn get_plugin_names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Check if a plugin is registered
    pub fn has_plugin(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    /// Get the number of registered plugins
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
}

impl Clone for ObsPluginManager {
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            plugins: HashMap::new(), // Plugins are not cloned, they need to be re-added
        }
    }
}

// Implement ObsPlugin trait for the manager itself
impl ObsPlugin for ObsPluginManager {
    fn name(&self) -> &str {
        "obs_plugin_manager"
    }

    fn init(&self) -> AppResult<()> {
        log::info!("🔧 Initializing OBS Plugin Manager");
        Ok(())
    }

    fn shutdown(&self) -> AppResult<()> {
        log::info!("🔧 Shutting down OBS Plugin Manager");
        Ok(())
    }
} 