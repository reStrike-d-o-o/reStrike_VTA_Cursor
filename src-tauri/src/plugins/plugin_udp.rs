use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::VecDeque;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use crate::types::{AppError, AppResult};
use crate::plugins::ProtocolManager;

/// Initialize the UDP plugin
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("🔧 Initializing UDP plugin...");
    Ok(())
}

// Re-export the main plugin type
pub type UdpPlugin = UdpServer;

// PSS Event Types based on protocol specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PssEvent {
    // Points events
    Points {
        athlete: u8,      // 1 or 2
        point_type: u8,   // 1=punch, 2=body, 3=head, 4=tech_body, 5=tech_head
    },
    
    // Hit level events
    HitLevel {
        athlete: u8,      // 1 or 2
        level: u8,        // 1-100
    },
    
    // Warnings/Gam-jeom events
    Warnings {
        athlete1_warnings: u8,
        athlete2_warnings: u8,
    },
    
    // Injury time events
    Injury {
        athlete: u8,      // 0=unidentified, 1=athlete1, 2=athlete2
        time: String,     // format: "m:ss"
        action: Option<String>, // show, hide, reset
    },
    
    // Challenge/IVR events
    Challenge {
        source: u8,       // 0=referee, 1=athlete1, 2=athlete2
        accepted: Option<bool>,
        won: Option<bool>,
        canceled: bool,
    },
    
    // Break events
    Break {
        time: String,     // format: "m:ss" or just seconds
        action: Option<String>, // stop, stopEnd
    },
    
    // Winner rounds events
    WinnerRounds {
        round1_winner: u8, // 0=none, 1=athlete1, 2=athlete2
        round2_winner: u8,
        round3_winner: u8,
    },
    
    // Final winner events
    Winner {
        name: String,
        classification: Option<String>,
    },
    
    // Match info events
    Athletes {
        athlete1_short: String,
        athlete1_long: String,
        athlete1_country: String,
        athlete2_short: String,
        athlete2_long: String,
        athlete2_country: String,
    },
    
    // Match configuration
    MatchConfig {
        number: u32,
        category: String,
        weight: String,
        rounds: u8,
        colors: (String, String, String, String), // bg1, fg1, bg2, fg2
        match_id: String,
        division: String,
        total_rounds: u8,
        round_duration: u32, // seconds
        countdown_type: String,
        count_up: u32,
        format: u8,
    },
    
    // Scores
    Scores {
        athlete1_r1: u8,
        athlete2_r1: u8,
        athlete1_r2: u8,
        athlete2_r2: u8,
        athlete1_r3: u8,
        athlete2_r3: u8,
    },
    
    // Current scores
    CurrentScores {
        athlete1_score: u8,
        athlete2_score: u8,
    },
    
    // Clock events
    Clock {
        time: String,     // format: "m:ss"
        action: Option<String>, // start, stop
    },
    
    // Round events
    Round {
        current_round: u8,
    },
    
    // System events
    FightLoaded,
    FightReady,
    
    // Raw message for unrecognized patterns
    Raw(String),
}

#[derive(Debug, Clone)]
pub struct UdpServerConfig {
    pub port: u16,
    pub bind_address: String,
    pub enabled: bool,
    pub auto_start: bool,
}

