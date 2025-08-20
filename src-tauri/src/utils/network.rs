//! Network utilities (interface discovery and settings application)
use std::net::{IpAddr, Ipv4Addr};
use crate::types::AppResult;
use crate::config::NetworkInterfaceSettings;

/// Network interface information
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub interface_type: InterfaceType,
    pub ip_addresses: Vec<IpAddr>,
    pub subnet_masks: Vec<String>,
    pub default_gateway: Option<String>,
    pub dns_suffix: Option<String>,
    pub media_state: MediaState,
    pub is_up: bool,
    pub is_loopback: bool,
    pub description: Option<String>,
}

/// Network interface types
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum InterfaceType {
    Ethernet,
    WiFi,
    Loopback,
    Bluetooth,
    Virtual,
    Unknown,
}

/// Media state of network interface
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum MediaState {
    Connected,
    Disconnected,
    Unknown,
}

impl From<&str> for InterfaceType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "ethernet" | "eth" | "lan" => InterfaceType::Ethernet,
            "wifi" | "wireless" | "wlan" => InterfaceType::WiFi,
            "loopback" | "lo" => InterfaceType::Loopback,
            "bluetooth" | "bt" => InterfaceType::Bluetooth,
            "virtual" | "vpn" | "tunnel" => InterfaceType::Virtual,
            _ => InterfaceType::Unknown,
        }
    }
}

