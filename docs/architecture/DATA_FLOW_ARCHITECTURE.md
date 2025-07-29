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

## Detailed Data Flow

### PSS Event Processing Flow

#### Complete Event Lifecycle
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   UDP       │───▶│   PSS       │───▶│  Database   │───▶│   Frontend  │
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
   
   // Update UI in real-time
   frontend_store.update_events(event_data);
   ```

### OBS Integration and Dual Protocol Support

#### OBS WebSocket Dual-Protocol Architecture

The system supports both OBS WebSocket v4 and v5 protocols simultaneously:

```
┌─────────────────────────────────────────────────────────────┐
│                    OBS Integration Layer                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   OBS v4    │  │   OBS v5    │  │  Protocol   │        │
│  │  Protocol   │  │  Protocol   │  │  Detector   │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Unified OBS Interface                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │ Connection  │  │   Scene     │  │ Recording   │        │
│  │ Management  │  │  Control    │  │  Control    │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
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