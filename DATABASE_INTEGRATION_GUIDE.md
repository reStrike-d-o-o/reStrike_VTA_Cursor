# Database Integration Guide - reStrike VTA

## Overview

This document provides a comprehensive guide to the database structure, models, relationships, and integration patterns for the reStrike VTA project. The database system is built on SQLite with a sophisticated migration system, supporting multiple subsystems including PSS events, UDP server management, flag management, and UI settings.

## 🏗️ Database Architecture

### **Technology Stack**
- **Database Engine**: SQLite 3 via `rusqlite`
- **Migration System**: Custom migration framework with version tracking
- **Connection Management**: Async connection pooling with `tokio::sync::Mutex`
- **Error Handling**: Custom `AppError` and `DatabaseResult` types
- **Integration**: Tauri v2 plugin architecture with frontend exposure

### **Current Schema Version**: 4
- **Migration 1**: Initial schema (PSS events, OBS connections, app config, flag mappings)
- **Migration 2**: Normalized settings schema (categories, keys, values, history)
- **Migration 3**: Comprehensive flag management system (253+ IOC flags)
- **Migration 4**: PSS and UDP subsystem integration with normalization

---

## 📊 Database Schema Overview

### **Core Tables**

#### **1. Schema Management**
- `schema_version` - Migration tracking and version history

#### **2. Settings System**
- `settings_categories` - Settings organization categories
- `settings_keys` - Setting definitions with validation rules
- `settings_values` - Current setting values
- `settings_history` - Audit trail for setting changes

#### **3. Flag Management System**
- `flags` - Flag file metadata and recognition status
- `flag_mappings` - IOC to PSS code mappings (253+ entries)
- `recognition_history` - Flag recognition attempt tracking

#### **4. Network & UDP System**
- `network_interfaces` - System network interface detection
- `udp_server_configs` - UDP server configuration settings
- `udp_server_sessions` - Runtime session tracking and statistics
- `udp_client_connections` - Client connection monitoring

#### **5. PSS Event System**
- `pss_event_types` - Normalized event type definitions
- `pss_matches` - Match information and configuration
- `pss_athletes` - Athlete information and metadata
- `pss_match_athletes` - Match-athlete relationships
- `pss_rounds` - Round information and timing
- `pss_events_v2` - Enhanced event storage with relationships
- `pss_event_details` - Event-specific data storage
- `pss_scores` - Score tracking and history
- `pss_warnings` - Warning/gam-jeom tracking

#### **6. Legacy Tables**
- `pss_events` - Original event storage (deprecated)
- `obs_connections` - OBS WebSocket connection configurations
- `app_config` - Application configuration (deprecated)

---

## 🔗 Entity Relationships

### **PSS Event System Relationships**
```
pss_events_v2
├── session_id → udp_server_sessions.id
├── match_id → pss_matches.id
├── round_id → pss_rounds.id
└── event_type_id → pss_event_types.id

pss_event_details
└── event_id → pss_events_v2.id

pss_match_athletes
├── match_id → pss_matches.id
└── athlete_id → pss_athletes.id

pss_rounds
└── match_id → pss_matches.id

pss_scores
├── match_id → pss_matches.id
└── round_id → pss_rounds.id

pss_warnings
├── match_id → pss_matches.id
└── round_id → pss_rounds.id
```

### **UDP Server System Relationships**
```
udp_server_configs
└── network_interface_id → network_interfaces.id

udp_server_sessions
└── server_config_id → udp_server_configs.id

udp_client_connections
└── session_id → udp_server_sessions.id
```

### **Settings System Relationships**
```
settings_keys
└── category_id → settings_categories.id

settings_values
└── key_id → settings_keys.id

settings_history
└── key_id → settings_keys.id
```

### **Flag Management Relationships**
```
recognition_history
└── flag_id → flags.id

pss_athletes
└── flag_id → flags.id
```

---

## 🗃️ Data Models

### **Core Models**

