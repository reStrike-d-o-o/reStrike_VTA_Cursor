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
    let scenes = app
        .database_plugin().get_database_connection()
        .get_active_obs_scenes()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    Ok(scenes)
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
