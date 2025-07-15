use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use tokio::sync::mpsc;

mod utils;
mod plugins;
mod commands;

use plugins::{udp::UdpPlugin, obs::ObsPlugin, playback::PlaybackPlugin};
use utils::logger::{log_info, log_error, log_warn, create_component_logger};

#[tokio::main]
async fn main() {
    let logger = create_component_logger("Main");
    
    logger.info("🎯 reStrike VTA - Starting Windows Desktop Application...", None);
    
    // Create event channel for PSS events
    let (pss_event_tx, mut pss_event_rx) = mpsc::unbounded_channel::<plugins::plugin_udp::PssEvent>();
    
    // Initialize UDP PSS Protocol Server
    logger.info("🚀 Starting UDP PSS Protocol Server on port 6000...", None);
    match UdpPlugin::new("0.0.0.0:6000") {
        Ok(mut udp_plugin) => {
            logger.info("✅ UDP PSS Server started successfully", None);
            
            // Start UDP server in background
            std::thread::spawn(move || {
                if let Err(e) = udp_plugin.start() {
                    logger.error("Failed to start UDP server", Some(&e.to_string()));
                }
            });
            
            // Monitor UDP server status
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(30));
                    match udp_plugin.get_status() {
                        Ok(status) => {
                            logger.info("📊 UDP Server Status: Running", None);
                            logger.info("📈 UDP Server Stats", Some(&format!("{} packets received, {} parsed", 
                                status.packets_received, status.packets_parsed)));
                        }
                        Err(e) => {
                            logger.warn("⚠️ UDP Server Status", Some(&format!("{:?}", e)));
                        }
                    }
                }
            });
        }
        Err(e) => {
            logger.error("❌ Failed to start UDP PSS Server", Some(&e.to_string()));
            logger.error("🔧 Make sure port 6000 is available", None);
            return;
        }
    }
    
    // Initialize OBS WebSocket Plugin
    logger.info("🎥 Initializing OBS WebSocket Plugin...", None);
    let (obs_event_tx, mut obs_event_rx) = mpsc::unbounded_channel();
    let obs_plugin = ObsPlugin::new(obs_event_tx);
    logger.info("✅ OBS Plugin initialized", None);
    
    // Initialize Playback Plugin
    let playback_plugin = PlaybackPlugin::new();
    
    // Start event processing tasks
    tokio::spawn(async move {
        while let Some(event) = pss_event_rx.recv().await {
            handle_pss_event(event).await;
        }
    });
    
    tokio::spawn(async move {
        while let Some(event) = obs_event_rx.recv().await {
            handle_obs_event(event).await;
        }
    });
    
    // Start TCP server for legacy compatibility
    let tcp_listener = TcpListener::bind("127.0.0.1:7878")
        .expect("Failed to bind TCP listener");
    
    for stream in tcp_listener.incoming() {
        let stream = stream.expect("Failed to accept TCP connection");
        thread::spawn(|| {
            handle_tcp_client(stream);
        });
    }
}

