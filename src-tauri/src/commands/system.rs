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
