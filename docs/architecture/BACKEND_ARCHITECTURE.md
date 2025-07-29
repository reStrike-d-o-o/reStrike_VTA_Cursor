# Backend Architecture

## Overview

The reStrike VTA backend is built with Rust and Tauri v2, providing a native Windows desktop application with a modular plugin architecture. The system is designed for high performance, real-time event processing, and seamless integration with external systems like OBS Studio.

## Architecture

### Core Architecture
- **Tauri v2**: Modern desktop application framework
- **Plugin System**: Modular architecture with clear separation of concerns
- **Async/Await**: Tokio-based asynchronous programming
- **Error Handling**: Comprehensive error management with AppResult<T>
- **State Management**: Thread-safe shared state with Arc<Mutex<T>>

### Technology Stack
- **Language**: Rust (latest stable)
- **Framework**: Tauri v2 for native Windows integration
- **Async Runtime**: Tokio for asynchronous operations
- **Database**: SQLite with rusqlite
- **WebSocket**: tokio-tungstenite for OBS integration
- **Serialization**: Serde for JSON handling
- **Logging**: Structured logging with file rotation

## Directory Structure

```
src-tauri/
├── src/
│   ├── main.rs              # Tauri app entry point
│   ├── lib.rs               # Library exports and plugin registration
│   ├── tauri_commands.rs    # Tauri command definitions (1835 lines)
│   ├── core/                # Core application functionality
│   │   ├── app.rs           # Application state and lifecycle
│   │   ├── config.rs        # Configuration management
│   │   └── state.rs         # Global state management
│   ├── config/              # Configuration system
│   │   ├── manager.rs       # Configuration manager
│   │   ├── types.rs         # Configuration types
│   │   └── mod.rs           # Configuration module
│   ├── logging/             # Logging system
│   │   ├── logger.rs        # Logging implementation
│   │   ├── rotation.rs      # Log rotation
│   │   ├── archival.rs      # Log archival
│   │   └── mod.rs           # Logging module
│   ├── plugins/             # Plugin modules
│   │   ├── mod.rs           # Plugin module registration
│   │   ├── plugin_obs.rs    # OBS WebSocket integration
│   │   ├── plugin_udp.rs    # UDP protocol handling
│   │   ├── plugin_database.rs # Database operations
│   │   ├── plugin_cpu_monitor.rs # System monitoring
│   │   └── plugin_license.rs # License management
│   ├── database/            # Database system
│   │   ├── connection.rs    # Database connection management
│   │   ├── migrations.rs    # Database migrations
│   │   ├── models.rs        # Data models
│   │   ├── operations.rs    # Database operations
│   │   └── mod.rs           # Database module
│   ├── types/               # Shared types
│   │   └── mod.rs           # Type definitions
│   └── utils/               # Utility functions
│       ├── logger.rs        # Logging utilities
│       └── network.rs       # Network interface detection
├── Cargo.toml               # Rust dependencies
├── tauri.conf.json          # Tauri configuration
├── capabilities.json        # Tauri capabilities
└── build.rs                 # Build script
```

## Plugin System

### Plugin Architecture

The backend uses a modular plugin architecture where each plugin is responsible for specific functionality:

```rust
// Plugin trait for common interface
pub trait Plugin {
    fn name(&self) -> &str;
    fn initialize(&mut self) -> AppResult<()>;
    fn shutdown(&mut self) -> AppResult<()>;
    fn get_status(&self) -> PluginStatus;
}

// Plugin status enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginStatus {
    Initialized,
    Running,
    Error(String),
    Stopped,
}
```

### Core Plugins

#### Database Plugin
```rust
#[derive(Clone)]
pub struct DatabasePlugin {
    connection: Arc<DatabaseConnection>,
    migration_strategy: MigrationStrategy,
    hybrid_provider: Arc<Mutex<HybridSettingsProvider>>,
}

impl DatabasePlugin {
    // Flag Management
    pub async fn get_flag_mappings_data(&self) -> AppResult<FlagMappingsData>
    pub async fn scan_and_populate_flags(&self) -> AppResult<FlagScanResult>
    pub async fn get_flags_data(&self) -> AppResult<Vec<Flag>>
    pub async fn clear_flags_table(&self) -> AppResult<()>

    // PSS Event Management
    pub async fn get_pss_events(&self, limit: Option<i64>) -> AppResult<Vec<PssEventV2>>
    pub async fn get_pss_event_types(&self) -> AppResult<Vec<PssEventType>>
    pub async fn create_pss_event(&self, event: PssEventV2) -> AppResult<i64>

    // UDP Management
    pub async fn get_udp_server_configs(&self) -> AppResult<Vec<UdpServerConfig>>
    pub async fn create_udp_server_config(&self, config: UdpServerConfig) -> AppResult<i64>
    pub async fn update_udp_server_config(&self, id: i64, config: UdpServerConfig) -> AppResult<()>
}
```

