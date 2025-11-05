use crate::core::app::App;
use crate::openapi::{OpenApiRuntime, SchemaFormat, SchemaValidationOutcome};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

#[derive(Deserialize)]
pub struct OpenApiStateRequest {
    pub format: Option<String>,
}

#[derive(Serialize)]
pub struct OpenApiEndpoints {
    pub base_url: String,
    pub json: String,
    pub yaml: String,
    pub health: String,
}

#[derive(Serialize)]
pub struct OpenApiStateResponse {
    pub schema: String,
    pub format: SchemaFormat,
    pub updated_at: String,
    pub validation: SchemaValidationOutcome,
    pub endpoints: OpenApiEndpoints,
}

#[derive(Deserialize)]
pub struct OpenApiSaveRequest {
    pub schema: String,
    pub format: String,
}

#[derive(Serialize)]
pub struct OpenApiSaveResponse {
    pub format: SchemaFormat,
    pub updated_at: String,
    pub validation: SchemaValidationOutcome,
}

#[derive(Deserialize)]
pub struct OpenApiValidateRequest {
    pub schema: String,
    pub format: String,
}

#[derive(Serialize)]
pub struct OpenApiValidateResponse {
    pub validation: SchemaValidationOutcome,
}

#[derive(Deserialize)]
pub struct OpenApiUploadRequest {
    pub path: String,
}

#[derive(Serialize)]
pub struct OpenApiUploadResponse {
    pub schema: String,
    pub format: SchemaFormat,
    pub validation: SchemaValidationOutcome,
}

#[derive(Deserialize)]
pub struct OpenApiExportRequest {
    pub path: String,
    pub format: String,
}

#[tauri::command]
pub async fn openapi_get_state(
    request: Option<OpenApiStateRequest>,
    app: State<'_, Arc<App>>,
) -> Result<OpenApiStateResponse, tauri::Error> {
    let target_format = parse_format_option(request.as_ref().and_then(|req| req.format.as_ref()))?;
    let manager = app.openapi_manager();
    let current = manager.current_document().await;
    let resolved_format = target_format.unwrap_or(current.format);
    let document = if resolved_format == current.format {
        current
    } else {
        manager.get_document_as(resolved_format).await?
    };

    let validation = manager.validate_schema(&document.raw, document.format);
    let runtime = app.openapi_runtime();

    Ok(OpenApiStateResponse {
        schema: document.raw,
        format: document.format,
        updated_at: document.updated_at.to_rfc3339(),
        validation,
        endpoints: map_endpoints(&runtime),
    })
}

#[tauri::command]
pub async fn openapi_save_schema(
    request: OpenApiSaveRequest,
    app: State<'_, Arc<App>>,
) -> Result<OpenApiSaveResponse, tauri::Error> {
    let format = parse_format(&request.format)?;
    let manager = app.openapi_manager();
    let validation = manager.save_schema(&request.schema, format).await?;

    if validation.valid {
        app.apply_openapi_schema().await?;
    }

    let document = manager.current_document().await;

    Ok(OpenApiSaveResponse {
        format: document.format,
        updated_at: document.updated_at.to_rfc3339(),
        validation,
    })
}

#[tauri::command]
pub async fn openapi_validate_schema(
    request: OpenApiValidateRequest,
    app: State<'_, Arc<App>>,
) -> Result<OpenApiValidateResponse, tauri::Error> {
    let format = parse_format(&request.format)?;
    let manager = app.openapi_manager();
    let validation = manager.validate_schema(&request.schema, format);
    Ok(OpenApiValidateResponse { validation })
}

#[tauri::command]
pub async fn openapi_upload_schema(
    request: OpenApiUploadRequest,
    app: State<'_, Arc<App>>,
) -> Result<OpenApiUploadResponse, tauri::Error> {
    let path = PathBuf::from(&request.path);
    let manager = app.openapi_manager();
    let (schema, format) = read_schema_file(&path)?;
    let validation = manager.validate_schema(&schema, format);
    Ok(OpenApiUploadResponse {
        schema,
        format,
        validation,
    })
}

#[tauri::command]
pub async fn openapi_export_schema(
    request: OpenApiExportRequest,
    app: State<'_, Arc<App>>,
) -> Result<(), tauri::Error> {
    let format = parse_format(&request.format)?;
    let path = PathBuf::from(&request.path);
    let manager = app.openapi_manager();
    manager.export_schema(&path, format).await?;
    Ok(())
}

fn parse_format(raw: &str) -> Result<SchemaFormat, tauri::Error> {
    SchemaFormat::from_extension(raw)
        .or_else(|| SchemaFormat::from_extension(&raw.to_ascii_lowercase()))
        .ok_or_else(|| {
            tauri::Error::from(anyhow::anyhow!(format!(
                "Unsupported schema format '{raw}'"
            )))
        })
}

fn parse_format_option(raw: Option<&String>) -> Result<Option<SchemaFormat>, tauri::Error> {
    match raw {
        Some(value) => Ok(Some(parse_format(value)?)),
        None => Ok(None),
    }
}

fn read_schema_file(path: &PathBuf) -> Result<(String, SchemaFormat), tauri::Error> {
    let bytes = std::fs::read(path).map_err(|e| {
        tauri::Error::from(anyhow::anyhow!(format!(
            "Failed to read schema file {}: {}",
            path.display(),
            e
        )))
    })?;
    let schema = String::from_utf8(bytes).map_err(|_| {
        tauri::Error::from(anyhow::anyhow!(
            "OpenAPI schema files must be UTF-8 encoded"
        ))
    })?;

    let format = path
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(SchemaFormat::from_extension)
        .unwrap_or_else(|| {
            if schema.trim_start().starts_with('{') {
                SchemaFormat::Json
            } else {
                SchemaFormat::Yaml
            }
        });

    Ok((schema, format))
}

fn map_endpoints(runtime: &Arc<OpenApiRuntime>) -> OpenApiEndpoints {
    OpenApiEndpoints {
        base_url: runtime.http_base_url(),
        json: runtime.json_endpoint(),
        yaml: runtime.yaml_endpoint(),
        health: format!("{}/api/health", runtime.http_base_url()),
    }
}
