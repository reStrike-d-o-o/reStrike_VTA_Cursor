use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::database::DatabaseConnection;
use crate::database::models::{OvrProvider, OvrTournament};
use sha2::{Sha256, Digest};
use chrono::Utc;

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
        let url = _base_url.unwrap_or_else(|| "https://worldtkd.simplycompete.com".to_string());
        self.fetch_from_jsonld_page(_provider_id, &url).await
    }

    async fn fetch_tpss(&self, _provider_id: i64, _base_url: Option<String>) -> Result<Vec<OvrTournament>, String> {
        let url = _base_url.unwrap_or_else(|| "https://www.tpss.eu".to_string());
        self.fetch_from_jsonld_page(_provider_id, &url).await
    }

    async fn fetch_martial_events(&self, _provider_id: i64, _base_url: Option<String>) -> Result<Vec<OvrTournament>, String> {
        let url = _base_url.unwrap_or_else(|| "https://www.martial.events".to_string());
        // First try a known JSON API endpoint; fall back to JSON-LD
        if let Ok(list) = self.fetch_martial_events_api(_provider_id).await { return Ok(list); }
        self.fetch_from_jsonld_page(_provider_id, &url).await
    }

    async fn fetch_martial_events_api(&self, provider_id: i64) -> Result<Vec<OvrTournament>, String> {
        // Speculative public API; if it fails, caller falls back
        let client = reqwest::Client::new();
        let resp = client
            .get("https://www.martial.events/api/events")
            .header("User-Agent", "reStrike-VTA/1.0")
            .send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() { return Err(format!("status {}", resp.status())); }
        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        let now = Utc::now();
        let mut out: Vec<OvrTournament> = Vec::new();
        if let Some(arr) = json.as_array() {
            for item in arr {
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if name.is_empty() { continue; }
                let start = item.get("startDate").or_else(|| item.get("start"))
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc));
                let end = item.get("endDate").or_else(|| item.get("end"))
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc));
                let city = item.get("city").or_else(|| item.get("location").and_then(|l| l.get("addressLocality"))).and_then(|v| v.as_str()).map(|s| s.to_string());
                let country = item.get("country").or_else(|| item.get("location").and_then(|l| l.get("addressCountry"))).and_then(|v| v.as_str()).map(|s| s.to_string());
                let url = item.get("url").and_then(|v| v.as_str()).map(|s| s.to_string());
                let status = item.get("status").and_then(|v| v.as_str()).map(|s| s.to_string());
                let key = Self::stable_key(&name, start.as_ref().map(|d| d.to_rfc3339()));
                out.push(OvrTournament {
                    id: None,
                    provider_id,
                    provider_tournament_id: key,
                    name,
                    start_date: start,
                    end_date: end,
                    city,
                    country,
                    url,
                    status,
                    last_seen_at: Some(now),
                    hash: None,
                    etag: None,
                    created_at: now,
                    updated_at: now,
                });
            }
        }
        Ok(out)
    }

    async fn fetch_from_jsonld_page(&self, provider_id: i64, url: &str) -> Result<Vec<OvrTournament>, String> {
        let client = reqwest::Client::new();
        let resp = client
            .get(url)
            .header("User-Agent", "reStrike-VTA/1.0")
            .send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() { return Err(format!("status {}", resp.status())); }
        let body = resp.text().await.map_err(|e| e.to_string())?;
        let jsons = Self::extract_jsonld(&body);
        let now = Utc::now();
        let mut out: Vec<OvrTournament> = Vec::new();
        for js in jsons {
            match serde_json::from_str::<serde_json::Value>(&js) {
                Ok(val) => {
                    // Could be a single Event or an array
                    if let Some(arr) = val.as_array() {
                        for item in arr { if let Some(t) = Self::jsonld_to_tournament(provider_id, item, now) { out.push(t); } }
                    } else if let Some(obj) = val.as_object() {
                        if let Some(t) = Self::jsonld_to_tournament(provider_id, &serde_json::Value::Object(obj.clone()), now) { out.push(t); }
                    }
                }
                Err(_) => { /* skip invalid json */ }
            }
        }
        Ok(out)
    }

    fn extract_jsonld(html: &str) -> Vec<String> {
        let mut res = Vec::new();
        let marker = "<script";
        let ty = "application/ld+json";
        let mut pos = 0;
        while let Some(idx) = html[pos..].find(marker) {
            let start = pos + idx;
            let tag_end = match html[start..].find('>') { Some(off) => start + off + 1, None => break };
            let tag = &html[start..tag_end];
            if tag.to_lowercase().contains(ty) {
                if let Some(close_idx) = html[tag_end..].find("</script>") {
                    let json_str = &html[tag_end..tag_end + close_idx];
                    res.push(json_str.trim().to_string());
                    pos = tag_end + close_idx + 9; // len("</script>")
                    continue;
                }
            }
            pos = tag_end;
        }
        res
    }

    fn jsonld_to_tournament(provider_id: i64, v: &serde_json::Value, now: chrono::DateTime<Utc>) -> Option<OvrTournament> {
        // Accept either @type == Event or objects with event-like fields
        let ty = v.get("@type").and_then(|s| s.as_str()).unwrap_or("");
        if !(ty.eq_ignore_ascii_case("Event") || v.get("name").is_some()) { return None; }
        let name = v.get("name").and_then(|s| s.as_str())?.to_string();
        let start = v.get("startDate").and_then(|s| s.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        let end = v.get("endDate").and_then(|s| s.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        let (city, country) = if let Some(loc) = v.get("location") {
            let city = loc.get("addressLocality").and_then(|s| s.as_str()).map(|s| s.to_string());
            let country = loc.get("addressCountry").and_then(|s| s.as_str()).map(|s| s.to_string());
            (city, country)
        } else { (None, None) };
        let url = v.get("url").and_then(|s| s.as_str()).map(|s| s.to_string());
        let status = v.get("eventStatus").or_else(|| v.get("status")).and_then(|s| s.as_str()).map(|s| s.to_string());
        let key = Self::stable_key(&name, start.as_ref().map(|d| d.to_rfc3339()));
        Some(OvrTournament {
            id: None,
            provider_id,
            provider_tournament_id: key,
            name,
            start_date: start,
            end_date: end,
            city,
            country,
            url,
            status,
            last_seen_at: Some(now),
            hash: None,
            etag: None,
            created_at: now,
            updated_at: now,
        })
    }

    fn stable_key(name: &str, start_iso: Option<String>) -> String {
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        if let Some(s) = start_iso { hasher.update(s.as_bytes()); }
        let bytes = hasher.finalize();
        hex::encode(&bytes[..16]) // short key
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OvrRefreshResult { pub providers_processed: usize }


