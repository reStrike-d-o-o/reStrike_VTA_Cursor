# Data Flow Architecture

## Overview

The reStrike VTA data flow architecture provides comprehensive real-time event processing, multi-protocol integration, and seamless communication between frontend and backend systems. The architecture is designed for high performance, reliability, and extensibility in sports broadcasting and event management scenarios.

## System Architecture Layers

### Layer Overview
```
┌─────────────────────────────────────────────────────────────┐
│                    Presentation Layer                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   React UI  │  │  DockBar    │  │ Advanced    │        │
│  │ Components  │  │  Sidebar    │  │   Panel     │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                     Bridge Layer                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   Tauri     │  │   Event     │  │   Command   │        │
│  │   API       │  │   System    │  │   System    │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Application Layer                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   Plugin    │  │   Core      │  │   State     │        │
│  │   System    │  │   Logic     │  │ Management  │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                     Data Layer                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   SQLite    │  │   Cache     │  │   File      │        │
│  │  Database   │  │   System    │  │   System    │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

## Event Table Data Flow

### Overview
The Event Table data flow handles real-time PSS event display with intelligent time and round tracking, manual override detection, and automatic event management.

### Recent Improvements (2025-01-29)

#### **Event Table Time & Round Display Fixes**
- **Backend Hardcoded Values**: Removed hardcoded "2:00" and round "1" values from JSON event creation
- **WebSocket-Only Processing**: Ensured all events go through WebSocket plugin for proper state management
- **Time Accuracy**: Event Table now displays actual PSS event times instead of fallback values
- **Round Display**: Added 'RND' to importantEventCodes array to display round events

#### **Event Table Management**
- **Automatic Clearing**: Event Table clears automatically on `rdy;FightReady` events
- **Counter Behavior**: Verified correct behavior (Round/Time preserved, Total/Table reset)
- **Clean UI**: Removed manual clear buttons, keeping only automatic clearing

#### **Manual Override Detection System**
- **Event Sequence Tracking**: Replaced time-based threshold with event sequence analysis
- **Break Event Exception**: Round changes after `brk;0:00;stopEnd` are NOT manual override
- **Normal Pattern**: `brk;0:00;stopEnd` → `rnd;3` → `clk;02:00;start`

### Event Table Data Flow

#### **Complete Event Processing Flow**
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   PSS       │───▶│  WebSocket  │───▶│  Live Data  │───▶│  Event      │
│  Events     │    │   Plugin    │    │   Store     │    │   Table     │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │                   │
       ▼                   ▼                   ▼                   ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ UDP Server  │    │ State       │    │ Event       │    │ Real-time   │
│ Parsing     │    │ Management  │    │ Filtering   │    │ Display     │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

#### **Step-by-Step Event Table Process**

1. **PSS Event Reception**
   ```rust
   // UDP server receives and parses PSS events
   let event = PssEvent::from_bytes(&buffer[..len])?;
   
   // Convert to WebSocket message (no hardcoded values)
   let ws_message = convert_pss_event_to_ws_message(&event);
   ```

2. **WebSocket Broadcasting**
   ```rust
   // Broadcast to all connected clients
   websocket_plugin.broadcast_message(ws_message)?;
   ```

3. **Frontend Event Processing**
   ```typescript
   // WebSocket event received
   if (data.type === 'pss_event' && data.data) {
     const eventData = data.data;
     
     // Handle fight_ready event - clear Event Table
     if (eventData.event_type === 'fight_ready') {
       useLiveDataStore.getState().clearEvents();
       return;
     }
     
     // Create and add event to store
     const event: PssEventData = {
       // ... event creation with current store time/round
     };
     useLiveDataStore.getState().addEvent(event);
   }
   ```

4. **Event Table Display**
   ```typescript
   // Filter important events for display
   const filteredEvents = allEvents.filter(event => {
     const importantEventCodes = ['K', 'P', 'H', 'TH', 'TB', 'R', 'RND'];
     return importantEventCodes.includes(event.eventCode);
   });
   ```

### Manual Override Detection Flow

#### **Scoreboard Overlay Manual Override Detection**
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Clock     │───▶│  Manual     │───▶│  Event      │───▶│  Override   │
│  Events     │    │  Override   │    │  Sequence   │    │  Detection  │
│             │    │  State      │    │  Tracking   │    │             │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │                   │
       ▼                   ▼                   ▼                   ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ clk;{};stop │    │ manualOver- │    │ eventsAfter │    │ Normal vs   │
│ clk;{};start│    │ rideActive  │    │ BreakStopEnd│    │ Manual      │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

#### **Manual Override Detection Logic**
```javascript
// Check if we're in manual override mode
function isInManualOverrideMode() {
  return manualOverrideState.clockState === 'stopped' && 
         manualOverrideState.manualOverrideActive;
}