#### UDP Plugin
```rust
pub struct UdpPlugin {
    config: UdpServerConfig,
    event_tx: UnboundedSender<PssEvent>,
    protocol_manager: Arc<ProtocolManager>,
    database: Arc<DatabasePlugin>,
    server: Arc<Mutex<Option<UdpServer>>>,
}

impl UdpPlugin {
    pub fn new(
        config: UdpServerConfig,
        event_tx: UnboundedSender<PssEvent>,
        protocol_manager: Arc<ProtocolManager>,
        database: Arc<DatabasePlugin>,
    ) -> Self {
        // Initialize UDP plugin with database integration
    }

    pub async fn start(&self) -> AppResult<()> {
        // Start UDP server with database session tracking
    }

    pub async fn stop(&self) -> AppResult<()> {
        // Stop UDP server and update database session
    }
}
```

#### OBS Plugin
```rust
pub struct ObsPlugin {
    connections: Arc<Mutex<HashMap<String, ObsConnection>>>,
    event_tx: UnboundedSender<ObsEvent>,
}

impl ObsPlugin {
    // Connection Management
    pub async fn add_connection(&mut self, config: ObsConnectionConfig) -> AppResult<()>
    pub async fn remove_connection(&mut self, name: &str) -> AppResult<()>
    pub async fn connect_obs(&mut self, name: &str) -> AppResult<()>
    pub async fn disconnect_obs(&mut self, name: &str) -> AppResult<()>

    // Scene Operations
    pub async fn get_current_scene(&self, name: &str) -> AppResult<String>
    pub async fn set_current_scene(&self, name: &str, scene: &str) -> AppResult<()>
    pub async fn get_scenes(&self, name: &str) -> AppResult<Vec<String>>

    // Recording Operations
    pub async fn start_recording(&self, name: &str) -> AppResult<()>
    pub async fn stop_recording(&self, name: &str) -> AppResult<()>
    pub async fn get_recording_status(&self, name: &str) -> AppResult<bool>

    // Replay Buffer Operations
    pub async fn start_replay_buffer(&self, name: &str) -> AppResult<()>
    pub async fn stop_replay_buffer(&self, name: &str) -> AppResult<()>
    pub async fn save_replay_buffer(&self, name: &str) -> AppResult<()>
}
```

#### CPU Monitor Plugin
```rust
pub struct CpuMonitorPlugin {
    interval: Duration,
    event_tx: UnboundedSender<CpuEvent>,
    running: Arc<AtomicBool>,
}

impl CpuMonitorPlugin {
    pub async fn start_monitoring(&self) -> AppResult<()> {
        // Start CPU monitoring with configurable interval
    }

    pub async fn stop_monitoring(&self) -> AppResult<()> {
        // Stop CPU monitoring
    }

    pub async fn get_system_info(&self) -> AppResult<SystemInfo> {
        // Get current system information
    }
}
```

#### License Plugin
```rust
pub struct LicensePlugin {
    license_key: Option<String>,
    hardware_id: String,
    validation_url: String,
    event_tx: UnboundedSender<LicenseEvent>,
}

impl LicensePlugin {
    pub async fn activate_license(&mut self, key: &str) -> AppResult<LicenseStatus>
    pub async fn validate_license(&self) -> AppResult<LicenseStatus>
    pub async fn get_license_info(&self) -> AppResult<LicenseInfo>
    pub async fn check_hardware_id(&self) -> AppResult<String>
}
```

## Development Environment Management

### Windows Development Setup

The project is optimized for Windows development with comprehensive tooling:

#### Prerequisites
- **Windows 10/11** (64-bit)
- **Node.js** v24+ LTS
- **Rust** (latest stable via rustup)
- **Tauri CLI** (`cargo install tauri-cli`)
- **VSCode** with recommended extensions
- **Git** (latest version)
- **Python** v3.8+ (for scripts)

