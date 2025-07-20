use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main application configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application version and metadata
    pub app: AppSettings,
    /// OBS WebSocket connection settings
    pub obs: ObsSettings,
    /// UDP/PSS protocol settings
    pub udp: UdpSettings,
    /// Logging and diagnostics settings
    pub logging: LoggingSettings,
    /// UI and overlay settings
    pub ui: UiSettings,
    /// Video playback settings
    pub video: VideoSettings,
    /// License and activation settings
    pub license: LicenseSettings,
    /// Flag management settings
    pub flags: FlagSettings,
    /// Advanced settings and features
    pub advanced: AdvancedSettings,
}

/// Application metadata and core settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Application version
    pub version: String,
    /// Last configuration save timestamp
    pub last_save: String,
    /// Application startup behavior
    pub startup: StartupSettings,
    /// Performance and optimization settings
    pub performance: PerformanceSettings,
}

/// Application startup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupSettings {
    /// Auto-start UDP listener on app launch
    pub auto_start_udp: bool,
    /// Auto-connect to OBS on app launch
    pub auto_connect_obs: bool,
    /// Show advanced panel on startup
    pub show_advanced_panel: bool,
    /// Minimize to system tray on startup
    pub minimize_to_tray: bool,
    /// Check for updates on startup
    pub check_updates: bool,
}

/// Performance optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Enable hardware acceleration
    pub hardware_acceleration: bool,
    /// Maximum memory usage (MB)
    pub max_memory_mb: u64,
    /// Enable background processing
    pub background_processing: bool,
    /// Log rotation size (MB)
    pub log_rotation_size_mb: u64,
}

/// OBS WebSocket connection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsSettings {
    /// List of OBS connections
    pub connections: Vec<ObsConnectionConfig>,
    /// Default connection settings
    pub defaults: ObsDefaultSettings,
    /// Connection behavior settings
    pub behavior: ObsBehaviorSettings,
    /// Integration settings
    pub integration: ObsIntegrationSettings,
}

/// OBS Integration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsIntegrationSettings {
    /// Auto-connect to OBS on startup
    pub auto_connect_on_startup: bool,
    /// Show OBS status in overlay
    pub show_status_in_overlay: bool,
    /// Auto-record when playing clips
    pub auto_record_on_clip_play: bool,
    /// Save replay buffer on clip creation
    pub save_replay_buffer_on_clip_creation: bool,
}

/// Individual OBS connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsConnectionConfig {
    /// Connection name (e.g., "OBS_REC", "OBS_STR")
    pub name: String,
    /// OBS host address
    pub host: String,
    /// OBS WebSocket port
    pub port: u16,
    /// WebSocket password (optional)
    pub password: Option<String>,
    /// Protocol version ("v5")
    pub protocol_version: String,
    /// Whether connection is enabled
    pub enabled: bool,
    /// Connection timeout (seconds)
    pub timeout_seconds: u64,
    /// Auto-reconnect settings
    pub auto_reconnect: bool,
    /// Maximum reconnection attempts
    pub max_reconnect_attempts: u32,
}

/// Default OBS connection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsDefaultSettings {
    /// Default host for new connections
    pub default_host: String,
    /// Default port for new connections
    pub default_port: u16,
    /// Default protocol version
    pub default_protocol: String,
    /// Default timeout (seconds)
    pub default_timeout: u64,
}

/// OBS connection behavior settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsBehaviorSettings {
    /// Auto-reconnect on connection loss
    pub auto_reconnect: bool,
    /// Reconnection delay (seconds)
    pub reconnect_delay: u64,
    /// Maximum reconnection attempts
    pub max_attempts: u32,
    /// Enable connection status monitoring
    pub status_monitoring: bool,
    /// Status check interval (seconds)
    pub status_interval: u64,
}

/// UDP/PSS protocol settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpSettings {
    /// UDP listener configuration
    pub listener: UdpListenerSettings,
    /// PSS protocol settings
    pub pss: PssSettings,
    /// Event processing settings
    pub events: EventSettings,
}

