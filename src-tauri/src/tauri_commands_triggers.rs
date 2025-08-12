use serde::{Serialize, Deserialize};
use tauri::{State, Error as TauriError};
use std::sync::Arc;
use chrono::Utc;

use crate::core::app::App;
use crate::database::models::{ObsScene, OverlayTemplate, EventTrigger};

// ---------------- LIST PSS EVENTS ----------------
#[tauri::command]
pub async fn triggers_list_pss_events() -> Result<Vec<String>, TauriError> {
    // Reduced authoritative list agreed with user
    Ok(vec!["pre", "rdy", "rnd", "sup", "wrd", "wmh"].into_iter().map(|s| s.to_string()).collect())
}

// ---------------- SCENES ----------------
#[tauri::command]
pub async fn triggers_list_obs_scenes(app: State<'_, Arc<App>>) -> Result<Vec<ObsScene>, TauriError> {
    // 1) Try database – preferred source
    let scenes_db = app
        .database_plugin().get_database_connection()
        .get_active_obs_scenes()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    if !scenes_db.is_empty() {
        return Ok(scenes_db);
    }

    // 2) Fallback: live query via obws (when legacy plugin has no synced scenes)
    #[cfg(feature = "obs-obws")]
    {
        let mut out: Vec<ObsScene> = Vec::new();
        let conn_names = app.obs_obws_plugin().get_connection_names().await;
        for name in conn_names {
            match app.obs_obws_plugin().get_scenes(Some(name.as_str())).await {
                Ok(list) => {
                    for s in list {
                        // Map to DB model shape (id None, active true, timestamps auto)
                        out.push(ObsScene::new(s.name.clone(), s.name.clone()));
                    }
                }
                Err(e) => {
                    log::warn!("Failed to fetch obws scenes for '{}': {}", name, e);
                }
            }
        }
        return Ok(out);
    }

    // 3) If obws feature is off, return empty
    #[cfg(not(feature = "obs-obws"))]
    {
        return Ok(Vec::new());
    }

    // If obs-obws feature is enabled, all code paths above returned already
    #[cfg(feature = "obs-obws")]
    {
        // This branch is only to satisfy type checker; it will never execute
        // because the obws-enabled path returns earlier.
        return Ok(Vec::new());
    }
}

// ---------------- OVERLAYS ----------------
#[tauri::command]
pub async fn triggers_list_active_overlays(app: State<'_, Arc<App>>) -> Result<Vec<OverlayTemplate>, TauriError> {
    let overlays = app
        .database_plugin().get_database_connection()
        .get_active_overlay_templates()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    Ok(overlays)
}

// ---------------- GET TRIGGERS ----------------
#[tauri::command]
pub async fn triggers_get(app: State<'_, Arc<App>>, tournament_id: Option<i64>, day_id: Option<i64>) -> Result<Vec<EventTrigger>, TauriError> {
    let conn = app.database_plugin().get_database_connection();
    let res = match (tournament_id, day_id) {
        (Some(_tid), Some(did)) => conn.get_event_triggers_for_tournament_day(did).await,
        (Some(tid), None) => conn.get_event_triggers_for_tournament(tid).await,
        _ => conn.get_event_triggers().await,
    };
    res.map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))
}

// ---------------- SAVE TRIGGERS ----------------
#[derive(Debug, Serialize, Deserialize)]
pub struct EventTriggerPayload {
    pub id: Option<i64>,
    pub tournament_id: Option<i64>,
    pub tournament_day_id: Option<i64>,
    pub event_type: String,
    pub action: String, // show / hide
    pub target_type: String, // scene / overlay
    pub obs_scene_id: Option<i64>,
    pub overlay_template_id: Option<i64>,
    pub delay_ms: Option<i64>,
    pub is_enabled: bool,
    pub priority: i32,
    // v2 additions
    pub action_kind: Option<String>,
    pub obs_connection_name: Option<String>,
    pub condition_round: Option<i64>,
    pub condition_once_per: Option<String>,
    pub debounce_ms: Option<i64>,
    pub cooldown_ms: Option<i64>,
}