#### VSCode Extensions
- **Rust Analyzer** (rust-lang.rust-analyzer)
- **TypeScript and JavaScript Language Features** (built-in)
- **ES7+ React/Redux/React-Native snippets** (dsznajder.es7-react-js-snippets)
- **Tailwind CSS IntelliSense** (bradlc.vscode-tailwindcss)
- **GitLens** (eamodio.gitlens)
- **Thunder Client** (rangav.vscode-thunder-client)
- **Error Lens** (usernamehw.errorlens)
- **Auto Rename Tag** (formulahendry.auto-rename-tag)

#### Development Scripts
```bash
# Main development wrapper
./scripts/dev.sh help                    # Show all commands
./scripts/dev.sh status                  # Check current status
./scripts/dev.sh start-all               # Start all services
./scripts/dev.sh stop-all                # Stop all services
./scripts/dev.sh cleanup                 # Full cleanup

# Cleanup script
./scripts/development/cleanup-dev-environment.sh --cleanup
./scripts/development/cleanup-dev-environment.sh --quick
./scripts/development/cleanup-dev-environment.sh --status

# Fast development
./scripts/development/fast-dev.sh        # Fast development server
./scripts/development/dev.sh             # Standard development server
```

#### Environment Configuration
```json
// .vscode/settings.json
{
  "typescript.preferences.importModuleSpecifier": "relative",
  "typescript.suggest.autoImports": true,
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  },
  "files.associations": {
    "*.rs": "rust"
  },
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.buildScripts.enable": true,
  "tailwindCSS.includeLanguages": {
    "typescript": "javascript",
    "typescriptreact": "javascript"
  }
}
```

#### Launch Configuration
```json
// .vscode/launch.json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Launch Tauri App",
      "type": "node",
      "request": "launch",
      "program": "${workspaceFolder}/ui/node_modules/.bin/react-scripts",
      "args": ["start"],
      "cwd": "${workspaceFolder}/ui",
      "env": {
        "REACT_APP_ENVIRONMENT": "windows"
      },
      "console": "integratedTerminal"
    },
    {
      "name": "Debug Rust Backend",
      "type": "lldb",
      "request": "launch",
      "program": "${workspaceFolder}/target/debug/restrike-vta",
      "args": [],
      "cwd": "${workspaceFolder}",
      "console": "integratedTerminal"
    }
  ]
}
```

### Development Workflow

#### Starting Development
```bash
# Single command (recommended)
cd src-tauri
cargo tauri dev

# Alternative manual start
# Terminal 1: Start React dev server
cd ui
npm run start:fast

# Terminal 2: Start Tauri app
cd src-tauri
cargo tauri dev
```

#### Build Commands
```bash
# Development build
cd ui && npm run build

# Production build
cd src-tauri && cargo tauri build

# Clean build
cargo clean
npm run build
```

#### Testing Commands
```bash
# Frontend tests
cd ui && npm test

# Backend tests
cargo test

# Integration tests
cargo test --test integration

# Code quality
cargo clippy
cargo fmt
npm run lint
```

### Performance Optimization

#### Rust Backend Optimization
```toml
# Cargo.toml optimization settings
[profile.dev]
opt-level = 1
codegen-units = 256
incremental = true
lto = false

[profile.release]
opt-level = 3
codegen-units = 1
lto = true
panic = "abort"
```

#### Development Scripts
```bash
# Fast development scripts
npm run start:fast          # Fast React development
npm run build:fast          # Fast build
npm run clean:all           # Clean all caches
npm run analyze             # Bundle analysis
```

### Troubleshooting

#### Common Issues

**Rust Compilation Errors**
```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build

# Check for missing dependencies
cargo check
```

**Node.js/npm Issues**
```bash
# Clear npm cache
npm cache clean --force

# Delete node_modules and reinstall
rm -rf node_modules
npm install

# Update npm
npm install -g npm@latest
```

**Tauri Build Issues**
```bash
# Check Tauri requirements
cargo tauri info

# Update Tauri CLI
cargo install tauri-cli --force

# Check system requirements
cargo tauri doctor
```

**Port Conflicts**
```bash
# Clean up ports before starting
./scripts/development/cleanup-dev-environment.sh --cleanup
```

#### Debug Information

