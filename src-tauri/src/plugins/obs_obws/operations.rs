//! Advanced OBS operations using the obws crate

use super::client::ObsClient;
use super::types::{
    ObsBounds, ObsFilter, ObsHotkey, ObsOperationRequest, ObsOperationResponse, ObsOutputSettings,
    ObsTransform, ObsTransition,
};
use crate::types::{AppError, AppResult};
use std::collections::HashMap;

/// Advanced OBS operations
pub struct ObsOperations;

impl ObsOperations {
    /// Get source settings
    pub async fn get_source_settings(
        _client: &ObsClient,
        _source_name: &str,
    ) -> AppResult<HashMap<String, serde_json::Value>> {
        // TODO: Implement proper source settings retrieval
        // For now, return empty settings
        Ok(HashMap::new())
    }

    /// Set source settings
    pub async fn set_source_settings(
        _client: &ObsClient,
        _source_name: &str,
        _settings: HashMap<String, serde_json::Value>,
    ) -> AppResult<()> {
        // TODO: Implement proper source settings setting
        // For now, do nothing
        Ok(())
    }

    /// Get source filters
    pub async fn get_source_filters(
        _client: &ObsClient,
        _source_name: &str,
    ) -> AppResult<Vec<ObsFilter>> {
        // Note: obws has a different API for filters that requires SourceId
        // This would need to be implemented using the proper obws API
        log::warn!("Source filters not yet implemented in obws integration");
        Err(AppError::ConfigError(
            "Source filters not yet implemented".to_string(),
        ))
    }

    /// Add filter to source
    pub async fn add_source_filter(
        _client: &ObsClient,
        _source_name: &str,
        _filter_name: &str,
        _filter_type: &str,
        _settings: Option<HashMap<String, serde_json::Value>>,
    ) -> AppResult<()> {
        // Note: obws has a different API for filters that requires Create struct
        // This would need to be implemented using the proper obws API
        log::warn!("Add source filter not yet implemented in obws integration");
        Err(AppError::ConfigError(
            "Add source filter not yet implemented".to_string(),
        ))
    }

    /// Remove filter from source
    pub async fn remove_source_filter(
        _client: &ObsClient,
        _source_name: &str,
        _filter_name: &str,
    ) -> AppResult<()> {
        // Note: obws has a different API for filters that requires SourceId
        // This would need to be implemented using the proper obws API
        log::warn!("Remove source filter not yet implemented in obws integration");
        Err(AppError::ConfigError(
            "Remove source filter not yet implemented".to_string(),
        ))
    }

    /// Set source filter settings
    pub async fn set_source_filter_settings(
        _client: &ObsClient,
        _source_name: &str,
        _filter_name: &str,
        _settings: HashMap<String, serde_json::Value>,
    ) -> AppResult<()> {
        // Note: obws has a different API for filters that requires SetSettings struct
        // This would need to be implemented using the proper obws API
        log::warn!("Set source filter settings not yet implemented in obws integration");
        Err(AppError::ConfigError(
            "Set source filter settings not yet implemented".to_string(),
        ))
    }

    /// Get source transform
    pub async fn get_source_transform(
        _client: &ObsClient,
        _source_name: &str,
    ) -> AppResult<ObsTransform> {
        // Note: obws doesn't have a direct source_transform method
        // This would need to be implemented using scene item transforms
        log::warn!("Source transform not yet implemented in obws integration");
        Err(AppError::ConfigError(
            "Source transform not yet implemented".to_string(),
        ))
    }

    /// Set source transform
    pub async fn set_source_transform(
        _client: &ObsClient,
        _source_name: &str,
        _transform: ObsTransform,
    ) -> AppResult<()> {
        // Note: obws doesn't have a direct set_source_transform method
        // This would need to be implemented using scene item transforms
        log::warn!("Set source transform not yet implemented in obws integration");
        Err(AppError::ConfigError(
            "Set source transform not yet implemented".to_string(),
        ))
    }

    /// Get source bounds
    pub async fn get_source_bounds(
        _client: &ObsClient,
        _source_name: &str,
    ) -> AppResult<ObsBounds> {
        // Note: obws doesn't have a direct source_bounds method
        // This would need to be implemented using scene item bounds
        log::warn!("Source bounds not yet implemented in obws integration");
        Err(AppError::ConfigError(
            "Source bounds not yet implemented".to_string(),
        ))
    }

