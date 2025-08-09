use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use crate::types::{AppError, AppResult};
use crate::plugins::plugin_udp::PssEvent;
use crate::plugins::obs_obws::ObsPathGenerator;
use crate::plugins::obs_obws::manager::ObsManager;
use crate::database::operations::{TournamentOperations, PssUdpOperations};
use chrono::Utc;

/// Recording session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordingState {
    Idle,
    Preparing,      // Generating path, setting up OBS
    Recording,      // Actively recording
    Stopping,       // Stopping recording
    Error(String),  // Error state with message
}

/// Recording session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSession {
    pub id: Option<i64>,
    pub match_id: String,
    pub tournament_name: Option<String>,
    pub tournament_day: Option<String>,
    pub match_number: Option<String>,
    pub player1_name: Option<String>,
    pub player1_flag: Option<String>,
    pub player2_name: Option<String>,
    pub player2_flag: Option<String>,
    pub recording_path: Option<String>,
    pub recording_filename: Option<String>,
    pub state: RecordingState,
    pub start_time: Option<chrono::DateTime<Utc>>,
    pub end_time: Option<chrono::DateTime<Utc>>,
    pub obs_connection_name: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// Configuration for automatic recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomaticRecordingConfig {
    pub enabled: bool,
    pub obs_connection_name: Option<String>,
    pub auto_stop_on_match_end: bool,
    pub auto_stop_on_winner: bool,
    pub stop_delay_seconds: u32,
    pub include_replay_buffer: bool,
}

impl Default for AutomaticRecordingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            obs_connection_name: None,
            auto_stop_on_match_end: true,
            auto_stop_on_winner: true,
            stop_delay_seconds: 30,
            include_replay_buffer: true,
        }
    }
}

/// Event types for recording control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordingEvent {
    StartRecording {
        match_id: String,
        obs_connection_name: String,
    },
    StopRecording {
        session_id: i64,
        obs_connection_name: String,
    },
    UpdateState {
        session_id: i64,
        state: RecordingState,
    },
    Error {
        session_id: Option<i64>,
        error: String,
    },
}

/// OBS Recording Event Handler
pub struct ObsRecordingEventHandler {
    config: Arc<Mutex<AutomaticRecordingConfig>>,
    pub current_session: Arc<Mutex<Option<RecordingSession>>>,
    pub event_tx: mpsc::UnboundedSender<RecordingEvent>,
    path_generator: ObsPathGenerator,
    database: Arc<crate::plugins::plugin_database::DatabasePlugin>,
    obs_manager: Arc<ObsManager>,
    last_applied_directory_day: Arc<Mutex<Option<String>>>,
}

impl ObsRecordingEventHandler {
    pub fn new(
        config: AutomaticRecordingConfig,
        event_tx: mpsc::UnboundedSender<RecordingEvent>,
        database: Arc<crate::plugins::plugin_database::DatabasePlugin>,
        obs_manager: Arc<ObsManager>,
    ) -> Self {
        let path_generator = ObsPathGenerator::new(None);
        
        Self {
            config: Arc::new(Mutex::new(config)),
            current_session: Arc::new(Mutex::new(None)),
            event_tx,
            path_generator,
            database,
            obs_manager,
            last_applied_directory_day: Arc::new(Mutex::new(None)),
        }
    }

