use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use crate::types::{AppError, AppResult};

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
            port: 6000,
            bind_address: "0.0.0.0".to_string(),
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
    pub fn new(config: UdpServerConfig, event_tx: mpsc::UnboundedSender<PssEvent>) -> Self {
        Self {
            config,
            status: Arc::new(Mutex::new(UdpServerStatus::Stopped)),
            event_tx,
            socket: Arc::new(Mutex::new(None)),
            stats: Arc::new(Mutex::new(UdpStats::default())),
        }
    }

    pub fn start(&self) -> AppResult<()> {
        let bind_addr = format!("{}:{}", self.config.bind_address, self.config.port);
        
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

        thread::spawn(move || {
            Self::listen_loop(socket_clone, event_tx, status_clone, stats_clone);
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

    fn listen_loop(
        socket: Arc<Mutex<Option<UdpSocket>>>,
        event_tx: mpsc::UnboundedSender<PssEvent>,
        status: Arc<Mutex<UdpServerStatus>>,
        stats: Arc<Mutex<UdpStats>>,
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
                Ok((size, addr)) => {
                    let data = String::from_utf8_lossy(&buffer[..size]);
                    let message = data.trim().to_string();

                    // Update stats
                    {
                        let mut stats_guard = stats.lock().unwrap();
                        stats_guard.packets_received += 1;
                        stats_guard.last_packet_time = Some(std::time::SystemTime::now());
                    }

                    // Parse and send the event
                    match Self::parse_pss_message(&message) {
                        Ok(event) => {
                            // Update parse stats
                            {
                                let mut stats_guard = stats.lock().unwrap();
                                stats_guard.packets_parsed += 1;
                            }

                            // Send event
                            if let Err(_) = event_tx.send(event) {
                                println!("⚠️ Failed to send PSS event - receiver may have been dropped");
                                break;
                            }
                        }
                        Err(e) => {
                            // Update error stats
                            {
                                let mut stats_guard = stats.lock().unwrap();
                                stats_guard.parse_errors += 1;
                            }
                            
                            println!("⚠️ Failed to parse PSS message '{}': {}", message, e);
                            
                            // Send raw message as fallback
                            let _ = event_tx.send(PssEvent::Raw(message));
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

    fn parse_pss_message(message: &str) -> AppResult<PssEvent> {
        let parts: Vec<&str> = message.split(';').collect();
        
        if parts.is_empty() {
            return Err(AppError::ConfigError("Empty message".to_string()));
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

// Public API for the plugin
pub fn start_udp_server() -> AppResult<UdpServer> {
    let config = UdpServerConfig::default();
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    
    let server = UdpServer::new(config, event_tx);
    
    // Start the server
    server.start()?;
    
    // Start event processing in a separate task
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            println!("🎯 PSS Event: {:?}", event);
            // Here you can process events, store them, trigger actions, etc.
            // For example, save to database, trigger OBS recordings, etc.
        }
    });
    
    Ok(server)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_points() {
        let event = UdpServer::parse_pss_message("pt1;3;").unwrap();
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
        let event = UdpServer::parse_pss_message("wg1;1;wg2;2;").unwrap();
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
        let event = UdpServer::parse_pss_message("clk;1:23;start;").unwrap();
        match event {
            PssEvent::Clock { time, action } => {
                assert_eq!(time, "1:23");
                assert_eq!(action, Some("start".to_string()));
            }
            _ => panic!("Expected Clock event"),
        }
    }
}
