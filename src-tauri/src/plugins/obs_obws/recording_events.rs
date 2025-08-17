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
use once_cell::sync::OnceCell;
use std::sync::Mutex as StdMutex;

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
    database: Arc<crate::plugins::plugin_database::DatabasePlugin>,
    obs_manager: Arc<ObsManager>,
    last_applied_directory_day: Arc<Mutex<Option<String>>>,
    // Live UDP-provided current match id (from MatchConfig)
    last_udp_match_id: Arc<Mutex<Option<String>>>,
    // Whether we are waiting for user to confirm path decision (Continue/New)
    awaiting_path_decision: Arc<Mutex<bool>>,
    // In-session memo of the active tournament/day chosen or created (prevents recomputing Day from disk)
    active_tournament_day: Arc<Mutex<Option<(String, String)>>>,
    // Pending UDP values captured before a session exists
    pending_p1_name: Arc<Mutex<Option<String>>>,
    pending_p1_flag: Arc<Mutex<Option<String>>>,
    pending_p2_name: Arc<Mutex<Option<String>>>,
    pending_p2_flag: Arc<Mutex<Option<String>>>,
    pending_match_number: Arc<Mutex<Option<String>>>,
    // Pending delayed stop task (aborted on new match)
    pending_stop_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl ObsRecordingEventHandler {
    pub fn new(
        config: AutomaticRecordingConfig,
        event_tx: mpsc::UnboundedSender<RecordingEvent>,
        database: Arc<crate::plugins::plugin_database::DatabasePlugin>,
        obs_manager: Arc<ObsManager>,
    ) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            current_session: Arc::new(Mutex::new(None)),
            event_tx,
            database,
            obs_manager,
            last_applied_directory_day: Arc::new(Mutex::new(None)),
            last_udp_match_id: Arc::new(Mutex::new(None)),
            awaiting_path_decision: Arc::new(Mutex::new(false)),
            active_tournament_day: Arc::new(Mutex::new(None)),
            pending_p1_name: Arc::new(Mutex::new(None)),
            pending_p1_flag: Arc::new(Mutex::new(None)),
            pending_p2_name: Arc::new(Mutex::new(None)),
            pending_p2_flag: Arc::new(Mutex::new(None)),
            pending_match_number: Arc::new(Mutex::new(None)),
            pending_stop_task: Arc::new(Mutex::new(None)),
        }
    }

    /// Handle PSS events and trigger recording actions
    pub async fn handle_pss_event(&self, event: &PssEvent) -> AppResult<()> {
        log::info!("🎥 ObsRecordingEventHandler::handle_pss_event called with: {:?}", event);
        println!("🎥 ObsRecordingEventHandler::handle_pss_event: {:?}", event);
        let config = {
            let config_guard = self.config.lock().unwrap();
            config_guard.clone()
        };

        if !config.enabled {
            return Ok(());
        }

        match event {
            // Capture live match id and number as soon as MatchConfig arrives
            PssEvent::MatchConfig { number, .. } => {
                {
                    let mut guard = self.last_udp_match_id.lock().unwrap();
                    // Map UDP number to DB match_id format (e.g., "mch:101")
                    *guard = Some(format!("mch:{}", number));
                }
                {
                    let mut m = self.pending_match_number.lock().unwrap();
                    *m = Some(number.clone());
                }
                // Optionally update current session's match number early
                {
                    let mut session_guard = self.current_session.lock().unwrap();
                    if let Some(ref mut session) = *session_guard {
                        session.match_number = Some(number.to_string());
                        session.updated_at = Utc::now();
                    }
                }
                log::info!("📣 MatchConfig received - set current match_id=mch:{} number={}", number, number);

                // Ensure any pending delayed stop is cancelled and enforce immediate stop
                let _ = self.cancel_pending_stop_and_stop_immediately().await;

                // If we don't have a prepared session for this match, prepare now
                if let Err(e) = self.handle_fight_loaded().await {
                    log::warn!("Failed to prepare session on MatchConfig: {}", e);
                }
            }
            // Capture athletes immediately to ensure filename uses current match names
            PssEvent::Athletes { athlete1_short, athlete1_country, athlete2_short, athlete2_country, .. } => {
                // Store pending UDP names/flags (used when session does not exist yet)
                {
                    *self.pending_p1_name.lock().unwrap() = Some(athlete1_short.clone());
                    *self.pending_p1_flag.lock().unwrap() = Some(athlete1_country.clone());
                    *self.pending_p2_name.lock().unwrap() = Some(athlete2_short.clone());
                    *self.pending_p2_flag.lock().unwrap() = Some(athlete2_country.clone());
                }
                // Also update current session if present
                let mut session_guard = self.current_session.lock().unwrap();
                if let Some(ref mut session) = *session_guard {
                    session.player1_name = Some(athlete1_short.clone());
                    session.player1_flag = Some(athlete1_country.clone());
                    session.player2_name = Some(athlete2_short.clone());
                    session.player2_flag = Some(athlete2_country.clone());
                    session.updated_at = Utc::now();
                }
            }
            // Match loaded - prepare recording
            PssEvent::FightLoaded => {
                log::info!("🎬 FightLoaded event received - preparing recording session");
                // Ensure any pending delayed stop is cancelled and enforce immediate stop
                let _ = self.cancel_pending_stop_and_stop_immediately().await;
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

    /// Cancel any pending delayed stop and stop OBS recording immediately
    async fn cancel_pending_stop_and_stop_immediately(&self) -> AppResult<()> {
        // Abort any scheduled delayed stop
        if let Some(handle) = self.pending_stop_task.lock().unwrap().take() {
            handle.abort();
            log::info!("⏹️ Aborted pending delayed stop task");
        }

        let config = { self.config.lock().unwrap().clone() };
        if let Some(connection_name) = config.obs_connection_name {
            // Attempt to stop recording immediately; ignore error if already stopped
            if let Err(e) = self.obs_manager.stop_recording(Some(&connection_name)).await {
                log::warn!("Stop recording on new match failed (may already be stopped): {}", e);
            } else {
                log::info!("🎬 Recording stopped immediately for connection: {}", connection_name);
            }
        }
        Ok(())
    }

    /// Handle FightLoaded event - prepare recording session
    async fn handle_fight_loaded(&self) -> AppResult<()> {
        // Before starting a new session, finalize and index the previous one (for delayed-stop cases)
        let _ = self.finalize_previous_session_index().await;

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
                        // Normalize path separators for OBS
                        let dir_norm = dir.replace('\\', "/");
                        // Print exactly what we're sending to OBS
                        println!("📤 Sending to OBS '{}' record directory (day apply): {}", conn_name, dir_norm);
                        log::info!("📤 Sending to OBS '{}' record directory (day apply): {}", conn_name, dir_norm);
                        // Release the mutex before awaiting
                        match self.obs_manager.set_record_directory(&dir_norm, Some(&conn_name)).await {
                            Ok(()) => {
                                let mut last = self.last_applied_directory_day.lock().unwrap();
                                *last = Some(day_key);
                                log::info!("📁 Applied recording directory to OBS: {}", dir_norm);
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

    /// Finalize and index the previous recording session (used on next FightLoaded after a delayed stop)
    async fn finalize_previous_session_index(&self) -> AppResult<()> {
        if let Some(prev) = self.get_current_session() {
            // Only index if we have path, filename and start_time
            if let (Some(dir), Some(fname), Some(start_time)) = (
                prev.recording_path.clone(),
                prev.recording_filename.clone(),
                prev.start_time,
            ) {
                let created = chrono::Utc::now();
                let duration = (prev.end_time.unwrap_or(created) - start_time)
                    .num_seconds()
                    .max(0) as i32;
                let full_path = std::path::PathBuf::from(&dir)
                    .join(&fname)
                    .to_string_lossy()
                    .to_string();
                if let Ok(conn_guard) = self.database.get_connection().await {
                    let conn_ref = &*conn_guard;
                    // Resolve active tournament/day IDs
                    let (tid_opt, day_opt) = {
                        let t = TournamentOperations::get_active_tournament(&*conn_ref).ok().flatten();
                        let d = t.as_ref().and_then(|tt| TournamentOperations::get_active_tournament_day(&*conn_ref, tt.id.unwrap()).ok().flatten());
                        (t.and_then(|tt| tt.id), d.and_then(|dd| dd.id))
                    };
                    let _ = conn_ref.execute(
                        "INSERT INTO recorded_videos (match_id, event_id, tournament_id, tournament_day_id, video_type, file_path, record_directory, filename_formatting, start_time, duration_seconds, created_at)\n                         SELECT (SELECT id FROM pss_matches WHERE match_id = ? ORDER BY created_at DESC LIMIT 1), NULL, ?, ?, 'recording', ?, ?, NULL, ?, ?, ?\n                         WHERE NOT EXISTS (SELECT 1 FROM recorded_videos rv WHERE rv.match_id = (SELECT id FROM pss_matches WHERE match_id = ? ORDER BY created_at DESC LIMIT 1) AND rv.start_time = ?)",
                        rusqlite::params![
                            prev.match_id,
                            tid_opt,
                            day_opt,
                            full_path,
                            dir,
                            start_time.to_rfc3339(),
                            duration,
                            created.to_rfc3339(),
                            prev.match_id,
                            start_time.to_rfc3339()
                        ],
                    );
                    // Link only important events (K,P,H,TH,TB,R) within window
                    let _ = conn_ref.execute(
                        "INSERT OR IGNORE INTO recorded_video_events (recorded_video_id, event_id, offset_ms, created_at)\n                         SELECT rv.id, e.id, CAST((julianday(e.timestamp) - julianday(?)) * 86400000 AS INTEGER), ?\n                         FROM recorded_videos rv\n                         JOIN pss_events_v2 e ON e.match_id = rv.match_id\n                         JOIN pss_event_types t ON t.id = e.event_type_id\n                         WHERE rv.file_path = ? AND e.timestamp >= ? AND e.timestamp <= ?\n                           AND t.event_code IN ('K','P','H','TH','TB','R')\n                         ORDER BY e.timestamp ASC",
                        rusqlite::params![ start_time.to_rfc3339(), chrono::Utc::now().to_rfc3339(), full_path, start_time.to_rfc3339(), (prev.end_time.unwrap_or(created)).to_rfc3339() ]
                    );
                }
            }
        }
        Ok(())
    }

    /// Handle FightReady event - start recording
    async fn handle_fight_ready(&self) -> AppResult<()> {
        let config = {
            let config_guard = self.config.lock().unwrap();
            config_guard.clone()
        };

        // Debug snapshot of config to trace issues in production logs
        log::info!(
            "🧩 FightReady config: enabled={} conn={:?} auto_start_rec={} include_rb={}",
            config.enabled,
            config.obs_connection_name,
            config.auto_start_recording_on_match_begin,
            config.include_replay_buffer
        );

        // If a path decision prompt was emitted, do not block auto-start; proceed with current/default path
        if *self.awaiting_path_decision.lock().unwrap() {
            log::info!("⏸️ Path decision pending, proceeding with current/default path to avoid blocking recording");
        }

        // Resolve connection name: config -> session -> default "OBS_REC"
        let connection_name = config
            .obs_connection_name
            .clone()
            .or_else(|| self.get_current_session().and_then(|s| s.obs_connection_name.clone()))
            .unwrap_or_else(|| {
                log::warn!("⚠️ No connection name in config/session; defaulting to 'OBS_REC'");
                "OBS_REC".to_string()
            });

        if !connection_name.is_empty() {
            log::info!("🎬 FightReady: using OBS connection '{}'", connection_name);
            println!("🎬 FightReady: using OBS connection '{}'", connection_name);
            // Apply recording directory and filename formatting BEFORE starting RB/recording
            // If session is missing fields (athletes/match number), try a quick refresh
            if {
                let s = self.get_current_session();
                s.as_ref().map(|ss| ss.player1_name.is_none() || ss.player2_name.is_none() || ss.match_number.is_none()).unwrap_or(true)
            } {
                if let Some(mid) = { self.get_current_session().map(|s| s.match_id) } {
                    let _ = self.generate_recording_path(&mid).await;
                }
            }
            if let Some(mut session) = self.get_current_session() {
                // Apply directory (normalize separators) to be sure OBS accepts formatting update
                if let Some(dir) = session.recording_path.clone() {
                    let dir_norm = dir.replace('\\', "/");
                    // Print exactly what we're sending to OBS
                    println!("📤 Sending to OBS '{}' record directory (pre-start): {}", connection_name, dir_norm);
                    log::info!("📤 Sending to OBS '{}' record directory (pre-start): {}", connection_name, dir_norm);
                    if let Err(e) = self.obs_manager.set_record_directory(&dir_norm, Some(&connection_name)).await {
                        log::warn!("Failed to set record directory before start: {}", e);
                    } else {
                        log::info!("📁 Record directory ensured before start: {}", dir_norm);
                    }
                }
                // Resolve filename template; if DB has none, fall back to default mapping
                let effective_template = match self.get_active_filename_template().await? {
                    Some(t) => t,
                    None => {
                        let def = "{matchNumber} {player1} ({country1}) VS {player2} ({country2}) - {date} - {time}".to_string();
                        println!("⚠️ No DB filename template; using default: {}", def);
                        def
                    }
                };
                println!("🧪 Active filename template: {}", effective_template);
                // Ensure we use the most recent UDP values for formatting
                if session.player1_name.is_none() { session.player1_name = self.pending_p1_name.lock().unwrap().clone(); }
                if session.player1_flag.is_none() { session.player1_flag = self.pending_p1_flag.lock().unwrap().clone(); }
                if session.player2_name.is_none() { session.player2_name = self.pending_p2_name.lock().unwrap().clone(); }
                if session.player2_flag.is_none() { session.player2_flag = self.pending_p2_flag.lock().unwrap().clone(); }
                if session.match_number.is_none() { session.match_number = self.pending_match_number.lock().unwrap().clone(); }
                let formatting = self.build_filename_formatting(&effective_template, &session);
                // Print exactly what we're sending to OBS
                println!("📤 Sending to OBS '{}' filename formatting: {}", connection_name, formatting);
                log::info!("📤 Sending to OBS '{}' filename formatting: {}", connection_name, formatting);
                if let Err(e) = self.obs_manager.set_filename_formatting(&formatting, Some(&connection_name)).await {
                    log::warn!("Failed to set filename formatting: {}", e);
                } else {
                    log::info!("🧾 Applied filename formatting to OBS: {}", formatting);
                    // Read-back verification
                    match self.obs_manager.get_filename_formatting(Some(&connection_name)).await {
                        Ok(current) => println!("🔎 OBS current filename formatting='{}'", current),
                        Err(err) => println!("⚠️ Failed to read back filename formatting: {}", err),
                    }
                }
            }

            // Small delay to allow OBS to commit profile updates before starting outputs
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            // Ensure replay buffer is enabled and active before recording
            match self.obs_manager.get_replay_buffer_status(Some(&connection_name)).await {
                Ok(ObsReplayBufferStatus::Active) => {
                    log::info!("▶️ Replay buffer already active");
                    println!("▶️ Replay buffer already active");
                }
                _ => {
                    log::info!("▶️ Starting replay buffer before recording...");
                    println!("▶️ Starting replay buffer before recording...");
                    if let Err(e) = self.obs_manager.start_replay_buffer(Some(&connection_name)).await {
                        log::warn!("Failed to start replay buffer: {}", e);
                        println!("Failed to start replay buffer: {}", e);
                    } else {
                        log::info!("▶️ Replay buffer started to satisfy recording invariant");
                        println!("▶️ Replay buffer started to satisfy recording invariant");
                    }
                }
            }

            // Update session state to recording
            if config.auto_start_recording_on_match_begin {
                self.update_session_state(RecordingState::Recording).await?;
                // Start recording immediately via obws manager (authoritative)
                log::info!("🎬 Starting OBS recording...");
                println!("🎬 Starting OBS recording...");
                match self.obs_manager.start_recording(Some(&connection_name)).await {
                    Ok(()) => { log::info!("🎬 Recording started for connection: {}", connection_name); println!("🎬 Recording started for connection: {}", connection_name); },
                    Err(e) => { log::error!("Failed to start recording via obws: {}", e); println!("Failed to start recording via obws: {}", e); },
                }
            } else {
                log::info!("🎬 Auto-start recording disabled by UI setting; not starting recording on FightReady");
            }
        } else {
            log::warn!("⚠️ FightReady: empty connection name resolved; skipping start");
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

            // Respect stop delay seconds, but allow cancellation by new match
            let delay_secs = config.stop_delay_seconds as u64;
            if delay_secs == 0 {
                // Stop immediately
                if let Err(e) = self.obs_manager.stop_recording(Some(&connection_name)).await {
                    log::error!("Failed to stop recording via obws: {}", e);
                } else {
                    log::info!("🎬 Recording stopped for connection: {}", connection_name);
                    // Index the recording synchronously (best-effort)
                    let _ = self.index_recording_after_stop().await;
                }
            } else {
                log::info!("⏳ Scheduling stop in {}s (will cancel if new match loads)", delay_secs);
                let mgr = self.obs_manager.clone();
                let conn = connection_name.clone();
                let _app_db = self.database.clone(); // reserved for future richer indexing
                let _session_before = self.get_current_session();
                // Abort any previous pending stop
                if let Some(handle) = self.pending_stop_task.lock().unwrap().take() { handle.abort(); }
                let handle = tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                    if let Err(e) = mgr.stop_recording(Some(&conn)).await {
                        log::error!("Delayed stop: failed to stop recording via obws: {}", e);
                    } else {
                        log::info!("🎬 Delayed stop: recording stopped for connection: {}", conn);
                    }
                });
                *self.pending_stop_task.lock().unwrap() = Some(handle);
            }
        }

        Ok(())
    }

    /// After a successful immediate stop, try to index the recording for the current session
    async fn index_recording_after_stop(&self) -> AppResult<()> {
        if let Some(session) = self.get_current_session() {
            let start_opt = session.start_time;
            let dir_opt = session.recording_path.clone();
            let fname_opt = session.recording_filename.clone();
            let mid = session.match_id.clone();
            let created = chrono::Utc::now();
            if let (Some(start_time), Some(record_dir)) = (start_opt, dir_opt) {
                let file_path: Option<String> = fname_opt.map(|f| std::path::PathBuf::from(&record_dir).join(f).to_string_lossy().to_string());
                if let Ok(conn_guard) = self.database.get_connection().await {
                    let conn_ref = &*conn_guard;
                    let duration = (created - start_time).num_seconds().max(0) as i32;
                    // Resolve active tournament/day IDs for better indexing
                    let (tid_opt, day_opt) = {
                        let t = TournamentOperations::get_active_tournament(&*conn_ref).ok().flatten();
                        let d = t.as_ref().and_then(|tt| TournamentOperations::get_active_tournament_day(&*conn_ref, tt.id.unwrap()).ok().flatten());
                        (t.and_then(|tt| tt.id), d.and_then(|dd| dd.id))
                    };
                    // Resolve match DB id robustly: try by match_id string, then by match_number, else most recent
                    let match_db_id: i64 = {
                        let by_mid = conn_ref
                            .query_row(
                                "SELECT id FROM pss_matches WHERE match_id = ? ORDER BY updated_at DESC LIMIT 1",
                                rusqlite::params![ mid ],
                                |r| r.get::<_, i64>(0),
                            )
                            .ok();
                        if let Some(id) = by_mid { id } else {
                            // Try by match_number from session
                            let maybe_num: Option<String> = {
                                let s = self.get_current_session();
                                s.and_then(|ss| ss.match_number.clone())
                            };
                            if let Some(mnum) = maybe_num {
                                if let Ok(id2) = conn_ref.query_row(
                                    "SELECT id FROM pss_matches WHERE match_number = ? ORDER BY updated_at DESC LIMIT 1",
                                    rusqlite::params![ mnum ],
                                    |r| r.get::<_, i64>(0),
                                ) { id2 } else {
                                    conn_ref.query_row(
                                        "SELECT id FROM pss_matches ORDER BY created_at DESC LIMIT 1",
                                        [],
                                        |r| r.get::<_, i64>(0),
                                    ).unwrap_or(0)
                                }
                            } else {
                                conn_ref.query_row(
                                    "SELECT id FROM pss_matches ORDER BY created_at DESC LIMIT 1",
                                    [],
                                    |r| r.get::<_, i64>(0),
                                ).unwrap_or(0)
                            }
                        }
                    };
                    if match_db_id > 0 {
                        let _ = conn_ref.execute(
                            "INSERT INTO recorded_videos (match_id, event_id, tournament_id, tournament_day_id, video_type, file_path, record_directory, filename_formatting, start_time, duration_seconds, created_at) VALUES (?, NULL, ?, ?, 'recording', ?, ?, NULL, ?, ?, ?)",
                            rusqlite::params![ match_db_id, tid_opt, day_opt, file_path, record_dir, start_time.to_rfc3339(), duration, created.to_rfc3339() ]
                        );
                    }
                    // Resolve recorded_video_id for this recording window
                    let rvid: i64 = conn_ref
                        .query_row(
                            "SELECT id FROM recorded_videos WHERE match_id = (SELECT id FROM pss_matches WHERE match_id = ? ORDER BY created_at DESC LIMIT 1) AND start_time = ?",
                            rusqlite::params![ mid, start_time.to_rfc3339() ],
                            |r| r.get(0),
                        )
                        .unwrap_or_else(|_| conn_ref.last_insert_rowid());
                    // Bulk-link events inside window with offset_ms
                    let end_time = start_time + chrono::Duration::seconds(duration as i64);
                    // Link only important events (K,P,H,TH,TB,R) for this recording window
                    let _ = conn_ref.execute(
                        "INSERT OR IGNORE INTO recorded_video_events (recorded_video_id, event_id, offset_ms, created_at)
                         SELECT ?, e.id, CAST((julianday(e.timestamp) - julianday(?)) * 86400000 AS INTEGER), ?
                         FROM pss_events_v2 e
                         JOIN pss_event_types t ON t.id = e.event_type_id
                         WHERE e.match_id = (SELECT id FROM pss_matches WHERE match_id = ? ORDER BY created_at DESC LIMIT 1)
                           AND e.timestamp >= ? AND e.timestamp <= ?
                           AND t.event_code IN ('K','P','H','TH','TB','R')
                         ORDER BY e.timestamp ASC",
                        rusqlite::params![ rvid, start_time.to_rfc3339(), chrono::Utc::now().to_rfc3339(), mid, start_time.to_rfc3339(), end_time.to_rfc3339() ]
                    );
                }
            }
        }
        Ok(())
    }

    // (removed) handle_match_end: WinnerRounds no longer auto-stops

    /// Generate recording path for current match
    pub async fn generate_recording_path(&self, match_id: &str) -> AppResult<()> {
        let conn = self.database.get_connection().await?;

        // Resolve Videos root and path generator settings first (we must verify folders on disk)
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

        // Determine tournament/day from DB and always use them; ensure folders exist instead of demoting
        let tournament = TournamentOperations::get_active_tournament(&*conn)?;
        let tournament_day = if let Some(ref t) = tournament {
            TournamentOperations::get_active_tournament_day(&*conn, t.id.unwrap()).ok()
        } else { None };

        // Get match details (support both raw db IDs and mch:<number> keys)
        let matches = PssUdpOperations::get_pss_matches(&*conn, Some(200))?;
        let match_info = matches.into_iter()
            .find(|m| m.match_id == match_id || m.match_id == format!("mch:{}", m.match_number.clone().unwrap_or_default()))
            .ok_or_else(|| AppError::ConfigError(format!("Match not found: {}", match_id)))?;

        // Get match athletes with a short retry window to allow UDP linking to persist
        let match_db_id = match_info.id.unwrap();
        let mut db_player1_name: Option<String> = None;
        let mut db_player1_flag: Option<String> = None;
        let mut db_player2_name: Option<String> = None;
        let mut db_player2_flag: Option<String> = None;
        for _ in 0..20 { // up to ~3s
            let match_athletes = PssUdpOperations::get_pss_match_athletes(&*conn, match_db_id)?;
            let mut found1 = false;
            let mut found2 = false;
            for (match_athlete, athlete) in &match_athletes {
                match match_athlete.athlete_position {
                    1 => {
                        db_player1_name = Some(athlete.short_name.clone());
                        db_player1_flag = athlete.country_code.clone(); // Option<String>
                        found1 = true;
                    },
                    2 => {
                        db_player2_name = Some(athlete.short_name.clone());
                        db_player2_flag = athlete.country_code.clone(); // Option<String>
                        found2 = true;
                    },
                    _ => {}
                }
            }
            if found1 && found2 { break; }
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        }

        // Build path generator from active recording config to avoid sending placeholders

        let gen_cfg = PathGeneratorConfig {
            videos_root: videos_root.clone(),
            default_format: recording_format,
            include_minutes_seconds: true,
            folder_pattern,
        };
        let path_generator = ObsPathGenerator::new(Some(gen_cfg));

        // Prefer live UDP/session values over DB when available
        let (live_p1_name, live_p1_flag, live_p2_name, live_p2_flag, live_match_number) = {
            // Start with pending UDP values
            let mut p1n = self.pending_p1_name.lock().unwrap().clone();
            let mut p1f = self.pending_p1_flag.lock().unwrap().clone();
            let mut p2n = self.pending_p2_name.lock().unwrap().clone();
            let mut p2f = self.pending_p2_flag.lock().unwrap().clone();
            let mut mnum = self.pending_match_number.lock().unwrap().clone();

            // Overlay current session values if present
            if let Some(s) = self.current_session.lock().unwrap().clone() {
                if p1n.is_none() { p1n = s.player1_name; }
                if p1f.is_none() { p1f = s.player1_flag; }
                if p2n.is_none() { p2n = s.player2_name; }
                if p2f.is_none() { p2f = s.player2_flag; }
                if mnum.is_none() { mnum = s.match_number; }
            }

            // Finally fallback to DB-derived values
            (
                p1n.or(db_player1_name.clone()),
                p1f.or(db_player1_flag.clone()),
                p2n.or(db_player2_name.clone()),
                p2f.or(db_player2_flag.clone()),
                mnum.or(match_info.match_number.clone()),
            )
        };

        // Resolve concrete tournament/day defaults if not provided by DB
        // First prefer in-session memo to avoid recomputing Day from disk after we just created Day 1
        let memo_td = { self.active_tournament_day.lock().unwrap().clone() };
        let has_memo_td = memo_td.is_some();
        let (tournament_name_resolved, tournament_day_resolved): (Option<String>, Option<String>) = if let Some((ref tn, ref td)) = memo_td {
            (Some(tn.clone()), Some(td.clone()))
        } else {
            // Determine base tournament name by scanning root when none active
            let tn: Option<String> = Some(match tournament {
                Some(t) => t.name,
                None => {
                    // Scan root for existing "Tournament N" directories
                    let mut max_tournament = 0u32;
                    if videos_root.is_dir() {
                        if let Ok(entries) = std::fs::read_dir(&videos_root) {
                            for e in entries.flatten() {
                                if let Ok(md) = e.metadata() {
                                    if md.is_dir() {
                                        if let Some(name) = e.file_name().to_str() {
                                            if let Some(rest) = name.strip_prefix("Tournament ") {
                                                if let Ok(n) = rest.trim().parse::<u32>() { if n > max_tournament { max_tournament = n; } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // If none exist, start at Tournament 1
                    if max_tournament == 0 { "Tournament 1".to_string() } else { format!("Tournament {}", max_tournament) }
                }
            });
            // Determine default day when DB has none
            let td: Option<String> = Some(match tournament_day {
                Some(td) => format!("Day {}", td.unwrap().day_number),
                None => {
                    // Compute next Day N under the chosen tournament folder if exists; else Day 1
                    let t_dir = videos_root.join(tn.clone().unwrap());
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
            (tn, td)
        };

        // If folders already exist on disk (returning user), emit a prompt; if not, create Tournament 1/Day 1 silently
        static ASKED_THIS_SESSION: OnceCell<StdMutex<bool>> = OnceCell::new();
        let asked_flag = ASKED_THIS_SESSION.get_or_init(|| StdMutex::new(false));
        let should_prompt_this_session = {
            let asked = asked_flag.lock().unwrap();
            !*asked
        };
        let has_existing_tournaments = {
            let t_dir = &videos_root;
            t_dir.is_dir() && std::fs::read_dir(t_dir).map(|mut it| it.any(|e| e.ok().and_then(|x| x.file_name().to_str().map(|s| s.starts_with("Tournament "))).unwrap_or(false))).unwrap_or(false)
        };
        if should_prompt_this_session && has_existing_tournaments && !has_memo_td {
            let tn = tournament_name_resolved.clone().unwrap_or_else(|| "Tournament 1".to_string());
            // Suggest next tournament as Tournament N+1
            let tn_suggest_new = if let Some(rest) = tn.strip_prefix("Tournament ") {
                if let Ok(n) = rest.trim().parse::<u32>() { format!("Tournament {}", n+1) } else { "Tournament 2".to_string() }
            } else { "Tournament 2".to_string() };
            // Continue with the highest existing Day under current tournament (or Day 1 if none)
            let resolved_day = {
                let t_dir = videos_root.join(&tn);
                if t_dir.is_dir() {
                    let mut max_day = 0u32;
                    if let Ok(entries) = std::fs::read_dir(&t_dir) {
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
                    }
                    if max_day == 0 { "Day 1".to_string() } else { format!("Day {}", max_day) }
                } else { "Day 1".to_string() }
            };
            let payload = serde_json::json!({
                "type": "obs_path_decision_needed",
                "continue": { "tournament": tn, "day": resolved_day },
                "new": { "tournament": tn_suggest_new, "day": "Day 1" }
            });
            // Emit on both pss_event (legacy) and a dedicated custom event to guarantee UI handling
            crate::core::app::App::emit_pss_event(payload.clone());
            crate::core::app::App::emit_custom_event("obs_path_decision_needed", payload.clone());
            println!(
                "📣 Emitted obs_path_decision_needed with options: Continue={}/{} New={}/{}",
                payload["continue"]["tournament"], payload["continue"]["day"],
                payload["new"]["tournament"], payload["new"]["day"],
            );
            // Mark asked for this session; do not block auto-start (we'll proceed with defaults)
            if let Ok(mut asked) = asked_flag.lock() { *asked = true; }
        }

        // Generate path
        let generated_path = path_generator.generate_recording_path(
            match_id,
            tournament_name_resolved,
            tournament_day_resolved,
            live_match_number.clone(),
            live_p1_name.clone(),
            live_p1_flag.clone(),
            live_p2_name.clone(),
            live_p2_flag.clone()
        )?;

        // Ensure directory exists. If we showed prompt, wait for user decision; if no prompt, create now
        let waiting = *self.awaiting_path_decision.lock().unwrap();
        if !waiting {
            path_generator.ensure_directory_exists(&generated_path.directory)?;
            // Suppress showing the modal later in the same session since we just created defaults
            if let Ok(mut asked) = ASKED_THIS_SESSION.get_or_init(|| StdMutex::new(false)).lock() { *asked = true; }
            // Memoize the active tournament/day for this session to avoid recomputing Day from disk again
            if let (Some(ref tn), Some(ref td)) = (&generated_path.tournament_name, &generated_path.tournament_day) {
                if let Ok(mut memo) = self.active_tournament_day.lock() { *memo = Some((tn.clone(), td.clone())); }
            }
        }

        // Update session with path information
        {
            let mut session_guard = self.current_session.lock().unwrap();
            if let Some(ref mut session) = *session_guard {
                session.recording_path = Some(generated_path.directory.to_string_lossy().to_string());
                session.recording_filename = Some(generated_path.filename);
                session.tournament_name = generated_path.tournament_name;
                session.tournament_day = generated_path.tournament_day;
                // Preserve live values if already set; otherwise use resolved/live/db values
                session.match_number = session.match_number.clone().or(live_match_number);
                session.player1_name = session.player1_name.clone().or(live_p1_name);
                session.player1_flag = session.player1_flag.clone().or(live_p1_flag);
                session.player2_name = session.player2_name.clone().or(live_p2_name);
                session.player2_flag = session.player2_flag.clone().or(live_p2_flag);
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
            match_info.match_number.clone(),
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

        // Release DB connection before awaiting any OBS/profile calls to avoid pool deadlocks
        drop(conn);

        // Apply directory to OBS (re-evaluate day boundary) and release the wait flag
        if let Some(session) = self.get_current_session() {
            if let (Some(dir), Some(conn_name)) = (session.recording_path.clone(), session.obs_connection_name.clone()) {
                // Normalize path separators to forward slashes for OBS compatibility
                let dir_norm = dir.replace('\\', "/");
                println!("📤 Sending to OBS '{}' record directory (override): {}", conn_name, dir_norm);
                log::info!("📤 Sending to OBS '{}' record directory (override): {}", conn_name, dir_norm);
                match self.obs_manager.set_record_directory(&dir_norm, Some(&conn_name)).await {
                    Ok(()) => log::info!("📁 Applied overridden recording directory to OBS: {}", dir),
                    Err(e) => log::warn!("Failed to set overridden record directory in OBS: {}", e),
                }
                // Re-apply filename formatting
                println!("🧪 Override: resolving filename template...");
                let eff_tmpl = match self.get_active_filename_template().await {
                    Ok(Some(t)) => t,
                    Ok(None) => {
                        let d = "{matchNumber} {player1} ({country1}) VS {player2} ({country2}) - {date} - {time}".to_string();
                        println!("⚠️ Override: no DB template -> using default: {}", d);
                        d
                    }
                    Err(e) => {
                        let d = "{matchNumber} {player1} ({country1}) VS {player2} ({country2}) - {date} - {time}".to_string();
                        println!("⚠️ Override: failed to get DB template ({}), using default: {}", e, d);
                        d
                    }
                };
                println!("🧪 Override: active filename template: {}", eff_tmpl);
                let formatting = self.build_filename_formatting(&eff_tmpl, &session);
                println!("📤 Sending to OBS '{}' filename formatting (override): {}", conn_name, formatting);
                log::info!("📤 Sending to OBS '{}' filename formatting (override): {}", conn_name, formatting);
                if let Err(e) = self.obs_manager.set_filename_formatting(&formatting, Some(&conn_name)).await {
                    log::warn!("Failed to set filename formatting after override: {}", e);
                }
                // Clear awaiting flag so FightReady will proceed next time
                if let Ok(mut wait_flag) = self.awaiting_path_decision.lock() { *wait_flag = false; }
                // Memoize the user-selected tournament/day for this session
                if let (Some(ref tn), Some(ref td)) = (session.tournament_name.clone(), session.tournament_day.clone()) {
                    if let Ok(mut memo) = self.active_tournament_day.lock() { *memo = Some((tn.clone(), td.clone())); }
                }
            }
        }

        // Now that the decision is applied and waiting flag cleared, proceed with the standard FightReady flow
        // to ensure filename formatting, RB, and recording start are executed.
        println!("⏭️ Calling FightReady immediately after override apply...");
        let _ = self.handle_fight_ready().await;

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
        let c1 = session.player1_flag.clone().unwrap_or_default();
        let c2 = session.player2_flag.clone().unwrap_or_default();
        if !p1.is_empty() && !p2.is_empty() {
            // Insert VS into a local copy for replacement convenience
            // We will map {player1} -> p1, {player2} -> p2 and let template include VS, but also patch common templates
        }

        let mut fmt = template.to_string();
        if let Some(ref n) = session.match_number {
            // Replace both {matchNumber}_{player1} and stand-alone {matchNumber}
            fmt = fmt.replace("{matchNumber}_{player1}", &format!("{} {}", n, p1));
            fmt = fmt.replace("{matchNumber}", n);
        }
        if let Some(ref f1) = session.player1_flag { fmt = fmt.replace("{player1Flag}", f1); }
        if let Some(ref f2) = session.player2_flag { fmt = fmt.replace("{player2Flag}", f2); }
        fmt = fmt.replace("{player1}", &p1);
        fmt = fmt.replace("{player2}", &p2);
        // New country placeholders
        fmt = fmt.replace("{country1}", &c1);
        fmt = fmt.replace("{country2}", &c2);

        // If template lacked VS, inject a sane default pattern
        if !fmt.contains("VS") && fmt.contains(&p1) && fmt.contains(&p2) {
            // Try to place VS between players when pattern is exactly p1 _ p2 or similar
            // Simple heuristic: if pattern contains "_" between players, replace first "_" between them with " VS "
            let combined = format!("{}_{}", p1, p2);
            if fmt.contains(&combined) {
                fmt = fmt.replace(&combined, &format!("{} VS {}", p1, p2));
            }
        }

        // Map app placeholders to OBS placeholders
        fmt = fmt.replace("{date} - {time}", "%DD-%MM-%CCYY %hh-%mm-%ss");
        fmt = fmt.replace("{date}_{time}", "%DD-%MM-%CCYY_%hh-%mm-%ss");
        fmt = fmt.replace("{date}", "%DD-%MM-%CCYY");
        fmt = fmt.replace("{time}", "%hh-%mm-%ss");
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
