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