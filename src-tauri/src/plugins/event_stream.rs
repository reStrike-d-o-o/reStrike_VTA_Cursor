use crate::database::models::PssEventV2 as DbPssEvent;
use crate::plugins::event_cache::{
    AthleteStatistics, EventCache, MatchStatistics, TournamentStatistics,
};
use crate::plugins::plugin_database::DatabasePlugin;
use crate::plugins::plugin_udp::PssEvent as UdpPssEvent;
use crate::AppResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant, SystemTime};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::{interval, Duration};

/// Event stream configuration
#[derive(Debug, Clone)]
pub struct EventStreamConfig {
    pub buffer_size: usize,
    pub processing_interval_ms: u64,
    pub max_concurrent_processors: usize,
    pub enable_real_time_analytics: bool,
    pub analytics_update_interval_ms: u64,
}

impl Default for EventStreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            processing_interval_ms: 100,
            max_concurrent_processors: 4,
            enable_real_time_analytics: true,
            analytics_update_interval_ms: 5000,
        }
    }
}

/// Event stream processor for real-time event handling
pub struct EventStreamProcessor {
    event_tx: mpsc::UnboundedSender<DbPssEvent>,
    event_rx: Option<mpsc::UnboundedReceiver<DbPssEvent>>,
    broadcast_tx: broadcast::Sender<DbPssEvent>,
    cache: Arc<EventCache>,
    database: Arc<DatabasePlugin>,
    config: EventStreamConfig,
    processors: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
    analytics_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    statistics: Arc<RwLock<StreamStatistics>>,
    raw_metrics: Arc<RwLock<RawMetrics>>,
    match_athlete_cache: Arc<RwLock<HashMap<i64, MatchAthleteCacheEntry>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStatistics {
    pub total_events_processed: u64,
    pub events_per_second: f64,
    pub average_processing_time_ms: f64,
    pub cache_hit_rate: f64,
    pub active_processors: usize,
    pub last_updated: std::time::SystemTime,
}

impl Default for StreamStatistics {
    fn default() -> Self {
        Self {
            total_events_processed: 0,
            events_per_second: 0.0,
            average_processing_time_ms: 0.0,
            cache_hit_rate: 0.0,
            active_processors: 0,
            last_updated: std::time::SystemTime::now(),
        }
    }
}

#[derive(Debug, Default)]
struct RawMetrics {
    last_tick: Option<Instant>,
    events_since_tick: u64,
}

const MATCH_ATHLETE_CACHE_TTL: StdDuration = StdDuration::from_secs(30);

#[derive(Debug, Clone)]
struct MatchAthleteCacheEntry {
    athlete1_id: Option<i64>,
    athlete2_id: Option<i64>,
    fetched_at: SystemTime,
}

impl MatchAthleteCacheEntry {
    fn is_stale(&self) -> bool {
        self.fetched_at
            .elapsed()
            .map(|elapsed| elapsed > MATCH_ATHLETE_CACHE_TTL)
            .unwrap_or(true)
    }
}

/// Real-time analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeAnalytics {
    pub tournament_id: Option<i64>,
    pub match_id: Option<i64>,
    pub current_athlete_stats: HashMap<i64, AthleteStatistics>,
    pub current_tournament_stats: Option<TournamentStatistics>,
    pub current_match_stats: Option<MatchStatistics>,
    pub event_rate_per_minute: f64,
    pub top_athletes_by_points: Vec<(i64, u64)>,
    pub last_updated: std::time::SystemTime,
}

impl EventStreamProcessor {
    pub fn new(cache: Arc<EventCache>, database: Arc<DatabasePlugin>) -> Self {
        Self::with_config(cache, database, EventStreamConfig::default())
    }