/// Network interface configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceSettings {
    /// Auto-detect network interface
    pub auto_detect: bool,
    /// Preferred interface type ("ethernet", "wifi", "any")
    pub preferred_type: String,
    /// Fallback to localhost if no suitable interface found
    pub fallback_to_localhost: bool,
    /// Manually selected interface name (if auto_detect is false)
    pub selected_interface: Option<String>,
}

impl Default for NetworkInterfaceSettings {
    fn default() -> Self {
        Self {
            auto_detect: true,
            preferred_type: "ethernet".to_string(),
            fallback_to_localhost: true,
            selected_interface: None,
        }
    }
}

/// UDP listener configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpListenerSettings {
    /// UDP listen port
    pub port: u16,
    /// Bind address (empty for all interfaces)
    pub bind_address: String,
    /// Enable UDP listener
    pub enabled: bool,
    /// Buffer size (bytes)
    pub buffer_size: usize,
    /// Timeout (seconds)
    pub timeout_seconds: u64,
    /// Network interface configuration
    #[serde(default)]
    pub network_interface: NetworkInterfaceSettings,
}

/// PSS protocol settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssSettings {
    /// PSS schema file path
    pub schema_file: String,
    /// Enable PSS parsing
    pub enabled: bool,
    /// Strict parsing mode
    pub strict_mode: bool,
    /// Unknown field handling
    pub unknown_fields: String, // "ignore", "warn", "error"
    /// Protocol version management
    #[serde(default)]
    pub protocol_versions: ProtocolVersionSettings,
    /// Protocol file storage
    #[serde(default)]
    pub storage: ProtocolStorageSettings,
}

/// Protocol version management settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersionSettings {
    /// Currently active protocol version
    pub active_version: String,
    /// Available protocol versions
    pub available_versions: Vec<ProtocolVersion>,
    /// Auto-update protocol versions
    pub auto_update: bool,
    /// Version validation on load
    pub validate_on_load: bool,
}

impl Default for ProtocolVersionSettings {
    fn default() -> Self {
        Self {
            active_version: "2.3".to_string(),
            available_versions: Vec::new(),
            auto_update: false,
            validate_on_load: true,
        }
    }
}

/// Individual protocol version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersion {
    /// Version identifier (e.g., "2.3", "2.4")
    pub version: String,
    /// Protocol file name
    pub filename: String,
    /// Full path to protocol file
    pub file_path: String,
    /// Protocol description
    pub description: String,
    /// Creation date
    pub created_date: String,
    /// Last modified date
    pub last_modified: String,
    /// Whether this version is active
    pub is_active: bool,
    /// Protocol file size in bytes
    pub file_size: u64,
    /// Protocol file checksum for validation
    pub checksum: Option<String>,
}

/// Protocol file storage settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolStorageSettings {
    /// Protocol files storage directory
    pub storage_directory: String,
    /// Maximum number of protocol versions to keep
    pub max_versions: usize,
    /// Auto-backup protocol files
    pub auto_backup: bool,
    /// Backup directory
    pub backup_directory: String,
    /// Enable protocol file compression
    pub compression: bool,
    /// Protocol file retention days
    pub retention_days: u64,
}

impl Default for ProtocolStorageSettings {
    fn default() -> Self {
        Self {
            storage_directory: "protocol".to_string(),
            max_versions: 10,
            auto_backup: true,
            backup_directory: "protocol/backups".to_string(),
            compression: false,
            retention_days: 30,
        }
    }
}

/// Event processing settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSettings {
    /// Enable event storage
    pub store_events: bool,
    /// Maximum events to store
    pub max_events: usize,
    /// Event cleanup interval (hours)
    pub cleanup_interval_hours: u64,
    /// Enable event filtering
    pub enable_filtering: bool,
    /// Default event filters
    pub default_filters: Vec<String>,
}

