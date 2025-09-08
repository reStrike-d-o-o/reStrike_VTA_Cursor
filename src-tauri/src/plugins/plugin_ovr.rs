use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::database::DatabaseConnection;
use crate::database::models::{OvrProvider, OvrTournament};
use sha2::{Sha256, Digest};
use chrono::Utc;
use regex::Regex;
use std::time::Duration;

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
        // Load providers in a short scope so the DB lock is released before per-provider refresh
        let providers = {
            let conn = self.database.get_connection().await.map_err(|e| e.to_string())?;
            crate::database::operations::OvrOperations::get_providers(&*conn).map_err(|e| e.to_string())?
        };
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
        // Read provider in a short scope so the DB lock is released before network calls
        let provider = {
            let conn = self.database.get_connection().await.map_err(|e| e.to_string())?;
            let mut stmt = conn.prepare("SELECT * FROM ovr_providers WHERE id = ?").map_err(|e| e.to_string())?;
            stmt.query_row([provider_id], |row| OvrProvider::from_row(row)).map_err(|e| e.to_string())?
        };

        // Mark provider as fetching
        {
            let mut connw = self.database.get_connection().await.map_err(|e| e.to_string())?;
            let _ = crate::database::operations::OvrOperations::set_provider_refresh_status(&mut *connw, provider_id, Some("fetching"), None)
                .map_err(|e| e.to_string())?;
        }
        let name_lc = provider.name.to_lowercase();

        // Adapters: scrape specific sites
        let tournaments: Vec<OvrTournament> = if name_lc.contains("simply") {
            self.fetch_simplycompete(provider.id.unwrap(), provider.base_url.clone()).await?
        } else if name_lc.contains("tpss") {
            self.fetch_tpss(provider.id.unwrap(), provider.base_url.clone()).await?
        } else if name_lc.contains("martial") {
            self.fetch_martial_events(provider.id.unwrap(), provider.base_url.clone()).await?
        } else if name_lc.contains("etu") || name_lc.contains("europe") {
            self.fetch_etu(provider.id.unwrap(), provider.base_url.clone()).await?
        } else {
            Vec::new()
        };

        // Persist tournaments (obtain a fresh DB lock now)
        let mut connw = self.database.get_connection().await.map_err(|e| e.to_string())?;
        for t in tournaments {
            let _ = crate::database::operations::OvrOperations::upsert_tournament(&mut *connw, &t)
                .map_err(|e| e.to_string())?;
        }
        let _ = crate::database::operations::OvrOperations::set_provider_refresh_status(&mut *connw, provider_id, Some("ok"), None)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn fetch_simplycompete(&self, provider_id: i64, base_url: Option<String>) -> Result<Vec<OvrTournament>, String> {
        let url = base_url.unwrap_or_else(|| "https://worldtkd.simplycompete.com/events?eventType=Tournament&invitationStatus=all&da&isArchived=false&pageNumber=1&itemsPerPage=1000".to_string());
        let html = self.fetch_html(&url).await?;
        let mut items = self.parse_jsonld_block(provider_id, &html);
        if items.is_empty() {
            items.extend(self.parse_generic_anchors(provider_id, &html, &url));
        }
        Ok(items)
    }

    async fn fetch_tpss(&self, provider_id: i64, base_url: Option<String>) -> Result<Vec<OvrTournament>, String> {
        let base = base_url.unwrap_or_else(|| "https://www.tpss.eu".to_string());
        let mut out: Vec<OvrTournament> = Vec::new();
        for path in ["/liveresults.asp?AR=1", "/liveresults.asp?AR=2", "/Results.asp?YR=All"] {
            let url = format!("{}{}", base, path);
            if let Ok(html) = self.fetch_html(&url).await {
                let parsed = self.parse_tpss_html(provider_id, &html, &base);
                out.extend(parsed);
            }
        }
        Ok(out)
    }

    async fn fetch_martial_events(&self, provider_id: i64, base_url: Option<String>) -> Result<Vec<OvrTournament>, String> {
        let url = base_url.unwrap_or_else(|| "https://www.martial.events/en".to_string());
        let html = self.fetch_html(&url).await?;
        let mut items = self.parse_jsonld_block(provider_id, &html);
        if items.is_empty() {
            items.extend(self.parse_martial_events_html(provider_id, &html, &url));
        }
        Ok(items)
    }
    async fn fetch_html(&self, url: &str) -> Result<String, String> {
        let client = reqwest::Client::builder()
            .user_agent("reStrike-VTA/1.0")
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(20))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build().map_err(|e| e.to_string())?;
        let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() { return Err(format!("status {}", resp.status())); }
        let body = resp.text().await.map_err(|e| e.to_string())?;
        Ok(body)
    }

    fn parse_jsonld_block(&self, provider_id: i64, body: &str) -> Vec<OvrTournament> {
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
        out
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

    fn absolute_url(base: &str, href: &str) -> String {
        if href.starts_with("http://") || href.starts_with("https://") { return href.to_string(); }
        let sep = if href.starts_with('/') { "" } else { "/" };
        format!("{}{}{}", base.trim_end_matches('/'), sep, href)
    }

    fn make_simple_tournament(&self, provider_id: i64, name: &str, url: Option<String>) -> OvrTournament {
        let now = Utc::now();
        OvrTournament {
            id: None,
            provider_id,
            provider_tournament_id: Self::stable_key(name, None),
            name: name.to_string(),
            start_date: None,
            end_date: None,
            city: None,
            country: None,
            url,
            status: None,
            last_seen_at: Some(now),
            hash: None,
            etag: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn parse_generic_anchors(&self, provider_id: i64, html: &str, base_url: &str) -> Vec<OvrTournament> {
        let re = Regex::new(r#"<a[^>]+href=\"([^\"]+)\"[^>]*>([^<]{3,})</a>"#).unwrap();
        let mut out = Vec::new();
        for cap in re.captures_iter(html) {
            let href = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let text = cap.get(2).map(|m| m.as_str()).unwrap_or("").trim();
            let h = href.to_lowercase();
            if !(h.contains("event") || h.contains("tourn") || h.contains("checktournament.asp")) { continue; }
            if text.is_empty() { continue; }
            let url = Some(Self::absolute_url(base_url, href));
            out.push(self.make_simple_tournament(provider_id, text, url));
        }
        out
    }

    fn parse_tpss_html(&self, provider_id: i64, html: &str, base_url: &str) -> Vec<OvrTournament> {
        let re = Regex::new(r#"<a[^>]+href=\"(CheckTournament\.asp\?Code=[^\"]+)\"[^>]*>\s*([^<][^<]+?)\s*</a>"#).unwrap();
        let mut out = Vec::new();
        for cap in re.captures_iter(html) {
            let href = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("").trim();
            if name.is_empty() { continue; }
            let full = Self::absolute_url(base_url, href);
            out.push(self.make_simple_tournament(provider_id, name, Some(full)));
        }
        out
    }

    fn parse_martial_events_html(&self, provider_id: i64, html: &str, base_url: &str) -> Vec<OvrTournament> {
        let re = Regex::new(r#"<a[^>]+href=\"(/en/[^\"]+)\"[^>]*>\s*<[^>]*>\s*([^<][^<]+?)\s*</[^>]*>"#).unwrap();
        let mut out = Vec::new();
        for cap in re.captures_iter(html) {
            let href = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("").trim();
            if name.is_empty() { continue; }
            let full = Self::absolute_url(base_url, href);
            out.push(self.make_simple_tournament(provider_id, name, Some(full)));
        }
        if out.is_empty() {
            out = self.parse_generic_anchors(provider_id, html, base_url);
        }
        out
    }

    async fn fetch_etu(&self, provider_id: i64, base_url: Option<String>) -> Result<Vec<OvrTournament>, String> {
        let url = base_url.unwrap_or_else(|| "https://europetaekwondo.org/events/list/".to_string());
        let html = self.fetch_html(&url).await?;
        let mut items = self.parse_jsonld_block(provider_id, &html);
        if items.is_empty() {
            items.extend(self.parse_generic_anchors(provider_id, &html, &url));
        }
        Ok(items)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OvrRefreshResult { pub providers_processed: usize }