impl Default for UdpServerConfig {
    fn default() -> Self {
        Self {
            port: 8888,
            bind_address: "127.0.0.1".to_string(),
            enabled: true,
            auto_start: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum UdpServerStatus {
    Stopped,
    Starting,
    Running,
    Error(String),
}

pub struct UdpServer {
    config: UdpServerConfig,
    status: Arc<Mutex<UdpServerStatus>>,
    event_tx: mpsc::UnboundedSender<PssEvent>,
    socket: Arc<Mutex<Option<UdpSocket>>>,
    stats: Arc<Mutex<UdpStats>>,
    protocol_manager: Arc<ProtocolManager>,
    recent_events: Arc<Mutex<VecDeque<PssEvent>>>,
}

#[derive(Debug, Clone, Default)]
pub struct UdpStats {
    pub packets_received: u64,
    pub packets_parsed: u64,
    pub parse_errors: u64,
    pub last_packet_time: Option<std::time::SystemTime>,
    pub connected_clients: usize,
}

impl UdpServer {
    pub fn new(config: UdpServerConfig, event_tx: mpsc::UnboundedSender<PssEvent>, protocol_manager: Arc<ProtocolManager>) -> Self {
        Self {
            config,
            status: Arc::new(Mutex::new(UdpServerStatus::Stopped)),
            event_tx,
            socket: Arc::new(Mutex::new(None)),
            stats: Arc::new(Mutex::new(UdpStats::default())),
            protocol_manager,
            recent_events: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn start(&self, config: &crate::config::types::AppConfig) -> AppResult<()> {
        let network_settings = &config.udp.listener.network_interface;
        
        // Determine the best IP address to bind to
        let bind_ip = if network_settings.auto_detect {
            match crate::utils::NetworkDetector::get_best_ip_address(network_settings) {
                Ok(ip) => ip.to_string(),
                Err(e) => {
                    println!("⚠️ Failed to auto-detect network interface: {}", e);
                    self.config.bind_address.clone()
                }
            }
        } else {
            self.config.bind_address.clone()
        };
        
        let bind_addr = format!("{}:{}", bind_ip, self.config.port);
        
        // Update status to starting
        {
            let mut status = self.status.lock().unwrap();
            *status = UdpServerStatus::Starting;
        }

        // Try to bind the socket
        let socket = match UdpSocket::bind(&bind_addr) {
            Ok(socket) => {
                socket.set_nonblocking(false).map_err(|e| AppError::ConfigError(e.to_string()))?;
                socket
            }
            Err(e) => {
                let error_msg = format!("Failed to bind UDP socket to {}: {}", bind_addr, e);
                let mut status = self.status.lock().unwrap();
                *status = UdpServerStatus::Error(error_msg.clone());
                return Err(AppError::ConfigError(error_msg));
            }
        };

        // Store the socket
        {
            let mut socket_guard = self.socket.lock().unwrap();
            *socket_guard = Some(socket);
        }

        // Update status to running
        {
            let mut status = self.status.lock().unwrap();
            *status = UdpServerStatus::Running;
        }

        // Start the listening thread
        let socket_clone = self.socket.clone();
        let event_tx = self.event_tx.clone();
        let status_clone = self.status.clone();
        let stats_clone = self.stats.clone();
        let protocol_manager = self.protocol_manager.clone();
        let recent_events_clone = self.recent_events.clone();

        thread::spawn(move || {
            Self::listen_loop(socket_clone, event_tx, status_clone, stats_clone, protocol_manager, recent_events_clone);
        });

        println!("🎯 UDP PSS Server started on {}", bind_addr);
        Ok(())
    }

    pub fn stop(&self) -> AppResult<()> {
        // Clear the socket (this will break the listening loop)
        {
            let mut socket_guard = self.socket.lock().unwrap();
            *socket_guard = None;
        }

        // Update status
        {
            let mut status = self.status.lock().unwrap();
            *status = UdpServerStatus::Stopped;
        }

        println!("🎯 UDP PSS Server stopped");
        Ok(())
    }

    pub fn get_status(&self) -> UdpServerStatus {
        let status = self.status.lock().unwrap();
        status.clone()
    }

    pub fn get_stats(&self) -> UdpStats {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }

    pub fn get_recent_events(&self) -> Vec<PssEvent> {
        let events = self.recent_events.lock().unwrap();
        events.iter().cloned().collect()
    }

    pub fn add_event(&self, event: PssEvent) {
        let mut events = self.recent_events.lock().unwrap();
        events.push_back(event);
        
        // Keep only the last 100 events
        if events.len() > 100 {
            events.pop_front();
        }
    }

    fn listen_loop(
        socket: Arc<Mutex<Option<UdpSocket>>>,
        event_tx: mpsc::UnboundedSender<PssEvent>,
        status: Arc<Mutex<UdpServerStatus>>,
        stats: Arc<Mutex<UdpStats>>,
        protocol_manager: Arc<ProtocolManager>,
        recent_events: Arc<Mutex<VecDeque<PssEvent>>>,
    ) {
        let mut buffer = [0; 1024];

        loop {
            // Check if socket is still available and get a reference
            let socket_ref = {
                let socket_guard = socket.lock().unwrap();
                if socket_guard.is_some() {
                    true
                } else {
                    false
                }
            };

            if !socket_ref {
                // Socket has been removed, stop listening
                break;
            }

            // Receive data (we need to access socket directly)
            let recv_result = {
                let socket_guard = socket.lock().unwrap();
                if let Some(ref s) = *socket_guard {
                    s.recv_from(&mut buffer)
                } else {
                    break;
                }
            };

            match recv_result {
                Ok((size, _addr)) => {
                    let data = String::from_utf8_lossy(&buffer[..size]);
                    let message = data.trim().to_string();

                    // Update stats
                    {
                        let mut stats_guard = stats.lock().unwrap();
                        stats_guard.packets_received += 1;
                        stats_guard.last_packet_time = Some(std::time::SystemTime::now());
                    }

                    // Parse and send the event
                    match Self::parse_pss_message(&message, &protocol_manager) {
                        Ok(event) => {
                            // Update parse stats
                            {
                                let mut stats_guard = stats.lock().unwrap();
                                stats_guard.packets_parsed += 1;
                            }

                            // Add event to recent events storage
                            {
                                let mut events_guard = recent_events.lock().unwrap();
                                events_guard.push_back(event.clone());
                                
                                // Keep only the last 100 events
                                if events_guard.len() > 100 {
                                    events_guard.pop_front();
                                }
                            }

                            // Send event (ignore errors if no receiver)
                            if let Err(_) = event_tx.send(event) {
                                // Don't break the loop, just log the warning
                                println!("⚠️ Failed to send PSS event - receiver may have been dropped");
                                // Continue listening for more packets
                            }
                        }
                        Err(e) => {
                            // Update error stats
                            {
                                let mut stats_guard = stats.lock().unwrap();
                                stats_guard.parse_errors += 1;
                            }
                            
                            println!("⚠️ Failed to parse PSS message '{}': {}", message, e);
                            
                            // Create raw event and add to storage
                            let raw_event = PssEvent::Raw(message.clone());
                            
                            // Add raw event to recent events storage
                            {
                                let mut events_guard = recent_events.lock().unwrap();
                                events_guard.push_back(raw_event.clone());
                                
                                // Keep only the last 100 events
                                if events_guard.len() > 100 {
                                    events_guard.pop_front();
                                }
                            }
                            
                            // Send raw message as fallback (ignore errors if no receiver)
                            if let Err(_) = event_tx.send(raw_event) {
                                // Don't break the loop, just continue
                            }
                        }
                    }
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        let error_msg = format!("UDP receive error: {}", e);
                        println!("❌ {}", error_msg);
                        
                        let mut status_guard = status.lock().unwrap();
                        *status_guard = UdpServerStatus::Error(error_msg);
                        break;
                    }
                }
            }
        }

        println!("🎯 UDP PSS Server listening loop ended");
    }

    fn parse_pss_message(message: &str, protocol_manager: &ProtocolManager) -> AppResult<PssEvent> {
        let parts: Vec<&str> = message.split(';').collect();
        
        if parts.is_empty() {
            return Err(AppError::ConfigError("Empty message".to_string()));
        }

        // Get protocol parsing rules from the protocol manager
        let protocol_rules = match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // We're in an async context, use block_in_place
                handle.block_on(async {
                    protocol_manager.get_parsing_rules().await
                })
            }
            Err(_) => {
                // We're not in an async context, create a new runtime
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    protocol_manager.get_parsing_rules().await
                })
            }
        }.unwrap_or_default();

        // TODO: Use protocol_rules for validation and enhanced parsing
        // For now, we'll use the existing parsing logic but log protocol usage
        if !protocol_rules.is_empty() {
            log::debug!("Using protocol rules for parsing: {:?}", protocol_rules);
        }

        match parts[0] {
            // Points events
            "pt1" => {
                if parts.len() >= 2 {
                    let point_type = parts[1].parse::<u8>()
                        .map_err(|_| AppError::ConfigError(format!("Invalid point type: {}", parts[1])))?;
                    Ok(PssEvent::Points { athlete: 1, point_type })
                } else {
                    Err(AppError::ConfigError("Missing point type for pt1".to_string()))
                }
            }
            "pt2" => {
                if parts.len() >= 2 {
                    let point_type = parts[1].parse::<u8>()
                        .map_err(|_| AppError::ConfigError(format!("Invalid point type: {}", parts[1])))?;
                    Ok(PssEvent::Points { athlete: 2, point_type })
                } else {
                    Err(AppError::ConfigError("Missing point type for pt2".to_string()))
                }
            }

            // Hit level events
            "hl1" => {
                if parts.len() >= 2 {
                    let level = parts[1].parse::<u8>()
                        .map_err(|_| AppError::ConfigError(format!("Invalid hit level: {}", parts[1])))?;
                    Ok(PssEvent::HitLevel { athlete: 1, level })
                } else {
                    Err(AppError::ConfigError("Missing hit level for hl1".to_string()))
                }
            }
            "hl2" => {
                if parts.len() >= 2 {
                    let level = parts[1].parse::<u8>()
                        .map_err(|_| AppError::ConfigError(format!("Invalid hit level: {}", parts[1])))?;
                    Ok(PssEvent::HitLevel { athlete: 2, level })
                } else {
                    Err(AppError::ConfigError("Missing hit level for hl2".to_string()))
                }
            }

            // Warning events (wg1;1;wg2;2;)
            "wg1" => {
                // This is a complex parsing as it includes both athletes
                // Expected format: wg1;1;wg2;2;
                if parts.len() >= 4 && parts[2] == "wg2" {
                    let athlete1_warnings = parts[1].parse::<u8>()
                        .map_err(|_| AppError::ConfigError(format!("Invalid athlete1 warnings: {}", parts[1])))?;
                    let athlete2_warnings = parts[3].parse::<u8>()
                        .map_err(|_| AppError::ConfigError(format!("Invalid athlete2 warnings: {}", parts[3])))?;
                    
                    Ok(PssEvent::Warnings { athlete1_warnings, athlete2_warnings })
                } else {
                    Err(AppError::ConfigError("Invalid warning format".to_string()))
                }
            }

            // Injury events
            "ij0" | "ij1" | "ij2" => {
                let athlete = match parts[0] {
                    "ij0" => 0,
                    "ij1" => 1,
                    "ij2" => 2,
                    _ => return Err(AppError::ConfigError("Invalid injury athlete".to_string())),
                };

                if parts.len() >= 2 {
                    let time = parts[1].to_string();
                    let action = if parts.len() >= 3 {
                        Some(parts[2].to_string())
                    } else {
                        None
                    };

                    Ok(PssEvent::Injury { athlete, time, action })
                } else {
                    Err(AppError::ConfigError("Missing injury time".to_string()))
                }
            }

            // Challenge events
            "ch0" | "ch1" | "ch2" => {
                let source = match parts[0] {
                    "ch0" => 0,
                    "ch1" => 1,
                    "ch2" => 2,
                    _ => return Err(AppError::ConfigError("Invalid challenge source".to_string())),
                };

                let (accepted, won, canceled) = match parts.len() {
                    1 => (None, None, false),
                    2 => {
                        if parts[1] == "-1" {
                            (None, None, true)
                        } else {
                            let acc = parts[1].parse::<u8>().ok().map(|v| v == 1);
                            (acc, None, false)
                        }
                    }
                    3 => {
                        let acc = parts[1].parse::<u8>().ok().map(|v| v == 1);
                        let won_val = parts[2].parse::<u8>().ok().map(|v| v == 1);
                        (acc, won_val, false)
                    }
                    _ => (None, None, false),
                };

                Ok(PssEvent::Challenge { source, accepted, won, canceled })
            }

            // Break events
            "brk" => {
                if parts.len() >= 2 {
                    let time = parts[1].to_string();
                    let action = if parts.len() >= 3 {
                        Some(parts[2].to_string())
                    } else {
                        None
                    };

                    Ok(PssEvent::Break { time, action })
                } else {
                    Err(AppError::ConfigError("Missing break time".to_string()))
                }
            }

            // Winner rounds
            "wrd" => {
                // Expected format: wrd;rd1;0;rd2;0;rd3;0
                if parts.len() >= 7 && parts[1] == "rd1" && parts[3] == "rd2" && parts[5] == "rd3" {
                    let round1_winner = parts[2].parse::<u8>()
                        .map_err(|_| AppError::ConfigError(format!("Invalid round1 winner: {}", parts[2])))?;
                    let round2_winner = parts[4].parse::<u8>()
                        .map_err(|_| AppError::ConfigError(format!("Invalid round2 winner: {}", parts[4])))?;
                    let round3_winner = parts[6].parse::<u8>()
                        .map_err(|_| AppError::ConfigError(format!("Invalid round3 winner: {}", parts[6])))?;

                    Ok(PssEvent::WinnerRounds { round1_winner, round2_winner, round3_winner })
                } else {
                    Err(AppError::ConfigError("Invalid winner rounds format".to_string()))
                }
            }

            // Final winner
            "wmh" => {
                if parts.len() >= 2 {
                    let name = parts[1].to_string();
                    let classification = if parts.len() >= 3 {
                        Some(parts[2].to_string())
                    } else {
                        None
                    };

                    Ok(PssEvent::Winner { name, classification })
                } else {
                    Err(AppError::ConfigError("Missing winner name".to_string()))
                }
            }

            // Athletes info
            "at1" => {
                // Expected format: at1;short;long;country;at2;short;long;country;
                if parts.len() >= 8 && parts[4] == "at2" {
                    Ok(PssEvent::Athletes {
                        athlete1_short: parts[1].to_string(),
                        athlete1_long: parts[2].to_string(),
                        athlete1_country: parts[3].to_string(),
                        athlete2_short: parts[5].to_string(),
                        athlete2_long: parts[6].to_string(),
                        athlete2_country: parts[7].to_string(),
                    })
                } else {
                    Err(AppError::ConfigError("Invalid athletes format".to_string()))
                }
            }

            // Clock events
            "clk" => {
                if parts.len() >= 2 {
                    let time = parts[1].to_string();
                    let action = if parts.len() >= 3 {
                        Some(parts[2].to_string())
                    } else {
                        None
                    };

                    Ok(PssEvent::Clock { time, action })
                } else {
                    Err(AppError::ConfigError("Missing clock time".to_string()))
                }
            }

            // Round events
            "rnd" => {
                if parts.len() >= 2 {
                    let current_round = parts[1].parse::<u8>()
                        .map_err(|_| AppError::ConfigError(format!("Invalid round: {}", parts[1])))?;
                    Ok(PssEvent::Round { current_round })
                } else {
                    Err(AppError::ConfigError("Missing round number".to_string()))
                }
            }

            // System events
            "pre" => {
                if parts.len() >= 2 && parts[1] == "FightLoaded" {
                    Ok(PssEvent::FightLoaded)
                } else {
                    Ok(PssEvent::Raw(message.to_string()))
                }
            }
            "rdy" => {
                if parts.len() >= 2 && parts[1] == "FightReady" {
                    Ok(PssEvent::FightReady)
                } else {
                    Ok(PssEvent::Raw(message.to_string()))
                }
            }

            // Default: return as raw message
            _ => Ok(PssEvent::Raw(message.to_string())),
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    // Mock protocol manager for testing
    fn create_mock_protocol_manager() -> ProtocolManager {
        ProtocolManager::new().unwrap()
    }

    #[test]
    fn test_parse_points() {
        let protocol_manager = create_mock_protocol_manager();
        let event = UdpServer::parse_pss_message("pt1;3;", &protocol_manager).unwrap();
        match event {
            PssEvent::Points { athlete, point_type } => {
                assert_eq!(athlete, 1);
                assert_eq!(point_type, 3);
            }
            _ => panic!("Expected Points event"),
        }
    }

    #[test]
    fn test_parse_warnings() {
        let protocol_manager = create_mock_protocol_manager();
        let event = UdpServer::parse_pss_message("wg1;1;wg2;2;", &protocol_manager).unwrap();
        match event {
            PssEvent::Warnings { athlete1_warnings, athlete2_warnings } => {
                assert_eq!(athlete1_warnings, 1);
                assert_eq!(athlete2_warnings, 2);
            }
            _ => panic!("Expected Warnings event"),
        }
    }

    #[test]
    fn test_parse_clock() {
        let protocol_manager = create_mock_protocol_manager();
        let event = UdpServer::parse_pss_message("clk;1:23;start;", &protocol_manager).unwrap();
        match event {
            PssEvent::Clock { time, action } => {
                assert_eq!(time, "1:23");
                assert_eq!(action, Some("start".to_string()));
            }
            _ => panic!("Expected Clock event"),
        }
    }
}
