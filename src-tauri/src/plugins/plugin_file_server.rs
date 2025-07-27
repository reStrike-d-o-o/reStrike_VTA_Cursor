use std::path::PathBuf;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::Value;

pub struct FileServerPlugin {
    port: u16,
    asset_path: PathBuf,
}

impl FileServerPlugin {
    pub fn new(port: u16) -> Self {
        let asset_path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("."))
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .join("assets");
        
        Self {
            port,
            asset_path,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        
        println!("📁 File server listening on http://{}", addr);
        
        loop {
            let (socket, addr) = listener.accept().await?;
            let _client_id = format!("{}", addr);
            
            // Handle client in separate task
            let asset_path = self.asset_path.clone();
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(socket, asset_path).await {
                    eprintln!("❌ File server error: {}", e);
                }
            });
        }
    }

    async fn handle_connection(
        mut socket: tokio::net::TcpStream,
        asset_path: PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut buffer = [0; 1024];
        let n = socket.read(&mut buffer).await?;
        
        if n == 0 {
            return Ok(());
        }
        
        let request = String::from_utf8_lossy(&buffer[..n]);
        let lines: Vec<&str> = request.lines().collect();
        
        if lines.is_empty() {
            return Ok(());
        }
        
        let request_line = lines[0];
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        
        if parts.len() < 2 {
            return Ok(());
        }
        
        let method = parts[0];
        let path = parts[1];
        
        if method != "GET" {
            let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
            socket.write_all(response.as_bytes()).await?;
            return Ok(());
        }
        
        // Map URL paths to file paths
        let file_path = match path {
            "/" | "/index.html" => asset_path.join("index.html"),
            "/scoreboard-overlay.html" => asset_path.join("scoreboard-overlay.html"),
            "/player-introduction-overlay.html" => asset_path.join("player-introduction-overlay.html"),
            "/test-scoreboard-fixes.html" => asset_path.join("test-scoreboard-fixes.html"),
            path if path.starts_with("/assets/") => {
                let relative_path = &path[8..]; // Remove "/assets/"
                asset_path.join(relative_path)
            }
            _ => {
                let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                socket.write_all(response.as_bytes()).await?;
                return Ok(());
            }
        };
        
        // Read file content
        match tokio::fs::read(&file_path).await {
            Ok(content) => {
                let content_type = Self::get_content_type(&file_path);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
                    content_type,
                    content.len()
                );
                
                socket.write_all(response.as_bytes()).await?;
                socket.write_all(&content).await?;
            }
            Err(_) => {
                let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                socket.write_all(response.as_bytes()).await?;
            }
        }
        
        Ok(())
    }

    fn get_content_type(file_path: &PathBuf) -> &'static str {
        match file_path.extension().and_then(|s| s.to_str()) {
            Some("html") => "text/html",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("svg") => "image/svg+xml",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("ico") => "image/x-icon",
            Some("json") => "application/json",
            _ => "application/octet-stream",
        }
    }

            pub async fn get_status(&self) -> Value {
            serde_json::json!({
                "type": "file_server",
                "port": self.port,
                "address": format!("http://0.0.0.0:{}", self.port),
                "network_accessible": true,
                "connected_clients": 0, // Not tracking individual clients for now
                "asset_path": self.asset_path.to_string_lossy()
            })
        }
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 File server plugin initialized");
    Ok(())
} 