// Check for break event exception
function isRoundChangeAfterBreakStopEnd() {
  if (!manualOverrideState.lastBreakStopEnd) return false;
  
  // Check if there are no other events between brk;0:00;stopEnd and rnd
  const hasOnlyRoundEvent = manualOverrideState.eventsAfterBreakStopEnd.length === 0;
  return hasOnlyRoundEvent;
}

// Manual round change detection
function isManualRoundChange(event) {
  if (!event || event.type !== 'round') return false;
  if (!isInManualOverrideMode()) return false;
  
  // Exception: Round changes after break stopEnd are NOT manual override
  if (isRoundChangeAfterBreakStopEnd()) return false;
  
  return true; // Manual override detected
}
```

### Data Flow Integration

#### **Event Table Integration Points**
- **WebSocket Plugin**: Primary event source for real-time display
- **Live Data Store**: Central state management for events and counters
- **Event Table Component**: Real-time display with filtering and auto-scroll
- **Scoreboard Overlay**: Manual override detection and exception handling

#### **State Management Flow**
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  PSS        │───▶│  WebSocket  │───▶│  Live Data  │
│  Events     │    │   Plugin    │    │   Store     │
└─────────────┘    └─────────────┘    └─────────────┘
                           │                   │
                           ▼                   ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Scoreboard │◀───│  Manual     │◀───│  Event      │
│  Overlay    │    │  Override   │    │  Table      │
│             │    │  Detection  │    │  Component  │
└─────────────┘    └─────────────┘    └─────────────┘
```

## Detailed Data Flow

### PSS Event Processing Flow

#### Complete Event Lifecycle
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   UDP       │───▶│   PSS       │───▶│  Database  │───▶│   Frontend  │
│  Server     │    │  Parser     │    │   Storage   │    │    UI       │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │                   │
       ▼                   ▼                   ▼                   ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ Network     │    │ Event       │    │ Cache       │    │ Real-time   │
│ Interface   │    │ Validation  │    │ Update      │    │ Display     │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

#### Step-by-Step Process

1. **UDP Reception**
   ```rust
   // UDP server receives PSS datagrams
   let mut buffer = [0; 8192];
   let (len, addr) = socket.recv_from(&mut buffer).await?;
   
   // Parse PSS event
   let event = PssEvent::from_bytes(&buffer[..len])?;
   ```

2. **Event Parsing and Validation**
   ```rust
   // Parse and validate PSS event
   let parsed_event = match event.event_type {
       "match_config" => parse_match_config(&event.data)?,
       "athletes" => parse_athletes(&event.data)?,
       "current_scores" => parse_scores(&event.data)?,
       "warnings" => parse_warnings(&event.data)?,
       _ => return Err(AppError::ValidationError("Unknown event type".to_string())),
   };
   ```

3. **Database Storage**
   ```rust
   // Store event in database
   let event_id = database.create_pss_event(parsed_event).await?;
   
   // Update caches
   update_event_type_cache(&event.event_type, event_id).await?;
   update_athlete_cache(&event.athlete_data).await?;
   ```

4. **Frontend Notification**
   ```rust
   // Emit event to frontend
   app_handle.emit_all("pss_event", event_data)?;
   ```

## Enhanced PSS Event System

### Status Mark System

#### Recognition Status Values
- **`recognized`**: Event is fully understood and parsed correctly
- **`unknown`**: Event format is not recognized or validation failed
- **`partial`**: Event partially parsed but some fields unknown
- **`deprecated`**: Event type is no longer used in current protocol

#### Status Tracking
- Automatic status assignment based on parsing and validation results
- Status change history tracking in `pss_event_recognition_history` table
- Ability to update status manually for protocol evolution

### Event Validation

#### Protocol Compliance
- Validation against PSS v2.3 specification
- Range checking for numeric values
- Format validation for time strings
- Required field validation
- Custom validation rules

#### Validation Rules
The system includes predefined validation rules for:
- Point types (1-5)
- Hit levels (1-100)
- Warning counts (0-4)
- Round numbers (1-3)
- Time formats (m:ss)
- Required fields

### Unknown Event Collection

#### Automatic Collection
- All unrecognized events are stored in `pss_unknown_events` table
- Pattern analysis for unknown events
- Occurrence counting and tracking
- Support for future protocol updates

#### Unknown Event Analysis
- Raw data storage for debugging
- Pattern hashing for similarity detection
- Suggested event type classification
- Notes and metadata for analysis

### Enhanced Event Details

#### Hit Level Tracking
- Automatic linking of hit levels with point events
- Time-window based hit level collection (5 seconds)
- Statistical analysis (max, average hit levels)
- Storage of all hit levels regardless of point events

#### Processing Metadata
- Processing time tracking
- Protocol version used for parsing
- Parser confidence scores
- Validation error details

### Database Schema Enhancements

#### New Tables
- `pss_event_recognition_history`: Status change tracking
- `pss_unknown_events`: Unknown event collection
- `pss_event_validation_rules`: Protocol validation rules
- `pss_event_validation_results`: Validation result storage
- `pss_event_statistics`: Event processing metrics

