use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use crate::types::{AppError, AppResult};
use std::process::Command;
use std::time::Duration;
use log;

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
                // Check if monitoring should continue without holding the lock
                let should_continue = {
                    let config = plugin.config.lock().await;
                    config.enabled
                };
                
                if !should_continue {
                    break;
                }
                
                // Update CPU data
                if let Err(e) = plugin.update_cpu_data().await {
                    log::warn!("[CPU_MONITOR] Error updating CPU data: {}", e);
                }
                
                // Get interval without holding the lock
                let interval = {
                    let config = plugin.config.lock().await;
                    config.update_interval_seconds
                };
                
                // Sleep for the configured interval
                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
            
            log::info!("[CPU_MONITOR] CPU monitoring stopped");
        });
    }

    // Update CPU data for all processes and system
    pub async fn update_cpu_data(&self) -> AppResult<()> {
        log::info!("[CPU_PLUGIN] ===== UPDATE CPU DATA STARTED =====");
        
        let config = self.config.lock().await;
        log::debug!("[CPU_PLUGIN] Config loaded - system_cpu: {}, enabled: {}", config.include_system_cpu, config.enabled);
        
        if !config.enabled {
            log::info!("[CPU_PLUGIN] CPU monitoring is disabled");
            return Ok(());
        }
        
        drop(config); // Release the lock before async calls
        
        // Update system CPU if enabled
        if self.config.lock().await.include_system_cpu {
            log::info!("[CPU_PLUGIN] Starting system CPU update...");
            self.update_system_cpu().await?;
            log::info!("[CPU_PLUGIN] System CPU update completed successfully");
        }
        
        // Update process data
        log::info!("[CPU_PLUGIN] Starting process update...");
        self.update_all_processes().await?;
        log::info!("[CPU_PLUGIN] Process update completed successfully");
        
        log::info!("[CPU_PLUGIN] ===== UPDATE CPU DATA COMPLETED =====");
        Ok(())
    }

    // Update system CPU usage
    async fn update_system_cpu(&self) -> AppResult<()> {
        log::info!("[CPU_SYSTEM] ===== UPDATE SYSTEM CPU STARTED =====");
        
        #[cfg(target_os = "windows")]
        {
            // Try using sysinfo crate first (more efficient than WMIC)
            if let Ok(cpu_percent) = self.get_system_cpu_sysinfo().await {
                log::info!("[CPU_SYSTEM] Using sysinfo: {:.1}%", cpu_percent);
                
                // Get actual number of CPU cores (not logical processors)
                // Use num_cpus::get() for physical cores, not sysinfo which might return logical processors
                let num_cores = num_cpus::get() as f64;
                
                // Create a vector with the system CPU percentage for each core
                let cores = vec![cpu_percent; num_cores as usize];
                
                log::info!("[CPU_SYSTEM] Detected {} physical CPU cores", num_cores);
                
                let system_data = SystemCpuData {
                    total_cpu_percent: cpu_percent,
                    cores,
                    last_update: chrono::Utc::now(),
                };
                
                let mut data = self.system_data.lock().await;
                *data = Some(system_data);
                
                log::info!("[CPU_SYSTEM] ===== UPDATE SYSTEM CPU COMPLETED =====");
                return Ok(());
            }
            
            // Fallback to WMIC if sysinfo fails
            log::info!("[CPU_SYSTEM] sysinfo failed, falling back to WMIC...");
            
            // Windows: Use wmic to get system CPU usage
            log::info!("[CPU_SYSTEM] Executing WMIC command...");
            
            let output = Command::new("wmic")
                .args(&["cpu", "get", "loadpercentage", "/format:csv"])
                .output()
                .map_err(|e| {
                    AppError::ConfigError(format!("Failed to get system CPU: {}", e))
                })?;
            
            log::info!("[CPU_SYSTEM] WMIC command completed successfully");
            
            let system_cpu = String::from_utf8_lossy(&output.stdout);
            log::debug!("[CPU_SYSTEM] WMIC system output: {}", system_cpu);
            
            let total_cpu_percent: f64 = system_cpu
                .lines()
                .find(|line| line.contains(",") && !line.contains("Node") && !line.contains("LoadPercentage"))
                .and_then(|line| {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 2 {
                        // The LoadPercentage is in the second column (index 1)
                        let value = parts[1].trim();
                        value.parse().ok()
                    } else {
                        None
                    }
                })
                .unwrap_or(0.0);

            log::info!("[CPU_SYSTEM] Parsed CPU percentage: {:.1}%", total_cpu_percent);

            // Get actual number of CPU cores (not logical processors)
            // Use num_cpus::get() for physical cores, not sysinfo which might return logical processors
            let num_cores = num_cpus::get() as f64;
            
            // Create a vector with the system CPU percentage for each core
            let cores = vec![total_cpu_percent; num_cores as usize];
            
            log::info!("[CPU_SYSTEM] Detected {} physical CPU cores", num_cores);
            
            let system_data = SystemCpuData {
                total_cpu_percent,
                cores,
                last_update: chrono::Utc::now(),
            };
            
            let mut data = self.system_data.lock().await;
            *data = Some(system_data);
            
            log::info!("[CPU_SYSTEM] ===== UPDATE SYSTEM CPU COMPLETED =====");
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Linux/macOS: Use ps command to get system CPU usage
            let output = Command::new("ps")
                .args(&["-p", "1", "-o", "%cpu"])
                .output()
                .map_err(|e| AppError::ConfigError(format!("Failed to get system CPU: {}", e)))?;
            
            let ps_output = String::from_utf8_lossy(&output.stdout);
            let cpu_percent: f64 = ps_output.lines().nth(1)
                .and_then(|line| line.trim().parse().ok())
                .unwrap_or(0.0);
            
            // Get actual number of CPU cores (not logical processors)
            let num_cores = num_cpus::get() as f64;
            let cores = vec![cpu_percent; num_cores as usize];
            
            let system_data = SystemCpuData {
                total_cpu_percent: cpu_percent,
                cores,
                last_update: chrono::Utc::now(),
            };
            
            let mut data = self.system_data.lock().await;
            *data = Some(system_data);
            
            log::debug!("[CPU_MONITOR] Updated system CPU: {}%", cpu_percent);
        }

        Ok(())
    }

    // More efficient system CPU monitoring using sysinfo crate
    async fn get_system_cpu_sysinfo(&self) -> AppResult<f64> {
        use sysinfo::{System, SystemExt, CpuExt};
        
        let mut sys = System::new_all();
        sys.refresh_cpu();
        
        // Wait a bit for accurate measurement
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        sys.refresh_cpu();
        
        let cpu_usage = sys.global_cpu_info().cpu_usage();
        Ok(cpu_usage as f64)
    }

    // Update all running processes
    async fn update_all_processes(&self) -> AppResult<()> {
        log::info!("[CPU_PROCESS] ===== UPDATE ALL PROCESSES STARTED =====");
        
        #[cfg(target_os = "windows")]
        {
            // Try using sysinfo crate first (more efficient than PowerShell)
            if let Ok(processes) = self.get_processes_sysinfo().await {
                log::info!("[CPU_PROCESS] Using sysinfo: {} processes", processes.len());
                
                // Update the process data
                let mut data = self.process_data.lock().await;
                *data = processes;
                
                log::info!("[CPU_PROCESS] Process data updated successfully (sysinfo)");
                log::info!("[CPU_PROCESS] ===== UPDATE ALL PROCESSES COMPLETED =====");
                return Ok(());
            }
            
            // Fallback to PowerShell if sysinfo fails
            log::info!("[CPU_PROCESS] sysinfo failed, falling back to PowerShell...");
            
            // Windows: Use PowerShell to get all processes
            log::info!("[CPU_PROCESS] Executing PowerShell command...");
            
            let output = Command::new("powershell")
                .args(&["-Command", "Get-Process | Select-Object Name, Id, CPU, WorkingSet | ConvertTo-Csv -NoTypeInformation"])
                .output()
                .map_err(|e| {
                    AppError::ConfigError(format!("Failed to get all processes: {}", e))
                })?;
            
            log::info!("[CPU_PROCESS] PowerShell command completed successfully");
            
            let process_info = String::from_utf8_lossy(&output.stdout);
            log::debug!("[CPU_PROCESS] PowerShell output: {}", process_info);
            
            let lines: Vec<&str> = process_info.lines().collect();
            log::info!("[CPU_PROCESS] Parsed {} lines from PowerShell output", lines.len());
            
            let mut new_process_data = HashMap::new();
            
            for (_i, line) in lines.iter().skip(1).enumerate() { // Skip header
                
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 4 {
                    let process_name = parts[0].trim_matches('"').to_string();
                    
                    // Parse PID (always present)
                    let _pid = parts[1].trim_matches('"').parse::<u32>().unwrap_or(0);
                    
                    // Parse CPU (may be empty or in European format with commas)
                    let cpu_str = parts[2].trim_matches('"');
                    let cpu_seconds: f64 = if cpu_str.is_empty() {
                        0.0
                    } else {
                        // Handle European number format (commas as decimal separators)
                        let normalized_cpu = cpu_str.replace(",", ".");
                        normalized_cpu.parse().unwrap_or(0.0)
                    };
                    
                    // Parse memory (always present)
                    let memory_bytes: u64 = parts[3].trim_matches('"').parse().unwrap_or(0);
                    
                    // The CPU value from PowerShell is in seconds of CPU time
                    // Use a much smaller scaling factor for realistic percentages
                    let cpu_percent = cpu_seconds * 0.005; // Much more reasonable scaling (divide by 200)
                    
                    // Only include processes with significant CPU usage (> 0.1%) or memory (> 50MB)
                    // This will reduce the number of processes and improve performance
                    if cpu_percent > 0.1 || memory_bytes > 50 * 1024 * 1024 {
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
            
            log::info!("[CPU_PROCESS] Found {} processes with significant usage", new_process_data.len());
            
            // Sort by CPU usage and take top 10
            let mut sorted_processes: Vec<CpuProcessData> = new_process_data.into_values().collect();
            sorted_processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));
            sorted_processes.truncate(10); // Keep only top 10
            
            // Convert back to HashMap
            let mut top_processes = HashMap::new();
            for process in sorted_processes {
                top_processes.insert(process.process_name.clone(), process);
            }
            
            // Update the process data
            let mut data = self.process_data.lock().await;
            *data = top_processes;
            
            log::info!("[CPU_PROCESS] Process data updated successfully (top 10 processes)");
            log::info!("[CPU_PROCESS] ===== UPDATE ALL PROCESSES COMPLETED =====");
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

    // More efficient process monitoring using sysinfo crate
    async fn get_processes_sysinfo(&self) -> AppResult<HashMap<String, CpuProcessData>> {
        use sysinfo::{System, SystemExt, ProcessExt};
        
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let mut processes = HashMap::new();
        
        for (_pid, process) in sys.processes() {
            // sysinfo returns CPU usage as a percentage, but it might be cumulative
            // We need to get the actual percentage by dividing by the number of cores
            let raw_cpu_usage = process.cpu_usage() as f64;
            let num_cores = sys.cpus().len() as f64;
            
            // Calculate actual CPU percentage (divide by number of cores for realistic values)
            let cpu_percent = if num_cores > 0.0 {
                raw_cpu_usage / num_cores
            } else {
                raw_cpu_usage
            };
            
            let memory_mb = process.memory() as f64 / 1024.0 / 1024.0; // Convert KB to MB
            
            // Only include processes with significant CPU usage (> 0.1%) or memory (> 50MB)
            if cpu_percent > 0.1 || memory_mb > 50.0 {
                let process_data = CpuProcessData {
                    process_name: process.name().to_string(),
                    cpu_percent,
                    memory_mb,
                    last_update: chrono::Utc::now(),
                };
                
                processes.insert(process_data.process_name.clone(), process_data);
            }
        }
        
        // Sort by CPU usage and take top 10
        let mut sorted_processes: Vec<CpuProcessData> = processes.into_values().collect();
        sorted_processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));
        sorted_processes.truncate(10);
        
        // Convert back to HashMap
        let mut top_processes = HashMap::new();
        for process in sorted_processes {
            top_processes.insert(process.process_name.clone(), process);
        }
        
        Ok(top_processes)
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

    // Enable monitoring
    pub async fn enable_monitoring(&self) -> AppResult<()> {
        let mut config = self.config.lock().await;
        if !config.enabled {
            config.enabled = true;
            drop(config);
            self.start_monitoring();
            log::info!("[CPU_MONITOR] Monitoring enabled and started");
        } else {
            log::info!("[CPU_MONITOR] Monitoring already enabled");
        }
        Ok(())
    }

    // Disable monitoring
    pub async fn disable_monitoring(&self) -> AppResult<()> {
        let mut config = self.config.lock().await;
        if config.enabled {
            config.enabled = false;
            drop(config);
            self.stop_monitoring().await;
            log::info!("[CPU_MONITOR] Monitoring disabled and stopped");
        } else {
            log::info!("[CPU_MONITOR] Monitoring already disabled");
        }
        Ok(())
    }

    // Check if monitoring is enabled
    pub async fn is_monitoring_enabled(&self) -> AppResult<bool> {
        let config = self.config.lock().await;
        Ok(config.enabled)
    }
}

// Default configuration
impl Default for CpuMonitorConfig {
    fn default() -> Self {
        // Use different intervals for debug vs release builds
        let update_interval = if cfg!(debug_assertions) {
            1 // 1 second for development
        } else {
            2 // 2 seconds for production (less CPU intensive)
        };
        
        Self {
            enabled: false, // Disabled by default
            update_interval_seconds: update_interval,
            monitored_processes: vec!["obs64.exe".to_string(), "obs-studio".to_string()],
            include_system_cpu: true,
        }
    }
} 