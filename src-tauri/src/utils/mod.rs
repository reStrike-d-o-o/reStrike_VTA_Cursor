pub mod logger;
pub mod network;
pub mod simulation_env;

pub use network::*;

/// Generate a new UUID v4 string
pub fn new_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Return current Unix timestamp (seconds since epoch)
pub fn now_unix() -> i64 {
    chrono::Utc::now().timestamp()
}