#### Enhanced Fields
- `recognition_status`: Event recognition status
- `protocol_version`: Protocol version used
- `parser_confidence`: Confidence score (0.0-1.0)
- `validation_errors`: Validation error details
- `processing_time_ms`: Processing time tracking

### Implementation Details

#### Database Operations

##### PssEventStatusOperations
```rust
// Store event with status
store_pss_event_with_status(conn, event) -> i64

// Update recognition status
update_event_recognition_status(conn, event_id, new_status, changed_by, reason) -> ()

// Store unknown event
store_unknown_event(conn, unknown_event) -> i64

// Get validation rules
get_validation_rules(conn, event_code, protocol_version) -> Vec<PssEventValidationRule>
```

#### Event Processing Pipeline
```rust
// Enhanced event processing with status tracking
pub async fn process_pss_event(event: PssEvent) -> AppResult<()> {
    // 1. Parse event
    let parsed_event = parse_pss_event(&event)?;
    
    // 2. Validate against rules
    let validation_result = validate_event(&parsed_event)?;
    
    // 3. Determine recognition status
    let status = determine_recognition_status(&parsed_event, &validation_result);
    
    // 4. Store with status
    let event_id = store_pss_event_with_status(&parsed_event, status).await?;
    
    // 5. Update statistics
    update_event_statistics(event_id, &validation_result).await?;
    
    Ok(())
}
```

#### Real-time Event Broadcasting
```rust
// Enhanced WebSocket message with action field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub event_type: String,
    pub event_code: String,
    pub athlete: String,
    pub round: u8,
    pub time: String,
    pub timestamp: String,
    pub raw_data: String,
    pub description: String,
    pub action: Option<String>, // New field for injury events
}

// Broadcast with enhanced data
pub async fn broadcast_enhanced_event(event: &PssEvent) -> AppResult<()> {
    let message = WebSocketMessage {
        event_type: event.event_type.clone(),
        event_code: get_event_code(event),
        athlete: get_athlete_string(event),
        round: get_current_round(),
        time: get_event_time(event),
        timestamp: Utc::now().to_rfc3339(),
        raw_data: event.raw_data.clone(),
        description: generate_description(event),
        action: get_action_field(event), // Include action for injury events
    };
    
         websocket_server.broadcast_message(message).await?;
     Ok(())
 }
```

## Hit Level Tracking System

### Overview
The hit level tracking system monitors hit level events (`hl1`, `hl2`) and links them with subsequent point events (`pt1`, `pt2`) for statistical analysis. This provides insights into the relationship between hit intensity and scoring.

### Implementation Details

#### Data Structure
```rust
recent_hit_levels: Arc<Mutex<std::collections::HashMap<u8, Vec<(u8, std::time::SystemTime)>>>>, // athlete -> [(level, timestamp)]
```

This stores:
- **Key**: Athlete number (1 or 2)
- **Value**: Vector of tuples containing (hit_level, timestamp)
- **Limit**: Maximum 10 hit levels per athlete to prevent memory bloat

#### Hit Level Tracking Logic
```rust
// Track hit level events for statistics
match &event {
    PssEvent::HitLevel { athlete, level } => {
        // Track this hit level for potential linking with point events
        let mut hit_levels = recent_hit_levels.lock().unwrap();
        let now = std::time::SystemTime::now();
        
        // Get or create the athlete's hit level history
        let athlete_hit_levels = hit_levels.entry(*athlete).or_insert_with(Vec::new);
        
        // Add the new hit level with timestamp
        athlete_hit_levels.push((*level, now));
        
        // Keep only the last 10 hit levels per athlete
        if athlete_hit_levels.len() > 10 {
            athlete_hit_levels.drain(0..athlete_hit_levels.len() - 10);
        }
        
        log::debug!("🎯 Tracked hit level for athlete {}: level {}", athlete, level);
    }
    PssEvent::FightLoaded | PssEvent::FightReady => {
        // Clear hit level tracking when a new fight starts
        let mut hit_levels = recent_hit_levels.lock().unwrap();
        hit_levels.clear();
        log::debug!("🧹 Cleared hit level tracking for new fight");
    }
    _ => {}
}
```

#### Enhanced Event Details
```rust
PssEvent::Points { athlete, point_type } => {
    let mut details = vec![
        ("athlete".to_string(), Some(athlete.to_string()), "u8".to_string()),
        ("point_type".to_string(), Some(point_type.to_string()), "u8".to_string()),
    ];
    
    // Add recent hit levels for this athlete (within last 5 seconds)
    let hit_levels_data = recent_hit_levels.lock().unwrap();
    if let Some(athlete_hit_levels) = hit_levels_data.get(athlete) {
        let now = std::time::SystemTime::now();
        let time_window_ms = 5000; // 5 seconds
        
        // Filter hit levels within the time window
        let recent_hit_levels: Vec<u8> = athlete_hit_levels
            .iter()
            .filter_map(|(level, timestamp)| {
                if let Ok(duration) = now.duration_since(*timestamp) {
                    if duration.as_millis() <= time_window_ms as u128 {
                        Some(*level)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        
        if !recent_hit_levels.is_empty() {
            let hit_levels_str = recent_hit_levels.iter()
                .map(|level| level.to_string())
                .collect::<Vec<_>>()
                .join(",");
            details.push(("recent_hit_levels".to_string(), Some(hit_levels_str), "String".to_string()));
        }
    }
    
    details
}
```