async fn handle_pss_event(event: plugins::plugin_udp::PssEvent) {
    let logger = create_component_logger("PSS");
    
    match event {
        plugins::plugin_udp::PssEvent::Points { athlete, point_type } => {
            logger.info("🥋 Point scored!", Some(&format!("Athlete {} scored {} points", athlete, get_point_value(point_type))));
            // Here you could trigger OBS recording, save clip, etc.
        }
        
        plugins::plugin_udp::PssEvent::HitLevel { athlete, level } => {
            logger.info("💥 Hit detected!", Some(&format!("Athlete {} hit level: {}", athlete, level)));
            // Trigger video replay if hit level is high enough
            if level >= 80 {
                logger.info("🎬 High impact hit! Consider saving replay buffer", None);
            }
        }
        
        plugins::plugin_udp::PssEvent::Warnings { athlete1_warnings, athlete2_warnings } => {
            logger.info("⚠️ Warnings updated", Some(&format!("Athlete 1: {}, Athlete 2: {}", athlete1_warnings, athlete2_warnings)));
        }
        
        plugins::plugin_udp::PssEvent::Clock { time, action } => {
            if let Some(action) = action {
                logger.info("⏰ Clock event", Some(&format!("{}: {}", action, time)));
                if action == "stop" {
                    logger.info("🛑 Match paused - good time for instant replay", None);
                }
            }
        }
        
        plugins::plugin_udp::PssEvent::Winner { name, classification } => {
            logger.info("🏆 Winner", Some(&format!("{}", name)));
            if let Some(class) = classification {
                logger.info("📊 Classification", Some(&format!("{}", class)));
            }
            logger.info("🎬 Match ended - saving final highlights", None);
        }
        
        plugins::plugin_udp::PssEvent::FightLoaded => {
            logger.info("📋 Fight loaded - ready for competition", None);
        }
        
        plugins::plugin_udp::PssEvent::FightReady => {
            logger.info("🚀 Fight ready - starting monitoring", None);
        }
        
        plugins::plugin_udp::PssEvent::Athletes { athlete1_short, athlete1_long, athlete1_country, 
                            athlete2_short, athlete2_long, athlete2_country } => {
            logger.info("🥋 Athletes", Some(&format!("{} ({}) vs {} ({})", 
                athlete1_short, athlete1_country, athlete2_short, athlete2_country)));
        }
        
        plugins::plugin_udp::PssEvent::Raw(message) => {
            logger.info("📨 Raw PSS message", Some(&message));
        }
        
        plugins::plugin_udp::PssEvent::Parsed(event_data) => {
            logger.info("📡 PSS Event", Some(&format!("{:?}", event_data)));
        }
    }
}

async fn handle_obs_event(event: plugins::plugin_obs::ObsEvent) {
    let logger = create_component_logger("OBS");
    
    match event {
        plugins::plugin_obs::ObsEvent::ConnectionStatusChanged { connection_name, status } => {
            logger.info("🎥 OBS Connection status", Some(&format!("'{}' status: {:?}", connection_name, status)));
        }
        
        plugins::plugin_obs::ObsEvent::RecordingStateChanged { connection_name, is_recording } => {
            if is_recording {
                logger.info("🔴 OBS started recording", Some(&connection_name));
            } else {
                logger.info("⏹️ OBS stopped recording", Some(&connection_name));
            }
        }
        
        plugins::plugin_obs::ObsEvent::ReplayBufferStateChanged { connection_name, is_active } => {
            if is_active {
                logger.info("📹 OBS replay buffer activated", Some(&connection_name));
            } else {
                logger.info("📹 OBS replay buffer deactivated", Some(&connection_name));
            }
        }
        
        _ => {
            logger.info("🎥 OBS Event", Some(&format!("{:?}", event)));
        }
    }
}

fn get_point_value(point_type: u8) -> &'static str {
    match point_type {
        1 => "Punch (1 point)",
        2 => "Body kick (2 points)",
        3 => "Head kick (3 points)",
        4 => "Technical body kick (3 points)",
        5 => "Technical head kick (4 points)",
        _ => "Unknown point type",
    }
}

fn handle_tcp_client(mut stream: TcpStream) {
    let logger = create_component_logger("TCP");
    let mut buffer = [0; 1024];
    
    match stream.read(&mut buffer) {
        Ok(size) => {
            let request = String::from_utf8_lossy(&buffer[..size]);
            logger.info("📡 TCP Request", Some(&request));

            let response = "HTTP/1.1 200 OK\r\n\r\nreStrike VTA Windows Desktop App - Running!";
            if let Err(e) = stream.write(response.as_bytes()) {
                logger.error("❌ Failed to send TCP response", Some(&e.to_string()));
            }
        }
        Err(e) => {
            logger.error("❌ Failed to read TCP stream", Some(&e.to_string()));
        }
    }
}
