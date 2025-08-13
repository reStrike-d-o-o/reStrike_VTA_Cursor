use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use crate::types::{AppError, AppResult};
use crate::plugins::plugin_udp::PssEvent;
use crate::plugins::obs_obws::ObsPathGenerator;
use crate::plugins::obs_obws::types::ObsReplayBufferStatus;
use crate::plugins::obs_obws::PathGeneratorConfig;
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
    // New flags to match frontend UI
    pub auto_start_recording_on_match_begin: bool,
    pub auto_start_replay_on_match_begin: bool,
    // removed: save_replay_on_match_end
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
            auto_start_recording_on_match_begin: true,
            auto_start_replay_on_match_begin: true,
            // removed: save_replay_on_match_end
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
    // Live UDP-provided current match id (from MatchConfig)
    last_udp_match_id: Arc<Mutex<Option<String>>>,
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
            last_udp_match_id: Arc::new(Mutex::new(None)),
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
            // Capture live match id and number as soon as MatchConfig arrives
            PssEvent::MatchConfig { match_id, number, .. } => {
                {
                    let mut guard = self.last_udp_match_id.lock().unwrap();
                    *guard = Some(match_id.clone());
                }
                // Optionally update current session's match number early
                {
                    let mut session_guard = self.current_session.lock().unwrap();
                    if let Some(ref mut session) = *session_guard {
                        session.match_number = Some(number.to_string());
                        session.updated_at = Utc::now();
                    }
                }
                log::info!("📣 MatchConfig received - set current match_id={} number={}", match_id, number);

                // If we don't have a prepared session for this match, prepare now
                let need_prepare = {
                    let session_guard = self.current_session.lock().unwrap();
                    match &*session_guard {
                        Some(s) => s.match_id != *match_id,
                        None => true,
                    }
                };
                if need_prepare {
                    if let Err(e) = self.handle_fight_loaded().await {
                        log::warn!("Failed to prepare session on MatchConfig: {}", e);
                    }
                }
            }
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

            // WinnerRounds should not auto-stop recording per requirement
            PssEvent::WinnerRounds { .. } => {
                log::info!("🎬 WinnerRounds received - no auto-stop per requirement");
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle FightLoaded event - prepare recording session
    async fn handle_fight_loaded(&self) -> AppResult<()> {
        // Get current match ID from UDP/DB context (fallback to most recent)
        let mut match_id = self.get_current_match_id().await?;
        if match_id.is_none() {
            // Fallback: use most recent match from DB
            let conn = self.database.get_connection().await?;
            let latest = PssUdpOperations::get_pss_matches(&*conn, Some(1)).unwrap_or_default();
            match_id = latest.into_iter().next().map(|m| m.match_id);
        }
        
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
            // Ensure replay buffer is running if requested and enabled by UI setting
            // Ensure replay buffer is active before recording (always-on invariant)
            match self.obs_manager.get_replay_buffer_status(Some(&connection_name)).await {
                Ok(ObsReplayBufferStatus::Active) => {
                    log::debug!("Replay buffer already active");
                }
                _ => {
                    if let Err(e) = self.obs_manager.start_replay_buffer(Some(&connection_name)).await {
                        log::warn!("Failed to start replay buffer: {}", e);
                    } else {
                        log::info!("▶️ Replay buffer started to satisfy recording invariant");
                    }
                }
            }

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
            if config.auto_start_recording_on_match_begin {
                self.update_session_state(RecordingState::Recording).await?;
                // Start recording immediately via obws manager (authoritative)
                if let Err(e) = self.obs_manager.start_recording(Some(&connection_name)).await {
                    log::error!("Failed to start recording via obws: {}", e);
                }
                log::info!("🎬 Recording started for connection: {}", connection_name);
            } else {
                log::info!("🎬 Auto-start recording disabled by UI setting; not starting recording on FightReady");
            }
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

            // Respect stop delay seconds before stopping recording
            let delay = std::time::Duration::from_secs(config.stop_delay_seconds as u64);
            if delay.as_secs() > 0 {
                log::info!("⏳ Waiting {}s before stopping recording", delay.as_secs());
                tokio::time::sleep(delay).await;
            }

            // Stop recording immediately via obws manager (authoritative)
            if let Err(e) = self.obs_manager.stop_recording(Some(&connection_name)).await {
                log::error!("Failed to stop recording via obws: {}", e);
            }

            log::info!("🎬 Recording stopped for connection: {}", connection_name);

            // Removed: save replay buffer on match end feature
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

        // Build path generator from active recording config to avoid sending placeholders
        let (videos_root, recording_format, folder_pattern) = {
            use crate::database::operations::ObsRecordingOperations as RecOps;
            // Resolve connection name from auto-config
            let conn_name = {
                let cfg = self.config.lock().unwrap();
                cfg.obs_connection_name.clone().unwrap_or_else(|| "OBS_REC".to_string())
            };
            if let Ok(Some(cfg)) = RecOps::get_recording_config(&*conn, &conn_name) {
                (std::path::PathBuf::from(cfg.recording_root_path), cfg.recording_format, Some(cfg.folder_pattern))
            } else {
                // Fallback to default Videos folder
                (PathGeneratorConfig::detect_windows_videos_folder(), "mp4".to_string(), Some("{tournament}/{tournamentDay}".to_string()))
            }
        };

        let gen_cfg = PathGeneratorConfig {
            videos_root: videos_root.clone(),
            default_format: recording_format,
            include_minutes_seconds: true,
            folder_pattern,
        };
        let path_generator = ObsPathGenerator::new(Some(gen_cfg));

        // Resolve concrete tournament/day defaults if not provided by DB
        let original_had_tournament = tournament.is_some();
        let tournament_name_resolved: Option<String> = Some(match tournament {
            Some(t) => t.name,
            None => {
                // Default tournament name when none active
                "Tournament 1".to_string()
            }
        });
        let original_had_day = tournament_day.is_some();
        let tournament_day_resolved: Option<String> = Some(match tournament_day {
            Some(td) => format!("Day {}", td.unwrap().day_number),
            None => {
                // Compute next Day N under the tournament folder if exists; else Day 1
                let t_dir = videos_root.join(tournament_name_resolved.clone().unwrap());
                let mut next_day_num = 1u32;
                if t_dir.is_dir() {
                    if let Ok(entries) = std::fs::read_dir(&t_dir) {
                        let mut max_day = 0u32;
                        for e in entries.flatten() {
                            if let Ok(md) = e.metadata() {
                                if md.is_dir() {
                                    if let Some(name) = e.file_name().to_str() {
                                        if let Some(rest) = name.strip_prefix("Day ") {
                                            if let Ok(n) = rest.trim().parse::<u32>() { if n > max_day { max_day = n; } }
                                        }
                                    }
                                }
                            }
                        }
                        next_day_num = if max_day == 0 { 1 } else { max_day + 1 };
                    }
                }
                format!("Day {}", next_day_num)
            }
        });

        // If we defaulted any value, emit a prompt to frontend via centralized messaging
        if !original_had_tournament || !original_had_day {
            let tn = tournament_name_resolved.clone().unwrap_or_else(|| "Tournament 1".to_string());
            // Suggest next tournament as Tournament N+1
            let tn_suggest_new = if let Some(rest) = tn.strip_prefix("Tournament ") {
                if let Ok(n) = rest.trim().parse::<u32>() { format!("Tournament {}", n+1) } else { "Tournament 2".to_string() }
            } else { "Tournament 2".to_string() };
            // Suggest next day as Day X+1 baseline is computed above as resolved day; for new tournament, Day 1
            let resolved_day = tournament_day_resolved.clone().unwrap_or_else(|| "Day 1".to_string());
            let payload = serde_json::json!({
                "type": "obs_path_decision_needed",
                "continue": { "tournament": tn, "day": resolved_day },
                "new": { "tournament": tn_suggest_new, "day": "Day 1" }
            });
            crate::core::app::App::emit_pss_event(payload);
        }

        // Generate path
        let generated_path = path_generator.generate_recording_path(
            match_id,
            tournament_name_resolved,
            tournament_day_resolved,
            match_info.match_number.clone(),
            player1_name.clone(),
            player1_flag.clone(),
            player2_name.clone(),
            player2_flag.clone()
        )?;

        // Ensure directory exists
        path_generator.ensure_directory_exists(&generated_path.directory)?;

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

    /// Regenerate recording path with explicit tournament/day overrides and apply to OBS
    pub async fn regenerate_path_with_overrides(&self, tournament_name: String, tournament_day: String) -> AppResult<()> {
        // Get current session and match_id
        let match_id = {
            let session_guard = self.current_session.lock().unwrap();
            session_guard.as_ref().map(|s| s.match_id.clone())
        }.ok_or_else(|| AppError::ConfigError("No active recording session".to_string()))?;

        let conn = self.database.get_connection().await?;
        // Build generator from config
        let (videos_root, recording_format, folder_pattern) = {
            use crate::database::operations::ObsRecordingOperations as RecOps;
            let conn_name = {
                let cfg = self.config.lock().unwrap();
                cfg.obs_connection_name.clone().unwrap_or_else(|| "OBS_REC".to_string())
            };
            if let Ok(Some(cfg)) = RecOps::get_recording_config(&*conn, &conn_name) {
                (std::path::PathBuf::from(cfg.recording_root_path), cfg.recording_format, Some(cfg.folder_pattern))
            } else {
                (PathGeneratorConfig::detect_windows_videos_folder(), "mp4".to_string(), Some("{tournament}/{tournamentDay}".to_string()))
            }
        };
        let gen_cfg = PathGeneratorConfig { videos_root, default_format: recording_format, include_minutes_seconds: true, folder_pattern };
        let path_generator = ObsPathGenerator::new(Some(gen_cfg));

        // Fetch match info for filename fields
        let matches = PssUdpOperations::get_pss_matches(&*conn, Some(100))?;
        let match_info = matches.into_iter().find(|m| m.match_id == match_id).ok_or_else(|| AppError::ConfigError("Match not found for override".to_string()))?;
        let match_athletes = PssUdpOperations::get_pss_match_athletes(&*conn, match_info.id.unwrap())?;
        let mut player1_name = None; let mut player1_flag = None; let mut player2_name = None; let mut player2_flag = None;
        for (match_athlete, athlete) in match_athletes { match match_athlete.athlete_position { 1 => { player1_name = Some(athlete.short_name); player1_flag = athlete.country_code; }, 2 => { player2_name = Some(athlete.short_name); player2_flag = athlete.country_code; }, _ => {} } }

        let generated_path = path_generator.generate_recording_path(
            &match_id,
            Some(tournament_name.clone()),
            Some(tournament_day.clone()),
            match_info.match_number,
            player1_name.clone(), player1_flag.clone(), player2_name.clone(), player2_flag.clone()
        )?;
        path_generator.ensure_directory_exists(&generated_path.directory)?;

        // Update session
        {
            let mut session_guard = self.current_session.lock().unwrap();
            if let Some(ref mut session) = *session_guard {
                session.recording_path = Some(generated_path.directory.to_string_lossy().to_string());
                session.recording_filename = Some(generated_path.filename);
                session.tournament_name = Some(tournament_name);
                session.tournament_day = Some(tournament_day);
                session.match_number = match_info.match_number.clone();
                session.player1_name = player1_name; session.player1_flag = player1_flag;
                session.player2_name = player2_name; session.player2_flag = player2_flag;
                session.updated_at = Utc::now();
            }
        }

        // Apply directory to OBS (re-evaluate day boundary)
        if let Some(session) = self.get_current_session() {
            if let (Some(dir), Some(conn_name)) = (session.recording_path.clone(), session.obs_connection_name.clone()) {
                match self.obs_manager.set_record_directory(&dir, Some(&conn_name)).await {
                    Ok(()) => log::info!("📁 Applied overridden recording directory to OBS: {}", dir),
                    Err(e) => log::warn!("Failed to set overridden record directory in OBS: {}", e),
                }
                // Re-apply filename formatting
                if let Some(template) = self.get_active_filename_template().await? {
                    let formatting = self.build_filename_formatting(&template, &session);
                    if let Err(e) = self.obs_manager.set_filename_formatting(&formatting, Some(&conn_name)).await {
                        log::warn!("Failed to set filename formatting after override: {}", e);
                    }
                }
            }
        }

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
        // Replace variables with concrete values and ensure "VS" is between players
        let p1 = session.player1_name.clone().unwrap_or_default();
        let p2 = session.player2_name.clone().unwrap_or_default();
        if !p1.is_empty() && !p2.is_empty() {
            // Insert VS into a local copy for replacement convenience
            // We will map {player1} -> p1, {player2} -> p2 and let template include VS, but also patch common templates
        }

        let mut fmt = template.to_string();
        if let Some(ref n) = session.match_number { fmt = fmt.replace("{matchNumber}", n); }
        if let Some(ref f1) = session.player1_flag { fmt = fmt.replace("{player1Flag}", f1); }
        if let Some(ref f2) = session.player2_flag { fmt = fmt.replace("{player2Flag}", f2); }
        fmt = fmt.replace("{player1}", &p1);
        fmt = fmt.replace("{player2}", &p2);

        // If template lacked VS, inject a sane default pattern
        if !fmt.contains("VS") && fmt.contains(&p1) && fmt.contains(&p2) {
            // Try to place VS between players when pattern is exactly p1 _ p2 or similar
            // Simple heuristic: if pattern contains "_" between players, replace first "_" between them with " VS "
            let combined = format!("{}_{}", p1, p2);
            if fmt.contains(&combined) {
                fmt = fmt.replace(&combined, &format!("{} VS {}", p1, p2));
            }
        }

        // Ensure we keep date/time placeholders for OBS to resolve
        // Recommend using %CCYY-%MM-%DD %hh-%mm-%ss in OBS; we simply pass through any {date}/{time}
        fmt
    }

    /// Get current match ID from UDP context
    async fn get_current_match_id(&self) -> AppResult<Option<String>> {
        // Prefer live UDP-provided match id
        let live = { self.last_udp_match_id.lock().unwrap().clone() };
        if live.is_some() {
            return Ok(live);
        }

        // Fallback: most recent match from DB
        let conn = self.database.get_connection().await?;
        let matches = PssUdpOperations::get_pss_matches(&*conn, Some(1)).unwrap_or_default();
        Ok(matches.into_iter().next().map(|m| m.match_id))
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
