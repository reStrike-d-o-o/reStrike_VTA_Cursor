use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::database::models::PssEventV2;
use crate::AppResult;

/// Cache entry with TTL (Time To Live)
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub data: T,
    pub created_at: std::time::SystemTime,
    pub ttl: std::time::Duration,
}

impl<T> CacheEntry<T> {
    pub fn new(data: T, ttl: std::time::Duration) -> Self {
        Self {
            data,
            created_at: std::time::SystemTime::now(),
            ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Ok(elapsed) = self.created_at.elapsed() {
            elapsed > self.ttl
        } else {
            true
        }
    }
}

/// Athlete statistics for caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthleteStatistics {
    pub athlete_id: i64,
    pub total_events: u64,
    pub total_points: u64,
    pub total_warnings: u64,
    pub total_injuries: u64,
    pub avg_hit_level: f64,
    pub last_updated: std::time::SystemTime,
}

/// Tournament statistics for caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentStatistics {
    pub tournament_id: i64,
    pub total_events: u64,
    pub events_per_day: HashMap<i32, u64>,
    pub active_athletes: u32,
    pub last_updated: std::time::SystemTime,
}

/// Match statistics for caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchStatistics {
    pub match_id: i64,
    pub event_count: u64,
    pub duration_seconds: u64,
    pub athlete1_score: u64,
    pub athlete2_score: u64,
    pub last_updated: std::time::SystemTime,
}

/// Advanced caching system for frequently accessed data
pub struct EventCache {
    tournament_events: Arc<RwLock<HashMap<i64, CacheEntry<Vec<PssEventV2>>>>>,
    match_events: Arc<RwLock<HashMap<i64, CacheEntry<Vec<PssEventV2>>>>>,
    athlete_stats: Arc<RwLock<HashMap<i64, CacheEntry<AthleteStatistics>>>>,
    tournament_stats: Arc<RwLock<HashMap<i64, CacheEntry<TournamentStatistics>>>>,
    match_stats: Arc<RwLock<HashMap<i64, CacheEntry<MatchStatistics>>>>,
    cache_config: CacheConfig,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub tournament_events_ttl: std::time::Duration,
    pub match_events_ttl: std::time::Duration,
    pub athlete_stats_ttl: std::time::Duration,
    pub tournament_stats_ttl: std::time::Duration,
    pub match_stats_ttl: std::time::Duration,
    pub max_tournament_events: usize,
    pub max_match_events: usize,
    pub max_athlete_stats: usize,
    pub max_tournament_stats: usize,
    pub max_match_stats: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            tournament_events_ttl: std::time::Duration::from_secs(300), // 5 minutes
            match_events_ttl: std::time::Duration::from_secs(180), // 3 minutes
            athlete_stats_ttl: std::time::Duration::from_secs(600), // 10 minutes
            tournament_stats_ttl: std::time::Duration::from_secs(300), // 5 minutes
            match_stats_ttl: std::time::Duration::from_secs(180), // 3 minutes
            max_tournament_events: 100,
            max_match_events: 200,
            max_athlete_stats: 500,
            max_tournament_stats: 50,
            max_match_stats: 100,
        }
    }
}