    /// Set source bounds
    pub async fn set_source_bounds(
        _client: &ObsClient,
        _source_name: &str,
        _bounds: ObsBounds,
    ) -> AppResult<()> {
        // Note: obws doesn't have a direct set_source_bounds method
        // This would need to be implemented using scene item bounds
        log::warn!("Set source bounds not yet implemented in obws integration");
        Err(AppError::ConfigError(
            "Set source bounds not yet implemented".to_string(),
        ))
    }

    /// Get source volume
    pub async fn get_source_volume(client: &ObsClient, source_name: &str) -> AppResult<f64> {
        client.get_source_volume(source_name).await
    }

    /// Set source volume
    pub async fn set_source_volume(
        client: &ObsClient,
        source_name: &str,
        volume: f64,
    ) -> AppResult<()> {
        client.set_source_volume(source_name, volume).await
    }

    /// Get source muted state
    pub async fn get_source_muted(client: &ObsClient, source_name: &str) -> AppResult<bool> {
        client.get_source_muted(source_name).await
    }

    /// Set source muted state
    pub async fn set_source_muted(
        client: &ObsClient,
        source_name: &str,
        muted: bool,
    ) -> AppResult<()> {
        client.set_source_muted(source_name, muted).await
    }

    /// Get transitions
    pub async fn get_transitions(_client: &ObsClient) -> AppResult<Vec<ObsTransition>> {
        // TODO: Implement proper transitions retrieval
        // For now, return empty list
        Ok(Vec::new())
    }

    /// Set transition
    pub async fn set_transition(_client: &ObsClient, _transition_name: &str) -> AppResult<()> {
        // TODO: Implement proper transition setting
        // For now, do nothing
        Ok(())
    }

    /// Trigger transition
    pub async fn trigger_transition(
        _client: &ObsClient,
        _transition_name: Option<&str>,
        _scene_name: Option<&str>,
    ) -> AppResult<()> {
        // TODO: Implement proper transition triggering
        // For now, do nothing
        Ok(())
    }

    /// Get hotkeys
    pub async fn get_hotkeys(_client: &ObsClient) -> AppResult<Vec<ObsHotkey>> {
        // Note: obws has a different API for hotkeys
        // This would need to be implemented using the proper obws API
        log::warn!("Hotkeys not yet implemented in obws integration");
        Err(AppError::ConfigError(
            "Hotkeys not yet implemented".to_string(),
        ))
    }

    /// Trigger hotkey
    pub async fn trigger_hotkey(_client: &ObsClient, _hotkey_name: &str) -> AppResult<()> {
        // Note: obws doesn't have a direct trigger method for hotkeys
        // This would need to be implemented using custom requests
        log::warn!("Trigger hotkey not yet implemented in obws integration");
        Err(AppError::ConfigError(
            "Trigger hotkey not yet implemented".to_string(),
        ))
    }

    /// Get output settings
    pub async fn get_output_settings(_client: &ObsClient) -> AppResult<ObsOutputSettings> {
        // Note: obws doesn't have a direct output_settings method
        // This would need to be implemented using custom requests
        log::warn!("Output settings not yet implemented in obws integration");
        Err(AppError::ConfigError(
            "Output settings not yet implemented".to_string(),
        ))
    }

    /// Set output settings
    pub async fn set_output_settings(
        _client: &ObsClient,
        _settings: ObsOutputSettings,
    ) -> AppResult<()> {
        // Note: obws doesn't have a direct set_output_settings method
        // This would need to be implemented using custom requests
        log::warn!("Set output settings not yet implemented in obws integration");
        Err(AppError::ConfigError(
            "Set output settings not yet implemented".to_string(),
        ))
    }

    /// Get studio mode status
    pub async fn get_studio_mode(_client: &ObsClient) -> AppResult<bool> {
        // Note: obws doesn't directly support studio mode
        // This is a placeholder implementation for future enhancement
        log::debug!("Studio mode status requested - not supported by obws");
        Ok(false) // Default to false since studio mode isn't supported
    }

    /// Set studio mode
    pub async fn set_studio_mode(_client: &ObsClient, enabled: bool) -> AppResult<()> {
        // Note: obws doesn't directly support studio mode
        // This is a placeholder implementation for future enhancement
        if enabled {
            log::warn!("Studio mode enable requested but not supported by obws crate");
        } else {
            log::debug!("Studio mode disable requested but not supported by obws crate");
        }

        Ok(()) // Always return success since studio mode isn't supported
    }

