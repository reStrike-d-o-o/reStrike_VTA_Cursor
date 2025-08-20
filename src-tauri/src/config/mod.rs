//! Configuration subsystem
//!
//! Purpose: App config load/save with backup, and typed config structures
//! consumed across the app.
pub mod manager;
pub mod types;

pub use manager::ConfigManager;
pub use types::*; 