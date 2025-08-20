//! PSS subsystem (deprecated placeholder)
//!
//! Note: Actual UDP ingest and event handling lives in `plugins::plugin_udp` and
//! `plugins::event_stream`. This module remains as a thin facade for future use.
pub mod protocol;
pub mod listener;
pub mod events;

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}