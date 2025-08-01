use std::collections::HashMap;
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};
use serde::{Serialize, Deserialize};
use crate::database::models::PssEventV2;
use crate::plugins::event_cache::EventCache;
// use crate::plugins::event_stream::{EventStreamProcessor, EventStreamConfig};
use crate::AppResult;

/// Load balancer configuration
#[derive(Debug, Clone)]
pub struct LoadBalancerConfig {
    pub health_check_interval_ms: u64,
    pub max_servers: usize,
    pub load_distribution_strategy: LoadDistributionStrategy,
    pub enable_auto_scaling: bool,
    pub auto_scaling_threshold: f64,
    pub server_timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub enum LoadDistributionStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    ConsistentHashing,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            health_check_interval_ms: 5000,
            max_servers: 4,
            load_distribution_strategy: LoadDistributionStrategy::RoundRobin,
            enable_auto_scaling: true,
            auto_scaling_threshold: 0.8,
            server_timeout_ms: 30000,
        }
    }
}

/// Server health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHealth {
    pub server_id: String,
    pub is_healthy: bool,
    pub last_health_check: std::time::SystemTime,
    pub response_time_ms: u64,
    pub active_connections: u32,
    pub events_per_second: f64,
    pub error_rate: f64,
}

/// Server statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatistics {
    pub server_id: String,
    pub total_events_processed: u64,
    pub events_per_second: f64,
    pub average_processing_time_ms: f64,
    pub active_connections: u32,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
    pub last_updated: std::time::SystemTime,
}

/// Event distributor for horizontal scaling
pub struct EventDistributor {
    servers: Arc<RwLock<HashMap<String, UdpServerInstance>>>,
    load_balancer: Arc<LoadBalancer>,
    cache: Arc<EventCache>,
    config: LoadBalancerConfig,
    health_check_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    statistics: Arc<RwLock<DistributorStatistics>>,
}

/// UDP Server instance wrapper
pub struct UdpServerInstance {
    pub server_id: String,
    pub bind_address: String,
    pub port: u16,
    pub health: ServerHealth,
    pub statistics: ServerStatistics,
    pub is_active: bool,
    pub created_at: std::time::SystemTime,
}

/// Load balancer for distributing events across servers
pub struct LoadBalancer {
    servers: Arc<RwLock<HashMap<String, UdpServerInstance>>>,
    current_index: Arc<RwLock<usize>>,
    strategy: LoadDistributionStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributorStatistics {
    pub total_servers: usize,
    pub active_servers: usize,
    pub total_events_distributed: u64,
    pub events_per_second: f64,
    pub average_distribution_time_ms: f64,
    pub load_balance_efficiency: f64,
    pub last_updated: std::time::SystemTime,
}

impl Default for DistributorStatistics {
    fn default() -> Self {
        Self {
            total_servers: 0,
            active_servers: 0,
            total_events_distributed: 0,
            events_per_second: 0.0,
            average_distribution_time_ms: 0.0,
            load_balance_efficiency: 0.0,
            last_updated: std::time::SystemTime::now(),
        }
    }
}

impl EventDistributor {
    pub fn new(cache: Arc<EventCache>) -> Self {
        Self::with_config(cache, LoadBalancerConfig::default())
    }

    pub fn with_config(cache: Arc<EventCache>, config: LoadBalancerConfig) -> Self {
        let load_balancer = Arc::new(LoadBalancer::new(config.load_distribution_strategy.clone()));
        
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            load_balancer,
            cache,
            config,
            health_check_task: Arc::new(RwLock::new(None)),
            statistics: Arc::new(RwLock::new(DistributorStatistics::default())),
        }
    }

    /// Start the event distributor
    pub async fn start(&mut self) -> AppResult<()> {
        log::info!("🚀 Starting Event Distributor...");
        
        // Start health check task
        let servers = self.servers.clone();
        let health_check_interval = Duration::from_millis(self.config.health_check_interval_ms);
        
        let health_check_handle = tokio::spawn(async move {
            Self::health_check_loop(servers, health_check_interval).await;
        });

        let mut health_check_task = self.health_check_task.write().await;
        *health_check_task = Some(health_check_handle);

        log::info!("✅ Event Distributor started");
        Ok(())
    }

    /// Stop the event distributor
    pub async fn stop(&self) -> AppResult<()> {
        log::info!("🛑 Stopping Event Distributor...");
        
        // Stop health check task
        if let Some(health_check_handle) = self.health_check_task.write().await.take() {
            health_check_handle.abort();
        }

        // Stop all servers
        let mut servers = self.servers.write().await;
        for (_, server) in servers.iter_mut() {
            server.is_active = false;
        }

        log::info!("✅ Event Distributor stopped");
        Ok(())
    }