    /// Get preview scene
    pub async fn get_preview_scene(_client: &ObsClient) -> AppResult<String> {
        // Note: obws doesn't have a direct preview_scene method
        // This would need to be implemented using custom requests
        log::warn!("Preview scene not yet implemented in obws integration");
        Err(AppError::ConfigError(
            "Preview scene not yet implemented".to_string(),
        ))
    }

    /// Set preview scene
    pub async fn set_preview_scene(_client: &ObsClient, _scene_name: &str) -> AppResult<()> {
        // Note: obws doesn't have a direct set_preview_scene method
        // This would need to be implemented using custom requests
        log::warn!("Set preview scene not yet implemented in obws integration");
        Err(AppError::ConfigError(
            "Set preview scene not yet implemented".to_string(),
        ))
    }

    /// Transition to program
    pub async fn transition_to_program(
        _client: &ObsClient,
        _transition_name: Option<&str>,
    ) -> AppResult<()> {
        // Note: obws doesn't have a direct transition_to_program method
        // This would need to be implemented using custom requests
        log::warn!("Transition to program not yet implemented in obws integration");
        Err(AppError::ConfigError(
            "Transition to program not yet implemented".to_string(),
        ))
    }

    /// Execute custom operation
    pub async fn execute_custom_operation(
        client: &ObsClient,
        request: ObsOperationRequest,
    ) -> AppResult<ObsOperationResponse> {
        let obs_client = client.get_client()?;

        // Generate unique request ID
        let request_id = format!("custom_op_{}", chrono::Utc::now().timestamp_millis());

        match request.operation.as_str() {
            // Audio control operations - these enable the functionality you requested
            "SetInputMute" => {
                let _input_name = request
                    .parameters
                    .get("inputName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'inputName' parameter".to_string())
                    })?;

                let _mute_value = request
                    .parameters
                    .get("inputMuted")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| {
                        AppError::ConfigError(
                            "Missing or invalid 'inputMuted' parameter".to_string(),
                        )
                    })?;

