use crate::config::types::OverlaySettings as UiOverlaySettings;
use crate::core::app::App;
use crate::database::models::OverlayTemplate;
use crate::types::OverlayRoutingRule;
use chrono::Utc;
use std::sync::Arc;
use tauri::{command, Error as TauriError, Manager, State, Window};

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

/// Close an overlay window by its label (e.g. "overlay_olympic").
#[command]
pub async fn close_overlay_window(window: Window, label: String) -> Result<(), TauriError> {
    let app_handle = window.app_handle();
    if let Some(webview) = app_handle.get_webview_window(&label) {
        if let Err(err) = webview.close() {
            log::warn!("Failed to close overlay window '{label}': {err}");
        }
    }
    Ok(())
}

#[command]
pub async fn overlays_sync_templates(
    app: State<'_, Arc<crate::App>>,
    templates: Vec<OverlayTemplatePayload>,
) -> Result<Vec<OverlayTemplate>, TauriError> {
    let db = app.database_plugin();

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
        db.upsert_overlay_template(&tpl)
            .await
            .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    }

    // Return fresh list
    let list = db
        .get_active_overlay_templates()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    Ok(list)
}

#[command]
pub async fn overlays_populate_from_files(
    app: State<'_, Arc<crate::App>>,
) -> Result<Vec<OverlayTemplate>, TauriError> {
    let db = app.database_plugin();

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
            url: Some("overlays/olympic/scoreboard.html".to_string()),
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
            url: Some("overlays/olympic/intro.html".to_string()),
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
            url: Some("overlays/olympic/intro.html".to_string()),
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
            url: Some("overlays/olympic/scoreboard.html".to_string()),
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
            url: Some("overlays/olympic/intro.html".to_string()),
        },
    ];

    // Clear existing templates by getting all and deleting them
    db.clear_overlay_templates()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

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
        db.upsert_overlay_template(&tpl)
            .await
            .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    }

    // Return fresh list
    let list = db
        .get_active_overlay_templates()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    Ok(list)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OverlayRoutingRuleConfig {
    pub trigger: String,
    pub overlay: String,
    pub action: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OverlayRoutingConfigPayload {
    pub rules: Vec<OverlayRoutingRuleConfig>,
}

#[command]
pub async fn get_overlay_routing_config(
    app: State<'_, Arc<App>>,
) -> Result<OverlayRoutingConfigPayload, TauriError> {
    let config = app.config_manager().get_config().await;
    let settings: &UiOverlaySettings = &config.ui.overlay;

    let rules: Vec<OverlayRoutingRule> = settings
        .routing_rules
        .clone()
        .unwrap_or_default();

    let payload = OverlayRoutingConfigPayload {
        rules: rules
            .into_iter()
            .map(|r| OverlayRoutingRuleConfig {
                trigger: r.trigger,
                overlay: r.overlay,
                action: r.action,
            })
            .collect(),
    };

    Ok(payload)
}

#[command]
pub async fn set_overlay_routing_config(
    app: State<'_, Arc<App>>,
    config: OverlayRoutingConfigPayload,
) -> Result<(), TauriError> {
    let rules: Vec<OverlayRoutingRule> = config
        .rules
        .into_iter()
        .map(|r| OverlayRoutingRule {
            trigger: r.trigger,
            overlay: r.overlay,
            action: r.action,
        })
        .collect();

    app.config_manager()
        .update_section(|cfg| {
            cfg.ui.overlay.routing_rules = Some(rules.clone());
            &mut cfg.ui.overlay
        })
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    Ok(())
}