### Statistical Analysis
- **Time-window based collection**: 5-second window for linking hit levels with points
- **Statistical analysis**: Max, average hit levels for each point event
- **Storage**: All hit levels stored regardless of point events for comprehensive analysis
- **Memory management**: Automatic cleanup to prevent memory bloat
```
   // Update UI in real-time
   frontend_store.update_events(event_data);
   ```

### OBS Integration and Dual Protocol Support

#### OBS WebSocket Dual-Protocol Architecture

The system supports both OBS WebSocket v4 and v5 protocols simultaneously:

```
┌─────────────────────────────────────────────────────────────┐
│                    OBS Integration Layer                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │   OBS v4    │  │   OBS v5    │  │  Protocol   │          │
│  │  Protocol   │  │  Protocol   │  │  Detector   │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Unified OBS Interface                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │ Connection  │  │   Scene     │  │ Recording   │          │
│  │ Management  │  │  Control    │  │  Control    │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
```

#### Protocol Differences Handled

**OBS WebSocket v4**
```json
// Request Format
{
  "request-type": "GetCurrentScene",
  "message-id": "uuid-here"
}

// Response Format
{
  "scene-name": "Scene Name",
  "is-recording": true
}
```

**OBS WebSocket v5**
```json
// Request Format
{
  "op": 6,
  "d": {
    "requestType": "GetCurrentProgramScene",
    "requestId": "uuid-here"
  }
}

// Response Format
{
  "requestStatus": {
    "result": true,
    "code": 100
  },
  "responseData": {
    "sceneName": "Scene Name",
    "outputActive": true
  }
}
```

#### Dual-Protocol Implementation

```rust
// Protocol-agnostic OBS operations
impl ObsPlugin {
    pub async fn get_current_scene(&self, name: &str) -> AppResult<String> {
        let connection = self.get_connection(name)?;
        match connection.protocol_version {
            ObsWebSocketVersion::V4 => self.get_current_scene_v4(connection).await,
            ObsWebSocketVersion::V5 => self.get_current_scene_v5(connection).await,
        }
    }

    pub async fn set_current_scene(&self, name: &str, scene: &str) -> AppResult<()> {
        let connection = self.get_connection(name)?;
        match connection.protocol_version {
            ObsWebSocketVersion::V4 => self.set_current_scene_v4(connection, scene).await,
            ObsWebSocketVersion::V5 => self.set_current_scene_v5(connection, scene).await,
        }
    }

    pub async fn start_recording(&self, name: &str) -> AppResult<()> {
        let connection = self.get_connection(name)?;
        match connection.protocol_version {
            ObsWebSocketVersion::V4 => self.start_recording_v4(connection).await,
            ObsWebSocketVersion::V5 => self.start_recording_v5(connection).await,
        }
    }
}
```

#### Multiple OBS Instance Support

```rust
// Support for multiple OBS instances
pub struct ObsPlugin {
    connections: Arc<Mutex<HashMap<String, ObsConnection>>>,
    event_tx: UnboundedSender<ObsEvent>,
}

impl ObsPlugin {
    pub async fn add_connection(&mut self, config: ObsConnectionConfig) -> AppResult<()> {
        let connection = ObsConnection::new(config).await?;
        self.connections.lock().unwrap().insert(
            connection.name.clone(), 
            connection
        );
        Ok(())
    }

    pub async fn connect_obs(&mut self, name: &str) -> AppResult<()> {
        if let Some(connection) = self.connections.lock().unwrap().get_mut(name) {
            connection.connect().await?;
            self.emit_event(ObsEvent::ConnectionStatusChanged {
                connection_name: name.to_string(),
                status: ObsConnectionStatus::Connected,
            })?;
        }
        Ok(())
    }
}
```

#### OBS Event Handling

```rust
// Handle events from both protocol versions
async fn handle_obs_events(plugin: &ObsPlugin) {
    while let Some(event) = event_rx.recv().await {
        match event {
            ObsEvent::ConnectionStatusChanged { connection_name, status } => {
                log::info!("{}: {:?}", connection_name, status);
                // Update UI with connection status
                frontend_store.update_obs_connection_status(connection_name, status);
            }
            ObsEvent::SceneChanged { connection_name, scene_name } => {
                log::info!("{} switched to scene: {}", connection_name, scene_name);
                // Update UI with scene change
                frontend_store.update_current_scene(connection_name, scene_name);
            }
            ObsEvent::RecordingStateChanged { connection_name, is_recording } => {
                log::info!("{} recording: {}", connection_name, is_recording);
                // Update UI with recording status
                frontend_store.update_recording_status(connection_name, is_recording);
            }
            ObsEvent::Error { connection_name, error } => {
                log::error!("{} error: {}", connection_name, error);
                // Show error in UI
                frontend_store.add_error_notification(connection_name, error);
            }
        }
    }
}
```