**Enable Debug Logging**
```rust
// Enable debug logging
env_logger::init();
log::set_max_level(log::LevelFilter::Debug);
```

**Check System Status**
```bash
# Check development environment status
./scripts/dev.sh status

# Check port usage
./scripts/dev.sh ports

# Run health checks
./scripts/dev.sh health
```

## Core Systems

### Application State Management
```rust
pub struct AppState {
    pub plugins: Arc<Mutex<HashMap<String, Box<dyn Plugin>>>>,
    pub event_tx: UnboundedSender<AppEvent>,
    pub config: Arc<ConfigManager>,
    pub database: Arc<DatabasePlugin>,
}

impl AppState {
    pub async fn initialize(&mut self) -> AppResult<()> {
        // Initialize all plugins in correct order
        self.initialize_database_plugin().await?;
        self.initialize_udp_plugin().await?;
        self.initialize_obs_plugin().await?;
        self.initialize_cpu_monitor_plugin().await?;
        self.initialize_license_plugin().await?;
        Ok(())
    }
}
```

### Configuration Management
```rust
pub struct ConfigManager {
    config_path: PathBuf,
    config: Arc<RwLock<AppConfig>>,
    backup_path: PathBuf,
}

impl ConfigManager {
    pub async fn load_config(&self) -> AppResult<AppConfig>
    pub async fn save_config(&self, config: &AppConfig) -> AppResult<()>
    pub async fn backup_config(&self) -> AppResult<()>
    pub async fn restore_config(&self) -> AppResult<()>
    pub async fn get_config_statistics(&self) -> AppResult<ConfigStats>
}
```

### Logging System
```rust
pub struct Logger {
    log_path: PathBuf,
    rotation_config: RotationConfig,
    archival_config: ArchivalConfig,
}

impl Logger {
    pub fn initialize(&self) -> AppResult<()>
    pub fn log(&self, level: Level, message: &str) -> AppResult<()>
    pub async fn rotate_logs(&self) -> AppResult<()>
    pub async fn archive_logs(&self) -> AppResult<()>
    pub async fn get_log_statistics(&self) -> AppResult<LogStats>
}
```

## Tauri Integration

### Command Registration
```rust
// tauri_commands.rs
#[tauri::command]
async fn get_app_status() -> AppResult<AppStatus> {
    // Return application status
}

#[tauri::command]
async fn obs_get_status(connection_name: String) -> AppResult<ObsStatus> {
    // Return OBS connection status
}

#[tauri::command]
async fn system_get_info() -> AppResult<SystemInfo> {
    // Return system information
}

#[tauri::command]
async fn get_flag_mappings_data() -> AppResult<FlagMappingsData> {
    // Return flag mapping data
}

#[tauri::command]
async fn scan_and_populate_flags() -> AppResult<FlagScanResult> {
    // Scan and populate flags
}
```

### Event System
```rust
// Event types for frontend communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEvent {
    PssEvent(PssEvent),
    ObsEvent(ObsEvent),
    CpuEvent(CpuEvent),
    LicenseEvent(LicenseEvent),
    DatabaseEvent(DatabaseEvent),
}

// Event emission
pub async fn emit_event(event: AppEvent) -> AppResult<()> {
    if let Some(app_handle) = APP_HANDLE.get() {
        app_handle.emit_all("app_event", event)?;
    }
    Ok(())
}
```

## Error Handling

### AppResult and AppError
```rust
pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    
    #[error("Plugin error: {0}")]
    PluginError(String),
}
```

### Error Propagation
```rust
// Proper error handling in plugins
impl DatabasePlugin {
    pub async fn create_pss_event(&self, event: PssEventV2) -> AppResult<i64> {
        let conn = self.connection.lock()
            .map_err(|e| AppError::DatabaseError(rusqlite::Error::InvalidPath(e.to_string())))?;
        
        // Database operation
        let event_id = conn.execute(
            "INSERT INTO pss_events_v2 (...) VALUES (...)",
            params![...]
        )?;
        
        Ok(event_id)
    }
}
```

## Performance Monitoring

### System Monitoring
```rust
pub struct SystemMonitor {
    cpu_usage: Arc<AtomicU64>,
    memory_usage: Arc<AtomicU64>,
    disk_usage: Arc<AtomicU64>,
    network_stats: Arc<Mutex<NetworkStats>>,
}

impl SystemMonitor {
    pub async fn start_monitoring(&self) -> AppResult<()>
    pub async fn get_system_stats(&self) -> AppResult<SystemStats>
    pub async fn get_performance_metrics(&self) -> AppResult<PerformanceMetrics>
}
```

