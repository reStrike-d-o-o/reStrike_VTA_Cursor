use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::database::DatabaseConnection;
use crate::database::models::{OvrProvider, OvrTournament};

#[derive(Clone)]
pub struct OvrScraperPlugin {
    database: Arc<DatabaseConnection>,
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("🛰️ Initializing OVR Scraper Plugin...");
    Ok(())
}

impl OvrScraperPlugin {
    pub fn new(database: Arc<DatabaseConnection>) -> Self { Self { database } }

    /// Refresh all enabled providers
    pub async fn refresh_all(&self) -> Result<usize, String> {
        let conn = self.database.get_connection().await.map_err(|e| e.to_string())?;
        let providers = crate::database::operations::OvrOperations::get_providers(&*conn).map_err(|e| e.to_string())?;
        let mut updated = 0usize;
        for p in providers.into_iter().filter(|p| p.enabled) {
            match self.refresh_provider(p.id.unwrap_or_default()).await {
                Ok(_) => updated += 1,
                Err(e) => {
                    let _ = self.database.get_connection().await
                        .map_err(|_| ())
                        .and_then(|mut c| crate::database::operations::OvrOperations::set_provider_refresh_status(&mut *c, p.id.unwrap_or_default(), Some("error"), Some(&e)).map_err(|_| ()));
                }
            }
        }
        Ok(updated)
    }

    /// Refresh single provider by basic adapter selection
    pub async fn refresh_provider(&self, provider_id: i64) -> Result<(), String> {
        let conn = self.database.get_connection().await.map_err(|e| e.to_string())?;
        let provider = {
            let mut stmt = conn.prepare("SELECT * FROM ovr_providers WHERE id = ?").map_err(|e| e.to_string())?;
            stmt.query_row([provider_id], |row| OvrProvider::from_row(row)).map_err(|e| e.to_string())?
        };
        let name_lc = provider.name.to_lowercase();

        // Minimal adapters: fetch public tournament lists where possible (placeholder endpoints)
        let tournaments: Vec<OvrTournament> = if name_lc.contains("simply") {
            self.fetch_simplycompete(provider.id.unwrap(), provider.base_url.clone()).await?
        } else if name_lc.contains("tpss") {
            self.fetch_tpss(provider.id.unwrap(), provider.base_url.clone()).await?
        } else if name_lc.contains("martial") {
            self.fetch_martial_events(provider.id.unwrap(), provider.base_url.clone()).await?
        } else {
            Vec::new()
        };

        // Persist tournaments
        let mut connw = self.database.get_connection().await.map_err(|e| e.to_string())?;
        for t in tournaments {
            let _ = crate::database::operations::OvrOperations::upsert_tournament(&mut *connw, &t)
                .map_err(|e| e.to_string())?;
        }
        let _ = crate::database::operations::OvrOperations::set_provider_refresh_status(&mut *connw, provider_id, Some("ok"), None)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn fetch_simplycompete(&self, _provider_id: i64, _base_url: Option<String>) -> Result<Vec<OvrTournament>, String> {
        // Placeholder: In real implementation, scrape API/list pages; here, no-op with empty result to avoid external calls without keys
        Ok(Vec::new())
    }

    async fn fetch_tpss(&self, _provider_id: i64, _base_url: Option<String>) -> Result<Vec<OvrTournament>, String> {
        Ok(Vec::new())
    }

    async fn fetch_martial_events(&self, _provider_id: i64, _base_url: Option<String>) -> Result<Vec<OvrTournament>, String> {
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OvrRefreshResult { pub providers_processed: usize }


