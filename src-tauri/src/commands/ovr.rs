use crate::core::app::App;
use crate::types::TauriError;
use std::sync::Arc;
use tauri::State;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as AsyncMutex;
use once_cell::sync::OnceCell;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OvrRefreshStatus {
    pub active: bool,
    pub total: usize,
    pub processed: usize,
    pub current_provider: Option<String>,
    pub cancelled: bool,
    pub last_error: Option<String>,
}

static OVR_REFRESH_STATE: OnceCell<Arc<AsyncMutex<OvrRefreshStatus>>> = OnceCell::new();

fn get_ovr_refresh_state() -> Arc<AsyncMutex<OvrRefreshStatus>> {
    OVR_REFRESH_STATE
        .get_or_init(|| {
            Arc::new(AsyncMutex::new(OvrRefreshStatus {
                active: false,
                total: 0,
                processed: 0,
                current_provider: None,
                cancelled: false,
                last_error: None,
            }))
        })
        .clone()
}

// Track currently running refresh task to allow cancellation
static OVR_REFRESH_TASK: OnceCell<Arc<AsyncMutex<Option<tokio::task::JoinHandle<()>>>>> =
    OnceCell::new();

fn get_ovr_refresh_task() -> Arc<AsyncMutex<Option<tokio::task::JoinHandle<()>>>> {
    OVR_REFRESH_TASK
        .get_or_init(|| Arc::new(AsyncMutex::new(None)))
        .clone()
}