### Database Performance
```rust
pub struct DatabaseMonitor {
    query_times: Arc<Mutex<Vec<Duration>>>,
    connection_pool: Arc<Mutex<ConnectionPool>>,
    cache_stats: Arc<Mutex<CacheStats>>,
}

impl DatabaseMonitor {
    pub async fn track_query_time(&self, duration: Duration)
    pub async fn get_performance_stats(&self) -> AppResult<DatabaseStats>
    pub async fn optimize_queries(&self) -> AppResult<()>
}
```

## Security

### License Validation
```rust
pub struct LicenseValidator {
    hardware_id: String,
    license_key: Option<String>,
    validation_url: String,
    offline_grace_period: Duration,
}

impl LicenseValidator {
    pub async fn validate_license(&self) -> AppResult<LicenseStatus>
    pub async fn activate_license(&mut self, key: &str) -> AppResult<LicenseStatus>
    pub async fn check_hardware_id(&self) -> AppResult<String>
    pub async fn get_offline_status(&self) -> AppResult<OfflineStatus>
}
```

### Data Protection
```rust
pub struct DataProtector {
    encryption_key: Vec<u8>,
    sensitive_fields: HashSet<String>,
}

impl DataProtector {
    pub fn encrypt_sensitive_data(&self, data: &str) -> AppResult<String>
    pub fn decrypt_sensitive_data(&self, encrypted_data: &str) -> AppResult<String>
    pub fn mask_sensitive_fields(&self, data: &mut serde_json::Value) -> AppResult<()>
}
```

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_plugin_creation() {
        let plugin = DatabasePlugin::new().await.unwrap();
        assert_eq!(plugin.name(), "database");
    }

    #[tokio::test]
    async fn test_udp_plugin_initialization() {
        let (event_tx, _) = mpsc::unbounded_channel();
        let config = UdpServerConfig::default();
        let plugin = UdpPlugin::new(config, event_tx, Arc::new(ProtocolManager::new()), Arc::new(DatabasePlugin::new().await.unwrap()));
        assert_eq!(plugin.name(), "udp");
    }
}
```

### Integration Tests
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_workflow() {
        // Test complete workflow from UDP event to database storage
        let app_state = AppState::new().await.unwrap();
        app_state.initialize().await.unwrap();
        
        // Simulate PSS event
        let event = PssEvent::new_test_event();
        app_state.process_pss_event(event).await.unwrap();
        
        // Verify database storage
        let events = app_state.database.get_pss_events(Some(1)).await.unwrap();
        assert_eq!(events.len(), 1);
    }
}
```

## Deployment

### Production Build
```bash
# Build for production
cargo tauri build --release

# Create installer
cargo tauri build --target x86_64-pc-windows-msvc

# Output location: src-tauri/target/release/bundle/
```

### Distribution
```bash
# Create distribution package
# - Windows executable (.exe)
# - MSI installer
# - Portable version
# - Documentation and licenses
```

## Future Enhancements

### Planned Features
1. **Plugin Marketplace**: Extensible plugin system
2. **Advanced Analytics**: Real-time analytics and reporting
3. **Cloud Integration**: Cloud backup and synchronization
4. **Multi-language Support**: Internationalization
5. **Advanced Security**: Enhanced security features

### Technical Improvements
1. **Performance Optimization**: Advanced performance tuning
2. **Memory Management**: Improved memory usage
3. **Error Recovery**: Enhanced error recovery mechanisms
4. **Monitoring**: Advanced monitoring and alerting
5. **Scalability**: Improved scalability for large deployments

## Support and Resources

### Documentation
- **Tauri Documentation**: https://tauri.app/docs/
- **Rust Documentation**: https://doc.rust-lang.org/
- **Tokio Documentation**: https://tokio.rs/

### Community
- **Tauri Discord**: https://discord.gg/tauri
- **Rust Community**: https://users.rust-lang.org/
- **GitHub Issues**: Project-specific issues and discussions

### Professional Support
- **Custom Development**: Tailored solutions and features
- **Training and Consulting**: Development team training
- **Enterprise Support**: Enterprise-level support and maintenance