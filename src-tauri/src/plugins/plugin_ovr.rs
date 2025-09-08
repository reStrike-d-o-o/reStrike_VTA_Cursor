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
        let rate_limit_ms = provider.rate_limit_ms;

        // Adapters: scrape specific sites
        let tournaments: Vec<OvrTournament> = if name_lc.contains("simply") {
            self.fetch_simplycompete(provider.id.unwrap(), provider.base_url.clone(), rate_limit_ms).await?
        } else if name_lc.contains("tpss") {
            self.fetch_tpss(provider.id.unwrap(), provider.base_url.clone(), rate_limit_ms).await?
        } else if name_lc.contains("martial") {
            self.fetch_martial_events(provider.id.unwrap(), provider.base_url.clone(), rate_limit_ms).await?
        } else if name_lc.contains("etu") || name_lc.contains("europe") {
            self.fetch_etu(provider.id.unwrap(), provider.base_url.clone(), rate_limit_ms).await?
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

    async fn fetch_simplycompete(&self, provider_id: i64, base_url: Option<String>, rate_limit_ms: i64) -> Result<Vec<OvrTournament>, String> {
        let url = base_url.unwrap_or_else(|| "https://worldtkd.simplycompete.com/events?eventType=Tournament&invitationStatus=all&da&isArchived=false&pageNumber=1&itemsPerPage=1000".to_string());
        // Try with main URL; if forbidden, attempt without some query flags and with alternate path
        let html = match self.fetch_html_rl(&url, rate_limit_ms).await {
            Ok(h) => h,
            Err(e) if e.contains("403") => {
                // Fallback URL variants commonly used on SimplyCompete
                let variants = [
                    "https://worldtkd.simplycompete.com/events",
                    "https://worldtkd.simplycompete.com/",
                ];
                let mut ok: Option<String> = None;
                for v in variants.iter() {
                    if let Ok(h) = self.fetch_html_rl(v, rate_limit_ms).await { ok = Some(h); break; }
                }
                ok.ok_or_else(|| e)?
            }
            Err(e) => return Err(e),
        };
        let mut items = self.parse_jsonld_block(provider_id, &html);
        if items.is_empty() {
            // Also probe detail links for richer data
            let anchors = self.parse_generic_anchors(provider_id, &html, &url);
            for a in anchors.into_iter().take(50) {
                if let Some(detail_url) = a.url.clone() {
                    if let Ok(detail_html) = self.fetch_html_rl(&detail_url, rate_limit_ms).await {
                        let enriched = self.parse_jsonld_block(provider_id, &detail_html);
                        if !enriched.is_empty() { items.extend(enriched); }
                    }
                }
            }
        }
        Ok(items)
    }

    async fn fetch_tpss(&self, provider_id: i64, base_url: Option<String>, rate_limit_ms: i64) -> Result<Vec<OvrTournament>, String> {
        let base = base_url.unwrap_or_else(|| "https://www.tpss.eu".to_string());
        let mut out: Vec<OvrTournament> = Vec::new();
        for path in ["/liveresults.asp?AR=1", "/liveresults.asp?AR=2", "/Results.asp?YR=All"] {
            let url = format!("{}{}", base, path);
            if let Ok(html) = self.fetch_html_rl(&url, rate_limit_ms).await {
                let parsed = self.parse_tpss_html(provider_id, &html, &base);
                // Enrich from detail page
                for mut t in parsed {
                    if let Some(ref detail) = t.url {
                        if let Ok(dhtml) = self.fetch_html_rl(detail, rate_limit_ms).await {
                            if let Some((city, country, start, end)) = Self::parse_tpss_detail(&dhtml) {
                                if city.is_some() { t.city = city; }
                                if country.is_some() { t.country = country; }
                                if start.is_some() { t.start_date = start; }
                                if end.is_some() { t.end_date = end; }
                            }
                        }
                    }
                    out.push(t);
                }
            }
        }
        Ok(out)
    }

    async fn fetch_martial_events(&self, provider_id: i64, base_url: Option<String>, rate_limit_ms: i64) -> Result<Vec<OvrTournament>, String> {
        let url = base_url.unwrap_or_else(|| "https://www.martial.events/en".to_string());
        let html = self.fetch_html_rl(&url, rate_limit_ms).await?;
        let mut items = self.parse_jsonld_block(provider_id, &html);
        if items.is_empty() {
            let anchors = self.parse_martial_events_html(provider_id, &html, &url);
            // Optionally follow a few detail pages to enrich
            for a in anchors.iter().take(30) {
                if let Some(detail_url) = a.url.clone() {
                    if let Ok(detail_html) = self.fetch_html_rl(&detail_url, rate_limit_ms).await {
                        let enriched = self.parse_jsonld_block(provider_id, &detail_html);
                        if !enriched.is_empty() { items.extend(enriched); }
                    }
                }
            }
            if items.is_empty() { items.extend(anchors); }
        }
        Ok(items)
    }
    async fn fetch_html(&self, url: &str) -> Result<String, String> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(20))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build().map_err(|e| e.to_string())?;
        let resp = client
            .get(url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Referer", url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() { return Err(format!("status {}", resp.status())); }
        let body = resp.text().await.map_err(|e| e.to_string())?;
        Ok(body)
    }

    async fn fetch_html_rl(&self, url: &str, delay_ms: i64) -> Result<String, String> {
        if delay_ms > 0 { tokio::time::sleep(Duration::from_millis(delay_ms as u64)).await; }
        self.fetch_html(url).await
    }

    fn parse_jsonld_block(&self, provider_id: i64, body: &str) -> Vec<OvrTournament> {
        let jsons = Self::extract_jsonld(&body);
        let now = Utc::now();
        let mut out: Vec<OvrTournament> = Vec::new();
        for js in jsons {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&js) {
                // Flatten common JSON-LD containers: @graph, ItemList.itemListElement[].item
                let mut candidates: Vec<serde_json::Value> = Vec::new();
                match &val {
                    serde_json::Value::Array(arr) => {
                        for v in arr { candidates.push(v.clone()); }
                    }
                    serde_json::Value::Object(map) => {
                        // @graph
                        if let Some(graph) = map.get("@graph").and_then(|g| g.as_array()) {
                            for v in graph { candidates.push(v.clone()); }
                        } else if map.get("@type").is_some() || map.get("name").is_some() {
                            candidates.push(val.clone());
                        }
                        // ItemList -> itemListElement -> item
                        if let Some(items) = map.get("itemListElement").and_then(|e| e.as_array()) {
                            for it in items {
                                if let Some(item) = it.get("item") {
                                    candidates.push(item.clone());
                                }
                            }
                        }
                    }
                    _ => {}
                }
                for cand in candidates {
                    if let Some(t) = Self::jsonld_to_tournament(provider_id, &cand, now) { out.push(t); }
                }
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
            match loc {
                serde_json::Value::String(s) => {
                    // Try to split "City, Country"
                    let parts: Vec<&str> = s.split(',').map(|x| x.trim()).collect();
                    if parts.len() >= 2 {
                        (Some(parts[0].to_string()), Some(parts[parts.len()-1].to_string()))
                    } else { (None, None) }
                }
                serde_json::Value::Object(obj) => {
                    // location.name may contain "City, Country"
                    if let Some(name_s) = obj.get("name").and_then(|s| s.as_str()) {
                        let parts: Vec<&str> = name_s.split(',').map(|x| x.trim()).collect();
                        if parts.len() >= 2 {
                            (Some(parts[0].to_string()), Some(parts[parts.len()-1].to_string()))
                        } else {
                            (None, None)
                        }
                    } else {
                        // Direct fields or nested address
                        let city = obj.get("addressLocality").and_then(|s| s.as_str()).map(|s| s.to_string())
                            .or_else(|| obj.get("address").and_then(|a| a.get("addressLocality")).and_then(|s| s.as_str()).map(|s| s.to_string()));
                        let country_val = obj.get("addressCountry").or_else(|| obj.get("address").and_then(|a| a.get("addressCountry")));
                        let country = match country_val {
                            Some(serde_json::Value::String(s)) => Some(s.to_string()),
                            Some(serde_json::Value::Object(o)) => o.get("name").and_then(|s| s.as_str()).map(|s| s.to_string()),
                            _ => None,
                        };
                        (city, country)
                    }
                }
                _ => (None, None)
            }
        } else { (None, None) };
        let url = v.get("url").and_then(|s| s.as_str()).map(|s| s.to_string());
        let status = v.get("eventStatus").or_else(|| v.get("status")).and_then(|s| s.as_str()).map(|s| s.to_string());
        let key = Self::stable_key(&name, start.as_ref().map(|d| d.to_rfc3339()));
        let dedupe_hash = Self::stable_dedupe_key(&name, start.as_ref().map(|d| d.to_rfc3339()), city.as_deref(), country.as_deref());
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
            hash: dedupe_hash,
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

    fn normalize_text(s: &str) -> String {
        s.trim().to_lowercase()
    }

    fn stable_dedupe_key(name: &str, start_iso: Option<String>, city: Option<&str>, country: Option<&str>) -> Option<String> {
        // Require at least name + start for cross-provider dedupe to avoid false positives
        if start_iso.is_none() { return None; }
        let mut hasher = Sha256::new();
        hasher.update(Self::normalize_text(name).as_bytes());
        if let Some(s) = &start_iso { hasher.update(s.as_bytes()); }
        if let Some(c) = city { hasher.update(Self::normalize_text(c).as_bytes()); }
        if let Some(cn) = country { hasher.update(Self::normalize_text(cn).as_bytes()); }
        let bytes = hasher.finalize();
        Some(hex::encode(&bytes[..16]))
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

    fn parse_tpss_detail(html: &str) -> Option<(Option<String>, Option<String>, Option<chrono::DateTime<Utc>>, Option<chrono::DateTime<Utc>>)> {
        let city = Regex::new(r"City[^\[]*\[([^\(\]]+)(?:\(([^\)]+)\))?").ok()
            .and_then(|re| re.captures(html).map(|c| (c.get(1).map(|m| m.as_str().trim().to_string()), c.get(2).map(|m| m.as_str().trim().to_string()))));
        let (city_val, country_val) = match city { Some((c, cn)) => (c, cn), None => (None, None) };
        let date_cap = Regex::new(r"Eventdate\s*(\d{2}-\d{2}-\d{4})(?:\s*[-–]\s*(\d{2}-\d{2}-\d{4}))?").ok()
            .and_then(|re| re.captures(html));
        let (start, end) = if let Some(dc) = date_cap { let sd = dc.get(1).map(|m| m.as_str()); let ed = dc.get(2).map(|m| m.as_str()); (Self::parse_dmy_to_utc(sd), Self::parse_dmy_to_utc(ed)) } else { (None, None) };
        if city_val.is_none() && country_val.is_none() && start.is_none() && end.is_none() { None } else { Some((city_val, country_val, start, end)) }
    }

    fn parse_dmy_to_utc(s: Option<&str>) -> Option<chrono::DateTime<Utc>> {
        if let Some(src) = s { if let Ok(nd) = chrono::NaiveDate::parse_from_str(src, "%d-%m-%Y") { let dt = nd.and_hms_opt(0, 0, 0)?.and_utc(); return Some(dt); } }
        None
    }

    fn find_next_page_url(html: &str, current_url: &str) -> Option<String> {
        let re = Regex::new(r#"<a[^>]+rel=\"next\"[^>]+href=\"([^\"]+)\""#).ok()?;
        if let Some(c) = re.captures(html) { return Some(Self::absolute_url(current_url, c.get(1)?.as_str())); }
        let re2 = Regex::new(r#"<a[^>]+class=\"[^\"]*next[^\"]*\"[^>]+href=\"([^\"]+)\""#).ok()?;
        if let Some(c) = re2.captures(html) { return Some(Self::absolute_url(current_url, c.get(1)?.as_str())); }
        None
    }

    async fn fetch_etu(&self, provider_id: i64, base_url: Option<String>, rate_limit_ms: i64) -> Result<Vec<OvrTournament>, String> {
        let base = base_url.unwrap_or_else(|| "https://europetaekwondo.org/events/".to_string());
        let mut collected: Vec<OvrTournament> = Vec::new();
        let mut page_url = base.clone();
        for _ in 0..10 {
            let html = self.fetch_html_rl(&page_url, rate_limit_ms).await?;
            let anchors = self.parse_generic_anchors(provider_id, &html, &page_url);
            let mut added = false;
            for a in anchors.iter().take(50) {
                if let Some(detail_url) = a.url.clone() {
                    if let Ok(detail_html) = self.fetch_html_rl(&detail_url, rate_limit_ms).await {
                        let enriched = self.parse_jsonld_block(provider_id, &detail_html);
                        if !enriched.is_empty() { collected.extend(enriched); added = true; }
                    }
                }
            }
            if let Some(next) = Self::find_next_page_url(&html, &page_url) { page_url = next; if !added { break; } } else { break; }
        }
        Ok(collected)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OvrRefreshResult { pub providers_processed: usize }