impl From<&str> for MediaState {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "media disconnected" => MediaState::Disconnected,
            "connected" | "up" => MediaState::Connected,
            _ => MediaState::Unknown,
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
            if !interface.is_up || interface.is_loopback || interface.media_state == MediaState::Disconnected {
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
            // First, try to find a private IPv4 address (local network)
            for ip in &interface.ip_addresses {
                if let IpAddr::V4(ipv4) = ip {
                    // Prefer private addresses for UDP server binding
                    if !ipv4.is_loopback() && ipv4.is_private() {
                        return Ok(*ip);
                    }
                }
            }
            
            // Second, try to find any non-loopback IPv4 address
            for ip in &interface.ip_addresses {
                if let IpAddr::V4(ipv4) = ip {
                    if !ipv4.is_loopback() {
                        return Ok(*ip);
                    }
                }
            }
            
            // Third, try to find any IPv4 address
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
        
        // Use ipconfig /all to get detailed network interface information
        let output = Command::new("ipconfig")
            .arg("/all")
            .output()
            .map_err(|e| crate::types::AppError::IoError(e))?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();
        
        let mut current_interface: Option<NetworkInterface> = None;
        
        for line in lines {
            let line = line.trim();
            
            // Interface name (ends with colon and contains adapter name)
            if line.ends_with(':') && !line.contains("IPv4") && !line.contains("IPv6") && !line.contains("Subnet Mask") && !line.contains("Default Gateway") && !line.contains("DNS Suffix") {
                // Save previous interface if it exists
                if let Some(interface) = current_interface.take() {
                    interfaces.push(interface);
                }
                
                let name = line.trim_end_matches(':').to_string();
                let interface_type = Self::detect_interface_type(&name);
                
                current_interface = Some(NetworkInterface {
                    name,
                    interface_type,
                    ip_addresses: Vec::new(),
                    subnet_masks: Vec::new(),
                    default_gateway: None,
                    dns_suffix: None,
                    media_state: MediaState::Unknown, // Initialize to Unknown, will be set later
                    is_up: false, // Initialize to false, will be set later
                    is_loopback: false,
                    description: None,
                });
            }
            // IPv4 address - enhanced parsing for different formats
            else if line.contains("IPv4") && line.contains(":") {
                if let Some(interface) = &mut current_interface {
                    // Try different parsing approaches
                    let ip_str = if let Some(ip_part) = line.split(':').nth(1) {
                        ip_part.trim()
                    } else {
                        continue;
                    };
                    
                    // Handle cases where there might be additional text after the IP (like "(Preferred)")
                    let clean_ip = ip_str.split_whitespace().next().unwrap_or(ip_str);
                    // Remove any parentheses and their contents
                    let clean_ip = clean_ip.split('(').next().unwrap_or(clean_ip).trim();
                    
                    if let Ok(ip) = clean_ip.parse::<IpAddr>() {
                        interface.ip_addresses.push(ip);
                        interface.is_loopback = ip.is_loopback();
                        // If we have an IP address, this interface is definitely connected
                        // (unless it's a loopback address)
                        if !ip.is_loopback() {
                            interface.media_state = MediaState::Connected;
                            interface.is_up = true;
                        }
                    } else {
                        // Try alternative parsing for different formats
                        if clean_ip.contains('.') {
                            let parts: Vec<&str> = clean_ip.split('.').collect();
                            if parts.len() == 4 {
                                // Try to construct IP manually
                                if let (Ok(a), Ok(b), Ok(c), Ok(d)) = (
                                    parts[0].parse::<u8>(),
                                    parts[1].parse::<u8>(),
                                    parts[2].parse::<u8>(),
                                    parts[3].parse::<u8>()
                                ) {
                                    let ip = IpAddr::V4(Ipv4Addr::new(a, b, c, d));
                                    interface.ip_addresses.push(ip);
                                    interface.is_loopback = ip.is_loopback();
                                    if !ip.is_loopback() {
                                        interface.media_state = MediaState::Connected;
                                        interface.is_up = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Subnet Mask
            else if line.contains("Subnet Mask") && line.contains(":") {
                if let Some(interface) = &mut current_interface {
                    if let Some(mask_str) = line.split(':').nth(1) {
                        let mask_str = mask_str.trim();
                        interface.subnet_masks.push(mask_str.to_string());
                    }
                }
            }
            // Default Gateway
            else if line.contains("Default Gateway") && line.contains(":") {
                if let Some(interface) = &mut current_interface {
                    if let Some(gateway_str) = line.split(':').nth(1) {
                        let gateway_str = gateway_str.trim();
                        if !gateway_str.is_empty() {
                            interface.default_gateway = Some(gateway_str.to_string());
                        }
                    }
                }
            }
            // DNS Suffix
            else if line.contains("DNS Suffix") && line.contains(":") {
                if let Some(interface) = &mut current_interface {
                    if let Some(dns_str) = line.split(':').nth(1) {
                        let dns_str = dns_str.trim();
                        if !dns_str.is_empty() && dns_str != "." {
                            interface.dns_suffix = Some(dns_str.to_string());
                        }
                    }
                }
            }
            // Media State
            else if line.contains("Media State") && line.contains(":") {
                if let Some(interface) = &mut current_interface {
                    if let Some(state_str) = line.split(':').nth(1) {
                        let state_str = state_str.trim();
                        interface.media_state = Self::detect_media_state(state_str);
                        interface.is_up = interface.media_state == MediaState::Connected;
                    }
                }
            }
            // Description
            else if line.contains("Description") && line.contains(":") {
                if let Some(interface) = &mut current_interface {
                    if let Some(desc_str) = line.split(':').nth(1) {
                        let desc_str = desc_str.trim();
                        if !desc_str.is_empty() {
                            interface.description = Some(desc_str.to_string());
                        }
                    }
                }
            }
        }
        
        // Add the last interface if it exists
        if let Some(interface) = current_interface {
            interfaces.push(interface);
        }
        
        // Final pass: ensure interfaces with IP addresses are marked as connected
        for interface in &mut interfaces {
            if !interface.ip_addresses.is_empty() && !interface.ip_addresses.iter().any(|ip| ip.is_loopback()) {
                interface.media_state = MediaState::Connected;
                interface.is_up = true;
            }
            // Fallback: if interface has a gateway but no IP addresses, mark as connected
            else if interface.default_gateway.is_some() && interface.ip_addresses.is_empty() {
                interface.media_state = MediaState::Connected;
                interface.is_up = true;
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
        
        let mut current_interface: Option<NetworkInterface> = None;
        
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
                    subnet_masks: Vec::new(),
                    default_gateway: None,
                    dns_suffix: None,
                    media_state: MediaState::Unknown,
                    is_up: false,
                    is_loopback: false,
                    description: None,
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
            interfaces.push(interface);
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
        } else if name_lower.contains("bluetooth") || name_lower.contains("bt") {
            InterfaceType::Bluetooth
        } else if name_lower.contains("virtual") || name_lower.contains("vpn") || name_lower.contains("tunnel") {
            InterfaceType::Virtual
        } else {
            InterfaceType::Unknown
        }
    }
    
    /// Detect media state from interface information
    fn detect_media_state(name: &str) -> MediaState {
        let name_lower = name.to_lowercase();
        
        if name_lower.contains("media disconnected") {
            MediaState::Disconnected
        } else if name_lower.contains("connected") || name_lower.contains("up") {
            MediaState::Connected
        } else {
            MediaState::Unknown
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
        assert_eq!(InterfaceType::from("bluetooth"), InterfaceType::Bluetooth);
        assert_eq!(InterfaceType::from("virtual"), InterfaceType::Virtual);
        assert_eq!(InterfaceType::from("unknown"), InterfaceType::Unknown);
    }
    
    #[test]
    fn test_media_state_detection() {
        assert_eq!(MediaState::from("media disconnected"), MediaState::Disconnected);
        assert_eq!(MediaState::from("connected"), MediaState::Connected);
        assert_eq!(MediaState::from("up"), MediaState::Connected);
        assert_eq!(MediaState::from("unknown"), MediaState::Unknown);
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