---

## 💾 Database Operations Flow

### **1. Settings Management Flow**
```
Frontend Settings UI
        │
        ▼ Setting Change
┌─────────────────┐
│ Tauri Command   │ ← db_set_ui_setting
│                 │ ← Validation & Sanitization
└─────────────────┘
        │
        ▼
┌─────────────────┐
│ Database Plugin │ ← Settings Operations
│                 │ ← Category/Key/Value Storage
└─────────────────┘
        │
        ▼
┌─────────────────┐
│ Settings Tables │ ← Normalized Storage
│                 │ ← History Tracking
└─────────────────┘
        │
        ▼
┌─────────────────┐
│ Frontend Update │ ← Settings Store Update
│                 │ ← UI Configuration
└─────────────────┘
```

### **2. Flag Management Flow**
```
Flag Upload/Selection
        │
        ▼ Flag Operation
┌─────────────────┐
│ Tauri Commands  │ ← get_flag_mappings_data
│                 │ ← scan_and_populate_flags
└─────────────────┘
        │
        ▼
┌─────────────────┐
│ Database Plugin │ ← Flag Operations
│                 │ ← IOC Code Mapping
└─────────────────┘
        │
        ▼
┌─────────────────┐
│ Flag Tables     │ ← Flag Metadata
│                 │ ← Recognition History
└─────────────────┘
        │
        ▼
┌─────────────────┐
│ Frontend Update │ ← Flag Store Update
│                 │ ← PSS Code Mapping
└─────────────────┘
```

---

## 🔄 Real-time Event Flow

### **1. Live Data Streaming Flow**
```
System Monitoring
        │
        ▼ System Events
┌─────────────────┐
│ CPU Monitor     │ ← System Resource Monitoring
│                 │ ← Performance Metrics
└─────────────────┘
        │
        ▼
┌─────────────────┐
│ Event Emitter   │ ← Tauri Event System
│                 │ ← Real-time Broadcasting
└─────────────────┘
        │
        ▼
┌─────────────────┐
│ Frontend Hook   │ ← useLiveDataEvents
│                 │ ← State Management
└─────────────────┘
        │
        ▼
┌─────────────────┐
│ Live Data Panel │ ← Real-time Display
│                 │ ← Auto-scroll Controls
└─────────────────┘
```

### **2. Event Table Update Flow**
```
PSS Event Stream
    │
    ▼ New Event
┌─────────────────┐
│ Event Parser    │ ← Event Type Detection
│                 │ ← Data Extraction
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Event Store     │ ← Zustand State Update
│                 │ ← Event Filtering
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Event Table     │ ← Component Update
│                 │ ← Real-time Display
└─────────────────┘
```

---

## 🎯 State Management Flow

### **1. Frontend State Flow**
```
User Interaction
    │
    ▼ Action
┌─────────────────┐
│ Component       │ ← User Input
│                 │ ← Event Handler
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Zustand Store   │ ← State Update
│                 │ ← Action Dispatch
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Tauri Command   │ ← Backend Communication
│                 │ ← Data Persistence
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ UI Update       │ ← Component Re-render
│                 │ ← State Synchronization
└─────────────────┘
```

### **2. Backend State Flow**
```
System Event
    │
    ▼ Event
┌─────────────────┐
│ Plugin Handler  │ ← Event Processing
│                 │ ← State Validation
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Database Store  │ ← State Persistence
│                 │ ← Transaction Management
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Event Emitter   │ ← Frontend Notification
│                 │ ← Real-time Updates
└─────────────────┘
```

---

## 🔧 Error Handling Flow

### **1. Error Propagation Flow**
```
Error Source
    │
    ▼ Error
┌─────────────────┐
│ Error Handler   │ ← Error Classification
│                 │ ← Context Collection
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Error Converter │ ← AppError Conversion
│                 │ ← Message Formatting
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Error Logger    │ ← Structured Logging
│                 │ ← Error Tracking
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Frontend Error  │ ← Error Display
│                 │ ← User Notification
└─────────────────┘
```

### **2. Recovery Flow**
```
Error Detection
    │
    ▼ Recovery Action
┌─────────────────┐
│ Retry Logic     │ ← Automatic Retry
│                 │ ← Exponential Backoff
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Fallback Handler│ ← Alternative Path
│                 │ ← Graceful Degradation
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ State Recovery  │ ← State Restoration
│                 │ ← Consistency Check
└─────────────────┘
```

---

## 📊 Performance Monitoring Flow

### **1. System Metrics Flow**
```
System Resources
    │
    ▼ Metrics Collection
┌─────────────────┐
│ CPU Monitor     │ ← Resource Monitoring
│                 │ ← Performance Tracking
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Metrics Store   │ ← Data Aggregation
│                 │ ← Trend Analysis
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Performance UI  │ ← Real-time Display
│                 │ ← Alert System
└─────────────────┘
```

