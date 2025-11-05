use super::types::{OpenApiDocument, SchemaFormat, SchemaMetadata, SchemaValidationOutcome};
use super::validation::{validate, ValidationOptions};
use crate::types::{AppError, AppResult};
use chrono::Utc;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

const DEFAULT_OPENAPI_YAML: &str = include_str!("../../resources/openapi/default.yaml");

pub struct OpenApiManager {
    schema_dir: PathBuf,
    yaml_path: PathBuf,
    json_path: PathBuf,
    meta_path: PathBuf,
    state: Arc<RwLock<OpenApiDocument>>,
}

impl OpenApiManager {
    pub fn new(schema_dir: &Path) -> AppResult<Self> {
        fs::create_dir_all(schema_dir)?;

        let yaml_path = schema_dir.join("schema.yaml");
        let json_path = schema_dir.join("schema.json");
        let meta_path = schema_dir.join("schema_state.json");

        if !yaml_path.exists() {
            fs::write(&yaml_path, DEFAULT_OPENAPI_YAML).map_err(|e| {
                AppError::OpenApiError(format!("Failed to write default schema: {e}"))
            })?;
        }

        if !json_path.exists() {
            let default_value =
                convert_to_json(DEFAULT_OPENAPI_YAML, SchemaFormat::Yaml).map_err(|e| {
                    AppError::OpenApiError(format!("Failed to convert default schema: {e}"))
                })?;
            let json_string = render_value(&default_value, SchemaFormat::Json).map_err(|e| {
                AppError::OpenApiError(format!("Failed to render default JSON: {e}"))
            })?;
            fs::write(&json_path, json_string).map_err(|e| {
                AppError::OpenApiError(format!("Failed to write default JSON: {e}"))
            })?;
        }

        let metadata = if meta_path.exists() {
            let raw_meta = fs::read_to_string(&meta_path).map_err(|e| {
                AppError::OpenApiError(format!("Failed to read schema metadata: {e}"))
            })?;
            serde_json::from_str::<SchemaMetadata>(&raw_meta).map_err(|e| {
                AppError::OpenApiError(format!("Invalid schema metadata format: {e}"))
            })?
        } else {
            let meta = SchemaMetadata {
                format: SchemaFormat::Yaml,
                updated_at: Utc::now(),
            };
            write_metadata(&meta_path, &meta)?;
            meta
        };

        let mut document = load_document(metadata.format, &yaml_path, &json_path)?;
        document.updated_at = metadata.updated_at;

        Ok(Self {
            schema_dir: schema_dir.to_path_buf(),
            yaml_path,
            json_path,
            meta_path,
            state: Arc::new(RwLock::new(document)),
        })
    }

    pub async fn current_document(&self) -> OpenApiDocument {
        self.state.read().await.clone()
    }

    pub async fn get_document_as(&self, format: SchemaFormat) -> AppResult<OpenApiDocument> {
        let doc = self.state.read().await.clone();
        if doc.format == format {
            return Ok(doc);
        }

        let rendered =
            render_value(&doc.json, format).map_err(|e| AppError::OpenApiError(e.to_string()))?;

        Ok(OpenApiDocument {
            raw: rendered,
            format,
            json: doc.json.clone(),
            updated_at: doc.updated_at,
        })
    }

    pub fn validate_schema(&self, content: &str, format: SchemaFormat) -> SchemaValidationOutcome {
        validate(content, format, ValidationOptions::default())
    }

    pub async fn save_schema(
        &self,
        content: &str,
        format: SchemaFormat,
    ) -> AppResult<SchemaValidationOutcome> {
        let validation = self.validate_schema(content, format);
        if !validation.valid {
            return Ok(validation);
        }

        let value =
            convert_to_json(content, format).map_err(|e| AppError::OpenApiError(e.to_string()))?;

        let canonical_current =
            render_value(&value, format).map_err(|e| AppError::OpenApiError(e.to_string()))?;
        let canonical_json = render_value(&value, SchemaFormat::Json)
            .map_err(|e| AppError::OpenApiError(e.to_string()))?;
        let canonical_yaml = render_value(&value, SchemaFormat::Yaml)
            .map_err(|e| AppError::OpenApiError(e.to_string()))?;

        fs::write(&self.json_path, &canonical_json)
            .map_err(|e| AppError::OpenApiError(format!("Failed to write JSON schema: {e}")))?;
        fs::write(&self.yaml_path, &canonical_yaml)
            .map_err(|e| AppError::OpenApiError(format!("Failed to write YAML schema: {e}")))?;

        let updated_at = Utc::now();
        let metadata = SchemaMetadata { format, updated_at };
        write_metadata(&self.meta_path, &metadata)?;

        let mut state = self.state.write().await;
        *state = OpenApiDocument {
            raw: canonical_current,
            format,
            json: value,
            updated_at,
        };

        Ok(validation)
    }

    pub async fn export_schema(&self, export_path: &Path, format: SchemaFormat) -> AppResult<()> {
        let document = self.get_document_as(format).await?;
        if let Some(parent) = export_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    AppError::OpenApiError(format!(
                        "Failed to create export directory {}: {}",
                        parent.display(),
                        e
                    ))
                })?;
            }
        }
        fs::write(export_path, document.raw.as_bytes()).map_err(|e| {
            AppError::OpenApiError(format!(
                "Failed to export schema to {}: {}",
                export_path.display(),
                e
            ))
        })
    }

    pub fn schema_directory(&self) -> &Path {
        &self.schema_dir
    }
}

fn load_document(
    format: SchemaFormat,
    yaml_path: &Path,
    json_path: &Path,
) -> AppResult<OpenApiDocument> {
    let raw = match format {
        SchemaFormat::Yaml => fs::read_to_string(yaml_path).map_err(|e| {
            AppError::OpenApiError(format!(
                "Failed to read YAML schema at {}: {}",
                yaml_path.display(),
                e
            ))
        })?,
        SchemaFormat::Json => fs::read_to_string(json_path).map_err(|e| {
            AppError::OpenApiError(format!(
                "Failed to read JSON schema at {}: {}",
                json_path.display(),
                e
            ))
        })?,
    };

    let value = convert_to_json(&raw, format)
        .map_err(|e| AppError::OpenApiError(format!("Invalid persisted schema: {e}")))?;

    let canonical =
        render_value(&value, format).map_err(|e| AppError::OpenApiError(e.to_string()))?;

    Ok(OpenApiDocument {
        raw: canonical,
        format,
        json: value,
        updated_at: Utc::now(),
    })
}

fn convert_to_json(
    content: &str,
    format: SchemaFormat,
) -> Result<Value, Box<dyn std::error::Error>> {
    match format {
        SchemaFormat::Json => Ok(serde_json::from_str::<Value>(content)?),
        SchemaFormat::Yaml => Ok(serde_yaml::from_str::<Value>(content)?),
    }
}

fn render_value(value: &Value, format: SchemaFormat) -> Result<String, Box<dyn std::error::Error>> {
    match format {
        SchemaFormat::Json => Ok(serde_json::to_string_pretty(value)?),
        SchemaFormat::Yaml => Ok(serde_yaml::to_string(value)?),
    }
}

fn write_metadata(path: &Path, metadata: &SchemaMetadata) -> AppResult<()> {
    let payload = serde_json::to_string_pretty(metadata).map_err(|e| {
        AppError::OpenApiError(format!("Failed to serialize schema metadata: {e}"))
    })?;
    fs::write(path, payload)
        .map_err(|e| AppError::OpenApiError(format!("Failed to persist schema metadata: {e}")))
}
