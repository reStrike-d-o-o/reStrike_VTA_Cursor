use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use tokio::sync::mpsc;

mod plugins;
use plugins::plugin_obs::ObsPlugin;
use plugins::plugin_udp::{start_udp_server, PssEvent};

mod commands;

#[tokio::main]
async fn main() {
    println!("🎯 reStrike VTA - Starting Windows Desktop Application...");

    // Create event channel for PSS events
    let (pss_event_tx, mut pss_event_rx) = mpsc::unbounded_channel::<PssEvent>();

    // Start UDP PSS Protocol Server
    println!("🚀 Starting UDP PSS Protocol Server on port 6000...");
    match start_udp_server() {
        Ok(udp_server) => {
            println!("✅ UDP PSS Server started successfully");

            // Log server status
            match udp_server.get_status() {
                plugins::plugin_udp::UdpServerStatus::Running => {
                    println!("📊 UDP Server Status: Running");
                    let stats = udp_server.get_stats();
                    println!(
                        "📈 UDP Server Stats: {} packets received, {} parsed",
                        stats.packets_received, stats.packets_parsed
                    );
                }
                status => {
                    println!("⚠️ UDP Server Status: {:?}", status);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to start UDP PSS Server: {}", e);
            println!("🔧 Make sure port 6000 is available");
        }
    }

    // Initialize OBS Plugin
    println!("🎥 Initializing OBS WebSocket Plugin...");
    let (obs_event_tx, mut obs_event_rx) = mpsc::unbounded_channel();
    let obs_plugin = ObsPlugin::new(obs_event_tx);
    println!("✅ OBS Plugin initialized");

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
    let tcp_listener = TcpListener::bind("127.0.0.1:7878").expect("Failed to bind TCP listener");

    for stream in tcp_listener.incoming() {
        let stream = stream.expect("Failed to accept TCP connection");
        thread::spawn(|| {
            handle_tcp_client(stream);
        });
    }
}

async fn handle_pss_event(event: PssEvent) {
    match event {
        PssEvent::Points {
            athlete,
            point_type,
        } => {
            println!(
                "🥋 Point scored! Athlete {} scored {} points",
                athlete,
                get_point_value(point_type)
            );
            // Here you could trigger OBS recording, save clip, etc.
        }

        PssEvent::HitLevel { athlete, level } => {
            println!("💥 Hit detected! Athlete {} hit level: {}", athlete, level);
            // Trigger video replay if hit level is high enough
            if level >= 80 {
                println!("🎬 High impact hit! Consider saving replay buffer");
            }
        }

        PssEvent::Warnings {
            athlete1_warnings,
            athlete2_warnings,
        } => {
            println!(
                "⚠️ Warnings updated: Athlete 1: {}, Athlete 2: {}",
                athlete1_warnings, athlete2_warnings
            );
        }

        PssEvent::Clock { time, action } => {
            if let Some(action) = action {
                println!("⏰ Clock {}: {}", action, time);
                if action == "stop" {
                    println!("🛑 Match paused - good time for instant replay");
                }
            }
        }

        PssEvent::Winner {
            name,
            classification,
        } => {
            println!("🏆 Winner: {}", name);
            if let Some(class) = classification {
                println!("📊 Classification: {}", class);
            }
            println!("🎬 Match ended - saving final highlights");
        }

        PssEvent::FightLoaded => {
            println!("📋 Fight loaded - ready for competition");
        }

        PssEvent::FightReady => {
            println!("🚀 Fight ready - starting monitoring");
        }

        PssEvent::Athletes {
            athlete1_short,
            athlete1_long,
            athlete1_country,
            athlete2_short,
            athlete2_long,
            athlete2_country,
        } => {
            println!(
                "🥋 Athletes: {} ({}) vs {} ({})",
                athlete1_short, athlete1_country, athlete2_short, athlete2_country
            );
        }

        PssEvent::Raw(message) => {
            println!("📨 Raw PSS message: {}", message);
        }

        _ => {
            println!("📡 PSS Event: {:?}", event);
        }
    }
}

async fn handle_obs_event(event: plugins::plugin_obs::ObsEvent) {
    match event {
        plugins::plugin_obs::ObsEvent::ConnectionStatusChanged {
            connection_name,
            status,
        } => {
            println!(
                "🎥 OBS Connection '{}' status: {:?}",
                connection_name, status
            );
        }

        plugins::plugin_obs::ObsEvent::RecordingStateChanged {
            connection_name,
            is_recording,
        } => {
            if is_recording {
                println!("🔴 OBS '{}' started recording", connection_name);
            } else {
                println!("⏹️ OBS '{}' stopped recording", connection_name);
            }
        }

        plugins::plugin_obs::ObsEvent::ReplayBufferStateChanged {
            connection_name,
            is_active,
        } => {
            if is_active {
                println!("📹 OBS '{}' replay buffer activated", connection_name);
            } else {
                println!("📹 OBS '{}' replay buffer deactivated", connection_name);
            }
        }

        _ => {
            println!("🎥 OBS Event: {:?}", event);
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
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(size) => {
            let request = String::from_utf8_lossy(&buffer[..size]);
            println!("📡 TCP Request: {}", request);

            let response = "HTTP/1.1 200 OK\r\n\r\nreStrike VTA Windows Desktop App - Running!";
            if let Err(e) = stream.write(response.as_bytes()) {
                println!("❌ Failed to send TCP response: {}", e);
            }
        }
        Err(e) => {
            println!("❌ Failed to read TCP stream: {}", e);
        }
    }
}