### **2. Database Performance Flow**
```
Database Operations
    │
    ▼ Query Execution
┌─────────────────┐
│ Query Monitor   │ ← Performance Tracking
│                 │ ← Slow Query Detection
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Performance Log │ ← Query Analysis
│                 │ ← Optimization Suggestions
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Admin Panel     │ ← Performance Dashboard
│                 │ ← Optimization Tools
└─────────────────┘
```

---

## 🔐 Security & Authentication Flow

### **1. Authentication Flow**
```
User Authentication
        │
        ▼ Password Input
┌─────────────────┐
│ Password Dialog │ ← User Input
│                 │ ← Validation
└─────────────────┘
        │
        ▼
┌─────────────────┐
│ Auth Handler    │ ← Password Verification
│                 │ ← Session Management
└─────────────────┘
        │
        ▼
┌─────────────────┐
│ State Update    │ ← Authentication State
│                 │ ← UI Mode Switch
└─────────────────┘
        │
        ▼
┌─────────────────┐
│ Feature Access  │ ← Advanced Mode
│                 │ ← Permission Check
└─────────────────┘
```

### **2. License Validation Flow**
```
Application Startup
        │
        ▼ License Check
┌─────────────────┐
│ License Plugin  │ ← License Key Validation
│                 │ ← Online/Offline Check
└─────────────────┘
        │
        ▼
┌─────────────────┐
│ Validation      │ ← License Status
│                 │ ← Feature Access
└─────────────────┘
        │
        ▼
┌─────────────────┐
│ UI Update       │ ← License Status Display
│                 │ ← Feature Availability
└─────────────────┘
```

---

## 🔄 Data Synchronization Flow

### **1. Settings Sync Flow**
```
Settings Change
    │
    ▼ Change Event
┌─────────────────┐
│ Settings Store  │ ← Local State Update
│                 │ ← Change Tracking
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Database Sync   │ ← Persistent Storage
│                 │ ← History Tracking
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ UI Sync         │ ← Component Update
│                 │ ← Real-time Reflection
└─────────────────┘
```

### **2. Event Sync Flow**
```
PSS Event
    │
    ▼ Event Processing
┌─────────────────┐
│ Event Store     │ ← In-memory Storage
│                 │ ← Real-time Processing
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Database Sync   │ ← Persistent Storage
│                 │ ← Event History
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Frontend Sync   │ ← UI Update
│                 │ ← Event Display
└─────────────────┘
```

---

## 🎯 Optimization Strategies

### **1. Caching Strategy**
```
Data Request
    │
    ▼ Cache Check
┌─────────────────┐
│ Cache Layer     │ ← In-memory Cache
│                 │ ← Cache Hit/Miss
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Database Query  │ ← Cache Miss Only
│                 │ ← Query Optimization
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Cache Update    │ ← Cache Population
│                 │ ← TTL Management
└─────────────────┘
```

### **2. Event Batching**
```
Event Stream
    │
    ▼ Event Collection
┌─────────────────┐
│ Event Buffer    │ ← Event Batching
│                 │ ← Batch Size Management
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Batch Processor │ ← Batch Processing
│                 │ ← Database Operations
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ UI Update       │ ← Batched Updates
│                 │ ← Performance Optimization
└─────────────────┘
```

---

## 🎯 Manual Override Detection System

### Overview
The manual override detection system provides intelligent detection and handling of manual changes made in the external PSS software. This system ensures that manual overrides (score changes, round changes, time corrections, warning adjustments) are properly detected and handled without disrupting normal event processing.

### Detection Methods

#### **1. Manual Round Change Detection**
```javascript
// Detect manual round changes
function isManualRoundChange(event) {
    // Check if this is a round change during stopped clock
    if (isClockStopped() && event.type === 'round') {
        return true;
    }
    
    // Check for rapid round changes (multiple within 5 seconds)
    const recentRoundEvents = getRecentEvents('round', 5000);
    return recentRoundEvents.length >= 2;
}
```

**Detection Criteria:**
- **Round changes during stopped clock**: Indicates manual intervention
- **Rapid round changes**: Multiple round changes within 5 seconds
- **Pattern analysis**: Unusual round change patterns

#### **2. Manual Score Change Detection**
```javascript
// Detect manual score changes
function isManualScoreChange(event, newBlueScore, newRedScore) {
    // 1. Score change during stopped clock
    if (isClockStopped() && event.type === 'points') {
        return true;
    }
    
    // 2. Score change during time correction
    if (hasRecentTimeCorrection() && event.type === 'points') {
        return true;
    }
    
    // 3. Large score jump (3 or more points)
    const scoreJump = calculateScoreJump(newBlueScore, newRedScore);
    if (scoreJump >= 3) {
        return true;
    }
    
    // 4. Rapid point messages
    if (hasRapidPointMessages()) {
        return true;
    }
    
    return false;
}
```