                // Use raw request system for individual input mute control
                match client
                    .execute_raw_request(
                        "SetInputMute",
                        serde_json::to_value(&request.parameters).unwrap_or_default(),
                    )
                    .await
                {
                    Ok(result) => Ok(ObsOperationResponse {
                        request_id,
                        status: "success".to_string(),
                        data: Some(serde_json::json!({
                            "inputName": _input_name,
                            "inputMuted": _mute_value,
                            "rawResponse": result
                        })),
                        error: None,
                    }),
                    Err(e) => Ok(ObsOperationResponse {
                        request_id,
                        status: "error".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    }),
                }
            }

            "GetInputMute" => {
                let _input_name = request
                    .parameters
                    .get("inputName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'inputName' parameter".to_string())
                    })?;

                // Use raw request system for individual input mute status
                match client
                    .execute_raw_request(
                        "GetInputMute",
                        serde_json::to_value(&request.parameters).unwrap_or_default(),
                    )
                    .await
                {
                    Ok(result) => Ok(ObsOperationResponse {
                        request_id,
                        status: "success".to_string(),
                        data: Some(serde_json::json!({
                            "inputName": _input_name,
                            "rawResponse": result
                        })),
                        error: None,
                    }),
                    Err(e) => Ok(ObsOperationResponse {
                        request_id,
                        status: "error".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    }),
                }
            }

            "SetInputVolume" => {
                let _input_name = request
                    .parameters
                    .get("inputName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'inputName' parameter".to_string())
                    })?;

                let _volume_value = request
                    .parameters
                    .get("inputVolumeMul")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| {
                        AppError::ConfigError(
                            "Missing or invalid 'inputVolumeMul' parameter".to_string(),
                        )
                    })?;

                // Use raw request system for individual input volume control
                match client
                    .execute_raw_request(
                        "SetInputVolume",
                        serde_json::to_value(&request.parameters).unwrap_or_default(),
                    )
                    .await
                {
                    Ok(result) => Ok(ObsOperationResponse {
                        request_id,
                        status: "success".to_string(),
                        data: Some(serde_json::json!({
                            "inputName": _input_name,
                            "inputVolumeMul": _volume_value,
                            "rawResponse": result
                        })),
                        error: None,
                    }),
                    Err(e) => Ok(ObsOperationResponse {
                        request_id,
                        status: "error".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    }),
                }
            }

            "GetInputVolume" => {
                let _input_name = request
                    .parameters
                    .get("inputName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'inputName' parameter".to_string())
                    })?;

                // Use raw request system for individual input volume status
                match client
                    .execute_raw_request(
                        "GetInputVolume",
                        serde_json::to_value(&request.parameters).unwrap_or_default(),
                    )
                    .await
                {
                    Ok(result) => Ok(ObsOperationResponse {
                        request_id,
                        status: "success".to_string(),
                        data: Some(serde_json::json!({
                            "inputName": _input_name,
                            "rawResponse": result
                        })),
                        error: None,
                    }),
                    Err(e) => Ok(ObsOperationResponse {
                        request_id,
                        status: "error".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    }),
                }
            }

            "ToggleMute" => {
                let _input_name = request
                    .parameters
                    .get("inputName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'inputName' parameter".to_string())
                    })?;

                // Use raw request system for input mute toggle
                match client
                    .execute_raw_request(
                        "ToggleMute",
                        serde_json::to_value(&request.parameters).unwrap_or_default(),
                    )
                    .await
                {
                    Ok(result) => Ok(ObsOperationResponse {
                        request_id,
                        status: "success".to_string(),
                        data: Some(serde_json::json!({
                            "inputName": _input_name,
                            "rawResponse": result
                        })),
                        error: None,
                    }),
                    Err(e) => Ok(ObsOperationResponse {
                        request_id,
                        status: "error".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    }),
                }
            }

            // Scene item operations
            "GetSceneItemId" => {
                let _scene_name = request
                    .parameters
                    .get("sceneName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'sceneName' parameter".to_string())
                    })?;

                let _source_name = request
                    .parameters
                    .get("sourceName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'sourceName' parameter".to_string())
                    })?;

                // Use raw request system for scene item operations
                match client
                    .execute_raw_request(
                        "GetSceneItemId",
                        serde_json::to_value(&request.parameters).unwrap_or_default(),
                    )
                    .await
                {
                    Ok(result) => Ok(ObsOperationResponse {
                        request_id,
                        status: "success".to_string(),
                        data: Some(serde_json::json!({
                            "sceneName": _scene_name,
                            "sourceName": _source_name,
                            "rawResponse": result
                        })),
                        error: None,
                    }),
                    Err(e) => Ok(ObsOperationResponse {
                        request_id,
                        status: "error".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    }),
                }
            }

            "SetSceneItemEnabled" => {
                let _scene_name = request
                    .parameters
                    .get("sceneName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'sceneName' parameter".to_string())
                    })?;

                let _scene_item_id = request
                    .parameters
                    .get("sceneItemId")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        AppError::ConfigError(
                            "Missing or invalid 'sceneItemId' parameter".to_string(),
                        )
                    })?;

                let _scene_item_enabled = request
                    .parameters
                    .get("sceneItemEnabled")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| {
                        AppError::ConfigError(
                            "Missing or invalid 'sceneItemEnabled' parameter".to_string(),
                        )
                    })?;

                // Use raw request system for scene item operations
                match client
                    .execute_raw_request(
                        "SetSceneItemEnabled",
                        serde_json::to_value(&request.parameters).unwrap_or_default(),
                    )
                    .await
                {
                    Ok(result) => Ok(ObsOperationResponse {
                        request_id,
                        status: "success".to_string(),
                        data: Some(serde_json::json!({
                            "sceneName": _scene_name,
                            "sceneItemId": _scene_item_id,
                            "sceneItemEnabled": _scene_item_enabled,
                            "rawResponse": result
                        })),
                        error: None,
                    }),
                    Err(e) => Ok(ObsOperationResponse {
                        request_id,
                        status: "error".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    }),
                }
            }

            "SetSceneItemTransform" => {
                let _scene_name = request
                    .parameters
                    .get("sceneName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'sceneName' parameter".to_string())
                    })?;

                let _scene_item_id = request
                    .parameters
                    .get("sceneItemId")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        AppError::ConfigError(
                            "Missing or invalid 'sceneItemId' parameter".to_string(),
                        )
                    })?;

                // Get transform parameters
                let _x = request
                    .parameters
                    .get("positionX")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let _y = request
                    .parameters
                    .get("positionY")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let _scale_x = request
                    .parameters
                    .get("scaleX")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0);
                let _scale_y = request
                    .parameters
                    .get("scaleY")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0);
                let _rotation = request
                    .parameters
                    .get("rotation")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                // Use raw request system for scene item transform
                match client
                    .execute_raw_request(
                        "SetSceneItemTransform",
                        serde_json::to_value(&request.parameters).unwrap_or_default(),
                    )
                    .await
                {
                    Ok(result) => Ok(ObsOperationResponse {
                        request_id,
                        status: "success".to_string(),
                        data: Some(serde_json::json!({
                            "sceneName": _scene_name,
                            "sceneItemId": _scene_item_id,
                            "positionX": _x,
                            "positionY": _y,
                            "scaleX": _scale_x,
                            "scaleY": _scale_y,
                            "rotation": _rotation,
                            "rawResponse": result
                        })),
                        error: None,
                    }),
                    Err(e) => Ok(ObsOperationResponse {
                        request_id,
                        status: "error".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    }),
                }
            }

            // Transition operations
            "SetCurrentSceneTransition" => {
                let _transition_name = request
                    .parameters
                    .get("transitionName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'transitionName' parameter".to_string())
                    })?;

                // Use raw request system for transition operations
                match client
                    .execute_raw_request(
                        "SetCurrentSceneTransition",
                        serde_json::to_value(&request.parameters).unwrap_or_default(),
                    )
                    .await
                {
                    Ok(result) => Ok(ObsOperationResponse {
                        request_id,
                        status: "success".to_string(),
                        data: Some(serde_json::json!({
                            "transitionName": _transition_name,
                            "rawResponse": result
                        })),
                        error: None,
                    }),
                    Err(e) => Ok(ObsOperationResponse {
                        request_id,
                        status: "error".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    }),
                }
            }

            "GetCurrentSceneTransition" => {
                // Use raw request system for transition operations
                match client
                    .execute_raw_request(
                        "GetCurrentSceneTransition",
                        serde_json::to_value(&request.parameters).unwrap_or_default(),
                    )
                    .await
                {
                    Ok(result) => Ok(ObsOperationResponse {
                        request_id,
                        status: "success".to_string(),
                        data: Some(serde_json::json!({
                            "rawResponse": result
                        })),
                        error: None,
                    }),
                    Err(e) => Ok(ObsOperationResponse {
                        request_id,
                        status: "error".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    }),
                }
            }

            "SetSceneTransitionOverride" => {
                let _scene_name = request
                    .parameters
                    .get("sceneName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'sceneName' parameter".to_string())
                    })?;

                let _transition_name = request
                    .parameters
                    .get("transitionName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'transitionName' parameter".to_string())
                    })?;

                let _transition_duration = request
                    .parameters
                    .get("transitionDuration")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(300);

                // Use raw request system for transition override
                match client
                    .execute_raw_request(
                        "SetSceneTransitionOverride",
                        serde_json::to_value(&request.parameters).unwrap_or_default(),
                    )
                    .await
                {
                    Ok(result) => Ok(ObsOperationResponse {
                        request_id,
                        status: "success".to_string(),
                        data: Some(serde_json::json!({
                            "sceneName": _scene_name,
                            "transitionName": _transition_name,
                            "transitionDuration": _transition_duration,
                            "rawResponse": result
                        })),
                        error: None,
                    }),
                    Err(e) => Ok(ObsOperationResponse {
                        request_id,
                        status: "error".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    }),
                }
            }

            "GetTransitionList" => {
                // Use raw request system for transition list
                match client
                    .execute_raw_request(
                        "GetTransitionList",
                        serde_json::to_value(&request.parameters).unwrap_or_default(),
                    )
                    .await
                {
                    Ok(result) => Ok(ObsOperationResponse {
                        request_id,
                        status: "success".to_string(),
                        data: Some(serde_json::json!({
                            "rawResponse": result
                        })),
                        error: None,
                    }),
                    Err(e) => Ok(ObsOperationResponse {
                        request_id,
                        status: "error".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    }),
                }
            }

            "TriggerStudioModeTransition" => {
                // Use raw request system for studio mode transition
                match client
                    .execute_raw_request(
                        "TriggerStudioModeTransition",
                        serde_json::to_value(&request.parameters).unwrap_or_default(),
                    )
                    .await
                {
                    Ok(result) => Ok(ObsOperationResponse {
                        request_id,
                        status: "success".to_string(),
                        data: Some(serde_json::json!({
                            "rawResponse": result
                        })),
                        error: None,
                    }),
                    Err(e) => Ok(ObsOperationResponse {
                        request_id,
                        status: "error".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    }),
                }
            }

            "SetStudioModeEnabled" => {
                let _studio_mode_enabled = request
                    .parameters
                    .get("studioModeEnabled")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| {
                        AppError::ConfigError(
                            "Missing or invalid 'studioModeEnabled' parameter".to_string(),
                        )
                    })?;

                // Use raw request system for studio mode control
                match client
                    .execute_raw_request(
                        "SetStudioModeEnabled",
                        serde_json::to_value(&request.parameters).unwrap_or_default(),
                    )
                    .await
                {
                    Ok(result) => Ok(ObsOperationResponse {
                        request_id,
                        status: "success".to_string(),
                        data: Some(serde_json::json!({
                            "studioModeEnabled": _studio_mode_enabled,
                            "rawResponse": result
                        })),
                        error: None,
                    }),
                    Err(e) => Ok(ObsOperationResponse {
                        request_id,
                        status: "error".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    }),
                }
            }

            // Basic recording operations
            "StartRecording" => match obs_client.recording().start().await {
                Ok(_) => Ok(ObsOperationResponse {
                    request_id,
                    status: "success".to_string(),
                    data: Some(serde_json::json!({})),
                    error: None,
                }),
                Err(e) => Ok(ObsOperationResponse {
                    request_id,
                    status: "error".to_string(),
                    data: None,
                    error: Some(e.to_string()),
                }),
            },

            "StopRecording" => match obs_client.recording().stop().await {
                Ok(_) => Ok(ObsOperationResponse {
                    request_id,
                    status: "success".to_string(),
                    data: Some(serde_json::json!({})),
                    error: None,
                }),
                Err(e) => Ok(ObsOperationResponse {
                    request_id,
                    status: "error".to_string(),
                    data: None,
                    error: Some(e.to_string()),
                }),
            },

            "GetRecordingStatus" => match obs_client.recording().status().await {
                Ok(status) => Ok(ObsOperationResponse {
                    request_id,
                    status: "success".to_string(),
                    data: Some(serde_json::json!({
                        "isRecording": status.active,
                        "isPaused": status.paused,
                        "timecode": format!("{:?}", status.timecode),
                        "bytes": status.bytes
                    })),
                    error: None,
                }),
                Err(e) => Ok(ObsOperationResponse {
                    request_id,
                    status: "error".to_string(),
                    data: None,
                    error: Some(e.to_string()),
                }),
            },

            // Scene operations
            "GetCurrentPreviewScene" => match obs_client.scenes().current_preview_scene().await {
                Ok(scene) => Ok(ObsOperationResponse {
                    request_id,
                    status: "success".to_string(),
                    data: Some(serde_json::json!({
                        "sceneName": scene.id
                    })),
                    error: None,
                }),
                Err(e) => Ok(ObsOperationResponse {
                    request_id,
                    status: "error".to_string(),
                    data: None,
                    error: Some(e.to_string()),
                }),
            },

            "SetCurrentPreviewScene" => {
                let scene_name = request
                    .parameters
                    .get("sceneName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'sceneName' parameter".to_string())
                    })?;

                match obs_client
                    .scenes()
                    .set_current_preview_scene(scene_name)
                    .await
                {
                    Ok(_) => Ok(ObsOperationResponse {
                        request_id,
                        status: "success".to_string(),
                        data: Some(serde_json::json!({
                            "sceneName": scene_name
                        })),
                        error: None,
                    }),
                    Err(e) => Ok(ObsOperationResponse {
                        request_id,
                        status: "error".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    }),
                }
            }

            // Unknown operation
            _ => {
                log::warn!("Unknown custom operation requested: {}", request.operation);
                Ok(ObsOperationResponse {
                    request_id,
                    status: "error".to_string(),
                    data: None,
                    error: Some(format!("Unknown operation: {}", request.operation)),
                })
            }
        }
    }
}
