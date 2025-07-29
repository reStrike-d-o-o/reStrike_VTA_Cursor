# Data Flow Architecture & System Integration

## Overview

The reStrike VTA system implements a sophisticated data flow architecture that handles real-time PSS events, OBS integration, database operations, and frontend updates. This document outlines the complete data flow patterns, event handling, and system integration points.

## 🔄 Data Flow Overview

### **System Architecture Layers**
```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend Layer (React)                   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │   UI State  │ │ Event Table │ │ Live Data   │ │ OBS UI  │ │
│  │  (Zustand)  │ │             │ │   Panel     │ │ Manager │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 Tauri Bridge Layer                          │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │   Commands  │ │   Events    │ │   IPC       │ │ Invoke  │ │
│  │             │ │             │ │             │ │         │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  Backend Layer (Rust)                       │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │ UDP Plugin  │ │ OBS Plugin  │ │ Database    │ │ CPU     │ │
│  │             │ │             │ │ Plugin      │ │ Monitor │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  Data Layer (SQLite)                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │ PSS Events  │ │ UI Settings │ │ Flag Data   │ │ OBS     │ │
│  │             │ │             │ │             │ │ Config  │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## 📡 PSS Event Data Flow

### **1. UDP Reception & Parsing**
```
External PSS System
        │
        ▼ UDP Datagrams
┌─────────────────┐
│   UDP Server    │ ← Network Interface Detection
│   (UDP Plugin)  │ ← Port 6000 (default)
└─────────────────┘
        │
        ▼ PSS Protocol Parsing
┌─────────────────┐
│ PSS Event Parser│ ← Protocol v2.3 Implementation
│                 │ ← Event Type Detection
└─────────────────┘
        │
        ▼ Event Classification
┌─────────────────┐
│ Event Handler   │ ← Match Config, Athletes, Scores, Warnings
│                 │ ← Real-time Processing
└─────────────────┘
```

### **2. Database Storage Flow**
```
PSS Event
    │
    ▼
┌─────────────────┐
│ Event Converter │ ← Convert to PssEventV2
│                 │ ← Extract Event Details
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Database Store  │ ← Async Storage Operation
│                 │ ← Transaction Management
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Cache Update    │ ← In-memory Caches
│                 │ ← Athlete & Event Type Caches
└─────────────────┘
```

### **3. Frontend Notification Flow**
```
Database Storage
    │
    ▼
┌─────────────────┐
│ Event Emitter   │ ← Tauri Event System
│                 │ ← Real-time Broadcasting
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Frontend Listener│ ← React Event Hook
│                 │ ← State Update
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ UI Update       │ ← Component Re-render
│                 │ ← Event Table Update
└─────────────────┘
```

---

## 🎥 OBS Integration Data Flow

### **1. WebSocket Connection Flow**
```
Frontend OBS Manager
        │
        ▼ Connect Request
┌─────────────────┐
│ Tauri Command   │ ← obs_connect(url)
│                 │ ← WebSocket v5 Protocol
└─────────────────┘
        │
        ▼
┌─────────────────┐
│ OBS Plugin      │ ← Connection Management
│                 │ ← Status Monitoring
└─────────────────┘
        │
        ▼
┌─────────────────┐
│ OBS Studio      │ ← WebSocket Connection
│                 │ ← Authentication
└─────────────────┘
```

### **2. OBS Status Monitoring Flow**
```
OBS Studio
    │
    ▼ Status Events
┌─────────────────┐
│ OBS Plugin      │ ← Recording Status
│                 │ ← Streaming Status
│                 │ ← Scene Information
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Status Emitter  │ ← Tauri Event System
│                 │ ← Real-time Updates
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Frontend Update │ ← OBS Store Update
│                 │ ← UI Status Display
└─────────────────┘
```

### **3. OBS Control Flow**
```
Frontend Controls
    │
    ▼ Control Commands
┌─────────────────┐
│ Tauri Commands  │ ← start_recording, stop_recording
│                 │ ← scene_switch, source_control
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ OBS Plugin      │ ← Command Execution
│                 │ ← Response Handling
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ OBS Studio      │ ← Action Execution
│                 │ ← Status Update
└─────────────────┘
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