    pub fn with_config(
        cache: Arc<EventCache>,
        database: Arc<DatabasePlugin>,
        config: EventStreamConfig,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (broadcast_tx, _) = broadcast::channel(config.buffer_size);

        Self {
            event_tx,
            event_rx: Some(event_rx),
            broadcast_tx,
            cache,
            database,
            config,
            processors: Arc::new(RwLock::new(Vec::new())),
            analytics_task: Arc::new(RwLock::new(None)),
            statistics: Arc::new(RwLock::new(StreamStatistics::default())),
            raw_metrics: Arc::new(RwLock::new(RawMetrics::default())),
            match_athlete_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the event stream processor
    pub async fn start(&mut self) -> AppResult<()> {
        log::info!("Starting Event Stream Processor...");

        // Start event processing loop
        let event_rx = self.event_rx.take().unwrap();
        let broadcast_tx = self.broadcast_tx.clone();
        let cache = self.cache.clone();
        let config = self.config.clone();
        let statistics = self.statistics.clone();

        let _processor_handle = tokio::spawn(async move {
            Self::event_processing_loop(event_rx, broadcast_tx, cache, config, statistics).await;
        });

        // Start analytics task if enabled
        if self.config.enable_real_time_analytics {
            let cache_clone = self.cache.clone();
            let analytics_interval =
                Duration::from_millis(self.config.analytics_update_interval_ms);

            let analytics_handle = tokio::spawn(async move {
                Self::analytics_update_loop(cache_clone, analytics_interval).await;
            });

            let mut analytics_task = self.analytics_task.write().await;
            *analytics_task = Some(analytics_handle);
        }

        // Start multiple event processors
        for i in 0..self.config.max_concurrent_processors {
            let broadcast_rx = self.broadcast_tx.subscribe();
            let cache_clone = self.cache.clone();
            let database_clone = self.database.clone();
            let match_cache_clone = self.match_athlete_cache.clone();
            let statistics_clone = self.statistics.clone();

            let processor_handle = tokio::spawn(async move {
                Self::event_processor_worker(
                    i,
                    broadcast_rx,
                    cache_clone,
                    database_clone,
                    match_cache_clone,
                    statistics_clone,
                )
                .await;
            });

            let mut processors = self.processors.write().await;
            processors.push(processor_handle);
        }

        log::info!(
            "Event Stream Processor started with {} workers",
            self.config.max_concurrent_processors
        );
        Ok(())
    }

    /// Stop the event stream processor
    pub async fn stop(&self) -> AppResult<()> {
        log::info!("Stopping Event Stream Processor...");

        // Stop analytics task
        if let Some(analytics_handle) = self.analytics_task.write().await.take() {
            analytics_handle.abort();
        }

        // Stop all processors
        let mut processors = self.processors.write().await;
        for processor in processors.drain(..) {
            processor.abort();
        }

        log::info!("Event Stream Processor stopped");
        Ok(())
    }

    /// Send an event to the stream
    pub async fn send_event(&self, event: DbPssEvent) -> AppResult<()> {
        self.event_tx.send(event).map_err(|e| {
            crate::AppError::ConfigError(format!("Failed to send event to stream: {}", e))
        })?;
        Ok(())
    }

    /// Record a live UDP event for downstream metrics without enqueueing database work.
    pub async fn record_pss_event(
        &self,
        _event: &UdpPssEvent,
        _event_json: &serde_json::Value,
    ) -> AppResult<()> {
        let now = Instant::now();

        {
            let mut metrics = self.raw_metrics.write().await;
            if let Some(last_tick) = metrics.last_tick {
                metrics.events_since_tick += 1;
                let elapsed = now.duration_since(last_tick);
                if elapsed >= std::time::Duration::from_secs(1) {
                    let elapsed_secs = elapsed.as_secs_f64().max(f64::EPSILON);
                    let events = metrics.events_since_tick as f64;
                    metrics.events_since_tick = 0;
                    metrics.last_tick = Some(now);

                    let mut stats = self.statistics.write().await;
                    stats.events_per_second = events / elapsed_secs;
                    stats.active_processors = self.config.max_concurrent_processors;
                    stats.last_updated = std::time::SystemTime::now();
                }
            } else {
                metrics.last_tick = Some(now);
                metrics.events_since_tick = 1;
            }
        }

        {
            let mut stats = self.statistics.write().await;
            stats.total_events_processed = stats.total_events_processed.saturating_add(1);
            stats.last_updated = std::time::SystemTime::now();
        }

        Ok(())
    }

    /// Subscribe to event stream
    pub fn subscribe(&self) -> broadcast::Receiver<DbPssEvent> {
        self.broadcast_tx.subscribe()
    }

    /// Get stream statistics
    pub async fn get_statistics(&self) -> StreamStatistics {
        self.statistics.read().await.clone()
    }

    /// Main event processing loop
    async fn event_processing_loop(
        mut event_rx: mpsc::UnboundedReceiver<DbPssEvent>,
        broadcast_tx: broadcast::Sender<DbPssEvent>,
        cache: Arc<EventCache>,
        config: EventStreamConfig,
        statistics: Arc<RwLock<StreamStatistics>>,
    ) {
        let mut interval = interval(Duration::from_millis(config.processing_interval_ms));
        let mut event_buffer = Vec::new();
        let start_time = std::time::Instant::now();
        let mut events_processed = 0u64;

        loop {
            tokio::select! {
                // Process incoming events
                event = event_rx.recv() => {
                    match event {
                        Some(event) => {
                            event_buffer.push(event);

                            // Process buffer if it's full
                            if event_buffer.len() >= config.buffer_size {
                                Self::process_event_batch(&event_buffer, &broadcast_tx, &cache).await;
                                events_processed += event_buffer.len() as u64;
                                event_buffer.clear();
                            }
                        }
                        None => {
                            log::info!("Event stream closed");
                            break;
                        }
                    }
                }

                // Periodic processing
                _ = interval.tick() => {
                    if !event_buffer.is_empty() {
                        Self::process_event_batch(&event_buffer, &broadcast_tx, &cache).await;
                        events_processed += event_buffer.len() as u64;
                        event_buffer.clear();
                    }

                    // Update statistics
                    let elapsed = start_time.elapsed();
                    if elapsed.as_secs() > 0 {
                        let mut stats = statistics.write().await;
                        stats.events_per_second = events_processed as f64 / elapsed.as_secs() as f64;
                        stats.total_events_processed = events_processed;
                        stats.last_updated = std::time::SystemTime::now();
                    }
                }
            }
        }
    }

    /// Process a batch of events
    async fn process_event_batch(
        events: &[DbPssEvent],
        broadcast_tx: &broadcast::Sender<DbPssEvent>,
        cache: &Arc<EventCache>,
    ) {
        for event in events {
            // Broadcast event to all subscribers
            if let Err(e) = broadcast_tx.send(event.clone()) {
                log::warn!("Failed to broadcast event: {}", e);
            }

            // Update cache based on event type
            if let Err(e) = Self::update_cache_for_event(cache, event).await {
                log::warn!("Failed to update cache for event: {}", e);
            }
        }
    }

    /// Event processor worker
    async fn event_processor_worker(
        worker_id: usize,
        mut broadcast_rx: broadcast::Receiver<DbPssEvent>,
        cache: Arc<EventCache>,
        database: Arc<DatabasePlugin>,
        match_cache: Arc<RwLock<HashMap<i64, MatchAthleteCacheEntry>>>,
        statistics: Arc<RwLock<StreamStatistics>>,
    ) {
        log::info!("Event processor worker {} started", worker_id);

        let mut processing_times = Vec::new();

        while let Ok(event) = broadcast_rx.recv().await {
            let start_time = std::time::Instant::now();

            // Process the event
            if let Err(e) =
                Self::process_single_event(&cache, &database, &match_cache, &event).await
            {
                log::error!("Worker {} failed to process event: {}", worker_id, e);
            }

            let processing_time = start_time.elapsed();
            processing_times.push(processing_time.as_millis() as f64);

            // Keep only last 100 processing times for average calculation
            if processing_times.len() > 100 {
                processing_times.remove(0);
            }

            // Update statistics
            let mut stats = statistics.write().await;
            stats.average_processing_time_ms =
                processing_times.iter().sum::<f64>() / processing_times.len() as f64;
        }

        log::info!("Event processor worker {} stopped", worker_id);
    }

    /// Analytics update loop
    async fn analytics_update_loop(cache: Arc<EventCache>, interval_duration: Duration) {
        let mut interval_timer = interval(interval_duration);

        loop {
            interval_timer.tick().await;

            // Update real-time analytics
            if let Err(e) = Self::update_real_time_analytics(&cache).await {
                log::warn!("Failed to update analytics: {}", e);
            }
        }
    }

    /// Update cache for a specific event
    async fn update_cache_for_event(cache: &Arc<EventCache>, event: &DbPssEvent) -> AppResult<()> {
        if let Some(match_id) = event.match_id {
            cache.invalidate_match(match_id).await?;
        }

        Ok(())
    }

    fn parse_udp_event(event: &DbPssEvent) -> Option<UdpPssEvent> {
        event
            .parsed_data
            .as_ref()
            .and_then(|payload| serde_json::from_str::<UdpPssEvent>(payload).ok())
    }

    async fn resolve_match_athletes(
        database: &Arc<DatabasePlugin>,
        cache: &Arc<RwLock<HashMap<i64, MatchAthleteCacheEntry>>>,
        match_id: i64,
    ) -> AppResult<Option<MatchAthleteCacheEntry>> {
        if let Some(entry) = cache.read().await.get(&match_id) {
            if !entry.is_stale() {
                return Ok(Some(entry.clone()));
            }
        }

        let lookup = database.get_pss_match_athletes(match_id).await?;
        if lookup.is_empty() {
            let placeholder = MatchAthleteCacheEntry {
                athlete1_id: None,
                athlete2_id: None,
                fetched_at: SystemTime::now(),
            };
            cache.write().await.insert(match_id, placeholder);
            return Ok(None);
        }

        let mut record = MatchAthleteCacheEntry {
            athlete1_id: None,
            athlete2_id: None,
            fetched_at: SystemTime::now(),
        };

        for (match_athlete, _athlete) in lookup {
            match match_athlete.athlete_position {
                1 => record.athlete1_id = Some(match_athlete.athlete_id),
                2 => record.athlete2_id = Some(match_athlete.athlete_id),
                _ => {}
            }
        }

        let mut cache_write = cache.write().await;
        cache_write.insert(match_id, record.clone());
        Ok(Some(record))
    }

    fn collect_athlete_targets(
        participants: &MatchAthleteCacheEntry,
        event: &UdpPssEvent,
    ) -> Vec<i64> {
        let mut targets = Vec::new();
        let mut push_side = |side: u8| match side {
            1 => {
                if let Some(id) = participants.athlete1_id {
                    targets.push(id);
                }
            }
            2 => {
                if let Some(id) = participants.athlete2_id {
                    targets.push(id);
                }
            }
            _ => {}
        };

        match event {
            UdpPssEvent::Points { athlete, .. }
            | UdpPssEvent::HitLevel { athlete, .. }
            | UdpPssEvent::Injury { athlete, .. } => push_side(*athlete),
            UdpPssEvent::Warnings { .. } | UdpPssEvent::WinnerRounds { .. } => {
                if let Some(id) = participants.athlete1_id {
                    targets.push(id);
                }
                if let Some(id) = participants.athlete2_id {
                    targets.push(id);
                }
            }
            _ => {}
        }

        targets
    }

    fn points_for_type(point_type: u8) -> u64 {
        match point_type {
            1 => 1,
            2 => 2,
            3 => 3,
            4 => 2,
            5 => 3,
            _ => 1,
        }
    }

    /// Process a single event
    async fn process_single_event(
        cache: &Arc<EventCache>,
        database: &Arc<DatabasePlugin>,
        match_cache: &Arc<RwLock<HashMap<i64, MatchAthleteCacheEntry>>>,
        event: &DbPssEvent,
    ) -> AppResult<()> {
        if let Some(match_id) = event.match_id {
            Self::update_match_statistics(cache, match_id, event).await?;

            if let Some(parsed_event) = Self::parse_udp_event(event) {
                if let Some(participants) =
                    Self::resolve_match_athletes(database, match_cache, match_id).await?
                {
                    let athlete_ids = Self::collect_athlete_targets(&participants, &parsed_event);
                    for athlete_id in athlete_ids {
                        Self::update_athlete_statistics(cache, athlete_id, event, &parsed_event)
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Update athlete statistics
    async fn update_athlete_statistics(
        cache: &Arc<EventCache>,
        athlete_id: i64,
        _event: &DbPssEvent,
        udp_event: &UdpPssEvent,
    ) -> AppResult<()> {
        // Get current stats or create new ones
        let mut stats = cache
            .get_athlete_stats(athlete_id)
            .await
            .unwrap_or_else(|| AthleteStatistics {
                athlete_id,
                total_events: 0,
                total_points: 0,
                total_warnings: 0,
                total_injuries: 0,
                avg_hit_level: 0.0,
                last_updated: SystemTime::now(),
            });

        stats.total_events = stats.total_events.saturating_add(1);
        stats.last_updated = SystemTime::now();

        match udp_event {
            UdpPssEvent::Points { point_type, .. } => {
                stats.total_points = stats
                    .total_points
                    .saturating_add(Self::points_for_type(*point_type));
            }
            UdpPssEvent::Warnings { .. } => {
                stats.total_warnings = stats.total_warnings.saturating_add(1);
            }
            UdpPssEvent::Injury { .. } => {
                stats.total_injuries = stats.total_injuries.saturating_add(1);
            }
            UdpPssEvent::HitLevel { level, .. } => {
                if stats.avg_hit_level <= 0.0 {
                    stats.avg_hit_level = *level as f64;
                } else {
                    // Lightweight smoothing to avoid needing a separate hit count tally
                    stats.avg_hit_level = (stats.avg_hit_level * 0.8) + (*level as f64 * 0.2);
                }
            }
            _ => {}
        }

        // Update cache
        cache.set_athlete_stats(athlete_id, stats).await?;
        Ok(())
    }

    /// Update match statistics
    async fn update_match_statistics(
        cache: &Arc<EventCache>,
        match_id: i64,
        _event: &DbPssEvent,
    ) -> AppResult<()> {
        // Get current stats or create new ones
        let mut stats = cache
            .get_match_stats(match_id)
            .await
            .unwrap_or_else(|| MatchStatistics {
                match_id,
                event_count: 0,
                duration_seconds: 0,
                athlete1_score: 0,
                athlete2_score: 0,
                last_updated: std::time::SystemTime::now(),
            });

        stats.event_count += 1;
        stats.last_updated = std::time::SystemTime::now();

        // Update cache
        cache.set_match_stats(match_id, stats).await?;
        Ok(())
    }

    /// Update real-time analytics
    async fn update_real_time_analytics(_cache: &Arc<EventCache>) -> AppResult<()> {
        // This would implement comprehensive real-time analytics
        // For now, we'll just log that analytics are being updated
        log::debug!("Updating real-time analytics...");
        Ok(())
    }
}

/// Event stream subscriber for consuming events
pub struct EventStreamSubscriber {
    rx: broadcast::Receiver<DbPssEvent>,
}

impl EventStreamSubscriber {
    pub fn new(rx: broadcast::Receiver<DbPssEvent>) -> Self {
        Self { rx }
    }

    /// Receive the next event
    pub async fn recv(&mut self) -> Result<DbPssEvent, broadcast::error::RecvError> {
        self.rx.recv().await
    }

    /// Try to receive an event without blocking
    pub fn try_recv(&mut self) -> Result<DbPssEvent, broadcast::error::TryRecvError> {
        self.rx.try_recv()
    }
}