**Detection Criteria:**
- **Score changes during stopped clock**: Manual intervention during paused time
- **Score changes during time correction**: Changes made during `clk;XX:XX;corr` events
- **Large score jumps**: Unusual score increases (3+ points at once)
- **Rapid point messages**: Multiple point events within 2 seconds

#### **3. Manual Time Change Detection**
```javascript
// Detect manual time changes
function isManualTimeChange(event) {
    // Check for time correction action
    if (event.type === 'clock' && event.action === 'corr') {
        return true;
    }
    return false;
}
```

**Detection Criteria:**
- **Time correction events**: `clk;XX:XX;corr` messages indicate manual time adjustments
- **Time changes during stopped periods**: Unusual time modifications

#### **4. Manual Warning Change Detection**
```javascript
// Detect manual warning changes
function isManualWarningChange(event, newBlueWarnings, newRedWarnings) {
    // Warning change during stopped clock
    if (isClockStopped()) {
        return true;
    }
    
    // Warning change during time correction
    if (hasRecentTimeCorrection()) {
        return true;
    }
    
    // Unusual warning pattern (multiple warnings in quick succession)
    const recentWarningEvents = getRecentEvents('warnings', 3000);
    return recentWarningEvents.length >= 2;
}
```

**Detection Criteria:**
- **Warning changes during stopped clock**: Manual intervention during paused time
- **Warning changes during time correction**: Changes made during time correction events
- **Rapid warning changes**: Multiple warning events within 3 seconds

### State Tracking System

#### **Clock State Management**
```javascript
const manualOverrideState = {
    // Clock state tracking
    clockState: 'stopped', // 'running', 'stopped'
    lastClockTime: null,
    lastClockAction: null,
    
    // Recent events tracking (last 10 events for pattern detection)
    recentEvents: [],
    maxRecentEvents: 10,
    
    // Manual change detection
    lastManualRoundChange: null,
    lastManualScoreChange: null,
    lastManualTimeChange: null,
    
    // Score tracking for jump detection
    lastBlueScore: 0,
    lastRedScore: 0,
    
    // Warning tracking
    lastBlueWarnings: 0,
    lastRedWarnings: 0,
    
    // Time correction tracking
    hasRecentTimeCorrection: false,
    timeCorrectionThreshold: 5000, // 5 seconds
    lastTimeCorrection: null
};
```

#### **Event Pattern Analysis**
```javascript
// Safely check for rapid point messages
function hasRapidPointMessages() {
    const recentPointEvents = getRecentEvents('points', 2000);
    return recentPointEvents.length >= 2;
}

// Safely calculate score jump
function calculateScoreJump(newBlueScore, newRedScore) {
    const blueJump = Math.abs(newBlueScore - lastBlueScore);
    const redJump = Math.abs(newRedScore - lastRedScore);
    return Math.max(blueJump, redJump);
}
```

### Handling Strategies

#### **1. Manual Round Change Handling**
```javascript
// Handle manual round change (preserve all data)
function handleManualRoundChange(event) {
    // Update round number but preserve all other data
    scoreboardInstance.updateRound(event.current_round);
    
    // DO NOT reset scores, warnings, or other data
    // This is the key difference from normal round changes
}
```

**Key Behavior:**
- **Update round number**: Change the displayed round
- **Preserve scores**: Keep current scores intact
- **Preserve warnings**: Keep current warnings intact
- **Preserve time**: Keep current time intact

#### **2. Manual Score Change Handling**
```javascript
// Handle manual score change (accept immediately)
function handleManualScoreChange(event, newBlueScore, newRedScore) {
    // Update scores immediately
    scoreboardInstance.updateScores(newBlueScore, newRedScore);
    
    // Update tracking state
    manualOverrideState.lastBlueScore = newBlueScore;
    manualOverrideState.lastRedScore = newRedScore;
}
```

**Key Behavior:**
- **Immediate acceptance**: Accept manual score changes without validation
- **Real-time updates**: Update scoreboard immediately
- **State tracking**: Update internal tracking state

#### **3. Manual Time Change Handling**
```javascript
// Handle manual time change (accept immediately)
function handleManualTimeChange(event) {
    // Update time immediately
    const timeParts = event.time.split(':');
    const minutes = parseInt(timeParts[0]) || 0;
    const seconds = parseInt(timeParts[1]) || 0;
    scoreboardInstance.updateTimer(minutes, seconds);
}
```

**Key Behavior:**
- **Immediate acceptance**: Accept manual time changes
- **Time correction tracking**: Track time correction events
- **Real-time updates**: Update scoreboard immediately

#### **4. Manual Warning Change Handling**
```javascript
// Handle manual warning change (accept immediately)
function handleManualWarningChange(event, newBlueWarnings, newRedWarnings) {
    // Update warnings immediately
    scoreboardInstance.updatePenalties(newBlueWarnings, newRedWarnings);
    
    // Update tracking state
    manualOverrideState.lastBlueWarnings = newBlueWarnings;
    manualOverrideState.lastRedWarnings = newRedWarnings;
}
```