/// Logging and diagnostics settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSettings {
    /// Global logging configuration
    pub global: GlobalLoggingSettings,
    /// Subsystem-specific logging
    pub subsystems: HashMap<String, SubsystemLoggingSettings>,
    /// Log file management
    pub files: LogFileSettings,
    /// Live data streaming
    pub live_data: LiveDataSettings,
}

/// Global logging settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalLoggingSettings {
    /// Global logging level
    pub level: String, // "trace", "debug", "info", "warn", "error"
    /// Enable console logging
    pub console_enabled: bool,
    /// Enable file logging
    pub file_enabled: bool,
    /// Enable timestamp in logs
    pub include_timestamp: bool,
    /// Enable subsystem prefix in logs
    pub include_subsystem: bool,
}

/// Subsystem-specific logging settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsystemLoggingSettings {
    /// Subsystem name
    pub name: String,
    /// Logging level for this subsystem
    pub level: String,
    /// Whether logging is enabled
    pub enabled: bool,
    /// Custom log file path (optional)
    pub custom_file: Option<String>,
}

/// Log file management settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFileSettings {
    /// Log directory path
    pub directory: String,
    /// Maximum log file size (MB)
    pub max_size_mb: u64,
    /// Maximum number of log files
    pub max_files: usize,
    /// Log file retention days
    pub retention_days: u64,
    /// Enable log compression
    pub compression: bool,
    /// Archive old logs
    pub archive_enabled: bool,
}

/// Live data streaming settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveDataSettings {
    /// Enable live data streaming
    pub enabled: bool,
    /// Streaming interval (seconds)
    pub interval_seconds: u64,
    /// Maximum data points
    pub max_data_points: usize,
    /// Enable real-time updates
    pub real_time: bool,
}

/// UI and overlay settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    /// Overlay configuration
    pub overlay: OverlaySettings,
    /// Theme settings
    pub theme: ThemeSettings,
    /// Layout settings
    pub layout: LayoutSettings,
    /// Animation settings
    pub animations: AnimationSettings,
}

/// Overlay display settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlaySettings {
    /// Overlay visibility
    pub visible: bool,
    /// Overlay opacity (0.0 - 1.0)
    pub opacity: f64,
    /// Overlay position
    pub position: String, // "top-left", "top-right", "bottom-left", "bottom-right"
    /// Overlay scale
    pub scale: f64,
    /// Overlay theme
    pub theme: String, // "dark", "light", "auto"
    /// Always on top
    pub always_on_top: bool,
    /// Click through overlay
    pub click_through: bool,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    /// Current theme
    pub current: String, // "dark", "light", "auto"
    /// Auto theme based on system
    pub auto_theme: bool,
    /// Custom theme colors
    pub custom_colors: HashMap<String, String>,
    /// High contrast mode
    pub high_contrast: bool,
}

/// Layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSettings {
    /// Sidebar position
    pub sidebar_position: String, // "left", "right"
    /// Sidebar width
    pub sidebar_width: u32,
    /// Status bar visibility
    pub status_bar_visible: bool,
    /// Task bar visibility
    pub task_bar_visible: bool,
    /// Window size
    pub window_size: WindowSize,
    /// Window position
    pub window_position: WindowPosition,
}

/// Window size settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

/// Window position settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

/// Animation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationSettings {
    /// Enable animations
    pub enabled: bool,
    /// Animation duration (ms)
    pub duration_ms: u64,
    /// Animation easing
    pub easing: String, // "linear", "ease-in", "ease-out", "ease-in-out"
    /// Reduce motion for accessibility
    pub reduce_motion: bool,
}

/// Video playback settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSettings {
    /// Video player settings
    pub player: VideoPlayerSettings,
    /// Replay buffer settings
    pub replay: ReplaySettings,
    /// Clip management
    pub clips: ClipSettings,
}

/// Video player configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoPlayerSettings {
    /// Default video player
    pub default_player: String, // "mpv", "vlc", "system"
    /// Player executable path
    pub player_path: Option<String>,
    /// Player arguments
    pub player_args: Vec<String>,
    /// Auto-hide UI during playback
    pub auto_hide_ui: bool,
    /// Fullscreen playback
    pub fullscreen: bool,
}

