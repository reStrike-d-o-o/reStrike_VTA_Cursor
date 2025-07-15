//! Main application class and lifecycle management

use crate::types::{AppResult, AppState, AppView};
use crate::obs::manager::ObsManager;
use crate::video::player::VideoPlayer;
use crate::pss::protocol::PssProtocol;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main application class that orchestrates all systems
pub struct App {
    state: Arc<RwLock<AppState>>,
    obs_manager: ObsManager,
    video_player: VideoPlayer,
    pss_protocol: PssProtocol,
}

impl App {
    /// Create a new application instance
    pub async fn new() -> AppResult<Self> {
        println!("🚀 Creating new application instance...");
        
        let state = Arc::new(RwLock::new(AppState::default()));
        let obs_manager = ObsManager::new(state.clone()).await?;
        let video_player = VideoPlayer::new(state.clone()).await?;
        let pss_protocol = PssProtocol::new(state.clone()).await?;
        
        Ok(Self {
            state,
            obs_manager,
            video_player,
            pss_protocol,
        })
    }
    
    /// Initialize the application
    pub async fn init(&self) -> AppResult<()> {
        println!("🔧 Initializing application...");
        
        // Initialize all subsystems
        self.obs_manager.init().await?;
        self.video_player.init().await?;
        self.pss_protocol.init().await?;
        
        println!("✅ Application initialized successfully");
        Ok(())
    }
    
    /// Start the application
    pub async fn start(&self) -> AppResult<()> {
        println!("▶️ Starting application...");
        
        // Start all subsystems
        self.obs_manager.start().await?;
        self.video_player.start().await?;
        self.pss_protocol.start().await?;
        
        println!("✅ Application started successfully");
        Ok(())
    }
    
    /// Stop the application
    pub async fn stop(&self) -> AppResult<()> {
        println!("⏹️ Stopping application...");
        
        // Stop all subsystems
        self.obs_manager.stop().await?;
        self.video_player.stop().await?;
        self.pss_protocol.stop().await?;
        
        println!("✅ Application stopped successfully");
        Ok(())
    }
    
    /// Get application state
    pub async fn get_state(&self) -> AppState {
        self.state.read().await.clone()
    }
    
    /// Update current view
    pub async fn set_view(&self, view: AppView) -> AppResult<()> {
        let mut state = self.state.write().await;
        state.current_view = view;
        Ok(())
    }
    
    /// Get OBS manager reference
    pub fn obs_manager(&self) -> &ObsManager {
        &self.obs_manager
    }
    
    /// Get video player reference
    pub fn video_player(&self) -> &VideoPlayer {
        &self.video_player
    }
    
    /// Get PSS protocol reference
    pub fn pss_protocol(&self) -> &PssProtocol {
        &self.pss_protocol
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            obs_connections: Vec::new(),
            active_obs_connection: None,
            obs_status: None,
            overlay_settings: crate::types::OverlaySettings::default(),
            video_clips: Vec::new(),
            current_clip: None,
            is_playing: false,
            current_view: AppView::SidebarTest,
            is_loading: false,
            error: None,
        }
    }
}

impl Default for crate::types::OverlaySettings {
    fn default() -> Self {
        Self {
            opacity: 0.9,
            position: crate::types::OverlayPosition::BottomRight,
            scale: 1.0,
            visible: true,
            theme: crate::types::OverlayTheme::Dark,
        }
    }
} 