**Key Behavior:**
- **Immediate acceptance**: Accept manual warning changes
- **Real-time updates**: Update scoreboard immediately
- **State tracking**: Update internal tracking state

### Error Handling and Safety

#### **Robust Error Handling**
```javascript
// Safely get current timestamp
function getCurrentTimestamp() {
    try {
        return Date.now();
    } catch (error) {
        console.warn('⚠️ Error getting timestamp:', error);
        return 0;
    }
}

// Safely add event to recent events
function addToRecentEvents(event) {
    try {
        manualOverrideState.recentEvents.push({
            event: event,
            timestamp: getCurrentTimestamp()
        });
        
        // Keep only the last N events
        if (manualOverrideState.recentEvents.length > maxRecentEvents) {
            manualOverrideState.recentEvents.shift();
        }
    } catch (error) {
        console.warn('⚠️ Error adding to recent events:', error);
    }
}
```

#### **Panic Prevention**
- **Try-catch blocks**: All detection functions wrapped in error handling
- **Safe utility functions**: Fallback values for all operations
- **Defensive programming**: Safe data access patterns
- **Graceful degradation**: Continue operation when errors occur

### Integration with Event Processing

#### **Event Handler Integration**
```javascript
// Main event handler with manual override detection
function handlePssEvent(event) {
    // Add event to recent events for pattern detection
    addToRecentEvents(event);
    
    // Update clock state tracking
    if (event.type === 'clock') {
        updateClockState(event);
    }
    
    // Check for manual overrides BEFORE normal processing
    if (isManualTimeChange(event)) {
        handleManualTimeChange(event);
        return;
    }
    
    if (isManualRoundChange(event)) {
        handleManualRoundChange(event);
        return;
    }
    
    // Normal event processing
    processNormalEvent(event);
}
```

#### **Scoreboard Overlay Integration**
```javascript
// Scoreboard overlay with manual override support
function handleScoresEvent(event) {
    // Check for manual score change
    if (isManualScoreChange(event, blueScore, redScore)) {
        handleManualScoreChange(event, blueScore, redScore);
    } else {
        // Normal score update
        scoreboardInstance.updateScores(blueScore, redScore);
    }
}
```

### Benefits and Impact

#### **1. Accurate Manual Change Detection**
- **Reliable detection**: Multiple detection methods for accuracy
- **Pattern recognition**: Intelligent pattern analysis
- **Context awareness**: Clock state and timing awareness

#### **2. Proper Data Handling**
- **Data preservation**: Manual round changes preserve all data
- **Immediate acceptance**: Manual changes accepted immediately
- **Real-time updates**: Instant scoreboard updates

#### **3. System Reliability**
- **Panic-free operation**: Comprehensive error handling
- **Graceful degradation**: Continue operation during errors
- **Robust state tracking**: Reliable state management

#### **4. User Experience**
- **Seamless operation**: Manual changes work as expected
- **No data loss**: All data preserved during manual changes
- **Real-time feedback**: Immediate visual updates

---

## 🔮 Future Data Flow Enhancements

### **1. Advanced Event Processing**
- **Event Correlation**: Cross-reference events for pattern detection
- **Predictive Analytics**: Machine learning for event prediction
- **Real-time Analytics**: Advanced statistical analysis
- **Event Filtering**: Intelligent event filtering and prioritization

### **2. Enhanced Synchronization**
- **Multi-device Sync**: Synchronization across multiple devices
- **Offline Support**: Offline event queuing and sync
- **Conflict Resolution**: Advanced conflict resolution strategies
- **Real-time Collaboration**: Multi-user real-time collaboration

### **3. Performance Optimizations**
- **Event Streaming**: Efficient event streaming protocols
- **Database Optimization**: Advanced database optimization techniques
- **Memory Management**: Intelligent memory management strategies
- **Load Balancing**: Distributed load balancing for high-performance scenarios

---

## 📞 Troubleshooting Data Flow

### **Common Issues**

#### **1. Event Loss**
- **Cause**: Network issues, buffer overflow, processing errors
- **Solution**: Implement event queuing, retry mechanisms, error recovery

#### **2. Performance Degradation**
- **Cause**: Large event volumes, inefficient queries, memory leaks
- **Solution**: Implement caching, query optimization, memory management

#### **3. Synchronization Issues**
- **Cause**: Race conditions, network latency, state inconsistencies
- **Solution**: Implement proper locking, conflict resolution, state validation

### **Monitoring & Debugging**
- **Event Tracing**: Comprehensive event tracing and logging
- **Performance Monitoring**: Real-time performance monitoring
- **Error Tracking**: Advanced error tracking and reporting
- **Health Checks**: System health monitoring and alerting

---

**Last Updated**: 2025-01-29  
**Architecture Version**: 2.0  
**Status**: Production Ready with Comprehensive Data Flow