/// Replay buffer settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaySettings {
    /// Default replay duration (seconds)
    pub default_duration: u64,
    /// Replay buffer size (MB)
    pub buffer_size_mb: u64,
    /// Auto-save replays
    pub auto_save: bool,
    /// Replay quality
    pub quality: String, // "low", "medium", "high"
}

/// Clip management settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipSettings {
    /// Clip storage directory
    pub storage_directory: String,
    /// Maximum clip size (MB)
    pub max_size_mb: u64,
    /// Clip retention days
    pub retention_days: u64,
    /// Auto-organize clips
    pub auto_organize: bool,
    /// Clip naming format
    pub naming_format: String,
}

/// License and activation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseSettings {
    /// License key
    pub license_key: Option<String>,
    /// License status
    pub status: String, // "valid", "invalid", "expired", "trial"
    /// License expiration date
    pub expiration_date: Option<String>,
    /// Auto-renewal settings
    pub auto_renewal: bool,
    /// Offline grace period (days)
    pub offline_grace_days: u32,
    /// Last validation check
    pub last_validation: Option<String>,
}

/// Flag management settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagSettings {
    /// Flag storage settings
    pub storage: FlagStorageSettings,
    /// Recognition settings
    pub recognition: FlagRecognitionSettings,
    /// Display settings
    pub display: FlagDisplaySettings,
}

/// Flag storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagStorageSettings {
    /// Flag storage directory
    pub directory: String,
    /// Auto-download flags
    pub auto_download: bool,
    /// Flag update interval (days)
    pub update_interval_days: u32,
    /// Maximum flag size (KB)
    pub max_size_kb: u64,
}

/// Flag recognition settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagRecognitionSettings {
    /// Enable automatic recognition
    pub auto_recognition: bool,
    /// Recognition confidence threshold
    pub confidence_threshold: f64,
    /// Recognition database path
    pub database_path: String,
    /// Enable machine learning
    pub machine_learning: bool,
}

/// Flag display settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagDisplaySettings {
    /// Flag size (pixels)
    pub size: u32,
    /// Flag quality
    pub quality: String, // "low", "medium", "high"
    /// Show country names
    pub show_names: bool,
    /// Flag animation
    pub animation: bool,
}

/// Advanced settings and features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSettings {
    /// Development settings
    pub development: DevelopmentSettings,
    /// Network settings
    pub network: NetworkSettings,
    /// Security settings
    pub security: SecuritySettings,
    /// Experimental features
    pub experimental: ExperimentalSettings,
}

/// Development settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentSettings {
    /// Enable debug mode
    pub debug_mode: bool,
    /// Enable verbose logging
    pub verbose_logging: bool,
    /// Enable hot reload
    pub hot_reload: bool,
    /// Development server port
    pub dev_server_port: u16,
}

/// Network settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    /// Proxy configuration
    pub proxy: Option<ProxySettings>,
    /// Connection timeout (seconds)
    pub connection_timeout: u64,
    /// Retry attempts
    pub retry_attempts: u32,
    /// Enable IPv6
    pub enable_ipv6: bool,
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettings {
    /// Proxy host
    pub host: String,
    /// Proxy port
    pub port: u16,
    /// Proxy username
    pub username: Option<String>,
    /// Proxy password
    pub password: Option<String>,
}

/// Security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// Enable SSL verification
    pub ssl_verification: bool,
    /// Allow insecure connections
    pub allow_insecure: bool,
    /// Certificate validation
    pub certificate_validation: bool,
    /// Security level
    pub security_level: String, // "low", "medium", "high"
}