#[tauri::command]
pub async fn triggers_save(app: State<'_, Arc<App>>, payload: Vec<EventTriggerPayload>, resume_delay_ms: Option<u64>) -> Result<(), TauriError> {
    let conn = app.database_plugin().get_database_connection();
    for p in payload {
        let now = Utc::now();
        let row = EventTrigger {
            action: p.action.clone(),
            target_type: p.target_type.clone(),
            delay_ms: p.delay_ms.unwrap_or(0),
            id: p.id,
            tournament_id: p.tournament_id,
            tournament_day_id: p.tournament_day_id,
            event_type: p.event_type.clone(),
            trigger_type: p.target_type.clone(), // legacy field still required elsewhere
            obs_scene_id: p.obs_scene_id,
            overlay_template_id: p.overlay_template_id,
            action_kind: p.action_kind.clone(),
            obs_connection_name: p.obs_connection_name.clone(),
            condition_round: p.condition_round,
            condition_once_per: p.condition_once_per.clone(),
            debounce_ms: p.debounce_ms,
            cooldown_ms: p.cooldown_ms,
            is_enabled: p.is_enabled,
            priority: p.priority,
            // Additional fields for new schema
            created_at: now,
            updated_at: now,
        };
        if row.id.is_some() {
            conn.update_event_trigger(&row).await.map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
        } else {
            conn.insert_event_trigger(&row).await.map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
        }
    }

    // Apply resume delay to plugin if provided
    if let Some(delay) = resume_delay_ms {
        if let Some(plugin) = crate::plugins::plugin_triggers::TRIGGER_PLUGIN_GLOBAL.get() {
            plugin.set_resume_delay(delay);
        }
    }
    Ok(())
}

// ---------------- RECENT EXECUTION LOGS ----------------
#[tauri::command]
pub async fn triggers_recent_logs(_app: State<'_, Arc<App>>, max: Option<usize>) -> Result<serde_json::Value, TauriError> {
    // SAFETY: keep interface stable; pull from global if available
    let logs = if let Some(p) = crate::plugins::plugin_triggers::TRIGGER_PLUGIN_GLOBAL.get() {
        p.get_recent_execution_logs(max.unwrap_or(50)).await
    } else { vec![] };
    Ok(serde_json::json!({
        "success": true,
        "logs": logs,
    }))
}

// ---------------- PREVIEW EVALUATION ----------------
#[tauri::command]
pub async fn triggers_preview_evaluate(
    _app: State<'_, Arc<App>>,
    trigger: EventTriggerPayload,
    consider_limits: Option<bool>,
) -> Result<serde_json::Value, TauriError> {
    // Build EventTrigger from payload
    let now = Utc::now();
    let row = EventTrigger {
        action: trigger.action.clone(),
        target_type: trigger.target_type.clone(),
        delay_ms: trigger.delay_ms.unwrap_or(0),
        id: trigger.id,
        tournament_id: trigger.tournament_id,
        tournament_day_id: trigger.tournament_day_id,
        event_type: trigger.event_type.clone(),
        trigger_type: trigger.target_type.clone(),
        obs_scene_id: trigger.obs_scene_id,
        overlay_template_id: trigger.overlay_template_id,
        action_kind: trigger.action_kind.clone(),
        obs_connection_name: trigger.obs_connection_name.clone(),
        condition_round: trigger.condition_round,
        condition_once_per: trigger.condition_once_per.clone(),
        debounce_ms: trigger.debounce_ms,
        cooldown_ms: trigger.cooldown_ms,
        is_enabled: trigger.is_enabled,
        priority: trigger.priority,
        created_at: now,
        updated_at: now,
    };

    let plugin = if let Some(p) = crate::plugins::plugin_triggers::TRIGGER_PLUGIN_GLOBAL.get() {
        p.clone()
    } else {
        return Ok(serde_json::json!({ "success": false, "error": "Trigger plugin not initialized" }));
    };

    let ok = plugin.should_fire_preview(&row, consider_limits.unwrap_or(false)).await;
    Ok(serde_json::json!({ "success": true, "can_fire": ok }))
}