#### **PSS Event System**
```rust
// Enhanced PSS Event with normalized relationships
pub struct PssEventV2 {
    pub id: Option<i64>,
    pub session_id: i64,
    pub match_id: Option<i64>,
    pub round_id: Option<i64>,
    pub event_type_id: i64,
    pub timestamp: DateTime<Utc>,
    pub raw_data: String,
    pub parsed_data: Option<String>,
    pub event_sequence: i32,
    pub processing_time_ms: Option<i32>,
    pub is_valid: bool,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

// PSS Match information
pub struct PssMatch {
    pub id: Option<i64>,
    pub match_id: String,
    pub match_number: Option<i32>,
    pub category: Option<String>,
    pub weight_class: Option<String>,
    pub division: Option<String>,
    pub total_rounds: i32,
    pub round_duration: Option<i32>,
    pub countdown_type: Option<String>,
    pub format_type: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// PSS Athlete information
pub struct PssAthlete {
    pub id: Option<i64>,
    pub athlete_code: String,
    pub short_name: String,
    pub long_name: Option<String>,
    pub country_code: Option<String>,
    pub flag_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### **UDP Server System**
```rust
// Network interface detection
pub struct NetworkInterface {
    pub id: Option<i64>,
    pub name: String,
    pub address: String,
    pub netmask: Option<String>,
    pub broadcast: Option<String>,
    pub is_loopback: bool,
    pub is_active: bool,
    pub is_recommended: bool,
    pub speed_mbps: Option<i32>,
    pub mtu: Option<i32>,
    pub mac_address: Option<String>,
    pub interface_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// UDP server configuration
pub struct UdpServerConfig {
    pub id: Option<i64>,
    pub name: String,
    pub port: u16,
    pub bind_address: String,
    pub network_interface_id: Option<i64>,
    pub enabled: bool,
    pub auto_start: bool,
    pub max_packet_size: i32,
    pub buffer_size: i32,
    pub timeout_ms: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// UDP server session tracking
pub struct UdpServerSession {
    pub id: Option<i64>,
    pub server_config_id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: String,
    pub packets_received: i32,
    pub packets_parsed: i32,
    pub parse_errors: i32,
    pub total_bytes_received: i32,
    pub average_packet_size: f64,
    pub max_packet_size_seen: i32,
    pub min_packet_size_seen: i32,
    pub unique_clients_count: i32,
    pub error_message: Option<String>,
}
```

#### **Flag Management System**
```rust
// Flag mapping for PSS to IOC code mapping
pub struct FlagMapping {
    pub id: Option<i64>,
    pub pss_code: String,
    pub ioc_code: String,
    pub country_name: String,
    pub is_custom: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Flag file metadata
pub struct Flag {
    pub id: Option<i64>,
    pub filename: String,
    pub ioc_code: Option<String>,
    pub country_name: Option<String>,
    pub recognition_status: String,
    pub recognition_confidence: Option<f64>,
    pub upload_date: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub file_size: Option<i64>,
    pub file_path: String,
    pub is_recognized: bool,
}
```

#### **Settings System**
```rust
// Settings category organization
pub struct SettingsCategory {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
}

// Settings key definition
pub struct SettingsKey {
    pub id: Option<i64>,
    pub category_id: i64,
    pub key_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub data_type: String,
    pub default_value: Option<String>,
    pub validation_rules: Option<String>,
    pub is_required: bool,
    pub is_sensitive: bool,
    pub created_at: DateTime<Utc>,
}

// Settings value storage
pub struct SettingsValue {
    pub id: Option<i64>,
    pub key_id: i64,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

## 🔧 Database Operations

### **Core Operations Structure**

#### **Database Plugin (`plugin_database.rs`)**
```rust
pub struct DatabasePlugin {
    connection: Arc<DatabaseConnection>,
    migration_strategy: MigrationStrategy,
    hybrid_provider: Arc<Mutex<HybridSettingsProvider>>,
}
```

#### **Operations Classes**
- `UiSettingsOperations` - UI settings management
- `PssUdpOperations` - PSS and UDP subsystem operations

### **Key Operation Patterns**

#### **1. Connection Management**
```rust
// Get database connection
let conn = self.connection.get_connection().await?;

// Use in transaction
let tx = conn.transaction()?;
// ... operations ...
tx.commit()?;
```

#### **2. Error Handling**
```rust
// Convert database errors to AppError
.map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get network interfaces: {}", e)))
```

#### **3. Async Operations**
```rust
// Async database operations
pub async fn get_network_interfaces(&self) -> AppResult<Vec<NetworkInterface>> {
    let conn = self.connection.get_connection().await
        .map_err(|e| AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
    PssUdpOperations::get_network_interfaces(&*conn)
        .map_err(|e| AppError::ConfigError(format!("Failed to get network interfaces: {}", e)))
}
```

---

## 🚀 Frontend Integration

### **Tauri Commands**

#### **Database Management Commands**
```rust
// UI Settings
#[tauri::command]
pub async fn db_initialize_ui_settings(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String>

#[tauri::command]
pub async fn db_get_ui_setting(key: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, String>

#[tauri::command]
pub async fn db_set_ui_setting(key: String, value: String, changed_by: String, change_reason: Option<String>, app: State<'_, Arc<App>>) -> Result<serde_json::Value, String>

// Flag Management
#[tauri::command]
pub async fn get_flag_mappings_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String>

#[tauri::command]
pub async fn scan_and_populate_flags(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String>

#[tauri::command]
pub async fn get_flags_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String>

#[tauri::command]
pub async fn clear_flags_table(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String>
```

#### **Frontend Usage**
```typescript
// Get flag mappings
const flagMappings = await window.__TAURI__.core.invoke('get_flag_mappings_data');

// Set UI setting
await window.__TAURI__.core.invoke('db_set_ui_setting', {
  key: 'window.position.x',
  value: '100',
  changed_by: 'user',
  change_reason: 'Manual adjustment'
});
```

---

## 📈 Migration System

### **Migration Framework**
```rust
pub trait Migration: Send + Sync {
    fn version(&self) -> u32;
    fn description(&self) -> &str;
    fn up(&self, conn: &Connection) -> SqliteResult<()>;
    fn down(&self, conn: &Connection) -> SqliteResult<()>;
}
```

### **Migration Manager**
```rust
pub struct MigrationManager {
    migrations: Vec<Box<dyn Migration>>,
}

impl MigrationManager {
    pub fn migrate(&self, conn: &Connection) -> DatabaseResult<()> {
        // Apply pending migrations in order
    }
    
    pub fn rollback(&self, conn: &Connection, target_version: u32) -> DatabaseResult<()> {
        // Rollback to specific version
    }
}
```

### **Automatic Migration**
```rust
// Database plugin automatically runs migrations on startup
tokio::spawn(async move {
    if let Err(e) = Self::run_migrations_internal(connection_clone).await {
        log::error!("Failed to run database migrations: {}", e);
    }
});
```

---

## 🔍 Performance Optimization

### **Indexing Strategy**
```sql
-- Event querying
CREATE INDEX idx_pss_events_v2_timestamp ON pss_events_v2(timestamp);
CREATE INDEX idx_pss_events_v2_match ON pss_events_v2(match_id, round_id);
CREATE INDEX idx_pss_events_v2_session ON pss_events_v2(session_id, event_sequence);

-- Settings queries
CREATE INDEX idx_settings_keys_category ON settings_keys(category_id);
CREATE INDEX idx_settings_keys_name ON settings_keys(key_name);

-- Flag lookups
CREATE INDEX idx_flag_mappings_pss_code ON flag_mappings(pss_code);
CREATE INDEX idx_flag_mappings_ioc_code ON flag_mappings(ioc_code);

-- Network interface selection
CREATE INDEX idx_network_interfaces_active ON network_interfaces(is_active, is_recommended);
```

### **Caching Strategy**
```rust
// In-memory caches for frequently accessed data
pub struct UdpServer {
    athlete_cache: Arc<Mutex<std::collections::HashMap<String, i64>>>,
    event_type_cache: Arc<Mutex<std::collections::HashMap<String, i64>>>,
}
```

---

## 🛡️ Data Integrity

### **Foreign Key Constraints**
```sql
-- PSS events relationships
FOREIGN KEY (session_id) REFERENCES udp_server_sessions(id)
FOREIGN KEY (match_id) REFERENCES pss_matches(id)
FOREIGN KEY (round_id) REFERENCES pss_rounds(id)
FOREIGN KEY (event_type_id) REFERENCES pss_event_types(id)

-- Settings relationships
FOREIGN KEY (category_id) REFERENCES settings_categories(id)
FOREIGN KEY (key_id) REFERENCES settings_keys(id)
```

### **Unique Constraints**
```sql
-- Prevent duplicate entries
UNIQUE(match_id, athlete_position)
UNIQUE(event_id, detail_key)
UNIQUE(session_id, client_address, client_port)
```

### **Validation Rules**
```rust
// Settings validation
pub struct SettingsKey {
    pub validation_rules: Option<String>, // JSON validation rules
    pub data_type: String, // string, integer, boolean, float, json
    pub is_required: bool,
}
```

---

## 📊 Statistics and Analytics

### **UDP Server Statistics**
```rust
pub async fn get_udp_server_statistics(&self) -> AppResult<serde_json::Value> {
    // Returns comprehensive statistics including:
    // - Total sessions, active sessions
    // - Total events, total matches
    // - Recent activity (last 24 hours)
}
```

### **Database Health Monitoring**
```rust
// Database accessibility check
pub async fn is_accessible(&self) -> bool

// File size monitoring
pub fn get_file_size(&self) -> AppResult<u64>

// Migration status
pub async fn get_migration_status(&self) -> AppResult<MigrationStatus>
```

---

## 🔄 Backup and Recovery

### **JSON Backup System**
```rust
// Create JSON backup
pub async fn create_json_backup(&self) -> AppResult<String>

// Restore from backup
pub async fn restore_from_json_backup(&self, backup_path: &str) -> AppResult<()>

// Google Drive integration
pub async fn drive_upload_backup_archive() -> Result<serde_json::Value, String>
```

### **Migration Rollback**
```rust
// Rollback to specific version
pub fn rollback(&self, conn: &Connection, target_version: u32) -> DatabaseResult<()>
```

---

## 🎯 Task Management Integration

### **Database Tasks for Development**

#### **High Priority Tasks**
1. **Database Performance Optimization**
   - Analyze query performance
   - Optimize indexes for common queries
   - Implement connection pooling improvements

2. **Data Validation Enhancement**
   - Add comprehensive validation rules
   - Implement data integrity checks
   - Add constraint validation

3. **Backup System Enhancement**
   - Implement automated backup scheduling
   - Add backup verification
   - Enhance Google Drive integration

#### **Medium Priority Tasks**
1. **Analytics Dashboard**
   - Create comprehensive statistics views
   - Implement real-time monitoring
   - Add performance metrics

2. **Data Export/Import**
   - Implement CSV/JSON export
   - Add bulk data import
   - Create data migration tools

3. **Advanced Querying**
   - Implement complex event queries
   - Add temporal data analysis
   - Create reporting functions

#### **Low Priority Tasks**
1. **Database Documentation**
   - Create ERD diagrams
   - Document query patterns
   - Add performance guidelines

2. **Testing Infrastructure**
   - Add database unit tests
   - Implement integration tests
   - Create performance benchmarks

---

## 📚 Best Practices

### **Development Guidelines**

#### **1. Model Design**
- Always include `created_at` and `updated_at` timestamps
- Use `Option<T>` for nullable fields
- Implement `from_row` methods for database mapping
- Use proper foreign key relationships

#### **2. Error Handling**
- Convert database errors to `AppError::ConfigError`
- Provide meaningful error messages
- Log errors with context
- Handle connection failures gracefully

#### **3. Performance**
- Use transactions for multiple operations
- Implement proper indexing
- Cache frequently accessed data
- Monitor query performance

#### **4. Migration Management**
- Always test migrations before deployment
- Include rollback procedures
- Document migration changes
- Version control migration files

#### **5. Frontend Integration**
- Use async/await for database operations
- Handle loading and error states
- Implement proper error boundaries
- Cache data appropriately

---

## 🔮 Future Enhancements

### **Planned Features**

#### **1. Advanced Analytics**
- Real-time event analysis
- Performance trend tracking
- Predictive analytics
- Custom reporting

#### **2. Data Synchronization**
- Multi-device sync
- Cloud backup integration
- Real-time collaboration
- Conflict resolution

#### **3. Advanced Querying**
- Full-text search
- Complex event filtering
- Temporal queries
- Aggregation functions

#### **4. Performance Optimization**
- Query optimization
- Connection pooling
- Read replicas
- Caching layers

---

## 📞 Support and Maintenance

### **Troubleshooting**

#### **Common Issues**
1. **Migration Failures**
   - Check schema version table
   - Verify migration files
   - Review error logs

2. **Performance Issues**
   - Analyze slow queries
   - Check index usage
   - Monitor connection pool

3. **Data Integrity**
   - Verify foreign key constraints
   - Check for orphaned records
   - Validate data types

#### **Maintenance Tasks**
1. **Regular Backups**
   - Daily automated backups
   - Weekly manual verification
   - Monthly archive cleanup

2. **Performance Monitoring**
   - Query performance tracking
   - Index usage analysis
   - Connection pool monitoring

3. **Data Cleanup**
   - Remove old sessions
   - Archive historical data
   - Clean up temporary files

---

**Last Updated**: 2025-01-29  
**Schema Version**: 4  
**Status**: Production Ready with Comprehensive Integration