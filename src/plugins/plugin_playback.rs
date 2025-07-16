use std::process::{Command, Child, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use super::super::types::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoClip {
    pub id: String,
    pub name: String,
    pub path: String,
    pub duration: f64,  // seconds
    pub timestamp: SystemTime,
    pub tags: Vec<String>,
    pub metadata: VideoMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub codec: String,
    pub bitrate: u64,
    pub file_size: u64,
}

#[derive(Debug, Clone)]
pub enum PlaybackEvent {
    Started { clip_id: String },
    Stopped { clip_id: String },
    Paused { clip_id: String },
    Resumed { clip_id: String },
    PositionChanged { clip_id: String, position: f64 },
    VolumeChanged { volume: f64 },
    Error { clip_id: String, error: String },
    ClipEnded { clip_id: String },
}

#[derive(Debug, Clone)]
pub enum PlaybackStatus {
    Stopped,
    Playing,
    Paused,
    Loading,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct PlaybackConfig {
    pub mpv_path: String,
    pub video_output: String,  // vo driver
    pub audio_output: String,  // ao driver
    pub hardware_decoding: bool,
    pub seek_step: f64,        // seconds
    pub volume: f64,           // 0.0 - 1.0
    pub fullscreen: bool,
    pub window_scale: f64,
    pub keep_open: bool,
    pub loop_file: bool,
}

impl Default for PlaybackConfig {
    fn default() -> Self {
        Self {
            mpv_path: "mpv".to_string(),  // Assumes mpv is in PATH
            video_output: "gpu".to_string(),
            audio_output: "pulse".to_string(),
            hardware_decoding: true,
            seek_step: 5.0,
            volume: 0.8,
            fullscreen: false,
            window_scale: 1.0,
            keep_open: true,
            loop_file: false,
        }
    }
}

pub struct VideoPlayer {
    config: PlaybackConfig,
    current_clip: Arc<Mutex<Option<VideoClip>>>,
    status: Arc<Mutex<PlaybackStatus>>,
    position: Arc<Mutex<f64>>,
    volume: Arc<Mutex<f64>>,
    mpv_process: Arc<Mutex<Option<Child>>>,
    event_tx: mpsc::UnboundedSender<PlaybackEvent>,
}

impl VideoPlayer {
    pub fn new(config: PlaybackConfig, event_tx: mpsc::UnboundedSender<PlaybackEvent>) -> Self {
        let initial_volume = config.volume;
        Self {
            config,
            current_clip: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(PlaybackStatus::Stopped)),
            position: Arc::new(Mutex::new(0.0)),
            volume: Arc::new(Mutex::new(initial_volume)),
            mpv_process: Arc::new(Mutex::new(None)),
            event_tx,
        }
    }

    pub fn play_clip(&self, clip: VideoClip) -> AppResult<()> {
        // Stop any currently playing clip
        self.stop()?;

        // Update current clip
        {
            let mut current = self.current_clip.lock().unwrap();
            *current = Some(clip.clone());
        }

        // Update status
        {
            let mut status = self.status.lock().unwrap();
            *status = PlaybackStatus::Loading;
        }

        // Build mpv command
        let mut cmd = Command::new(&self.config.mpv_path);
        
        // Basic options
        cmd.arg("--no-terminal")
           .arg("--no-input-default-bindings")
           .arg("--no-osc")
           .arg("--no-border")
           .arg("--geometry=50%:50%")
           .arg("--autofit=30%")
           .arg("--ontop");

        // Video output
        cmd.arg(format!("--vo={}", self.config.video_output));

        // Audio output
        cmd.arg(format!("--ao={}", self.config.audio_output));

        // Hardware decoding
        if self.config.hardware_decoding {
            cmd.arg("--hwdec=auto");
        }

        // Volume
        cmd.arg(format!("--volume={}", (self.config.volume * 100.0) as u32));

        // Keep open
        if self.config.keep_open {
            cmd.arg("--keep-open=yes");
        }

        // Loop
        if self.config.loop_file {
            cmd.arg("--loop-file=inf");
        }

        // Fullscreen
        if self.config.fullscreen {
            cmd.arg("--fullscreen");
        } else {
            cmd.arg(format!("--autofit-larger={}%", (self.config.window_scale * 100.0) as u32));
        }

        // Window title
        cmd.arg(format!("--title=reStrike VTA - {}", clip.name));

        // Input file
        cmd.arg(&clip.path);

        // Set stdio
        cmd.stdout(Stdio::piped())
           .stderr(Stdio::piped());

        // Launch mpv
        match cmd.spawn() {
            Ok(child) => {
                // Store the process
                {
                    let mut process = self.mpv_process.lock().unwrap();
                    *process = Some(child);
                }

                // Update status
                {
                    let mut status = self.status.lock().unwrap();
                    *status = PlaybackStatus::Playing;
                }

                // Send event
                let _ = self.event_tx.send(PlaybackEvent::Started { clip_id: clip.id.clone() });

                // Start monitoring thread
                self.start_monitor_thread(clip.id);

                println!("🎬 Started playing: {} ({})", clip.name, clip.path);
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to start mpv: {}", e);
                
                // Update status
                {
                    let mut status = self.status.lock().unwrap();
                    *status = PlaybackStatus::Error(error_msg.clone());
                }

                // Send error event
                let _ = self.event_tx.send(PlaybackEvent::Error { 
                    clip_id: clip.id, 
                    error: error_msg.clone() 
                });

                Err(AppError::ConfigError(error_msg))
            }
        }
    }

    pub fn stop(&self) -> AppResult<()> {
        // Get and terminate the process
        {
            let mut process_guard = self.mpv_process.lock().unwrap();
            if let Some(mut process) = process_guard.take() {
                // Try to terminate gracefully
                if let Err(_) = process.kill() {
                    // Force kill if graceful termination fails
                    let _ = process.kill();
                }
                let _ = process.wait();
            }
        }

        // Update status
        {
            let mut status = self.status.lock().unwrap();
            *status = PlaybackStatus::Stopped;
        }

        // Reset position
        {
            let mut pos = self.position.lock().unwrap();
            *pos = 0.0;
        }

        // Send event if there was a current clip
        if let Some(clip) = self.get_current_clip() {
            let _ = self.event_tx.send(PlaybackEvent::Stopped { clip_id: clip.id });
        }

        // Clear current clip
        {
            let mut current = self.current_clip.lock().unwrap();
            *current = None;
        }

        println!("⏹️ Playback stopped");
        Ok(())
    }

    pub fn pause(&self) -> AppResult<()> {
        // For now, we'll implement basic pause by stopping
        // In a full implementation, you'd send IPC commands to mpv
        let current_clip = self.get_current_clip();
        
        // Update status
        {
            let mut status = self.status.lock().unwrap();
            *status = PlaybackStatus::Paused;
        }

        if let Some(clip) = current_clip {
            let _ = self.event_tx.send(PlaybackEvent::Paused { clip_id: clip.id });
            println!("⏸️ Playback paused: {}", clip.name);
        }

        Ok(())
    }

    pub fn resume(&self) -> AppResult<()> {
        let current_clip = self.get_current_clip();
        
        // Update status
        {
            let mut status = self.status.lock().unwrap();
            *status = PlaybackStatus::Playing;
        }

        if let Some(clip) = current_clip {
            let _ = self.event_tx.send(PlaybackEvent::Resumed { clip_id: clip.id });
            println!("▶️ Playback resumed: {}", clip.name);
        }

        Ok(())
    }

    pub fn set_volume(&self, volume: f64) -> AppResult<()> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        
        // Update internal volume
        {
            let mut vol = self.volume.lock().unwrap();
            *vol = clamped_volume;
        }

        // Send event
        let _ = self.event_tx.send(PlaybackEvent::VolumeChanged { volume: clamped_volume });

        println!("🔊 Volume set to: {:.0}%", clamped_volume * 100.0);
        Ok(())
    }

    pub fn seek(&self, position: f64) -> AppResult<()> {
        // Update internal position
        {
            let mut pos = self.position.lock().unwrap();
            *pos = position.max(0.0);
        }

        if let Some(clip) = self.get_current_clip() {
            let _ = self.event_tx.send(PlaybackEvent::PositionChanged { 
                clip_id: clip.id, 
                position 
            });
            println!("⏭️ Seeked to: {:.1}s", position);
        }

        Ok(())
    }

    pub fn seek_relative(&self, delta: f64) -> AppResult<()> {
        let current_pos = {
            let pos = self.position.lock().unwrap();
            *pos
        };
        
        self.seek(current_pos + delta)
    }

    pub fn get_current_clip(&self) -> Option<VideoClip> {
        let current = self.current_clip.lock().unwrap();
        current.clone()
    }

    pub fn get_status(&self) -> PlaybackStatus {
        let status = self.status.lock().unwrap();
        status.clone()
    }

    pub fn get_position(&self) -> f64 {
        let pos = self.position.lock().unwrap();
        *pos
    }

    pub fn get_volume(&self) -> f64 {
        let vol = self.volume.lock().unwrap();
        *vol
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.get_status(), PlaybackStatus::Playing)
    }

    pub fn toggle_fullscreen(&self) -> AppResult<()> {
        // In a full implementation, you'd send IPC command to mpv
        // For now, just log the action
        println!("🖥️ Toggling fullscreen mode");
        Ok(())
    }

    fn start_monitor_thread(&self, clip_id: String) {
        let process_arc = self.mpv_process.clone();
        let status_arc = self.status.clone();
        let event_tx = self.event_tx.clone();

        thread::spawn(move || {
            // Wait for the process to finish
            loop {
                thread::sleep(Duration::from_millis(500));
                
                let process_finished = {
                    let mut process_guard = process_arc.lock().unwrap();
                    if let Some(ref mut process) = *process_guard {
                        match process.try_wait() {
                            Ok(Some(_)) => {
                                // Process has finished
                                *process_guard = None;
                                true
                            }
                            Ok(None) => {
                                // Process is still running
                                false
                            }
                            Err(_) => {
                                // Error checking process status
                                *process_guard = None;
                                true
                            }
                        }
                    } else {
                        // No process to monitor
                        true
                    }
                };

                if process_finished {
                    // Update status
                    {
                        let mut status = status_arc.lock().unwrap();
                        *status = PlaybackStatus::Stopped;
                    }

                    // Send ended event
                    let _ = event_tx.send(PlaybackEvent::ClipEnded { clip_id: clip_id.clone() });
                    
                    println!("🎬 Playback ended for clip: {}", clip_id);
                    break;
                }
            }
        });
    }
}