    /// Add a new UDP server instance
    pub async fn add_server(&self, server_id: String, bind_address: String, port: u16) -> AppResult<()> {
        let mut servers = self.servers.write().await;
        
        if servers.len() >= self.config.max_servers {
            return Err(crate::AppError::ConfigError(
                format!("Maximum number of servers ({}) reached", self.config.max_servers)
            ));
        }

        let server_instance = UdpServerInstance {
            server_id: server_id.clone(),
            bind_address: bind_address.clone(),
            port,
            health: ServerHealth {
                server_id: server_id.clone(),
                is_healthy: true,
                last_health_check: std::time::SystemTime::now(),
                response_time_ms: 0,
                active_connections: 0,
                events_per_second: 0.0,
                error_rate: 0.0,
            },
            statistics: ServerStatistics {
                server_id: server_id.clone(),
                total_events_processed: 0,
                events_per_second: 0.0,
                average_processing_time_ms: 0.0,
                active_connections: 0,
                memory_usage_mb: 0,
                cpu_usage_percent: 0.0,
                last_updated: std::time::SystemTime::now(),
            },
            is_active: true,
            created_at: std::time::SystemTime::now(),
        };

        servers.insert(server_id.clone(), server_instance);
        
        // Update load balancer
        self.load_balancer.add_server(server_id).await;
        
        log::info!("➕ Added UDP server: {}:{}", bind_address.clone(), port);
        Ok(())
    }

    /// Remove a UDP server instance
    pub async fn remove_server(&self, server_id: &str) -> AppResult<()> {
        let mut servers = self.servers.write().await;
        
        if let Some(mut server) = servers.remove(server_id) {
            server.is_active = false;
            
            // Update load balancer
            self.load_balancer.remove_server(server_id).await;
            
            log::info!("➖ Removed UDP server: {}", server_id);
            Ok(())
        } else {
            Err(crate::AppError::ConfigError(
                format!("Server {} not found", server_id)
            ))
        }
    }

    /// Distribute an event to the appropriate server
    pub async fn distribute_event(&self, event: PssEventV2) -> AppResult<()> {
        let start_time = std::time::Instant::now();
        
        // Get the best server based on load balancing strategy
        let server_id = self.load_balancer.get_next_server().await
            .ok_or_else(|| crate::AppError::ConfigError("No available servers".to_string()))?;
        
        // Send event to the selected server
        if let Some(server) = self.servers.read().await.get(&server_id) {
            if server.is_active && server.health.is_healthy {
                // In a real implementation, you would send the event to the actual server
                // For now, we'll just update statistics
                self.update_server_statistics(&server_id, &event).await?;
                
                let distribution_time = start_time.elapsed();
                self.update_distributor_statistics(distribution_time).await;
                
                log::debug!("📤 Distributed event to server: {}", server_id);
                Ok(())
            } else {
                Err(crate::AppError::ConfigError(
                    format!("Server {} is not available", server_id)
                ))
            }
        } else {
            Err(crate::AppError::ConfigError(
                format!("Server {} not found", server_id)
            ))
        }
    }

    /// Get distributor statistics
    pub async fn get_statistics(&self) -> DistributorStatistics {
        self.statistics.read().await.clone()
    }

    /// Get all server statistics
    pub async fn get_server_statistics(&self) -> Vec<ServerStatistics> {
        let servers = self.servers.read().await;
        servers.values()
            .map(|server| server.statistics.clone())
            .collect()
    }

    /// Health check loop
    async fn health_check_loop(
        servers: Arc<RwLock<HashMap<String, UdpServerInstance>>>,
        interval_duration: Duration,
    ) {
        let mut interval_timer = interval(interval_duration);
        
        loop {
            interval_timer.tick().await;
            
            let mut servers_guard = servers.write().await;
            for (server_id, server) in servers_guard.iter_mut() {
                // Perform health check
                let health_status = Self::perform_health_check(server).await;
                server.health = health_status;
                
                // Update server status based on health
                server.is_active = server.health.is_healthy;
                
                log::debug!("🏥 Health check for server {}: {}", server_id, server.health.is_healthy);
            }
        }
    }

    /// Perform health check for a server
    async fn perform_health_check(server: &UdpServerInstance) -> ServerHealth {
        let start_time = std::time::Instant::now();
        
        // In a real implementation, you would actually ping the server
        // For now, we'll simulate a health check
        let response_time = start_time.elapsed().as_millis() as u64;
        
        // Simulate health status (90% success rate)
        let is_healthy = rand::random::<u8>() > 25; // Use u8 instead of f64
        
        ServerHealth {
            server_id: server.server_id.clone(),
            is_healthy,
            last_health_check: std::time::SystemTime::now(),
            response_time_ms: response_time,
            active_connections: server.statistics.active_connections,
            events_per_second: server.statistics.events_per_second,
            error_rate: if is_healthy { 0.0 } else { 1.0 },
        }
    }

