use std::net::{IpAddr, Ipv4Addr};
use crate::types::AppResult;
use crate::config::NetworkInterfaceSettings;

/// Network interface information
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub interface_type: InterfaceType,
    pub ip_addresses: Vec<IpAddr>,
    pub is_up: bool,
    pub is_loopback: bool,
}

/// Network interface types
#[derive(Debug, Clone, PartialEq)]
pub enum InterfaceType {
    Ethernet,
    WiFi,
    Loopback,
    Unknown,
}

impl From<&str> for InterfaceType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "ethernet" | "eth" | "lan" => InterfaceType::Ethernet,
            "wifi" | "wireless" | "wlan" => InterfaceType::WiFi,
            "loopback" | "lo" => InterfaceType::Loopback,
            _ => InterfaceType::Unknown,
        }
    }
}

/// Network interface detector
pub struct NetworkDetector;

impl NetworkDetector {
    /// Get all available network interfaces
    pub fn get_interfaces() -> AppResult<Vec<NetworkInterface>> {
        // Use Windows-specific network interface detection
        #[cfg(target_os = "windows")]
        {
            Self::get_windows_interfaces()
        }
        
        // Use Unix-specific network interface detection
        #[cfg(not(target_os = "windows"))]
        {
            Self::get_unix_interfaces()
        }
    }
    
    /// Get the best network interface based on preferences
    pub fn get_best_interface(settings: &NetworkInterfaceSettings) -> AppResult<Option<NetworkInterface>> {
        if !settings.auto_detect {
            // Use manually selected interface
            if let Some(interface_name) = &settings.selected_interface {
                let interfaces = Self::get_interfaces()?;
                return Ok(interfaces.into_iter()
                    .find(|iface| iface.name == *interface_name));
            }
            return Ok(None);
        }
        
        let interfaces = Self::get_interfaces()?;
        let mut candidates = Vec::new();
        
        for interface in interfaces {
            if !interface.is_up || interface.is_loopback {
                continue;
            }
            
            // Check if interface matches preferred type
            let matches_preference = match settings.preferred_type.as_str() {
                "ethernet" => interface.interface_type == InterfaceType::Ethernet,
                "wifi" => interface.interface_type == InterfaceType::WiFi,
                "any" => interface.interface_type != InterfaceType::Loopback,
                _ => true, // Unknown preference, accept all
            };
            
            if matches_preference {
                candidates.push(interface);
            }
        }
        
        // Sort by preference: Ethernet first, then WiFi
        candidates.sort_by(|a, b| {
            match (&a.interface_type, &b.interface_type) {
                (InterfaceType::Ethernet, InterfaceType::WiFi) => std::cmp::Ordering::Less,
                (InterfaceType::WiFi, InterfaceType::Ethernet) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            }
        });
        
        Ok(candidates.into_iter().next())
    }
    
