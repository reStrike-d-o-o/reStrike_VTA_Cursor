use super::types::{OpenApiDocument, SchemaFormat};
use crate::types::{AppError, AppResult};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

#[derive(Clone)]
struct SharedDocument(Arc<RwLock<OpenApiDocument>>);

pub struct OpenApiRuntime {
    shared: SharedDocument,
    address: SocketAddr,
    server_handle: tokio::task::JoinHandle<()>,
}

impl OpenApiRuntime {
    pub async fn start(initial: OpenApiDocument, address: SocketAddr) -> AppResult<Self> {
        let shared = SharedDocument(Arc::new(RwLock::new(initial)));
        let router = Router::new()
            .route("/openapi.json", get(get_openapi_json))
            .route("/openapi.yaml", get(get_openapi_yaml))
            .route("/api/health", get(get_health))
            .with_state(shared.clone());

        let listener = TcpListener::bind(address)
            .await
            .map_err(|e| AppError::OpenApiError(format!("Failed to bind OpenAPI server: {}", e)))?;
        let server = axum::serve(listener, router.into_make_service());
        let handle = tokio::spawn(async move {
            if let Err(err) = server.await {
                log::error!("OpenAPI server terminated unexpectedly: {}", err);
            }
        });

        Ok(Self {
            shared,
            address,
            server_handle: handle,
        })
    }

    pub async fn update(&self, document: OpenApiDocument) {
        *self.shared.0.write().await = document;
    }

    pub fn http_base_url(&self) -> String {
        format!("http://{}", self.address)
    }

    pub fn json_endpoint(&self) -> String {
        format!("{}/openapi.json", self.http_base_url())
    }

    pub fn yaml_endpoint(&self) -> String {
        format!("{}/openapi.yaml", self.http_base_url())
    }

    pub fn shutdown(&self) {
        self.server_handle.abort();
    }
}

impl Drop for OpenApiRuntime {
    fn drop(&mut self) {
        self.server_handle.abort();
    }
}

async fn get_openapi_json(State(shared): State<SharedDocument>) -> impl IntoResponse {
    let document = shared.0.read().await;
    Json(document.json.clone())
}

async fn get_openapi_yaml(State(shared): State<SharedDocument>) -> impl IntoResponse {
    let document = shared.0.read().await;
    match serde_yaml::to_string(&document.json) {
        Ok(body) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                format!("{}; charset=utf-8", SchemaFormat::Yaml.content_type()),
            )
            .body(body.into())
            .unwrap(),
        Err(err) => {
            log::error!("Failed to render YAML OpenAPI document: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_health(State(shared): State<SharedDocument>) -> impl IntoResponse {
    let document = shared.0.read().await;
    Json(json!({
        "status": "ok",
        "timestamp": document.updated_at.to_rfc3339(),
    }))
}
