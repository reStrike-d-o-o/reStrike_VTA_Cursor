use crate::database::models::OverlayTemplate;
use chrono::Utc;
use tauri::{State, command, Error as TauriError};
use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OverlayTemplatePayload {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub theme: Option<String>,
    pub colors: Option<String>,
    pub animation_type: Option<String>,
    pub duration_ms: Option<i32>,
    pub is_active: Option<bool>,
    pub url: Option<String>,
}

#[command]
pub async fn overlays_sync_templates(app: State<'_, Arc<crate::App>>, templates: Vec<OverlayTemplatePayload>) -> Result<Vec<OverlayTemplate>, TauriError> {
    let conn = app.database_plugin().get_database_connection();

    // Insert or update each template
    for t in templates {
        let now = Utc::now();
        let tpl = OverlayTemplate {
            id: t.id,
            name: t.name,
            description: t.description,
            theme: t.theme.unwrap_or_else(|| "default".to_string()),
            colors: t.colors,
            animation_type: t.animation_type.unwrap_or_else(|| "fade".to_string()),
            duration_ms: t.duration_ms.unwrap_or(3000),
            is_active: t.is_active.unwrap_or(true),
            url: t.url,
            created_at: now,
            updated_at: now,
        };
        conn.insert_overlay_template(&tpl)
            .await
            .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    }

    // Return fresh list
    let list = conn
        .get_active_overlay_templates()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    Ok(list)
}

#[command]
pub async fn overlays_populate_from_files(app: State<'_, Arc<crate::App>>) -> Result<Vec<OverlayTemplate>, TauriError> {
    let conn = app.database_plugin().get_database_connection();

    // Define overlay templates based on existing SVG files
    let overlay_templates = vec![
        OverlayTemplatePayload {
            id: None,
            name: "Live Scoreboard".to_string(),
            description: Some("Real-time match scoreboard overlay".to_string()),
            theme: Some("default".to_string()),
            colors: Some("blue,red".to_string()),
            animation_type: Some("fade".to_string()),
            duration_ms: Some(3000),
            is_active: Some(true),
            url: Some("assets/scoreboard/scoreboard-overlay.svg".to_string()),
        },
        OverlayTemplatePayload {
            id: None,
            name: "Player Introduction".to_string(),
            description: Some("Player introduction overlay".to_string()),
            theme: Some("default".to_string()),
            colors: Some("blue,red".to_string()),
            animation_type: Some("slide".to_string()),
            duration_ms: Some(5000),
            is_active: Some(true),
            url: Some("assets/scoreboard/player-introduction-overlay.svg".to_string()),
        },
        OverlayTemplatePayload {
            id: None,
            name: "Winner Announcement".to_string(),
            description: Some("Winner announcement overlay".to_string()),
            theme: Some("default".to_string()),
            colors: Some("gold,silver".to_string()),
            animation_type: Some("zoom".to_string()),
            duration_ms: Some(4000),
            is_active: Some(true),
            url: Some("assets/scoreboard/winner-announcement-overlay.svg".to_string()),
        },
        OverlayTemplatePayload {
            id: None,
            name: "Previous Results".to_string(),
            description: Some("Player match history overlay".to_string()),
            theme: Some("default".to_string()),
            colors: Some("gray,white".to_string()),
            animation_type: Some("fade".to_string()),
            duration_ms: Some(3000),
            is_active: Some(true),
            url: Some("assets/scoreboard/previous-results-overlay.svg".to_string()),
        },
        OverlayTemplatePayload {
            id: None,
            name: "Victory Ceremony".to_string(),
            description: Some("4-player medal ceremony overlay".to_string()),
            theme: Some("olympic".to_string()),
            colors: Some("gold,silver,bronze".to_string()),
            animation_type: Some("reveal".to_string()),
            duration_ms: Some(6000),
            is_active: Some(true),
            url: Some("assets/scoreboard/victory-ceremony-overlay.svg".to_string()),
        },
    ];

    // Clear existing templates by getting all and deleting them
    let existing_templates = conn.get_overlay_templates().await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    
    for template in existing_templates {
        if let Some(id) = template.id {
            conn.delete_overlay_template(id).await
                .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
        }
    }

    // Insert each template
    for t in overlay_templates {
        let now = Utc::now();
        let tpl = OverlayTemplate {
            id: t.id,
            name: t.name,
            description: t.description,
            theme: t.theme.unwrap_or_else(|| "default".to_string()),
            colors: t.colors,
            animation_type: t.animation_type.unwrap_or_else(|| "fade".to_string()),
            duration_ms: t.duration_ms.unwrap_or(3000),
            is_active: t.is_active.unwrap_or(true),
            url: t.url,
            created_at: now,
            updated_at: now,
        };
        conn.insert_overlay_template(&tpl)
            .await
            .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    }

    // Return fresh list
    let list = conn
        .get_active_overlay_templates()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    Ok(list)
}