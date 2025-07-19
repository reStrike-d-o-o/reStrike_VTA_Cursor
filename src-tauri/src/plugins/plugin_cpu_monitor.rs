use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use crate::types::{AppError, AppResult};
use std::process::Command;
use std::time::Duration;

/// Initialize the CPU monitoring plugin
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("🔧 Initializing CPU monitoring plugin...");
    Ok(())
}

// CPU usage data for a specific process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuProcessData {
    pub process_name: String,
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

// System CPU data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCpuData {
    pub total_cpu_percent: f64,
    pub cores: Vec<f64>,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

// CPU monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMonitorConfig {
    pub enabled: bool,
    pub update_interval_seconds: u64,
    pub monitored_processes: Vec<String>, // Process names to monitor (e.g., ["obs64.exe", "obs-studio"])
    pub include_system_cpu: bool,
}

// CPU monitoring plugin
pub struct CpuMonitorPlugin {
    config: Arc<Mutex<CpuMonitorConfig>>,
    process_data: Arc<Mutex<HashMap<String, CpuProcessData>>>,
    system_data: Arc<Mutex<Option<SystemCpuData>>>,
    monitoring_active: Arc<Mutex<bool>>,
}

impl Clone for CpuMonitorPlugin {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            process_data: self.process_data.clone(),
            system_data: self.system_data.clone(),
            monitoring_active: self.monitoring_active.clone(),
        }
    }
}

impl CpuMonitorPlugin {
    pub fn new(config: CpuMonitorConfig) -> Self {
        let enabled = config.enabled;
        let plugin = Self {
            config: Arc::new(Mutex::new(config)),
            process_data: Arc::new(Mutex::new(HashMap::new())),
            system_data: Arc::new(Mutex::new(None)),
            monitoring_active: Arc::new(Mutex::new(false)),
        };
        
        // Start monitoring if enabled
        if enabled {
            plugin.start_monitoring();
        }
        
        plugin
    }

    // Start CPU monitoring background task
    fn start_monitoring(&self) {
        let plugin = self.clone();
        
        tokio::spawn(async move {
            let mut active = plugin.monitoring_active.lock().await;
            if *active {
                log::warn!("[CPU_MONITOR] Monitoring already active");
                return;
            }
            *active = true;
            drop(active);

            log::info!("[CPU_MONITOR] Starting CPU monitoring...");
            
            loop {
                // Check if monitoring should continue
                {
                    let config = plugin.config.lock().await;
                    if !config.enabled {
                        break;
                    }
                    
                    // Update CPU data
                    if let Err(e) = plugin.update_cpu_data().await {
                        log::warn!("[CPU_MONITOR] Error updating CPU data: {}", e);
                    }
                    
                    // Sleep for the configured interval
                    tokio::time::sleep(Duration::from_secs(config.update_interval_seconds)).await;
                }
            }
            
            log::info!("[CPU_MONITOR] CPU monitoring stopped");
        });
    }

    // Update CPU data for all processes and system
    async fn update_cpu_data(&self) -> AppResult<()> {
        let config = self.config.lock().await;
        
        // Update system CPU if enabled
        if config.include_system_cpu {
            if let Err(e) = self.update_system_cpu().await {
                log::warn!("[CPU_MONITOR] Failed to update system CPU: {}", e);
            }
        }
        
        // Update all running processes
        if let Err(e) = self.update_all_processes().await {
            log::error!("[CPU_MONITOR] Failed to update all processes: {}", e);
        }
        
        Ok(())
    }

    // Update system CPU usage
    async fn update_system_cpu(&self) -> AppResult<()> {
        #[cfg(target_os = "windows")]
        {
            // Windows: Use wmic to get system CPU usage
            let output = Command::new("wmic")
                .args(&["cpu", "get", "loadpercentage", "/value"])
                .output()
                .map_err(|e| AppError::ConfigError(format!("Failed to get system CPU: {}", e)))?;
            
            let system_cpu = String::from_utf8_lossy(&output.stdout);
            log::debug!("[CPU_MONITOR] WMIC output: {}", system_cpu);
            
            let total_cpu_percent: f64 = system_cpu
                .lines()
                .find(|line| line.starts_with("LoadPercentage="))
                .and_then(|line| line.split('=').nth(1))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);

            // For now, use single core data (can be extended to multi-core)
            let cores = vec![total_cpu_percent];
            
            let system_data = SystemCpuData {
                total_cpu_percent,
                cores,
                last_update: chrono::Utc::now(),
            };
            
            let mut data = self.system_data.lock().await;
            *data = Some(system_data);
            
            log::info!("[CPU_MONITOR] System CPU: {:.1}%", total_cpu_percent);
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Linux/macOS: Use top command or /proc/stat
            let output = Command::new("top")
                .args(&["-l", "1", "-n", "0"])
                .output()
                .map_err(|e| AppError::ConfigError(format!("Failed to get system CPU: {}", e)))?;
            
            // Parse top output for CPU usage (simplified)
            let output_str = String::from_utf8_lossy(&output.stdout);
            let cpu_percent = 0.0; // TODO: Parse top output properly
            
            let system_data = SystemCpuData {
                total_cpu_percent: cpu_percent,
                cores: vec![cpu_percent],
                last_update: chrono::Utc::now(),
            };
            
            let mut data = self.system_data.lock().await;
            *data = Some(system_data);
        }

