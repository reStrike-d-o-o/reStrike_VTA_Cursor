use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

mod plugin_license;
mod plugin_obs;
mod plugin_playback;
mod plugin_store;
mod plugin_udp;
mod tauri_commands;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(n) => {
            let request = String::from_utf8_lossy(&buffer[..n]);
            println!("Received request: {}", request);
            
            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"ok\",\"message\":\"reStrike VTA Backend Running\"}";
            stream.write_all(response.as_bytes()).unwrap();
        }
        Err(e) => {
            eprintln!("Error reading from stream: {}", e);
        }
    }
}

fn main() {
    println!("reStrike VTA backend starting...");
    
    // Initialize our plugins (with error handling)
    println!("Initializing plugins...");
    plugin_udp::start_udp_server();
    plugin_obs::connect_obs();
    plugin_license::check_license();
    println!("Plugins initialized successfully");
    
    // Start a simple HTTP server on port 1420
    let listener = TcpListener::bind("127.0.0.1:1420").unwrap();
    println!("Backend server listening on port 1420");
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