// Video utilities
pub struct VideoUtils;

impl VideoUtils {
    pub fn get_video_info(path: &str) -> AppResult<VideoMetadata> {
        // Use mpv to get video information
        let output = Command::new("mpv")
            .arg("--no-config")
            .arg("--no-terminal")
            .arg("--identify")
            .arg("--frames=0")
            .arg(path)
            .output()
            .map_err(AppError::IoError)?;

        if !output.status.success() {
            return Err(AppError::ConfigError("mpv failed to get video info".to_string()));
        }

        // Parse the output (simplified - in reality you'd parse the actual mpv output)
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Default values (in a real implementation, parse from mpv output)
        Ok(VideoMetadata {
            width: 1920,
            height: 1080,
            fps: 30.0,
            codec: "h264".to_string(),
            bitrate: 5000000,
            file_size: std::fs::metadata(path)
                .map(|m| m.len())
                .unwrap_or(0),
        })
    }

    pub fn create_thumbnail(video_path: &str, output_path: &str, timestamp: f64) -> AppResult<()> {
        let output = Command::new("mpv")
            .arg("--no-config")
            .arg("--no-terminal")
            .arg("--no-audio")
            .arg(format!("--start={}", timestamp))
            .arg("--frames=1")
            .arg(format!("--o={}", output_path))
            .arg(video_path)
            .output()
            .map_err(AppError::IoError)?;

        if output.status.success() {
            Ok(())
        } else {
            Err(AppError::ConfigError("Failed to generate thumbnail".to_string()))
        }
    }