impl EventCache {
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            tournament_events: Arc::new(RwLock::new(HashMap::new())),
            match_events: Arc::new(RwLock::new(HashMap::new())),
            athlete_stats: Arc::new(RwLock::new(HashMap::new())),
            tournament_stats: Arc::new(RwLock::new(HashMap::new())),
            match_stats: Arc::new(RwLock::new(HashMap::new())),
            cache_config: config,
        }
    }

    /// Get tournament events from cache
    pub async fn get_tournament_events(&self, tournament_id: i64) -> Option<Vec<PssEventV2>> {
        let cache = self.tournament_events.read().await;
        if let Some(entry) = cache.get(&tournament_id) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    /// Set tournament events in cache
    pub async fn set_tournament_events(&self, tournament_id: i64, events: Vec<PssEventV2>) -> AppResult<()> {
        let mut cache = self.tournament_events.write().await;
        
        // Clean up expired entries and enforce size limit
        self.cleanup_cache(&mut cache, self.cache_config.max_tournament_events);
        
        let entry = CacheEntry::new(events, self.cache_config.tournament_events_ttl);
        cache.insert(tournament_id, entry);
        Ok(())
    }

    /// Get match events from cache
    pub async fn get_match_events(&self, match_id: i64) -> Option<Vec<PssEventV2>> {
        let cache = self.match_events.read().await;
        if let Some(entry) = cache.get(&match_id) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    /// Set match events in cache
    pub async fn set_match_events(&self, match_id: i64, events: Vec<PssEventV2>) -> AppResult<()> {
        let mut cache = self.match_events.write().await;
        
        // Clean up expired entries and enforce size limit
        self.cleanup_cache(&mut cache, self.cache_config.max_match_events);
        
        let entry = CacheEntry::new(events, self.cache_config.match_events_ttl);
        cache.insert(match_id, entry);
        Ok(())
    }

    /// Get athlete statistics from cache
    pub async fn get_athlete_stats(&self, athlete_id: i64) -> Option<AthleteStatistics> {
        let cache = self.athlete_stats.read().await;
        if let Some(entry) = cache.get(&athlete_id) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    /// Set athlete statistics in cache
    pub async fn set_athlete_stats(&self, athlete_id: i64, stats: AthleteStatistics) -> AppResult<()> {
        let mut cache = self.athlete_stats.write().await;
        
        // Clean up expired entries and enforce size limit
        self.cleanup_cache(&mut cache, self.cache_config.max_athlete_stats);
        
        let entry = CacheEntry::new(stats, self.cache_config.athlete_stats_ttl);
        cache.insert(athlete_id, entry);
        Ok(())
    }

    /// Get tournament statistics from cache
    pub async fn get_tournament_stats(&self, tournament_id: i64) -> Option<TournamentStatistics> {
        let cache = self.tournament_stats.read().await;
        if let Some(entry) = cache.get(&tournament_id) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    /// Set tournament statistics in cache
    pub async fn set_tournament_stats(&self, tournament_id: i64, stats: TournamentStatistics) -> AppResult<()> {
        let mut cache = self.tournament_stats.write().await;
        
        // Clean up expired entries and enforce size limit
        self.cleanup_cache(&mut cache, self.cache_config.max_tournament_stats);
        
        let entry = CacheEntry::new(stats, self.cache_config.tournament_stats_ttl);
        cache.insert(tournament_id, entry);
        Ok(())
    }

    /// Get match statistics from cache
    pub async fn get_match_stats(&self, match_id: i64) -> Option<MatchStatistics> {
        let cache = self.match_stats.read().await;
        if let Some(entry) = cache.get(&match_id) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    /// Set match statistics in cache
    pub async fn set_match_stats(&self, match_id: i64, stats: MatchStatistics) -> AppResult<()> {
        let mut cache = self.match_stats.write().await;
        
        // Clean up expired entries and enforce size limit
        self.cleanup_cache(&mut cache, self.cache_config.max_match_stats);
        
        let entry = CacheEntry::new(stats, self.cache_config.match_stats_ttl);
        cache.insert(match_id, entry);
        Ok(())
    }

    /// Invalidate cache entries for a specific tournament
    pub async fn invalidate_tournament(&self, tournament_id: i64) -> AppResult<()> {
        let mut tournament_cache = self.tournament_events.write().await;
        tournament_cache.remove(&tournament_id);
        
        let mut tournament_stats_cache = self.tournament_stats.write().await;
        tournament_stats_cache.remove(&tournament_id);
        
        Ok(())
    }

    /// Invalidate cache entries for a specific match
    pub async fn invalidate_match(&self, match_id: i64) -> AppResult<()> {
        let mut match_cache = self.match_events.write().await;
        match_cache.remove(&match_id);
        
        let mut match_stats_cache = self.match_stats.write().await;
        match_stats_cache.remove(&match_id);
        
        Ok(())
    }

    /// Invalidate cache entries for a specific athlete
    pub async fn invalidate_athlete(&self, athlete_id: i64) -> AppResult<()> {
        let mut athlete_cache = self.athlete_stats.write().await;
        athlete_cache.remove(&athlete_id);
        Ok(())
    }

    /// Clear all caches
    pub async fn clear_all(&self) -> AppResult<()> {
        let mut tournament_cache = self.tournament_events.write().await;
        tournament_cache.clear();
        
        let mut match_cache = self.match_events.write().await;
        match_cache.clear();
        
        let mut athlete_cache = self.athlete_stats.write().await;
        athlete_cache.clear();
        
        let mut tournament_stats_cache = self.tournament_stats.write().await;
        tournament_stats_cache.clear();
        
        let mut match_stats_cache = self.match_stats.write().await;
        match_stats_cache.clear();
        
        Ok(())
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStatistics {
        let tournament_events_count = self.tournament_events.read().await.len();
        let match_events_count = self.match_events.read().await.len();
        let athlete_stats_count = self.athlete_stats.read().await.len();
        let tournament_stats_count = self.tournament_stats.read().await.len();
        let match_stats_count = self.match_stats.read().await.len();

        CacheStatistics {
            tournament_events_count,
            match_events_count,
            athlete_stats_count,
            tournament_stats_count,
            match_stats_count,
            total_entries: tournament_events_count + match_events_count + athlete_stats_count + tournament_stats_count + match_stats_count,
        }
    }

    /// Cleanup expired entries and enforce size limits
    fn cleanup_cache<T>(&self, cache: &mut HashMap<i64, CacheEntry<T>>, max_size: usize) {
        // Remove expired entries
        cache.retain(|_, entry| !entry.is_expired());
        
        // If still over limit, remove oldest entries
        if cache.len() > max_size {
            let mut entries: Vec<_> = cache.iter().collect();
            entries.sort_by(|a, b| a.1.created_at.cmp(&b.1.created_at));
            
            let to_remove = entries.len() - max_size;
            let keys_to_remove: Vec<i64> = entries.iter().take(to_remove).map(|(key, _)| **key).collect();
            
            for key in keys_to_remove {
                cache.remove(&key);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub tournament_events_count: usize,
    pub match_events_count: usize,
    pub athlete_stats_count: usize,
    pub tournament_stats_count: usize,
    pub match_stats_count: usize,
    pub total_entries: usize,
}

impl Default for EventCache {
    fn default() -> Self {
        Self::new()
    }
} 