use std::net::{IpAddr, Ipv4Addr};
use re_strike_vta::config::NetworkInterfaceSettings;
use re_strike_vta::utils::NetworkDetector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Network Interface Detection...");
    
    // Test 1: Get all interfaces
    println!("\n1. Getting all network interfaces:");
    match NetworkDetector::get_interfaces() {
        Ok(interfaces) => {
            println!("✅ Found {} network interfaces:", interfaces.len());
            for (i, interface) in interfaces.iter().enumerate() {
                println!("   {}. {} ({:?}) - IPs: {:?}", 
                    i + 1, 
                    interface.name, 
                    interface.interface_type,
                    interface.ip_addresses.iter().map(|ip| ip.to_string()).collect::<Vec<_>>()
                );
            }
        }
        Err(e) => {
            println!("❌ Failed to get interfaces: {}", e);
        }
    }
    
    // Test 2: Get best interface with Ethernet preference
    println!("\n2. Getting best interface (Ethernet preferred):");
    let settings = NetworkInterfaceSettings {
        auto_detect: true,
        preferred_type: "ethernet".to_string(),
        fallback_to_localhost: true,
        selected_interface: None,
    };
    
    match NetworkDetector::get_best_interface(&settings) {
        Ok(Some(interface)) => {
            println!("✅ Best interface: {} ({:?})", interface.name, interface.interface_type);
            println!("   IP addresses: {:?}", 
                interface.ip_addresses.iter().map(|ip| ip.to_string()).collect::<Vec<_>>()
            );
        }
        Ok(None) => {
            println!("⚠️ No suitable interface found");
        }
        Err(e) => {
            println!("❌ Error getting best interface: {}", e);
        }
    }
    
    // Test 3: Get best IP address
    println!("\n3. Getting best IP address:");
    match NetworkDetector::get_best_ip_address(&settings) {
        Ok(ip) => {
            println!("✅ Best IP address: {}", ip);
        }
        Err(e) => {
            println!("❌ Error getting best IP address: {}", e);
        }
    }
    
    // Test 4: Test WiFi preference
    println!("\n4. Getting best interface (WiFi preferred):");
    let wifi_settings = NetworkInterfaceSettings {
        auto_detect: true,
        preferred_type: "wifi".to_string(),
        fallback_to_localhost: true,
        selected_interface: None,
    };
    
    match NetworkDetector::get_best_interface(&wifi_settings) {
        Ok(Some(interface)) => {
            println!("✅ Best WiFi interface: {} ({:?})", interface.name, interface.interface_type);
        }
        Ok(None) => {
            println!("⚠️ No suitable WiFi interface found");
        }
        Err(e) => {
            println!("❌ Error getting best WiFi interface: {}", e);
        }
    }
    
    println!("\n✅ Network detection test completed!");
    Ok(())
} 