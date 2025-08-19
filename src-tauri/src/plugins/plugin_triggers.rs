use crate::database::{DatabaseConnection, models::{OverlayTemplate, EventTrigger}};
use once_cell::sync::OnceCell;
use crate::plugins::obs_obws::manager::ObsManager;
use crate::types::AppResult;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// Trigger system plugin for handling PSS event-driven automation
pub static TRIGGER_PLUGIN_GLOBAL: OnceCell<std::sync::Arc<TriggerPlugin>> = OnceCell::new();

pub struct TriggerPlugin {
    db: Arc<DatabaseConnection>,
    obs_plugin_manager: Arc<ObsManager>,
    enabled_triggers: Arc<RwLock<HashMap<String, Vec<EventTrigger>>>>,
    current_tournament_id: Arc<RwLock<Option<i64>>>,
    current_tournament_day_id: Arc<RwLock<Option<i64>>>,
    paused: std::sync::Arc<std::sync::atomic::AtomicBool>,
    buffered_rdy: Arc<RwLock<Option<String>>>,
    resume_delay_ms: std::sync::Arc<std::sync::atomic::AtomicU64>,
    // v2 runtime state
    current_round: Arc<RwLock<Option<i64>>>,
    last_fired_at: Arc<RwLock<HashMap<i64, std::time::Instant>>>,
    last_fired_round: Arc<RwLock<HashMap<i64, i64>>>,
    executed_once_match: Arc<RwLock<HashSet<i64>>>,
}

/// PSS Event types extracted from protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PssEventType {
    // Points
    Pt1(String), // pt1;point_value;
    Pt2(String), // pt2;point_value;
    
    // Hit Levels
    Hl1(String), // hl1;hit_level;
    Hl2(String), // hl2;hit_level;
    
    // Warnings
    Wg1(String), // wg1;warning_count;
    Wg2(String), // wg2;warning_count;
    
    // Injury
    Ij0(String), // ij0;time;
    Ij1(String), // ij1;time;
    Ij2(String), // ij2;time;
    
    // Challenges
    Ch0(String), // ch0;result;
    Ch1(String), // ch1;result;
    Ch2(String), // ch2;result;
    
    // Break
    Brk(String), // brk;time;
    
    // Winner Rounds
    Wrd(String), // wrd;rd1;winner1;rd2;winner2;rd3;winner3;
    
    // Winner
    Wmh(String), // wmh;winner_name;classification;
    
    // Athletes
    At1(String), // at1;short_name;long_name;country;at2;short_name2;long_name2;country2;
    
    // Match Configuration
    Mch(String), // mch;match_number;category;weight_class;...
    
    // Scores
    S11(String), // s11;score;
    S21(String), // s21;score;
    S12(String), // s12;score;
    S22(String), // s22;score;
    S13(String), // s13;score;
    S23(String), // s23;score;
    
    // Current Scores
    Sc1(String), // sc1;current_score;
    Sc2(String), // sc2;current_score;
    
    // Athlete Video Time
    Avt(String), // avt;video_time;
    
    // Clock
    Clk(String), // clk;time;
    
    // Round
    Rnd(String), // rnd;round_number;
    
    // Fight Ready
    Rdy(String), // rdy;FightReady;
    
    // Fight Loaded
    Pre(String), // pre;FightLoaded;
    
    // Winner
    Win(String), // win;BLUE/RED;
}

/// Trigger types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    Scene,
    Overlay,
    Both,
}

impl From<String> for TriggerType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "scene" => TriggerType::Scene,
            "overlay" => TriggerType::Overlay,
            "both" => TriggerType::Both,
            _ => TriggerType::Scene, // Default
        }
    }
}

impl From<TriggerType> for String {
    fn from(t: TriggerType) -> Self {
        match t {
            TriggerType::Scene => "scene".to_string(),
            TriggerType::Overlay => "overlay".to_string(),
            TriggerType::Both => "both".to_string(),
        }
    }
}

/// Trigger execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerExecutionResult {
    pub trigger_id: i64,
    pub event_type: String,
    pub trigger_type: TriggerType,
    pub success: bool,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
}

