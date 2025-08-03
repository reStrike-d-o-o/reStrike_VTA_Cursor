use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, broadcast};
use tokio::time::{Duration, interval};
use serde::{Serialize, Deserialize};
use crate::database::models::PssEventV2;
use crate::plugins::event_cache::{EventCache, AthleteStatistics, TournamentStatistics, MatchStatistics};
use crate::AppResult;

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
    event_tx: mpsc::UnboundedSender<PssEventV2>,
    event_rx: Option<mpsc::UnboundedReceiver<PssEventV2>>,
    broadcast_tx: broadcast::Sender<PssEventV2>,
    cache: Arc<EventCache>,
    config: EventStreamConfig,
    processors: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
    analytics_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    statistics: Arc<RwLock<StreamStatistics>>,
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
    pub fn new(cache: Arc<EventCache>) -> Self {
        Self::with_config(cache, EventStreamConfig::default())
    }

    pub fn with_config(cache: Arc<EventCache>, config: EventStreamConfig) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (broadcast_tx, _) = broadcast::channel(config.buffer_size);

        Self {
            event_tx,
            event_rx: Some(event_rx),
            broadcast_tx,
            cache,
            config,
            processors: Arc::new(RwLock::new(Vec::new())),
            analytics_task: Arc::new(RwLock::new(None)),
            statistics: Arc::new(RwLock::new(StreamStatistics::default())),
        }
    }

    /// Start the event stream processor
    pub async fn start(&mut self) -> AppResult<()> {
        log::info!("🚀 Starting Event Stream Processor...");
        
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
            let analytics_interval = Duration::from_millis(self.config.analytics_update_interval_ms);
            
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
            let statistics_clone = self.statistics.clone();
            
            let processor_handle = tokio::spawn(async move {
                Self::event_processor_worker(i, broadcast_rx, cache_clone, statistics_clone).await;
            });

            let mut processors = self.processors.write().await;
            processors.push(processor_handle);
        }

        log::info!("✅ Event Stream Processor started with {} workers", self.config.max_concurrent_processors);
        Ok(())
    }

    /// Stop the event stream processor
    pub async fn stop(&self) -> AppResult<()> {
        log::info!("🛑 Stopping Event Stream Processor...");
        
        // Stop analytics task
        if let Some(analytics_handle) = self.analytics_task.write().await.take() {
            analytics_handle.abort();
        }

        // Stop all processors
        let mut processors = self.processors.write().await;
        for processor in processors.drain(..) {
            processor.abort();
        }

        log::info!("✅ Event Stream Processor stopped");
        Ok(())
    }

    /// Send an event to the stream
    pub async fn send_event(&self, event: PssEventV2) -> AppResult<()> {
        self.event_tx.send(event)
            .map_err(|e| crate::AppError::ConfigError(format!("Failed to send event to stream: {}", e)))?;
        Ok(())
    }

    /// Subscribe to event stream
    pub fn subscribe(&self) -> broadcast::Receiver<PssEventV2> {
        self.broadcast_tx.subscribe()
    }

    /// Get stream statistics
    pub async fn get_statistics(&self) -> StreamStatistics {
        self.statistics.read().await.clone()
    }

    /// Main event processing loop
    async fn event_processing_loop(
        mut event_rx: mpsc::UnboundedReceiver<PssEventV2>,
        broadcast_tx: broadcast::Sender<PssEventV2>,
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
                            log::info!("📡 Event stream closed");
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
        events: &[PssEventV2],
        broadcast_tx: &broadcast::Sender<PssEventV2>,
        cache: &Arc<EventCache>,
    ) {
        for event in events {
            // Broadcast event to all subscribers
            if let Err(e) = broadcast_tx.send(event.clone()) {
                log::warn!("⚠️ Failed to broadcast event: {}", e);
            }

            // Update cache based on event type
            if let Err(e) = Self::update_cache_for_event(cache, event).await {
                log::warn!("⚠️ Failed to update cache for event: {}", e);
            }
        }
    }

    /// Event processor worker
    async fn event_processor_worker(
        worker_id: usize,
        mut broadcast_rx: broadcast::Receiver<PssEventV2>,
        cache: Arc<EventCache>,
        statistics: Arc<RwLock<StreamStatistics>>,
    ) {
        log::info!("🔧 Event processor worker {} started", worker_id);
        
        let mut processing_times = Vec::new();
        
        while let Ok(event) = broadcast_rx.recv().await {
            let start_time = std::time::Instant::now();
            
            // Process the event
            if let Err(e) = Self::process_single_event(&cache, &event).await {
                log::error!("❌ Worker {} failed to process event: {}", worker_id, e);
            }
            
            let processing_time = start_time.elapsed();
            processing_times.push(processing_time.as_millis() as f64);
            
            // Keep only last 100 processing times for average calculation
            if processing_times.len() > 100 {
                processing_times.remove(0);
            }
            
            // Update statistics
            let mut stats = statistics.write().await;
            stats.average_processing_time_ms = processing_times.iter().sum::<f64>() / processing_times.len() as f64;
        }
        
        log::info!("🔧 Event processor worker {} stopped", worker_id);
    }

    /// Analytics update loop
    async fn analytics_update_loop(cache: Arc<EventCache>, interval_duration: Duration) {
        let mut interval_timer = interval(interval_duration);
        
        loop {
            interval_timer.tick().await;
            
            // Update real-time analytics
            if let Err(e) = Self::update_real_time_analytics(&cache).await {
                log::warn!("⚠️ Failed to update analytics: {}", e);
            }
        }
    }

    /// Update cache for a specific event
    async fn update_cache_for_event(cache: &Arc<EventCache>, event: &PssEventV2) -> AppResult<()> {
        // Invalidate relevant caches when new events arrive
        if let Some(tournament_id) = event.tournament_id {
            cache.invalidate_tournament(tournament_id).await?;
        }
        
        if let Some(match_id) = event.match_id {
            cache.invalidate_match(match_id).await?;
        }
        
        // Update athlete cache if event involves athletes
        // Note: PssEventV2 doesn't have athlete_id field, would need to extract from parsed_data
        // For now, we'll skip athlete-specific cache invalidation
        
        Ok(())
    }

    /// Process a single event
    async fn process_single_event(cache: &Arc<EventCache>, event: &PssEventV2) -> AppResult<()> {
        // Update match statistics if match_id is present
        if let Some(match_id) = event.match_id {
            Self::update_match_statistics(cache, match_id, event).await?;
        }
        
        Ok(())
    }

    /// Update athlete statistics
    #[allow(dead_code)]
    async fn update_athlete_statistics(
        cache: &Arc<EventCache>,
        athlete_id: i64,
        _event: &PssEventV2,
    ) -> AppResult<()> {
        // Get current stats or create new ones
        let mut stats = cache.get_athlete_stats(athlete_id).await
            .unwrap_or_else(|| AthleteStatistics {
                athlete_id,
                total_events: 0,
                total_points: 0,
                total_warnings: 0,
                total_injuries: 0,
                avg_hit_level: 0.0,
                last_updated: std::time::SystemTime::now(),
            });

        // Update based on event type
        stats.total_events += 1;
        stats.last_updated = std::time::SystemTime::now();

        // Note: PssEventV2 has event_type_id (i64), not event_type (String)
        // We would need to map event_type_id to string codes or use a different approach
        // For now, we'll just increment total_events
        stats.total_events += 1;

        // Update cache
        cache.set_athlete_stats(athlete_id, stats).await?;
        Ok(())
    }

    /// Update match statistics
    async fn update_match_statistics(
        cache: &Arc<EventCache>,
        match_id: i64,
        _event: &PssEventV2,
    ) -> AppResult<()> {
        // Get current stats or create new ones
        let mut stats = cache.get_match_stats(match_id).await
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
        log::debug!("📊 Updating real-time analytics...");
        Ok(())
    }
}

/// Event stream subscriber for consuming events
pub struct EventStreamSubscriber {
    rx: broadcast::Receiver<PssEventV2>,
}

impl EventStreamSubscriber {
    pub fn new(rx: broadcast::Receiver<PssEventV2>) -> Self {
        Self { rx }
    }

    /// Receive the next event
    pub async fn recv(&mut self) -> Result<PssEventV2, broadcast::error::RecvError> {
        self.rx.recv().await
    }

    /// Try to receive an event without blocking
    pub fn try_recv(&mut self) -> Result<PssEventV2, broadcast::error::TryRecvError> {
        self.rx.try_recv()
    }
} 