    /// Get the best IP address for binding
    pub fn get_best_ip_address(settings: &NetworkInterfaceSettings) -> AppResult<IpAddr> {
        if let Some(interface) = Self::get_best_interface(settings)? {
            // Prefer IPv4 addresses
            for ip in &interface.ip_addresses {
                if let IpAddr::V4(ipv4) = ip {
                    // Skip localhost and private addresses if we want external
                    if !ipv4.is_loopback() && !ipv4.is_private() {
                        return Ok(*ip);
                    }
                }
            }
            
            // Fallback to any IPv4 address
            for ip in &interface.ip_addresses {
                if let IpAddr::V4(_) = ip {
                    return Ok(*ip);
                }
            }
            
            // Fallback to any address
            if let Some(ip) = interface.ip_addresses.first() {
                return Ok(*ip);
            }
        }
        
        // Fallback to localhost if enabled
        if settings.fallback_to_localhost {
            Ok(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
        } else {
            Err(crate::types::AppError::ConfigError(
                "No suitable network interface found".to_string()
            ))
        }
    }
    
    #[cfg(target_os = "windows")]
    fn get_windows_interfaces() -> AppResult<Vec<NetworkInterface>> {
        use std::process::Command;
        
        let mut interfaces = Vec::new();
        
        // Use ipconfig to get network interface information
        let output = Command::new("ipconfig")
            .output()
            .map_err(|e| crate::types::AppError::IoError(e))?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();
        
        let mut current_interface = None;
        
        for line in lines {
            let line = line.trim();
            
            // Interface name
            if line.ends_with(':') && !line.contains("IPv4") && !line.contains("IPv6") {
                let name = line.trim_end_matches(':').to_string();
                let interface_type = Self::detect_interface_type(&name);
                
                current_interface = Some(NetworkInterface {
                    name,
                    interface_type,
                    ip_addresses: Vec::new(),
                    is_up: true, // Assume up if we can see it
                    is_loopback: false,
                });
            }
            // IPv4 address
            else if line.contains("IPv4") && line.contains(":") {
                if let Some(interface) = &mut current_interface {
                    if let Some(ip_str) = line.split(':').nth(1) {
                        let ip_str = ip_str.trim();
                        if let Ok(ip) = ip_str.parse::<IpAddr>() {
                            interface.ip_addresses.push(ip);
                            interface.is_loopback = ip.is_loopback();
                        }
                    }
                }
            }
            // Empty line or section end
            else if line.is_empty() {
                if let Some(interface) = current_interface.take() {
                    if !interface.ip_addresses.is_empty() {
                        interfaces.push(interface);
                    }
                }
            }
        }
        
        // Add the last interface if it exists
        if let Some(interface) = current_interface {
            if !interface.ip_addresses.is_empty() {
                interfaces.push(interface);
            }
        }
        
        Ok(interfaces)
    }
    
    #[cfg(not(target_os = "windows"))]
    fn get_unix_interfaces() -> AppResult<Vec<NetworkInterface>> {
        use std::process::Command;
        
        let mut interfaces = Vec::new();
        
        // Use ifconfig or ip command to get network interface information
        let output = Command::new("ifconfig")
            .output()
            .or_else(|_| Command::new("ip").arg("addr").output())
            .map_err(|e| crate::types::AppError::IoError(e))?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();
        
        let mut current_interface = None;
        
        for line in lines {
            let line = line.trim();
            
            // Interface name (starts with alphanumeric, ends with colon)
            if line.chars().next().map_or(false, |c| c.is_alphanumeric()) && line.ends_with(':') {
                let name = line.trim_end_matches(':').to_string();
                let interface_type = Self::detect_interface_type(&name);
                
                current_interface = Some(NetworkInterface {
                    name,
                    interface_type,
                    ip_addresses: Vec::new(),
                    is_up: true,
                    is_loopback: false,
                });
            }
            // IP address line
            else if line.contains("inet ") && current_interface.is_some() {
                if let Some(interface) = &mut current_interface {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(ip) = parts[1].parse::<IpAddr>() {
                            interface.ip_addresses.push(ip);
                            interface.is_loopback = ip.is_loopback();
                        }
                    }
                }
            }
        }
        
        // Add the last interface if it exists
        if let Some(interface) = current_interface {
            if !interface.ip_addresses.is_empty() {
                interfaces.push(interface);
            }
        }
        
        Ok(interfaces)
    }
    
    /// Detect interface type from name
    fn detect_interface_type(name: &str) -> InterfaceType {
        let name_lower = name.to_lowercase();
        
        if name_lower.contains("ethernet") || name_lower.contains("eth") || name_lower.starts_with("e") {
            InterfaceType::Ethernet
        } else if name_lower.contains("wireless") || name_lower.contains("wifi") || name_lower.contains("wlan") || name_lower.starts_with("w") {
            InterfaceType::WiFi
        } else if name_lower.contains("loopback") || name_lower == "lo" {
            InterfaceType::Loopback
        } else {
            InterfaceType::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_interface_type_detection() {
        assert_eq!(InterfaceType::from("ethernet"), InterfaceType::Ethernet);
        assert_eq!(InterfaceType::from("wifi"), InterfaceType::WiFi);
        assert_eq!(InterfaceType::from("loopback"), InterfaceType::Loopback);
        assert_eq!(InterfaceType::from("unknown"), InterfaceType::Unknown);
    }
    
    #[test]
    fn test_get_best_ip_address() {
        let settings = NetworkInterfaceSettings {
            auto_detect: true,
            preferred_type: "ethernet".to_string(),
            fallback_to_localhost: true,
            selected_interface: None,
        };
        
        let result = NetworkDetector::get_best_ip_address(&settings);
        assert!(result.is_ok());
        
        let ip = result.unwrap();
        assert!(ip.is_ipv4());
    }
} 