#[tauri::command]
pub async fn ovr_get_providers(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    let db = app.database_plugin();

    if let Err(e) = db.ensure_default_overlay_providers().await {
        return Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }));
    }

    match db.get_overlay_providers().await {
        Ok(items) => Ok(serde_json::json!({"success": true, "providers": items})),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
pub async fn ovr_refresh_all(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    let plugin = crate::plugins::plugin_ovr::OvrScraperPlugin::new(app.database_plugin().clone());
    match plugin.refresh_all().await {
        Ok(n) => Ok(serde_json::json!({"success": true, "providers_processed": n})),
        Err(e) => Ok(serde_json::json!({"success": false, "error": e})),
    }
}

#[tauri::command]
pub async fn ovr_refresh_provider(
    app: State<'_, Arc<App>>,
    provider_id: i64,
) -> Result<serde_json::Value, TauriError> {
    let plugin = crate::plugins::plugin_ovr::OvrScraperPlugin::new(app.database_plugin().clone());
    match plugin.refresh_provider(provider_id).await {
        Ok(_) => Ok(serde_json::json!({"success": true})),
        Err(e) => {
            // Persist error status for direct refresh calls as well
            let _ = app
                .database_plugin()
                .set_overlay_provider_refresh_status(provider_id, Some("error"), Some(&e))
                .await;
            Ok(serde_json::json!({"success": false, "error": e}))
        }
    }
}

#[tauri::command]
pub async fn ovr_start_refresh_all(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    let state = get_ovr_refresh_state();
    app.database_plugin()
        .ensure_default_overlay_providers()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    let providers = app
        .database_plugin()
        .get_overlay_providers()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    {
        let mut st = state.lock().await;
        if st.active {
            return Ok(serde_json::json!({"success": false, "error": "already_running"}));
        }
        let total = providers.iter().filter(|p| p.enabled).count();
        st.active = true;
        st.cancelled = false;
        st.total = total;
        st.processed = 0;
        st.current_provider = None;
        st.last_error = None;
    }

    let app_arc = app.inner().clone();
    tokio::spawn(async move {
        let state = get_ovr_refresh_state();
        let providers = {
            match app_arc.database_plugin().get_overlay_providers().await {
                Ok(list) => list,
                Err(_) => vec![],
            }
        };
        let plugin =
            crate::plugins::plugin_ovr::OvrScraperPlugin::new(app_arc.database_plugin().clone());
        for p in providers.into_iter().filter(|p| p.enabled) {
            {
                let st = state.lock().await;
                if st.cancelled {
                    break;
                }
            }
            {
                let mut st = state.lock().await;
                st.current_provider = Some(p.name.clone());
            }
            let id = p.id.unwrap_or_default();
            // Spawn a cancellable subtask for this provider
            let app_clone = app_arc.clone();
            let state_clone = get_ovr_refresh_state();
            let plugin_clone = plugin.clone();
            let handle = tokio::spawn(async move {
                if let Err(e) = plugin_clone.refresh_provider(id).await {
                    let _ = app_clone
                        .database_plugin()
                        .set_overlay_provider_refresh_status(id, Some("error"), Some(&e))
                        .await;
                    let mut st = state_clone.lock().await;
                    st.last_error = Some(e);
                }
            });
            {
                let task_store = get_ovr_refresh_task();
                let mut guard = task_store.lock().await;
                *guard = Some(handle);
            }
            // Poll for cancellation while waiting
            loop {
                {
                    let st = state.lock().await;
                    if st.cancelled {
                        break;
                    }
                }
                let is_done = {
                    let task_store = get_ovr_refresh_task();
                    let guard = task_store.lock().await;
                    if let Some(h) = guard.as_ref() {
                        h.is_finished()
                    } else {
                        true
                    }
                };
                if is_done {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            }
            {
                let mut st = state.lock().await;
                st.processed = st.processed.saturating_add(1);
                if st.cancelled {
                    st.active = false;
                    st.current_provider = None;
                    break;
                }
            }
        }
        {
            let mut st = state.lock().await;
            st.active = false;
            st.current_provider = None;
        }
    });

    Ok(serde_json::json!({"success": true}))
}

#[tauri::command]
pub async fn ovr_start_refresh_provider(
    app: State<'_, Arc<App>>,
    provider_id: i64,
) -> Result<serde_json::Value, TauriError> {
    let state = get_ovr_refresh_state();
    {
        let mut st = state.lock().await;
        if st.active {
            return Ok(serde_json::json!({"success": false, "error": "already_running"}));
        }
        st.active = true;
        st.cancelled = false;
        st.total = 1;
        st.processed = 0;
        st.current_provider = None;
        st.last_error = None;
    }

    let app_arc = app.inner().clone();
    tokio::spawn(async move {
        let state = get_ovr_refresh_state();
        let provider_name: Option<String> = match app_arc
            .database_plugin()
            .get_overlay_provider_by_id(provider_id)
            .await
        {
            Ok(Some(provider)) => Some(provider.name),
            Ok(None) | Err(_) => None,
        };
        {
            let mut st = state.lock().await;
            st.current_provider = provider_name;
        }
        let plugin =
            crate::plugins::plugin_ovr::OvrScraperPlugin::new(app_arc.database_plugin().clone());
        // Spawn cancellable task
        let app_clone = app_arc.clone();
        let state_clone = get_ovr_refresh_state();
        let handle = tokio::spawn(async move {
            if let Err(e) = plugin.refresh_provider(provider_id).await {
                let _ = app_clone
                    .database_plugin()
                    .set_overlay_provider_refresh_status(provider_id, Some("error"), Some(&e))
                    .await;
                let mut st = state_clone.lock().await;
                st.last_error = Some(e);
            }
        });
        {
            let task_store = get_ovr_refresh_task();
            let mut guard = task_store.lock().await;
            *guard = Some(handle);
        }
        // Await completion or cancellation
        {
            let task_store = get_ovr_refresh_task();
            let mut guard = task_store.lock().await;
            if let Some(h) = guard.take() {
                let _ = h.await;
            }
        }
        {
            let mut st = state.lock().await;
            st.processed = 1;
            st.active = false;
            st.current_provider = None;
        }
    });

    Ok(serde_json::json!({"success": true}))
}

#[tauri::command]
pub async fn ovr_get_refresh_status() -> Result<serde_json::Value, TauriError> {
    let st = get_ovr_refresh_state();
    let s = st.lock().await.clone();
    Ok(serde_json::json!({"success": true, "status": s}))
}

#[tauri::command]
pub async fn ovr_cancel_refresh() -> Result<serde_json::Value, TauriError> {
    let st = get_ovr_refresh_state();
    let mut s = st.lock().await;
    s.cancelled = true;
    // Abort running task if any
    let task_store = get_ovr_refresh_task();
    let mut guard = task_store.lock().await;
    if let Some(h) = guard.take() {
        h.abort();
    }
    // Immediately mark inactive for UI responsiveness
    s.active = false;
    s.current_provider = None;
    Ok(serde_json::json!({"success": true}))
}

#[derive(Debug, Deserialize)]
pub struct OvrProviderPayload {
    pub id: Option<i64>,
    pub name: String,
    pub base_url: Option<String>,
    pub enabled: bool,
    pub rate_limit_ms: Option<i64>,
}

#[tauri::command]
pub async fn ovr_upsert_provider(
    payload: OvrProviderPayload,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    use crate::database::models::OvrProvider;
    let now = chrono::Utc::now();
    let model = OvrProvider {
        id: payload.id,
        name: payload.name,
        base_url: payload.base_url,
        enabled: payload.enabled,
        rate_limit_ms: payload.rate_limit_ms.unwrap_or(1000),
        last_refreshed_at: None,
        last_status: None,
        last_error: None,
        created_at: now,
        updated_at: now,
    };
    match app.database_plugin().upsert_overlay_provider(&model).await {
        Ok(saved) => Ok(serde_json::json!({
            "success": true,
            "id": saved.id.unwrap_or_default()
        })),
        Err(e) => Ok(serde_json::json!({"success": false, "error": e.to_string()})),
    }
}

#[tauri::command]
pub async fn ovr_remove_provider(
    id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    match app.database_plugin().delete_overlay_provider(id).await {
        Ok(_) => Ok(serde_json::json!({"success": true})),
        Err(e) => Ok(serde_json::json!({"success": false, "error": e.to_string()})),
    }
}

#[tauri::command]
pub async fn ovr_clear_all_tournaments(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    let mut conn = app.database_plugin().get_connection().await?;
    use crate::database::operations::OvrOperations as Ops;
    match Ops::clear_all_tournaments(&mut conn) {
        Ok(_) => Ok(serde_json::json!({"success": true})),
        Err(e) => Ok(serde_json::json!({"success": false, "error": e.to_string()})),
    }
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
pub async fn ovr_list_tournaments(
    app: State<'_, Arc<App>>,
    provider_id: Option<i64>,
    q: Option<String>,
    from: Option<String>,
    to: Option<String>,
    country: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<serde_json::Value, TauriError> {
    let conn = app.database_plugin().get_connection().await?;
    use crate::database::operations::OvrOperations as Ops;
    match Ops::list_tournaments(
        &conn,
        provider_id,
        q.as_deref(),
        from.as_deref(),
        to.as_deref(),
        country.as_deref(),
        limit,
        offset,
    ) {
        Ok(rows) => Ok(serde_json::json!({"success": true, "tournaments": rows})),
        Err(e) => Ok(serde_json::json!({"success": false, "error": e.to_string()})),
    }
}

#[tauri::command]
pub async fn ovr_get_categories(
    app: State<'_, Arc<App>>,
    tournament_id: i64,
) -> Result<serde_json::Value, TauriError> {
    let conn = app.database_plugin().get_connection().await?;
    use crate::database::operations::OvrOperations as Ops;
    match Ops::get_categories(&conn, tournament_id) {
        Ok(rows) => Ok(serde_json::json!({"success": true, "categories": rows})),
        Err(e) => Ok(serde_json::json!({"success": false, "error": e.to_string()})),
    }
}

#[tauri::command]
pub async fn ovr_promote_tournament(
    app: State<'_, Arc<App>>,
    ovr_tournament_id: i64,
    local_name: Option<String>,
) -> Result<serde_json::Value, TauriError> {
    let mut conn = app.database_plugin().get_connection().await?;
    use crate::database::operations::OvrOperations as Ops;
    match Ops::promote_to_local(&mut conn, ovr_tournament_id, local_name.as_deref()) {
        Ok(local_id) => Ok(serde_json::json!({"success": true, "local_tournament_id": local_id})),
        Err(e) => Ok(serde_json::json!({"success": false, "error": e.to_string()})),
    }
}