    /// Handle PSS events and trigger recording actions
    pub async fn handle_pss_event(&self, event: &PssEvent) -> AppResult<()> {
        let config = {
            let config_guard = self.config.lock().unwrap();
            config_guard.clone()
        };

        if !config.enabled {
            return Ok(());
        }

        match event {
            // Match loaded - prepare recording
            PssEvent::FightLoaded => {
                log::info!("🎬 FightLoaded event received - preparing recording session");
                self.handle_fight_loaded().await?;
            }

            // Match ready - start recording
            PssEvent::FightReady => {
                log::info!("🎬 FightReady event received - starting recording");
                self.handle_fight_ready().await?;
            }

            // Clock start - ensure recording is active
            PssEvent::Clock { action: Some(action), .. } if action == "start" => {
                log::info!("🎬 Clock start event received - ensuring recording is active");
                self.handle_clock_start().await?;
            }

            // Clock stop - consider stopping recording
            PssEvent::Clock { action: Some(action), .. } if action == "stop" => {
                log::info!("🎬 Clock stop event received - considering recording stop");
                self.handle_clock_stop().await?;
            }

            // Winner event - stop recording
            PssEvent::Winner { .. } => {
                if config.auto_stop_on_winner {
                    log::info!("🎬 Winner event received - stopping recording");
                    self.handle_winner().await?;
                }
            }

            // Match end indicators
            PssEvent::WinnerRounds { .. } => {
                if config.auto_stop_on_match_end {
                    log::info!("🎬 WinnerRounds event received - considering recording stop");
                    self.handle_match_end().await?;
                }
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle FightLoaded event - prepare recording session
    async fn handle_fight_loaded(&self) -> AppResult<()> {
        // Get current match ID from UDP context
        let match_id = self.get_current_match_id().await?;
        
        if let Some(match_id) = match_id {
            let config = {
                let config_guard = self.config.lock().unwrap();
                config_guard.clone()
            };

            // Create new recording session
            let session = RecordingSession {
                id: None,
                match_id: match_id.clone(),
                tournament_name: None,
                tournament_day: None,
                match_number: None,
                player1_name: None,
                player1_flag: None,
                player2_name: None,
                player2_flag: None,
                recording_path: None,
                recording_filename: None,
                state: RecordingState::Preparing,
                start_time: None,
                end_time: None,
                obs_connection_name: config.obs_connection_name.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            // Update current session
            {
                let mut session_guard = self.current_session.lock().unwrap();
                *session_guard = Some(session.clone());
            }

            // Generate recording path
            if let Err(e) = self.generate_recording_path(&match_id).await {
                log::error!("Failed to generate recording path: {}", e);
                self.update_session_state(RecordingState::Error(e.to_string())).await?;
            }

            // Apply directory at the start of tournament day if changed
            if let Some(session) = self.get_current_session() {
                let day_key = session.tournament_day.clone().unwrap_or_default();
                let should_apply = {
                    let last = self.last_applied_directory_day.lock().unwrap();
                    last.as_deref() != Some(&day_key)
                };
                if should_apply {
                    if let (Some(dir), Some(conn_name)) = (session.recording_path.clone(), session.obs_connection_name.clone()) {
                        // Release the mutex before awaiting
                        match self.obs_manager.set_record_directory(&dir, Some(&conn_name)).await {
                            Ok(()) => {
                                let mut last = self.last_applied_directory_day.lock().unwrap();
                                *last = Some(day_key);
                                log::info!("📁 Applied recording directory to OBS: {}", dir);
                            }
                            Err(e) => log::warn!("Failed to set record directory in OBS: {}", e),
                        }
                    }
                }
            }

            log::info!("🎬 Recording session prepared for match: {}", match_id);
        }

        Ok(())
    }

    /// Handle FightReady event - start recording
    async fn handle_fight_ready(&self) -> AppResult<()> {
        let config = {
            let config_guard = self.config.lock().unwrap();
            config_guard.clone()
        };

        if let Some(connection_name) = config.obs_connection_name {
            // Update filename formatting per match before starting recording
            if let Some(session) = self.get_current_session() {
                if let Some(template) = self.get_active_filename_template().await? {
                    let formatting = self.build_filename_formatting(&template, &session);
                    if let Err(e) = self.obs_manager.set_filename_formatting(&formatting, Some(&connection_name)).await {
                        log::warn!("Failed to set filename formatting: {}", e);
                    } else {
                        log::info!("🧾 Applied filename formatting to OBS: {}", formatting);
                    }
                }
            }

            // Update session state to recording
            self.update_session_state(RecordingState::Recording).await?;

            // Send start recording event
            if let Err(e) = self.event_tx.send(RecordingEvent::StartRecording {
                match_id: self.get_current_match_id().await?.unwrap_or_default(),
                obs_connection_name: connection_name.clone(),
            }) {
                log::error!("Failed to send start recording event: {}", e);
            }

            log::info!("🎬 Recording started for connection: {}", connection_name);
        }

        Ok(())
    }

    /// Handle Clock start event - ensure recording is active
    async fn handle_clock_start(&self) -> AppResult<()> {
        let current_state = {
            let session_guard = self.current_session.lock().unwrap();
            session_guard.as_ref().map(|s| s.state.clone()).unwrap_or(RecordingState::Idle)
        };

        match current_state {
            RecordingState::Preparing => {
                // If still preparing, start recording now
                self.handle_fight_ready().await?;
            }
            RecordingState::Recording => {
                // Already recording, do nothing
                log::debug!("🎬 Already recording, clock start ignored");
            }
            _ => {
                log::warn!("🎬 Clock start received but not in recording state: {:?}", current_state);
            }
        }

        Ok(())
    }

    /// Handle Clock stop event - consider stopping recording
    async fn handle_clock_stop(&self) -> AppResult<()> {
        let _config = {
            let config_guard = self.config.lock().unwrap();
            config_guard.clone()
        };

        // Don't stop immediately, wait for winner or match end
        log::debug!("🎬 Clock stop received, waiting for match end before stopping recording");
        Ok(())
    }

    /// Handle Winner event - stop recording
    async fn handle_winner(&self) -> AppResult<()> {
        let config = {
            let config_guard = self.config.lock().unwrap();
            config_guard.clone()
        };

        if let Some(connection_name) = config.obs_connection_name {
            // Update session state to stopping
            self.update_session_state(RecordingState::Stopping).await?;

            // Send stop recording event
            if let Some(session_id) = self.get_current_session_id().await? {
                if let Err(e) = self.event_tx.send(RecordingEvent::StopRecording {
                    session_id,
                    obs_connection_name: connection_name.clone(),
                }) {
                    log::error!("Failed to send stop recording event: {}", e);
                }
            }

            log::info!("🎬 Recording stopped for connection: {}", connection_name);
        }

        Ok(())
    }

    /// Handle match end - stop recording
    async fn handle_match_end(&self) -> AppResult<()> {
        // Similar to winner event
        self.handle_winner().await
    }

    /// Generate recording path for current match
    pub async fn generate_recording_path(&self, match_id: &str) -> AppResult<()> {
        let conn = self.database.get_connection().await?;

        // Get active tournament and tournament day
        let tournament = TournamentOperations::get_active_tournament(&*conn)?;
        let tournament_day = if let Some(ref tournament) = tournament {
            TournamentOperations::get_active_tournament_day(&*conn, tournament.id.unwrap()).ok()
        } else {
            None
        };

        // Get match details
        let matches = PssUdpOperations::get_pss_matches(&*conn, Some(100))?;
        let match_info = matches.into_iter()
            .find(|m| m.match_id == match_id)
            .ok_or_else(|| AppError::ConfigError(format!("Match not found: {}", match_id)))?;

        // Get match athletes
        let match_athletes = PssUdpOperations::get_pss_match_athletes(&*conn, match_info.id.unwrap())?;

        // Extract player information
        let mut player1_name = None;
        let mut player1_flag = None;
        let mut player2_name = None;
        let mut player2_flag = None;

        for (match_athlete, athlete) in match_athletes {
            match match_athlete.athlete_position {
                1 => {
                    player1_name = Some(athlete.short_name);
                    player1_flag = athlete.country_code;
                },
                2 => {
                    player2_name = Some(athlete.short_name);
                    player2_flag = athlete.country_code;
                },
                _ => {}
            }
        }

        // Generate path
        let generated_path = self.path_generator.generate_recording_path(
            match_id,
            tournament.map(|t| t.name),
            tournament_day.map(|td| format!("Day {}", td.unwrap().day_number)),
            match_info.match_number,
            player1_name.clone(),
            player1_flag.clone(),
            player2_name.clone(),
            player2_flag.clone()
        )?;

        // Ensure directory exists
        self.path_generator.ensure_directory_exists(&generated_path.directory)?;

        // Update session with path information
        {
            let mut session_guard = self.current_session.lock().unwrap();
            if let Some(ref mut session) = *session_guard {
                session.recording_path = Some(generated_path.directory.to_string_lossy().to_string());
                session.recording_filename = Some(generated_path.filename);
                session.tournament_name = generated_path.tournament_name;
                session.tournament_day = generated_path.tournament_day;
                session.match_number = generated_path.match_number;
                session.player1_name = player1_name;
                session.player1_flag = player1_flag;
                session.player2_name = player2_name;
                session.player2_flag = player2_flag;
                session.updated_at = Utc::now();
            }
        }

        log::info!("🎬 Generated recording path: {}", generated_path.full_path.to_string_lossy());
        Ok(())
    }

    async fn get_active_filename_template(&self) -> AppResult<Option<String>> {
        let conn = self.database.get_connection().await?;
        // Default to OBS_REC if unspecified
        let config_name = {
            let cfg = self.config.lock().unwrap();
            cfg.obs_connection_name.clone().unwrap_or_else(|| "OBS_REC".to_string())
        };
        let config = crate::database::operations::ObsRecordingOperations::get_recording_config(&*conn, &config_name).ok().flatten();
        Ok(config.map(|c| c.filename_template))
    }

    fn build_filename_formatting(&self, template: &str, session: &RecordingSession) -> String {
        let mut fmt = template.to_string();
        if let Some(ref n) = session.match_number { fmt = fmt.replace("{matchNumber}", n); }
        if let Some(ref p1) = session.player1_name { fmt = fmt.replace("{player1}", p1); }
        if let Some(ref p2) = session.player2_name { fmt = fmt.replace("{player2}", p2); }
        if let Some(ref f1) = session.player1_flag { fmt = fmt.replace("{player1Flag}", f1); }
        if let Some(ref f2) = session.player2_flag { fmt = fmt.replace("{player2Flag}", f2); }
        // Keep {date} and {time} placeholders intact for OBS to resolve
        fmt
    }

    /// Get current match ID from UDP context
    async fn get_current_match_id(&self) -> AppResult<Option<String>> {
        // This would need to be integrated with the UDP plugin's current match tracking
        // For now, return None - this will be implemented when we integrate with UDP
        Ok(None)
    }

    /// Get current session ID
    pub async fn get_current_session_id(&self) -> AppResult<Option<i64>> {
        let session_guard = self.current_session.lock().unwrap();
        Ok(session_guard.as_ref().and_then(|s| s.id))
    }

    /// Update session state
    pub async fn update_session_state(&self, state: RecordingState) -> AppResult<()> {
        {
            let mut session_guard = self.current_session.lock().unwrap();
            if let Some(ref mut session) = *session_guard {
                session.state = state.clone();
                session.updated_at = Utc::now();
                
                match state {
                    RecordingState::Recording => {
                        session.start_time = Some(Utc::now());
                    }
                    RecordingState::Stopping => {
                        session.end_time = Some(Utc::now());
                    }
                    _ => {}
                }
            }
        }

        log::info!("🎬 Recording session state updated: {:?}", state);
        Ok(())
    }

    /// Update configuration
    pub fn update_config(&self, config: AutomaticRecordingConfig) -> AppResult<()> {
        let mut config_guard = self.config.lock().unwrap();
        *config_guard = config;
        log::info!("🎬 Automatic recording configuration updated");
        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> AutomaticRecordingConfig {
        let config_guard = self.config.lock().unwrap();
        config_guard.clone()
    }

    /// Get current session
    pub fn get_current_session(&self) -> Option<RecordingSession> {
        let session_guard = self.current_session.lock().unwrap();
        session_guard.clone()
    }

    /// Clear current session
    pub fn clear_session(&self) -> AppResult<()> {
        let mut session_guard = self.current_session.lock().unwrap();
        *session_guard = None;
        log::info!("🎬 Recording session cleared");
        Ok(())
    }
}