/// Experimental features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentalSettings {
    /// Enable experimental features
    pub enabled: bool,
    /// List of enabled experimental features
    pub enabled_features: Vec<String>,
    /// Beta channel updates
    pub beta_updates: bool,
    /// Telemetry collection
    pub telemetry: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppSettings {
                version: env!("CARGO_PKG_VERSION").to_string(),
                last_save: chrono::Utc::now().to_rfc3339(),
                startup: StartupSettings {
                    auto_start_udp: false,
                    auto_connect_obs: false,
                    show_advanced_panel: false,
                    minimize_to_tray: false,
                    check_updates: true,
                },
                performance: PerformanceSettings {
                    hardware_acceleration: true,
                    max_memory_mb: 1024,
                    background_processing: true,
                    log_rotation_size_mb: 100,
                },
            },
            obs: ObsSettings {
                connections: vec![
                    ObsConnectionConfig {
                        name: "OBS_REC".to_string(),
                        host: "localhost".to_string(),
                        port: 4455,
                        password: Some("cekPIbj@245".to_string()),
                        protocol_version: "v5".to_string(),
                        enabled: true,
                        timeout_seconds: 30,
                        auto_reconnect: true,
                        max_reconnect_attempts: 5,
                    },
                    ObsConnectionConfig {
                        name: "OBS_STR".to_string(),
                        host: "localhost".to_string(),
                        port: 4466,
                        password: Some("cekPIbj@245".to_string()),
                        protocol_version: "v5".to_string(),
                        enabled: true,
                        timeout_seconds: 30,
                        auto_reconnect: true,
                        max_reconnect_attempts: 5,
                    },
                ],
                defaults: ObsDefaultSettings {
                    default_host: "localhost".to_string(),
                    default_port: 4455,
                    default_protocol: "v5".to_string(),
                    default_timeout: 30,
                },
                behavior: ObsBehaviorSettings {
                    auto_reconnect: true,
                    reconnect_delay: 5,
                    max_attempts: 5,
                    status_monitoring: true,
                    status_interval: 10,
                },
                integration: ObsIntegrationSettings {
                    auto_connect_on_startup: false,
                    show_status_in_overlay: false,
                    auto_record_on_clip_play: false,
                    save_replay_buffer_on_clip_creation: false,
                },
            },
            udp: UdpSettings {
                        listener: UdpListenerSettings {
            port: 8888,
            bind_address: "127.0.0.1".to_string(),
            enabled: false,
            buffer_size: 8192,
            timeout_seconds: 30,
            network_interface: NetworkInterfaceSettings::default(),
        },
                pss: PssSettings {
                    schema_file: "protocol/pss_v2.3.txt".to_string(),
                    enabled: true,
                    strict_mode: false,
                    unknown_fields: "warn".to_string(),
                    protocol_versions: ProtocolVersionSettings {
                        active_version: "2.3".to_string(),
                        available_versions: vec![
                            ProtocolVersion {
                                version: "2.3".to_string(),
                                filename: "pss_v2.3.json".to_string(),
                                file_path: "protocol/pss_v2.3.json".to_string(),
                                description: "Version 2.3 of the PSS protocol".to_string(),
                                created_date: chrono::Utc::now().to_rfc3339(),
                                last_modified: chrono::Utc::now().to_rfc3339(),
                                is_active: true,
                                file_size: 12345,
                                checksum: Some("abc123".to_string()),
                            },
                            ProtocolVersion {
                                version: "2.4".to_string(),
                                filename: "pss_v2.4.json".to_string(),
                                file_path: "protocol/pss_v2.4.json".to_string(),
                                description: "Version 2.4 of the PSS protocol".to_string(),
                                created_date: chrono::Utc::now().to_rfc3339(),
                                last_modified: chrono::Utc::now().to_rfc3339(),
                                is_active: false,
                                file_size: 12346,
                                checksum: Some("def456".to_string()),
                            },
                        ],
                        auto_update: true,
                        validate_on_load: true,
                    },
                    storage: ProtocolStorageSettings {
                        storage_directory: "protocol_files".to_string(),
                        max_versions: 5,
                        auto_backup: true,
                        backup_directory: "protocol_backups".to_string(),
                        compression: true,
                        retention_days: 30,
                    },
                },
                events: EventSettings {
                    store_events: true,
                    max_events: 10000,
                    cleanup_interval_hours: 24,
                    enable_filtering: false,
                    default_filters: vec![],
                },
            },
            logging: LoggingSettings {
                global: GlobalLoggingSettings {
                    level: "info".to_string(),
                    console_enabled: true,
                    file_enabled: true,
                    include_timestamp: true,
                    include_subsystem: true,
                },
                subsystems: HashMap::new(),
                files: LogFileSettings {
                    directory: "log".to_string(),
                    max_size_mb: 100,
                    max_files: 10,
                    retention_days: 30,
                    compression: true,
                    archive_enabled: true,
                },
                live_data: LiveDataSettings {
                    enabled: false,
                    interval_seconds: 5,
                    max_data_points: 1000,
                    real_time: false,
                },
            },
            ui: UiSettings {
                overlay: OverlaySettings {
                    visible: true,
                    opacity: 0.9,
                    position: "bottom-right".to_string(),
                    scale: 1.0,
                    theme: "dark".to_string(),
                    always_on_top: true,
                    click_through: false,
                },
                theme: ThemeSettings {
                    current: "dark".to_string(),
                    auto_theme: false,
                    custom_colors: HashMap::new(),
                    high_contrast: false,
                },
                layout: LayoutSettings {
                    sidebar_position: "left".to_string(),
                    sidebar_width: 300,
                    status_bar_visible: true,
                    task_bar_visible: true,
                    window_size: WindowSize { width: 1200, height: 800 },
                    window_position: WindowPosition { x: 100, y: 100 },
                },
                animations: AnimationSettings {
                    enabled: true,
                    duration_ms: 300,
                    easing: "ease-in-out".to_string(),
                    reduce_motion: false,
                },
            },
            video: VideoSettings {
                player: VideoPlayerSettings {
                    default_player: "mpv".to_string(),
                    player_path: None,
                    player_args: vec![],
                    auto_hide_ui: true,
                    fullscreen: false,
                },
                replay: ReplaySettings {
                    default_duration: 30,
                    buffer_size_mb: 500,
                    auto_save: false,
                    quality: "medium".to_string(),
                },
                clips: ClipSettings {
                    storage_directory: "clips".to_string(),
                    max_size_mb: 1000,
                    retention_days: 90,
                    auto_organize: true,
                    naming_format: "{date}_{time}_{event}".to_string(),
                },
            },
            license: LicenseSettings {
                license_key: None,
                status: "trial".to_string(),
                expiration_date: None,
                auto_renewal: false,
                offline_grace_days: 7,
                last_validation: None,
            },
            flags: FlagSettings {
                storage: FlagStorageSettings {
                    directory: "ui/public/assets/flags".to_string(),
                    auto_download: false,
                    update_interval_days: 30,
                    max_size_kb: 100,
                },
                recognition: FlagRecognitionSettings {
                    auto_recognition: false,
                    confidence_threshold: 0.8,
                    database_path: "scripts/media/flag_database.json".to_string(),
                    machine_learning: false,
                },
                display: FlagDisplaySettings {
                    size: 32,
                    quality: "medium".to_string(),
                    show_names: true,
                    animation: false,
                },
            },
            advanced: AdvancedSettings {
                development: DevelopmentSettings {
                    debug_mode: false,
                    verbose_logging: false,
                    hot_reload: false,
                    dev_server_port: 3000,
                },
                network: NetworkSettings {
                    proxy: None,
                    connection_timeout: 30,
                    retry_attempts: 3,
                    enable_ipv6: false,
                },
                security: SecuritySettings {
                    ssl_verification: true,
                    allow_insecure: false,
                    certificate_validation: true,
                    security_level: "medium".to_string(),
                },
                experimental: ExperimentalSettings {
                    enabled: false,
                    enabled_features: vec![],
                    beta_updates: false,
                    telemetry: false,
                },
            },
        }
    }
} 