impl Clone for TriggerPlugin {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            obs_plugin_manager: self.obs_plugin_manager.clone(),
            enabled_triggers: self.enabled_triggers.clone(),
            current_tournament_id: self.current_tournament_id.clone(),
            current_tournament_day_id: self.current_tournament_day_id.clone(),
            paused: self.paused.clone(),
            buffered_rdy: self.buffered_rdy.clone(),
            resume_delay_ms: self.resume_delay_ms.clone(),
            current_round: self.current_round.clone(),
            last_fired_at: self.last_fired_at.clone(),
            last_fired_round: self.last_fired_round.clone(),
            executed_once_match: self.executed_once_match.clone(),
        }
    }
}

impl TriggerPlugin {
    /// Create a new trigger plugin
    pub fn new(db: Arc<DatabaseConnection>, obs_plugin_manager: Arc<ObsManager>) -> Self {
        Self {
            db,
            obs_plugin_manager,
            enabled_triggers: Arc::new(RwLock::new(HashMap::new())),
            current_tournament_id: Arc::new(RwLock::new(None)),
            current_tournament_day_id: Arc::new(RwLock::new(None)),
            paused: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            buffered_rdy: Arc::new(RwLock::new(None)),
            resume_delay_ms: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(2000)),
            current_round: Arc::new(RwLock::new(None)),
            last_fired_at: Arc::new(RwLock::new(HashMap::new())),
            last_fired_round: Arc::new(RwLock::new(HashMap::new())),
            executed_once_match: Arc::new(RwLock::new(std::collections::HashSet::new())),
        }
    }
    
    /// Initialize the trigger plugin
    pub async fn initialize(&self) -> AppResult<()> {
        log::info!("🎯 Initializing Trigger Plugin");
        // Register global shortcuts for pause/resume
        // Store global reference
        let _ = TRIGGER_PLUGIN_GLOBAL.set(std::sync::Arc::new(self.clone()));
        // Global shortcuts require the optional plugin; wrap in cfg to compile when present
        #[cfg(feature = "custom-protocol")]
        {
            use tauri::Manager;
            if let Some(app_handle) = tauri::AppHandle::try_get() {
                let plugin_clone = self.clone();
                // Pause shortcut
                let _ = tauri_plugin_global_shortcut::register(&app_handle, "Ctrl+Shift+P", move || {
                    plugin_clone.set_paused(true);
                });
                let plugin_clone2 = self.clone();
                let _ = tauri_plugin_global_shortcut::register(&app_handle, "Ctrl+Shift+R", move || {
                    plugin_clone2.set_paused(false);
                });
            }
        }
        
        // Load all enabled triggers
        self.load_enabled_triggers().await?;
        
        // Initialize default overlay templates if none exist
        self.initialize_default_overlay_templates().await?;
        
        log::info!("✅ Trigger Plugin initialized successfully");
        Ok(())
    }
    
    /// Load all enabled triggers into memory
    async fn load_enabled_triggers(&self) -> AppResult<()> {
        let mut triggers = self.enabled_triggers.write().await;
        triggers.clear();
        
        // Load global triggers
        let global_triggers = self.db.get_global_event_triggers().await?;
        for trigger in global_triggers {
            if trigger.is_enabled {
                triggers.entry(trigger.event_type.clone())
                    .or_insert_with(Vec::new)
                    .push(trigger);
            }
        }
        
        // Load tournament-specific triggers
        let tournament_id = *self.current_tournament_id.read().await;
        if let Some(tid) = tournament_id {
            let tournament_triggers = self.db.get_event_triggers_for_tournament(tid).await?;
            for trigger in tournament_triggers {
                if trigger.is_enabled {
                    triggers.entry(trigger.event_type.clone())
                        .or_insert_with(Vec::new)
                        .push(trigger);
                }
            }
        }
        
        // Load tournament day-specific triggers
        let tournament_day_id = *self.current_tournament_day_id.read().await;
        if let Some(tdid) = tournament_day_id {
            let day_triggers = self.db.get_event_triggers_for_tournament_day(tdid).await?;
            for trigger in day_triggers {
                if trigger.is_enabled {
                    triggers.entry(trigger.event_type.clone())
                        .or_insert_with(Vec::new)
                        .push(trigger);
                }
            }
        }
        
        log::info!("📋 Loaded {} trigger types with {} total triggers", triggers.len(), triggers.values().map(|v| v.len()).sum::<usize>());
        Ok(())
    }
    
    /// Initialize default overlay templates
    async fn initialize_default_overlay_templates(&self) -> AppResult<()> {
        let existing_templates = self.db.get_overlay_templates().await?;
        if !existing_templates.is_empty() {
            return Ok(());
        }
        
        log::info!("🎨 Creating default overlay templates");
        
        let default_templates = vec![
            OverlayTemplate {
                id: None,
                name: "Scoreboard Overlay".to_string(),
                description: Some("Real-time scoreboard with player information".to_string()),
                theme: "dark".to_string(),
                colors: Some("blue,red,white".to_string()),
                animation_type: "fade_in".to_string(),
                duration_ms: 3000,
                is_active: true,
                url: Some("assets/scoreboard/scoreboard-overlay.svg".to_string()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            OverlayTemplate {
                id: None,
                name: "Player Introduction".to_string(),
                description: Some("Player introduction with flags and names".to_string()),
                theme: "dark".to_string(),
                colors: Some("blue,red,white".to_string()),
                animation_type: "slide_in".to_string(),
                duration_ms: 5000,
                is_active: true,
                url: Some("assets/scoreboard/player-introduction-overlay.svg".to_string()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            OverlayTemplate {
                id: None,
                name: "Winner Announcement".to_string(),
                description: Some("Winner announcement with celebration effects".to_string()),
                theme: "dark".to_string(),
                colors: Some("gold,white".to_string()),
                animation_type: "zoom_in".to_string(),
                duration_ms: 8000,
                is_active: true,
                url: Some("assets/scoreboard/winner-announcement-overlay.svg".to_string()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            OverlayTemplate {
                id: None,
                name: "Point Scored".to_string(),
                description: Some("Point scored animation with sound effects".to_string()),
                theme: "dark".to_string(),
                colors: Some("green,white".to_string()),
                animation_type: "pulse".to_string(),
                duration_ms: 2000,
                is_active: true,
                url: Some("assets/scoreboard/scoreboard-overlay.svg".to_string()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            OverlayTemplate {
                id: None,
                name: "Warning Issued".to_string(),
                description: Some("Warning/gam-jeom notification".to_string()),
                theme: "dark".to_string(),
                colors: Some("red,yellow".to_string()),
                animation_type: "shake".to_string(),
                duration_ms: 3000,
                is_active: true,
                url: Some("assets/scoreboard/scoreboard-overlay.svg".to_string()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ];
        
        for template in &default_templates {
            self.db.insert_overlay_template(template).await?;
        }
        
        log::info!("✅ Created {} default overlay templates", default_templates.len());
        Ok(())
    }
    
    /// Public setter to pause/resume system; emits Tauri event and handles buffered rdy replay
    pub fn set_resume_delay(&self, ms: u64) {
        self.resume_delay_ms.store(ms, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_paused(&self, paused: bool) {
        let was = self.paused.swap(paused, std::sync::atomic::Ordering::SeqCst);
        if was == paused {
            return;
        }
        // Emit tauri event on state change
                #[cfg(feature = "custom-protocol")]
        if let Some(app) = tauri::AppHandle::try_get() {
            let _ = app.emit_all("triggers_paused_changed", paused);
        }
        if !paused {
            // resumed – spawn task to process buffered event after delay
            let delay_ms = self.resume_delay_ms.load(std::sync::atomic::Ordering::SeqCst);
            let plugin = self.clone();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                let mut buf = plugin.buffered_rdy.write().await;
                if let Some(rdy_raw) = buf.take() {
                    let msg = format!("rdy;{}", rdy_raw);
                    let _ = plugin.process_pss_event(&msg).await;
                }
            });
        }
    }

    /// Parse PSS message and extract event type
    pub fn parse_pss_message(&self, message: &str) -> Option<PssEventType> {
        let parts: Vec<&str> = message.split(';').collect();
        if parts.is_empty() {
            return None;
        }
        
        let event_code = parts[0];
        let args = parts.get(1..).unwrap_or(&[]).join(";");
        
        match event_code {
            "pt1" => Some(PssEventType::Pt1(args)),
            "pt2" => Some(PssEventType::Pt2(args)),
            "hl1" => Some(PssEventType::Hl1(args)),
            "hl2" => Some(PssEventType::Hl2(args)),
            "wg1" => Some(PssEventType::Wg1(args)),
            "wg2" => Some(PssEventType::Wg2(args)),
            "ij0" => Some(PssEventType::Ij0(args)),
            "ij1" => Some(PssEventType::Ij1(args)),
            "ij2" => Some(PssEventType::Ij2(args)),
            "ch0" => Some(PssEventType::Ch0(args)),
            "ch1" => Some(PssEventType::Ch1(args)),
            "ch2" => Some(PssEventType::Ch2(args)),
            "brk" => Some(PssEventType::Brk(args)),
            "wrd" => Some(PssEventType::Wrd(args)),
            "wmh" => Some(PssEventType::Wmh(args)),
            "at1" => Some(PssEventType::At1(args)),
            "mch" => Some(PssEventType::Mch(args)),
            "s11" => Some(PssEventType::S11(args)),
            "s21" => Some(PssEventType::S21(args)),
            "s12" => Some(PssEventType::S12(args)),
            "s22" => Some(PssEventType::S22(args)),
            "s13" => Some(PssEventType::S13(args)),
            "s23" => Some(PssEventType::S23(args)),
            "sc1" => Some(PssEventType::Sc1(args)),
            "sc2" => Some(PssEventType::Sc2(args)),
            "avt" => Some(PssEventType::Avt(args)),
            "clk" => Some(PssEventType::Clk(args)),
            "rnd" => Some(PssEventType::Rnd(args)),
            "rdy" => Some(PssEventType::Rdy(args)),
            "pre" => Some(PssEventType::Pre(args)),
            "win" => Some(PssEventType::Win(args)),
            _ => None,
        }
    }
    
    /// Process PSS event and execute triggers
    pub async fn process_pss_event(&self, message: &str) -> AppResult<Vec<TriggerExecutionResult>> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::new();
        
        // Parse the PSS message
        let event_type = match self.parse_pss_message(message) {
            Some(event) => event,
            None => {
                log::debug!("⚠️ Could not parse PSS message: {}", message);
                return Ok(results);
            }
        };
        
        // capture round/context updates & resets
        match &event_type {
            PssEventType::Rnd(args) => {
                // args is like "3" or similar
                if let Ok(r) = args.trim().parse::<i64>() {
                    let mut cr = self.current_round.write().await;
                    *cr = Some(r);
                }
            }
            PssEventType::Pre(_) => {
                // New fight loaded → reset once-per-match tracking
                self.executed_once_match.write().await.clear();
                self.last_fired_round.write().await.clear();
            }
            _ => {}
        }

        // If system is paused, only buffer the last 'rdy' message and ignore others
        if self.paused.load(std::sync::atomic::Ordering::SeqCst) {
            if let PssEventType::Rdy(raw) = &event_type {
                let mut buf = self.buffered_rdy.write().await;
                *buf = Some(raw.clone());
            }
            return Ok(results);
        }
        // Get event type string for trigger lookup
        let event_type_str = match &event_type {
            PssEventType::Pt1(_) => "pt1",
            PssEventType::Pt2(_) => "pt2",
            PssEventType::Hl1(_) => "hl1",
            PssEventType::Hl2(_) => "hl2",
            PssEventType::Wg1(_) => "wg1",
            PssEventType::Wg2(_) => "wg2",
            PssEventType::Ij0(_) => "ij0",
            PssEventType::Ij1(_) => "ij1",
            PssEventType::Ij2(_) => "ij2",
            PssEventType::Ch0(_) => "ch0",
            PssEventType::Ch1(_) => "ch1",
            PssEventType::Ch2(_) => "ch2",
            PssEventType::Brk(_) => "brk",
            PssEventType::Wrd(_) => "wrd",
            PssEventType::Wmh(_) => "wmh",
            PssEventType::At1(_) => "at1",
            PssEventType::Mch(_) => "mch",
            PssEventType::S11(_) => "s11",
            PssEventType::S21(_) => "s21",
            PssEventType::S12(_) => "s12",
            PssEventType::S22(_) => "s22",
            PssEventType::S13(_) => "s13",
            PssEventType::S23(_) => "s23",
            PssEventType::Sc1(_) => "sc1",
            PssEventType::Sc2(_) => "sc2",
            PssEventType::Avt(_) => "avt",
            PssEventType::Clk(_) => "clk",
            PssEventType::Rnd(_) => "rnd",
            PssEventType::Rdy(_) => "rdy",
            PssEventType::Pre(_) => "pre",
            PssEventType::Win(_) => "win",
        };
        
        // Get triggers for this event type
        let triggers = self.enabled_triggers.read().await;
        let event_triggers = triggers.get(event_type_str).cloned().unwrap_or_default();
        drop(triggers);
        
        if event_triggers.is_empty() {
            log::debug!("📭 No triggers found for event type: {}", event_type_str);
            return Ok(results);
        }
        
        log::info!("🎯 Processing {} triggers for event: {}", event_triggers.len(), event_type_str);
        
        // Execute each trigger
        for trigger in event_triggers {
            // Simple condition matcher (round/once-per/debounce/cooldown)
            if !self.should_fire(&trigger).await {
                continue;
            }
            let trigger_start = std::time::Instant::now();
            let trigger_type: TriggerType = trigger.trigger_type.clone().into();
            let mut result = TriggerExecutionResult {
                trigger_id: trigger.id.unwrap_or(0),
                event_type: event_type_str.to_string(),
                trigger_type: trigger_type.clone(),
                success: false,
                error_message: None,
                execution_time_ms: 0,
            };
            
            match self.execute_trigger(&trigger, &event_type).await {
                Ok(_) => {
                    result.success = true;
                    // mark fired
                    if let Some(id) = trigger.id { self.mark_fired(id).await; }
                    log::info!("✅ Trigger {} executed successfully", trigger.id.unwrap_or(0));
                }
                Err(e) => {
                    result.error_message = Some(e.to_string());
                    log::error!("❌ Trigger {} failed: {}", trigger.id.unwrap_or(0), e);
                }
            }
            
            result.execution_time_ms = trigger_start.elapsed().as_millis() as u64;
            results.push(result);
        }
        
        let total_time = start_time.elapsed();
        log::info!("🎯 Processed {} triggers in {:?}", results.len(), total_time);
        
        Ok(results)
    }

    /// Evaluate basic Triggers v2 conditions and rate limits
    async fn should_fire(&self, trigger: &EventTrigger) -> bool {
        let id = match trigger.id { Some(v) => v, None => return true };
        // Round condition
        if let Some(req_round) = trigger.condition_round {
            let cr = *self.current_round.read().await;
            if cr != Some(req_round) { return false; }
        }
        // Once-per scope
        if let Some(scope) = trigger.condition_once_per.as_deref() {
            match scope {
                "match" => {
                    if self.executed_once_match.read().await.contains(&id) { return false; }
                }
                "round" => {
                    if let (Some(cr), Some(last_r)) = (*self.current_round.read().await, self.last_fired_round.read().await.get(&id).cloned()) {
                        if last_r == cr { return false; }
                    }
                }
                _ => {}
            }
        }
        // Debounce/Cooldown
        let now = std::time::Instant::now();
        if let Some(prev) = self.last_fired_at.read().await.get(&id).cloned() {
            let elapsed_ms = now.duration_since(prev).as_millis() as i64;
            let need_gap = trigger.cooldown_ms.unwrap_or(0).max(trigger.debounce_ms.unwrap_or(0));
            if need_gap > 0 && elapsed_ms < need_gap { return false; }
        }
        true
    }

    /// Preview evaluation without mutating runtime state.
    pub async fn should_fire_preview(&self, trigger: &EventTrigger, consider_limits: bool) -> bool {
        // Round check
        if let Some(req_round) = trigger.condition_round {
            let cr = *self.current_round.read().await;
            if cr != Some(req_round) { return false; }
        }
        if consider_limits {
            // Once-per and rate limits similar to live path, but do not mutate
            if let Some(scope) = trigger.condition_once_per.as_deref() {
                if let Some(id) = trigger.id {
                    match scope {
                        "match" => {
                            if self.executed_once_match.read().await.contains(&id) { return false; }
                        }
                        "round" => {
                            if let (Some(cr), Some(last_r)) = (*self.current_round.read().await, self.last_fired_round.read().await.get(&id).cloned()) {
                                if last_r == cr { return false; }
                            }
                        }
                        _ => {}
                    }
                }
            }
            if let Some(id) = trigger.id {
                if let Some(prev) = self.last_fired_at.read().await.get(&id).cloned() {
                    let elapsed_ms = std::time::Instant::now().duration_since(prev).as_millis() as i64;
                    let need_gap = trigger.cooldown_ms.unwrap_or(0).max(trigger.debounce_ms.unwrap_or(0));
                    if need_gap > 0 && elapsed_ms < need_gap { return false; }
                }
            }
        }
        true
    }

    /// Mark trigger as fired for rate limits
    async fn mark_fired(&self, id: i64) {
        self.last_fired_at.write().await.insert(id, std::time::Instant::now());
        if let Some(cr) = *self.current_round.read().await {
            self.last_fired_round.write().await.insert(id, cr);
        }
        self.executed_once_match.write().await.insert(id);
    }

    /// Return a snapshot of recent execution logs
    pub async fn get_recent_execution_logs(&self, _max: usize) -> Vec<serde_json::Value> {
        // Minimal stub until recent_executions queue is introduced
        vec![]
    }
    
    /// Execute a single trigger
    async fn execute_trigger(&self, trigger: &EventTrigger, event: &PssEventType) -> AppResult<()> {
        let trigger_type: TriggerType = trigger.trigger_type.clone().into();
        
        // Action-kind aware executor (v2). Falls back to legacy trigger_type.
        if let Some(kind) = trigger.action_kind.clone() {
            match kind.as_str() {
                "scene" => { self.execute_scene_trigger(trigger).await?; }
                "overlay" => { self.execute_overlay_trigger(trigger, event).await?; }
                "record_start" => { self.execute_record_action(trigger, true).await?; }
                "record_stop" => { self.execute_record_action(trigger, false).await?; }
                "replay_save" => { self.execute_replay_save(trigger).await?; }
                _ => { self.execute_scene_trigger(trigger).await?; }
            }
        } else {
            match trigger_type {
                TriggerType::Scene => { self.execute_scene_trigger(trigger).await?; }
                TriggerType::Overlay => { self.execute_overlay_trigger(trigger, event).await?; }
                TriggerType::Both => {
                    self.execute_scene_trigger(trigger).await?;
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    self.execute_overlay_trigger(trigger, event).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Execute scene change trigger
    async fn execute_scene_trigger(&self, trigger: &EventTrigger) -> AppResult<()> {
        let scene_id = match trigger.obs_scene_id {
            Some(id) => id,
            None => {
                return Err(crate::types::AppError::ConfigError("No OBS scene ID specified for trigger".to_string()));
            }
        };
        
        // Get scene details from database
        let scenes = self.db.get_obs_scenes().await?;
        let scene = scenes.iter().find(|s| s.id == Some(scene_id))
            .ok_or_else(|| crate::types::AppError::ConfigError(format!("OBS scene with ID {} not found", scene_id)))?;
        
        if !scene.is_active {
            return Err(crate::types::AppError::ConfigError(format!("OBS scene '{}' is not active", scene.scene_name)));
        }
        
        // Use targeted connection if provided; fallback to default
        let conn_name = trigger.obs_connection_name.as_deref().unwrap_or("default");
        self.obs_plugin_manager.set_current_scene(&scene.scene_name, Some(conn_name)).await?;
        
        log::info!("🎬 Changed OBS scene to: {}", scene.scene_name);
        Ok(())
    }

    async fn execute_record_action(&self, trigger: &EventTrigger, start: bool) -> AppResult<()> {
        let conn_name = trigger.obs_connection_name.as_deref().unwrap_or("OBS_REC");
        if start {
            self.obs_plugin_manager.start_recording(Some(conn_name)).await?;
            log::info!("🎥 Started recording on {}", conn_name);
        } else {
            self.obs_plugin_manager.stop_recording(Some(conn_name)).await?;
            log::info!("🛑 Stopped recording on {}", conn_name);
        }
        Ok(())
    }

    async fn execute_replay_save(&self, trigger: &EventTrigger) -> AppResult<()> {
        let conn_name = trigger.obs_connection_name.as_deref().unwrap_or("OBS_REC");
        // Use obws manager to save replay buffer
        if let Err(e) = self.obs_plugin_manager.save_replay_buffer(Some(conn_name)).await {
            log::warn!("Failed to save replay buffer on {}: {}", conn_name, e);
            return Err(e);
        }
        log::info!("💾 Save Replay Buffer executed on {}", conn_name);
        Ok(())
    }
    
    /// Execute overlay animation trigger
    async fn execute_overlay_trigger(&self, trigger: &EventTrigger, event: &PssEventType) -> AppResult<()> {
        let template_id = match trigger.overlay_template_id {
            Some(id) => id,
            None => {
                return Err(crate::types::AppError::ConfigError("No overlay template ID specified for trigger".to_string()));
            }
        };
        
        // Get overlay template from database
        let templates = self.db.get_overlay_templates().await?;
        let template = templates.iter().find(|t| t.id == Some(template_id))
            .ok_or_else(|| crate::types::AppError::ConfigError(format!("Overlay template with ID {} not found", template_id)))?;
        
        if !template.is_active {
            return Err(crate::types::AppError::ConfigError(format!("Overlay template '{}' is not active", template.name)));
        }
        
        // Execute overlay animation
        self.execute_overlay_animation(template, event).await?;
        
        log::info!("🎨 Executed overlay animation: {}", template.name);
        Ok(())
    }
    
    /// Execute overlay animation
    async fn execute_overlay_animation(&self, template: &OverlayTemplate, _event: &PssEventType) -> AppResult<()> {
        // This would integrate with the existing overlay system
        // For now, we'll log the animation details
        log::info!("🎨 Overlay Animation: {} ({}) - Duration: {}ms", 
            template.name, 
            template.animation_type, 
            template.duration_ms
        );
        
        // TODO: Integrate with existing overlay system
        // - Send WebSocket message to overlay HTML pages
        // - Apply theme and color configurations
        // - Handle animation timing and effects
        
        Ok(())
    }
    
    /// Set current tournament context
    pub async fn set_tournament_context(&self, tournament_id: Option<i64>, tournament_day_id: Option<i64>) -> AppResult<()> {
        let mut current_tournament = self.current_tournament_id.write().await;
        let mut current_day = self.current_tournament_day_id.write().await;
        
        *current_tournament = tournament_id;
        *current_day = tournament_day_id;
        
        // Reload triggers for new context
        self.load_enabled_triggers().await?;
        
        log::info!("🎯 Set tournament context: tournament_id={:?}, day_id={:?}", tournament_id, tournament_day_id);
        Ok(())
    }
    
    /// Sync OBS scenes from WebSocket connection
    pub async fn sync_obs_scenes(&self, scene_names: Vec<String>) -> AppResult<()> {
        // Update database with current OBS scenes
        self.db.sync_obs_scenes(&scene_names).await?;
        
        log::info!("🔄 Synced {} OBS scenes", scene_names.len());
        Ok(())
    }
    
    /// Get trigger statistics
    pub async fn get_trigger_statistics(&self) -> AppResult<serde_json::Value> {
        let triggers = self.db.get_event_triggers().await?;
        let enabled_count = triggers.iter().filter(|t| t.is_enabled).count();
        let disabled_count = triggers.len() - enabled_count;
        
        let mut trigger_counts = std::collections::HashMap::new();
        for trigger in &triggers {
            *trigger_counts.entry(&trigger.event_type).or_insert(0) += 1;
        }
        
        let stats = serde_json::json!({
            "total_triggers": triggers.len(),
            "enabled_triggers": enabled_count,
            "disabled_triggers": disabled_count,
            "trigger_types": trigger_counts,
            "current_tournament_id": *self.current_tournament_id.read().await,
            "current_tournament_day_id": *self.current_tournament_day_id.read().await,
        });
        
        Ok(stats)
    }
}

/// Initialize the trigger plugin
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Initializing Trigger Plugin");
    // The actual initialization happens when the plugin is created
    Ok(())
} 