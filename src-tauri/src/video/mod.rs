//! Video subsystem modules
//!
//! Purpose: Helpers for recorded video handling, overlays, and clip management.
pub mod player;
pub mod overlay;
pub mod clips;

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    // Reserved for future initialization hooks
    Ok(())
}