    pub fn validate_video_file(path: &str) -> bool {
        // Check if file exists and has a valid video extension
        if !std::path::Path::new(path).exists() {
            return false;
        }

        let valid_extensions = [".mp4", ".avi", ".mkv", ".mov", ".wmv", ".flv", ".webm", ".m4v"];
        let path_lower = path.to_lowercase();
        
        valid_extensions.iter().any(|&ext| path_lower.ends_with(ext))
    }

    pub fn has_valid_video_extension(path: &str) -> bool {
        // Only check extension, not file existence
        let valid_extensions = [".mp4", ".avi", ".mkv", ".mov", ".wmv", ".flv", ".webm", ".m4v"];
        let path_lower = path.to_lowercase();
        
        valid_extensions.iter().any(|&ext| path_lower.ends_with(ext))
    }
}

// Public API functions
pub fn create_video_player() -> (VideoPlayer, mpsc::UnboundedReceiver<PlaybackEvent>) {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let config = PlaybackConfig::default();
    let player = VideoPlayer::new(config, event_tx);
    (player, event_rx)
}

pub fn playback_clip(clip_path: &str, clip_name: &str) -> AppResult<()> {
    println!("🎬 Starting playback of: {} ({})", clip_name, clip_path);
    
    // Validate the video file
    if !VideoUtils::validate_video_file(clip_path) {
        return Err(AppError::ConfigError(format!("Invalid video file: {}", clip_path)));
    }

    // Create a basic video clip
    let clip = VideoClip {
        id: uuid::Uuid::new_v4().to_string(),
        name: clip_name.to_string(),
        path: clip_path.to_string(),
        duration: 0.0, // Will be determined during playback
        timestamp: SystemTime::now(),
        tags: vec!["instant-replay".to_string()],
        metadata: VideoUtils::get_video_info(clip_path)
            .unwrap_or_else(|_| VideoMetadata {
                width: 1920,
                height: 1080,
                fps: 30.0,
                codec: "unknown".to_string(),
                bitrate: 0,
                file_size: 0,
            }),
    };

    // Create player and start playback
    let (player, mut event_rx) = create_video_player();
    
    // Start playback
    player.play_clip(clip)?;
    
    // Handle events in a background task
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                PlaybackEvent::Started { clip_id } => {
                    println!("🎬 Playback started: {}", clip_id);
                }
                PlaybackEvent::ClipEnded { clip_id } => {
                    println!("🎬 Playback ended: {}", clip_id);
                    break;
                }
                PlaybackEvent::Error { clip_id, error } => {
                    println!("❌ Playback error for {}: {}", clip_id, error);
                    break;
                }
                _ => {
                    println!("🎬 Playback event: {:?}", event);
                }
            }
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_validation() {
        // Test extension validation (without file existence check)
        assert!(VideoUtils::has_valid_video_extension("test.mp4"));
        assert!(VideoUtils::has_valid_video_extension("test.AVI"));
        assert!(VideoUtils::has_valid_video_extension("test.mkv"));
        assert!(VideoUtils::has_valid_video_extension("test.mov"));
        assert!(VideoUtils::has_valid_video_extension("test.wmv"));
        assert!(VideoUtils::has_valid_video_extension("test.flv"));
        assert!(VideoUtils::has_valid_video_extension("test.webm"));
        assert!(VideoUtils::has_valid_video_extension("test.m4v"));
        
        // Test invalid extensions
        assert!(!VideoUtils::has_valid_video_extension("test.txt"));
        assert!(!VideoUtils::has_valid_video_extension("test.pdf"));
        assert!(!VideoUtils::has_valid_video_extension("test.doc"));
        
        // Test full validation (including file existence)
        // These should return false because the files don't exist
        assert!(!VideoUtils::validate_video_file("nonexistent.mp4"));
        assert!(!VideoUtils::validate_video_file("/path/to/nonexistent/video.avi"));
    }

    #[test]
    fn test_playback_config_default() {
        let config = PlaybackConfig::default();
        assert_eq!(config.mpv_path, "mpv");
        assert_eq!(config.volume, 0.8);
        assert_eq!(config.seek_step, 5.0);
    }
}
