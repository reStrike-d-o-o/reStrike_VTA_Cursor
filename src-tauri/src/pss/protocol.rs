//! PSS protocol facade (placeholder)
pub struct PssProtocol;

impl PssProtocol {
    pub async fn new(_state: std::sync::Arc<tokio::sync::RwLock<crate::types::AppState>>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(PssProtocol)
    }
    pub async fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
} 