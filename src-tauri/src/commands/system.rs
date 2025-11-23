use crate::core::app::App;
use anyhow;
use std::sync::Arc;
use tauri::State;
use crate::types::TauriError;

// ============================================================================
// System Commands
// ============================================================================

/// Get the current application status
#[tauri::command]
pub async fn get_app_status(_app: State<'_, Arc<App>>) -> Result<String, TauriError> {
    log::info!("Getting app status");
    Ok("Running".to_string())
}

/// Shutdown the application
#[tauri::command]
pub async fn shutdown_app(app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Shutting down app");
    app.stop()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{e}")))?;
    Ok(())
}

/// Get machine identity information (UID and machine hash)
#[tauri::command]
pub async fn get_machine_identity(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    let uid = machine_uid::get().unwrap_or_else(|_| "unknown".to_string());
    let mh = app
        .license_plugin()
        .compute_machine_hash()
        .unwrap_or_else(|_| "".to_string());
    Ok(serde_json::json!({
        "uid": uid,
        "machine_hash": mh,
    }))
}

/// Get system information
#[tauri::command]
pub async fn system_get_info(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("System get info called");
    Ok(serde_json::json!({
        "success": true,
        "platform": "windows",
        "version": "1.0.0"
    }))
}

// Network interface commands

/// Get all network interfaces
#[tauri::command]
pub async fn get_network_interfaces() -> Result<serde_json::Value, TauriError> {
    match crate::utils::NetworkDetector::get_interfaces() {
        Ok(interfaces) => {
            let interface_data: Vec<serde_json::Value> = interfaces
                .into_iter()
                .map(|iface| {
                    serde_json::json!({
                        "name": iface.name,
                        "type": match iface.interface_type {
                            crate::utils::InterfaceType::Ethernet => "ethernet",
                            crate::utils::InterfaceType::WiFi => "wifi",
                            crate::utils::InterfaceType::Loopback => "loopback",
                            crate::utils::InterfaceType::Bluetooth => "bluetooth",
                            crate::utils::InterfaceType::Virtual => "virtual",
                            crate::utils::InterfaceType::Unknown => "unknown",
                        },
                        "ip_addresses": iface.ip_addresses.iter().map(|ip| ip.to_string()).collect::<Vec<_>>(),
                        "subnet_masks": iface.subnet_masks,
                        "default_gateway": iface.default_gateway,
                        "dns_suffix": iface.dns_suffix,
                        "media_state": match iface.media_state {
                            crate::utils::MediaState::Connected => "connected",
                            crate::utils::MediaState::Disconnected => "disconnected",
                            crate::utils::MediaState::Unknown => "unknown",
                        },
                        "is_up": iface.is_up,
                        "is_loopback": iface.is_loopback,
                        "description": iface.description,
                    })
                })
                .collect();

            Ok(serde_json::json!({
                "success": true,
                "interfaces": interface_data
            }))
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

/// Get the best network interface based on current configuration
#[tauri::command]
pub async fn get_best_network_interface() -> Result<serde_json::Value, TauriError> {
    let settings = crate::config::NetworkInterfaceSettings::default();
    match crate::utils::NetworkDetector::get_best_interface(&settings) {
        Ok(Some(interface)) => Ok(serde_json::json!({
            "success": true,
            "interface": {
                "name": interface.name,
                "type": match interface.interface_type {
                    crate::utils::InterfaceType::Ethernet => "ethernet",
                    crate::utils::InterfaceType::WiFi => "wifi",
                    crate::utils::InterfaceType::Loopback => "loopback",
                    crate::utils::InterfaceType::Bluetooth => "bluetooth",
                    crate::utils::InterfaceType::Virtual => "virtual",
                    crate::utils::InterfaceType::Unknown => "unknown",
                },
                "ip_addresses": interface.ip_addresses.iter().map(|ip| ip.to_string()).collect::<Vec<_>>(),
                "subnet_masks": interface.subnet_masks,
                "default_gateway": interface.default_gateway,
                "dns_suffix": interface.dns_suffix,
                "media_state": match interface.media_state {
                    crate::utils::MediaState::Connected => "connected",
                    crate::utils::MediaState::Disconnected => "disconnected",
                    crate::utils::MediaState::Unknown => "unknown",
                },
                "is_up": interface.is_up,
                "is_loopback": interface.is_loopback,
                "description": interface.description,
            }
        })),
        Ok(None) => Ok(serde_json::json!({
            "success": false,
            "error": "No suitable network interface found"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

/// Get the best IP address for a specific interface
#[tauri::command]
pub async fn get_best_ip_address_for_interface(
    interface_name: String,
) -> Result<serde_json::Value, TauriError> {
    let _settings = crate::config::NetworkInterfaceSettings::default();
    match crate::utils::NetworkDetector::get_interfaces() {
        Ok(interfaces) => {
            // Find the specified interface
            if let Some(interface) = interfaces
                .into_iter()
                .find(|iface| iface.name == interface_name)
            {
                // Get the best IP address for this interface
                let best_ip = interface
                    .ip_addresses
                    .iter()
                    .find(|ip| {
                        if let std::net::IpAddr::V4(ipv4) = ip {
                            // Prefer private addresses for UDP server binding
                            !ipv4.is_loopback() && ipv4.is_private()
                        } else {
                            false
                        }
                    })
                    .or_else(|| {
                        interface.ip_addresses.iter().find(|ip| {
                            if let std::net::IpAddr::V4(ipv4) = ip {
                                !ipv4.is_loopback()
                            } else {
                                false
                            }
                        })
                    })
                    .or_else(|| interface.ip_addresses.first());

                if let Some(ip) = best_ip {
                    Ok(serde_json::json!({
                        "success": true,
                        "ip_address": ip.to_string()
                    }))
                } else {
                    Ok(serde_json::json!({
                        "success": false,
                        "error": "No suitable IP address found for interface"
                    }))
                }
            } else {
                Ok(serde_json::json!({
                    "success": false,
                    "error": "Interface not found"
                }))
            }
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

/// Set window startup position
#[tauri::command]
pub async fn set_window_startup_position(
    window: tauri::Window,
    app: State<'_, Arc<App>>,
) -> Result<(), TauriError> {
    log::info!("Setting window startup position from current window state");
    let position = window.outer_position().map_err(|e| TauriError::from(anyhow::anyhow!("Failed to get window position: {e}")))?;
    
    let mut config = app.config_manager().get_config().await;
    config.ui.layout.window_position.x = position.x;
    config.ui.layout.window_position.y = position.y;
    
    app.config_manager()
        .update_config(config)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to save config: {e}")))?;
    Ok(())
}

/// Set window to fullscreen
#[tauri::command]
pub async fn set_window_fullscreen(window: tauri::Window) -> Result<(), TauriError> {
    log::info!("Setting window to fullscreen");
    window.set_fullscreen(true).map_err(|e| TauriError::from(anyhow::anyhow!("Failed to set fullscreen: {e}")))?;
    Ok(())
}

/// Set window to compact mode
#[tauri::command]
pub async fn set_window_compact(
    window: tauri::Window,
    width: Option<f64>,
    height: Option<f64>,
) -> Result<(), TauriError> {
    log::info!("Setting window to compact mode - received width: {:?}, height: {:?}", width, height);
    
    // Exit fullscreen and unmaximize to ensure we can resize
    window.set_fullscreen(false).map_err(|e| TauriError::from(anyhow::anyhow!("Failed to exit fullscreen: {e}")))?;
    window.unmaximize().map_err(|e| TauriError::from(anyhow::anyhow!("Failed to unmaximize: {e}")))?;
    
    let mut width = width.unwrap_or(350.0);
    let mut height = height.unwrap_or(1080.0);
    
    // MIGRATION FIX: If we receive the old default values (1200x800), force them to the new correct defaults (350x1080)
    // This handles cases where the frontend has cached/persisted the old wrong defaults.
    if (width - 1200.0).abs() < 0.1 && (height - 800.0).abs() < 0.1 {
        log::warn!("Detected old default window size (1200x800). Migrating to new defaults (350x1080).");
        width = 350.0;
        height = 1080.0;
    }
    
    log::info!("Setting window size to: {}x{}", width, height);
    
    window.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }))
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to set window size: {e}")))?;
        
    Ok(())
}

/// Set window to custom size
#[tauri::command]
pub async fn set_window_custom_size(
    window: tauri::Window,
    width: f64,
    height: f64,
) -> Result<(), TauriError> {
    log::info!("Setting window to custom size: {width}x{height}");
    
    // Exit fullscreen and unmaximize to ensure we can resize
    window.set_fullscreen(false).map_err(|e| TauriError::from(anyhow::anyhow!("Failed to exit fullscreen: {e}")))?;
    window.unmaximize().map_err(|e| TauriError::from(anyhow::anyhow!("Failed to unmaximize: {e}")))?;
    
    window.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }))
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to set window size: {e}")))?;
        
    Ok(())
}

/// Get screen size
#[tauri::command]
pub async fn get_screen_size(window: tauri::Window) -> Result<serde_json::Value, TauriError> {
    if let Some(monitor) = window.current_monitor().map_err(|e| TauriError::from(anyhow::anyhow!("Failed to get monitor: {e}")))? {
        let size = monitor.size();
        let scale_factor = monitor.scale_factor();
        Ok(serde_json::json!({
            "width": size.width as f64 / scale_factor,
            "height": size.height as f64 / scale_factor
        }))
    } else {
        Err(TauriError::from(anyhow::anyhow!("No monitor found")))
    }
}

/// Save window settings
#[tauri::command]
pub async fn save_window_settings(
    app: State<'_, Arc<App>>,
    settings: serde_json::Value,
) -> Result<(), TauriError> {
    log::info!("Saving window settings");
    let mut config = app.config_manager().get_config().await;
    
    // Update window size if provided
    if let Some(width) = settings.get("compactWidth").and_then(|v| v.as_f64()) {
        config.ui.layout.window_size.width = width as u32;
    }
    if let Some(height) = settings.get("compactHeight").and_then(|v| v.as_f64()) {
        config.ui.layout.window_size.height = height as u32;
    }
    // Note: fullscreenWidth/fullscreenHeight are not stored separately in config
    // The app uses window_size for both modes
    
    app.config_manager()
        .update_config(config)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to save config: {e}")))?;
        
    Ok(())
}

/// Load window settings
#[tauri::command]
pub async fn load_window_settings(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    let config = app.config_manager().get_config().await;
    // Return current window size for both compact and fullscreen
    // Frontend can use different values, but config stores one size
    Ok(serde_json::json!({
        "compactWidth": config.ui.layout.window_size.width,
        "compactHeight": config.ui.layout.window_size.height,
        "fullscreenWidth": 1920,  // Default fullscreen width
        "fullscreenHeight": 1080, // Default fullscreen height
    }))
}

// ============================================================================
// CPU Monitoring Commands
// ============================================================================

/// Get CPU monitoring status
#[tauri::command]
pub async fn cpu_get_monitoring_status(app: State<'_, Arc<App>>) -> Result<bool, TauriError> {
    app.cpu_monitor_plugin()
        .is_monitoring_enabled()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to get monitoring status: {e}")))
}

/// Enable CPU monitoring
#[tauri::command]
pub async fn cpu_enable_monitoring(app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    app.cpu_monitor_plugin()
        .enable_monitoring()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to enable monitoring: {e}")))
}

/// Disable CPU monitoring
#[tauri::command]
pub async fn cpu_disable_monitoring(app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    app.cpu_monitor_plugin()
        .disable_monitoring()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to disable monitoring: {e}")))
}

/// Get process CPU data
#[tauri::command]
pub async fn cpu_get_process_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    let processes = app.cpu_monitor_plugin().get_process_cpu_data().await;
    Ok(serde_json::json!({
        "success": true,
        "processes": processes
    }))
}

/// Get system CPU data
#[tauri::command]
pub async fn cpu_get_system_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    let system = app.cpu_monitor_plugin().get_system_cpu_data().await;
    Ok(serde_json::json!({
        "success": true,
        "system": system
    }))
}