        Ok(())
    }

    // Update all running processes
    async fn update_all_processes(&self) -> AppResult<()> {
        #[cfg(target_os = "windows")]
        {
            // Windows: Use wmic to get all processes with CPU and memory usage
            let output = Command::new("wmic")
                .args(&["process", "get", "name,processid,workingsetsize,percentprocessortime", "/format:csv"])
                .output()
                .map_err(|e| AppError::ConfigError(format!("Failed to get all processes: {}", e)))?;
            
            let process_info = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = process_info.lines().collect();
            
            let mut new_process_data = HashMap::new();
            
            for line in &lines[1..] { // Skip header
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 4 {
                    let process_name = parts[0].trim().to_string();
                    if let (Ok(_pid), Ok(memory_bytes), Ok(cpu_percent)) = (
                        parts[1].trim().parse::<u32>(),
                        parts[2].trim().parse::<u64>(),
                        parts[3].trim().parse::<f64>()
                    ) {
                        // Only include processes with significant CPU usage (> 0.1%) or memory (> 10MB)
                        if cpu_percent > 0.1 || memory_bytes > 10 * 1024 * 1024 {
                            let memory_mb = memory_bytes as f64 / (1024.0 * 1024.0);
                            
                            let process_data = CpuProcessData {
                                process_name,
                                cpu_percent,
                                memory_mb,
                                last_update: chrono::Utc::now(),
                            };
                            
                            new_process_data.insert(process_data.process_name.clone(), process_data);
                        }
                    }
                }
            }
            
            // Update the process data
            let mut data = self.process_data.lock().await;
            *data = new_process_data;
            
            log::info!("[CPU_MONITOR] Updated {} processes", data.len());
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Linux/macOS: Use ps command to get all processes
            let output = Command::new("ps")
                .args(&["-eo", "comm,%cpu,%mem,rss"])
                .output()
                .map_err(|e| AppError::ConfigError(format!("Failed to get all processes: {}", e)))?;
            
            let ps_output = String::from_utf8_lossy(&output.stdout);
            let mut new_process_data = HashMap::new();
            
            for line in ps_output.lines().skip(1) { // Skip header
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let (Ok(cpu_percent), Ok(memory_percent), Ok(rss_kb)) = (
                        parts[1].parse::<f64>(),
                        parts[2].parse::<f64>(),
                        parts[3].parse::<u64>()
                    ) {
                        // Only include processes with significant CPU usage (> 0.1%) or memory (> 10MB)
                        if cpu_percent > 0.1 || rss_kb > 10 * 1024 {
                            let process_name = parts[0].to_string();
                            let memory_mb = rss_kb as f64 / 1024.0; // Convert KB to MB
                            
                            let process_data = CpuProcessData {
                                process_name,
                                cpu_percent,
                                memory_mb,
                                last_update: chrono::Utc::now(),
                            };
                            
                            new_process_data.insert(process_data.process_name.clone(), process_data);
                        }
                    }
                }
            }
            
            // Update the process data
            let mut data = self.process_data.lock().await;
            *data = new_process_data;
            
            log::debug!("[CPU_MONITOR] Updated {} processes", data.len());
        }

        Ok(())
    }

    // Update CPU usage for a specific process (kept for backward compatibility)
    async fn update_process_cpu(&self, process_name: &str) -> AppResult<()> {
        #[cfg(target_os = "windows")]
        {
            // Windows: Use wmic to get process CPU usage
            let output = Command::new("wmic")
                .args(&["process", "where", &format!("name='{}'", process_name), "get", "processid,workingsetsize", "/format:csv"])
                .output()
                .map_err(|e| AppError::ConfigError(format!("Failed to get process info for '{}': {}", process_name, e)))?;
            
            let process_info = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = process_info.lines().collect();
            
            if lines.len() > 1 {
                // Parse CSV format: Node,ProcessId,WorkingSetSize
                let parts: Vec<&str> = lines[1].split(',').collect();
                if parts.len() >= 3 {
                    if let Ok(pid) = parts[1].trim().parse::<u32>() {
                        // Get CPU usage for specific process
                        let cpu_output = Command::new("wmic")
                            .args(&["process", "where", &format!("processid={}", pid), "get", "percentprocessortime", "/value"])
                            .output()
                            .map_err(|e| AppError::ConfigError(format!("Failed to get CPU for process '{}': {}", process_name, e)))?;
                        
                        let cpu_info = String::from_utf8_lossy(&cpu_output.stdout);
                        let cpu_percent: f64 = cpu_info
                            .lines()
                            .find(|line| line.starts_with("PercentProcessorTime="))
                            .and_then(|line| line.split('=').nth(1))
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0.0);

                        // Get memory usage
                        let memory_mb: f64 = parts[2].trim().parse().unwrap_or(0.0) / (1024.0 * 1024.0);

                        let process_data = CpuProcessData {
                            process_name: process_name.to_string(),
                            cpu_percent,
                            memory_mb,
                            last_update: chrono::Utc::now(),
                        };
                        
                        let mut data = self.process_data.lock().await;
                        data.insert(process_name.to_string(), process_data);
                        
                        log::debug!("[CPU_MONITOR] Process '{}': CPU {:.1}%, Memory {:.1}MB", process_name, cpu_percent, memory_mb);
                    }
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Linux/macOS: Use ps command
            let output = Command::new("ps")
                .args(&["-eo", "comm,%cpu,%mem"])
                .output()
                .map_err(|e| AppError::ConfigError(format!("Failed to get process CPU: {}", e)))?;
            
            let ps_output = String::from_utf8_lossy(&output.stdout);
            
            for line in ps_output.lines() {
                if line.contains(process_name) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        if let (Ok(cpu_percent), Ok(memory_percent)) = (parts[1].parse::<f64>(), parts[2].parse::<f64>()) {
                            let process_data = CpuProcessData {
                                process_name: process_name.to_string(),
                                cpu_percent,
                                memory_mb: memory_percent, // Simplified for now
                                last_update: chrono::Utc::now(),
                            };
                            
                            let mut data = self.process_data.lock().await;
                            data.insert(process_name.to_string(), process_data);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // Get current CPU data for all monitored processes
    pub async fn get_process_cpu_data(&self) -> Vec<CpuProcessData> {
        let data = self.process_data.lock().await;
        data.values().cloned().collect()
    }

    // Get current system CPU data
    pub async fn get_system_cpu_data(&self) -> Option<SystemCpuData> {
        let data = self.system_data.lock().await;
        data.clone()
    }

    // Get CPU data for a specific process
    pub async fn get_process_cpu(&self, process_name: &str) -> Option<CpuProcessData> {
        let data = self.process_data.lock().await;
        data.get(process_name).cloned()
    }

    // Get total CPU usage for OBS processes (for backward compatibility)
    pub async fn get_obs_cpu_usage(&self) -> f64 {
        let data = self.process_data.lock().await;
        let mut max_cpu: f64 = 0.0;
        
        for (process_name, process_data) in data.iter() {
            if process_name.contains("obs") || process_name.contains("obs64.exe") {
                // Check if data is recent (within last 10 seconds)
                let age = chrono::Utc::now() - process_data.last_update;
                if age < chrono::Duration::seconds(10) {
                    max_cpu = max_cpu.max(process_data.cpu_percent);
                }
            }
        }
        
        max_cpu
    }

    // Update configuration
    pub async fn update_config(&self, new_config: CpuMonitorConfig) -> AppResult<()> {
        let mut config = self.config.lock().await;
        let was_enabled = config.enabled;
        *config = new_config;
        
        // Start/stop monitoring based on new config
        if config.enabled && !was_enabled {
            drop(config);
            self.start_monitoring();
        } else if !config.enabled && was_enabled {
            let mut active = self.monitoring_active.lock().await;
            *active = false;
        }
        
        Ok(())
    }

    // Stop monitoring
    pub async fn stop_monitoring(&self) {
        let mut active = self.monitoring_active.lock().await;
        *active = false;
        log::info!("[CPU_MONITOR] Monitoring stopped");
    }
}

// Default configuration
impl Default for CpuMonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            update_interval_seconds: 2,
            monitored_processes: vec![], // Empty since we monitor all processes now
            include_system_cpu: true,
        }
    }
} 