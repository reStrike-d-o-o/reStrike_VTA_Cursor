use crate::database::models::OverlayTemplate;
use chrono::Utc;
use tauri::{State, command, Error as TauriError};
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct OverlayTemplatePayload {
    pub name: String,
    pub theme: Option<String>,
    pub description: Option<String>,
    pub animation_type: Option<String>,
    pub duration_ms: Option<i32>,
    pub colors: Option<String>,
    pub is_active: Option<bool>,
}

#[command]
pub async fn overlays_sync_templates(app: State<'_, Arc<crate::App>>, templates: Vec<OverlayTemplatePayload>) -> Result<Vec<OverlayTemplate>, TauriError> {
    let conn = app.database_plugin().get_database_connection();

    // Insert or update each template
    for t in templates {
        let now = Utc::now();
        let tpl = OverlayTemplate {
            id: None,
            name: t.name,
            description: t.description,
            theme: t.theme.unwrap_or_else(|| "default".into()),
            colors: t.colors,
            animation_type: t.animation_type.unwrap_or_else(|| "fade".into()),
            duration_ms: t.duration_ms.unwrap_or(3000),
            is_active: t.is_active.unwrap_or(true),
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