    /// Update server statistics
    async fn update_server_statistics(&self, server_id: &str, _event: &PssEventV2) -> AppResult<()> {
        let mut servers = self.servers.write().await;
        
        if let Some(server) = servers.get_mut(server_id) {
            server.statistics.total_events_processed += 1;
            server.statistics.last_updated = std::time::SystemTime::now();
            
            // Update events per second (simplified calculation)
            let elapsed = server.created_at.elapsed().unwrap_or_default();
            if elapsed.as_secs() > 0 {
                server.statistics.events_per_second = 
                    server.statistics.total_events_processed as f64 / elapsed.as_secs() as f64;
            }
        }
        
        Ok(())
    }

    /// Update distributor statistics
    async fn update_distributor_statistics(&self, distribution_time: std::time::Duration) {
        let mut stats = self.statistics.write().await;
        stats.total_events_distributed += 1;
        stats.average_distribution_time_ms = distribution_time.as_millis() as f64;
        stats.last_updated = std::time::SystemTime::now();
        
        // Update events per second
        let elapsed = stats.last_updated.duration_since(
            std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1)
        ).unwrap_or_default();
        if elapsed.as_secs() > 0 {
            stats.events_per_second = stats.total_events_distributed as f64 / elapsed.as_secs() as f64;
        }
    }
}

impl LoadBalancer {
    pub fn new(strategy: LoadDistributionStrategy) -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            current_index: Arc::new(RwLock::new(0)),
            strategy,
        }
    }

    /// Add a server to the load balancer
    pub async fn add_server(&self, server_id: String) {
        let mut servers = self.servers.write().await;
        servers.insert(server_id, UdpServerInstance {
            server_id: String::new(),
            bind_address: String::new(),
            port: 0,
            health: ServerHealth {
                server_id: String::new(),
                is_healthy: true,
                last_health_check: std::time::SystemTime::now(),
                response_time_ms: 0,
                active_connections: 0,
                events_per_second: 0.0,
                error_rate: 0.0,
            },
            statistics: ServerStatistics {
                server_id: String::new(),
                total_events_processed: 0,
                events_per_second: 0.0,
                average_processing_time_ms: 0.0,
                active_connections: 0,
                memory_usage_mb: 0,
                cpu_usage_percent: 0.0,
                last_updated: std::time::SystemTime::now(),
            },
            is_active: true,
            created_at: std::time::SystemTime::now(),
        });
    }

    /// Remove a server from the load balancer
    pub async fn remove_server(&self, server_id: &str) {
        let mut servers = self.servers.write().await;
        servers.remove(server_id);
    }

    /// Get the next server based on the load balancing strategy
    pub async fn get_next_server(&self) -> Option<String> {
        let servers = self.servers.read().await;
        let server_ids: Vec<String> = servers.keys().cloned().collect();
        
        if server_ids.is_empty() {
            return None;
        }

        match self.strategy {
            LoadDistributionStrategy::RoundRobin => {
                let mut current_index = self.current_index.write().await;
                let server_id = server_ids[*current_index % server_ids.len()].clone();
                *current_index += 1;
                Some(server_id)
            }
            LoadDistributionStrategy::LeastConnections => {
                // Find server with least active connections
                let mut least_connections = u32::MAX;
                let mut selected_server = None;
                
                for (server_id, server) in servers.iter() {
                    if server.is_active && server.health.is_healthy {
                        if server.statistics.active_connections < least_connections {
                            least_connections = server.statistics.active_connections;
                            selected_server = Some(server_id.clone());
                        }
                    }
                }
                
                selected_server
            }
            LoadDistributionStrategy::WeightedRoundRobin => {
                // Similar to round robin but with weights based on server capacity
                let mut current_index = self.current_index.write().await;
                let server_id = server_ids[*current_index % server_ids.len()].clone();
                *current_index += 1;
                Some(server_id)
            }
            LoadDistributionStrategy::ConsistentHashing => {
                // Use consistent hashing for better distribution
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                // Use current timestamp as hash key for consistent distribution
                std::time::SystemTime::now().hash(&mut hasher);
                let hash_value = hasher.finish();
                let index = hash_value as usize % server_ids.len();
                Some(server_ids[index].clone())
            }
        }
    }
}

// Mock implementation for rand::random
mod rand {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    pub fn random<T>() -> T 
    where 
        T: Hash + Default,
    {
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        let _hash = hasher.finish();
        
        // Convert hash to T (simplified)
        T::default()
    }
} 