# Event Table Database Enhancement Plan - reStrike VTA

## Overview
This document outlines the comprehensive plan to enhance the PSS events database schema and create a database-driven Event table with real-time updates. The goal is to replace the current in-memory event storage with a robust database solution that supports advanced filtering, analytics, and real-time updates.

## Current Issues Identified

### 1. Event Table Not Populated
- **Problem**: Event table in DockBar/SidebarBig shows no events
- **Root Cause**: `useLiveDataEvents` hook not used in main App component
- **Current Flow**: PSS Events → Backend WebSocket (port 3001) → ❌ NO FRONTEND CONNECTION ❌ → Event Table (empty)
- **Expected Flow**: PSS Events → Backend WebSocket (port 3001) → useLiveDataEvents hook → liveDataStore → Event Table

### 2. Database Storage Issues
- **Problem**: Frontend events not stored to database
- **Root Cause**: `store_pss_event` Tauri command only logs events, doesn't store them
- **Working**: UDP events are stored automatically via UDP plugin
- **Not Working**: WebSocket events are not stored (command incomplete)

### 3. In-Memory Limitations
- **Problem**: Events stored in memory only
- **Issues**: Data lost on app restart, memory bloat, no persistence
- **Solution**: Database-driven approach with real-time updates

## PSS Protocol Event Analysis

### Event Type Mapping to Categories

Based on protocol files (`protocol/pss_v2.3.txt` and `protocol/pss_schema.txt`):

| **PSS Event** | **Protocol Stream** | **Category** | **Description** |
|---------------|-------------------|--------------|-----------------|
| **Points** | `pt1;1;` / `pt2;1;` | **P (Punch)** | Punch point (value 1) |
| **Points** | `pt1;2;` / `pt2;2;` | **TB (Technical Body)** | Body point (value 2) |
| **Points** | `pt1;3;` / `pt2;3;` | **H (Head)** | Head point (value 3) |
| **Points** | `pt1;4;` / `pt2;4;` | **TB (Technical Body)** | Technical body point (value 4) |
| **Points** | `pt1;5;` / `pt2;5;` | **TH (Technical Head)** | Technical head point (value 5) |
| **Hit Level** | `hl1;50;` / `hl2;50;` | **K (Kick)** | Hit level events (1-100) |
| **Warnings** | `wg1;1;wg2;2;` | **R (Referee)** | Warnings/gam-jeom |
| **Challenges** | `ch0;` / `ch1;` / `ch2;` | **R (Referee)** | IVR challenges |
| **Injury** | `ij1;1:23;` / `ij2;1:23;` | **R (Referee)** | Injury time |
| **Break** | `brk;0:59;` | **R (Referee)** | Break time |
| **Clock** | `clk;2:00;` | **R (Referee)** | Clock events |
| **Round** | `rnd;1;` | **R (Referee)** | Round changes |
| **Winner** | `win;BLUE;` | **R (Referee)** | Winner determination |
| **Athletes** | `at1;...;at2;...;` | **O (Other)** | Athlete info |
| **Match Config** | `mch;101;...;` | **O (Other)** | Match configuration |
| **Scores** | `s11;0;` / `s21;0;` | **O (Other)** | Score updates |
| **Current Scores** | `sc1;0;` / `sc2;0;` | **O (Other)** | Current score |
| **Winner Rounds** | `wrd;rd1;0;rd2;0;rd3;0;` | **R (Referee)** | Round winners |
| **Fight Ready** | `rdy;FightReady;` | **O (Other)** | Fight ready status |
| **Fight Loaded** | `pre;FightLoaded;` | **O (Other)** | Fight loaded status |

## Database Schema Enhancement Plan

### Phase 1: Add New Fields to `pss_events_v2` Table

**New Migration: Migration15**
```sql
-- Add event category field for easy filtering
ALTER TABLE pss_events_v2 ADD COLUMN event_category TEXT NOT NULL DEFAULT 'O' 
CHECK (event_category IN ('K', 'P', 'H', 'TH', 'TB', 'R', 'O'));

-- Add tournament context fields
ALTER TABLE pss_events_v2 ADD COLUMN tournament_id INTEGER REFERENCES tournaments(id);
ALTER TABLE pss_events_v2 ADD COLUMN tournament_day_id INTEGER REFERENCES tournament_days(id);

-- Add match number field (extracted from match configuration)
ALTER TABLE pss_events_v2 ADD COLUMN match_number TEXT;

-- Add indices for performance
CREATE INDEX IF NOT EXISTS idx_pss_events_v2_category ON pss_events_v2(event_category);
CREATE INDEX IF NOT EXISTS idx_pss_events_v2_tournament ON pss_events_v2(tournament_id, tournament_day_id);
CREATE INDEX IF NOT EXISTS idx_pss_events_v2_match_number ON pss_events_v2(match_number);
```

### Phase 2: Update Event Processing Logic

**File: `src-tauri/src/plugins/plugin_udp.rs`**

1. **Add Event Category Mapping Function:**
```rust
fn map_event_to_category(event: &PssEvent) -> &'static str {
    match event {
        PssEvent::Points { point_type, .. } => {
            match point_type {
                1 => "P",   // Punch
                2 => "TB",  // Technical Body
                3 => "H",   // Head
                4 => "TB",  // Technical Body
                5 => "TH",  // Technical Head
                _ => "O",   // Other
            }
        }
        PssEvent::HitLevel { .. } => "K",      // Kick
        PssEvent::Warnings { .. } => "R",      // Referee
        PssEvent::Challenge { .. } => "R",     // Referee
        PssEvent::Injury { .. } => "R",        // Referee
        PssEvent::Break { .. } => "R",         // Referee
        PssEvent::Clock { .. } => "R",         // Referee
        PssEvent::Round { .. } => "R",         // Referee
        PssEvent::Winner { .. } => "R",        // Referee
        PssEvent::WinnerRounds { .. } => "R",  // Referee
        PssEvent::Athletes { .. } => "O",      // Other
        PssEvent::MatchConfig { .. } => "O",   // Other
        PssEvent::Scores { .. } => "O",        // Other
        PssEvent::CurrentScores { .. } => "O", // Other
        PssEvent::FightReady => "O",           // Other
        PssEvent::FightLoaded => "O",          // Other
        PssEvent::Raw(_) => "O",               // Other
    }
}
```

2. **Update Event Storage Function:**
```rust
async fn convert_pss_event_to_db_model(
    event: &PssEvent,
    session_id: i64,
    current_match_id: &Arc<Mutex<Option<i64>>>,
    event_type_cache: &Arc<Mutex<std::collections::HashMap<String, i64>>>,
    database: &DatabasePlugin,
    current_tournament_id: &Arc<Mutex<Option<i64>>>,
    current_tournament_day_id: &Arc<Mutex<Option<i64>>>,
) -> AppResult<PssEventV2> {
    // ... existing code ...
    
    // Extract match number from match configuration
    let match_number = if let PssEvent::MatchConfig { number, .. } = event {
        Some(number.clone())
    } else {
        None
    };
    
    // Get tournament context
    let tournament_id = *current_tournament_id.lock().await;
    let tournament_day_id = *current_tournament_day_id.lock().await;
    
    // Map event to category
    let event_category = map_event_to_category(event);
    
    Ok(PssEventV2 {
        // ... existing fields ...
        event_category: event_category.to_string(),
        tournament_id,
        tournament_day_id,
        match_number,
        // ... rest of fields ...
    })
}
```

### Phase 3: Update Database Models

**File: `src-tauri/src/database/models.rs`**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssEventV2 {
    pub id: Option<i64>,
    pub session_id: i64,
    pub match_id: Option<i64>,
    pub round_id: Option<i64>,
    pub event_type_id: i64,
    pub timestamp: String,
    pub raw_data: String,
    pub parsed_data: Option<String>,
    pub event_sequence: Option<i64>,
    pub processing_time_ms: Option<i64>,
    pub is_valid: bool,
    pub error_message: Option<String>,
    pub created_at: String,
    // NEW FIELDS
    pub event_category: String,        // K, P, H, TH, TB, R, O
    pub tournament_id: Option<i64>,    // Tournament context
    pub tournament_day_id: Option<i64>, // Tournament day context
    pub match_number: Option<String>,  // Extracted match number
}
```

### Phase 4: Create New Database Queries

**File: `src-tauri/src/database/operations.rs`**
```rust
impl PssUdpOperations {
    // Get events by category for Event table
    pub fn get_events_by_category(
        conn: &Connection, 
        session_id: i64, 
        categories: &[String], 
        limit: Option<i64>
    ) -> DatabaseResult<Vec<PssEventV2>> {
        let limit = limit.unwrap_or(100);
        let categories_str = categories.join("','");
        
        let mut stmt = conn.prepare(&format!(
            "SELECT * FROM pss_events_v2 
             WHERE session_id = ? AND event_category IN ('{}')
             ORDER BY timestamp DESC LIMIT ?",
            categories_str
        ))?;
        
        let events = stmt.query_map(params![session_id, limit], |row| {
            PssEventV2::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(events)
    }
    
    // Get events by tournament context
    pub fn get_events_by_tournament(
        conn: &Connection,
        tournament_id: i64,
        tournament_day_id: Option<i64>,
        categories: &[String],
        limit: Option<i64>
    ) -> DatabaseResult<Vec<PssEventV2>> {
        let limit = limit.unwrap_or(100);
        let categories_str = categories.join("','");
        
        let query = if let Some(day_id) = tournament_day_id {
            format!(
                "SELECT * FROM pss_events_v2 
                 WHERE tournament_id = ? AND tournament_day_id = ? AND event_category IN ('{}')
                 ORDER BY timestamp DESC LIMIT ?",
                categories_str
            )
        } else {
            format!(
                "SELECT * FROM pss_events_v2 
                 WHERE tournament_id = ? AND event_category IN ('{}')
                 ORDER BY timestamp DESC LIMIT ?",
                categories_str
            )
        };
        
        let mut stmt = conn.prepare(&query)?;
        
        let events = if let Some(day_id) = tournament_day_id {
            stmt.query_map(params![tournament_id, day_id, limit], |row| {
                PssEventV2::from_row(row)
            })?
        } else {
            stmt.query_map(params![tournament_id, limit], |row| {
                PssEventV2::from_row(row)
            })?
        }
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(events)
    }
    
    // Get match events by match number
    pub fn get_events_by_match_number(
        conn: &Connection,
        match_number: &str,
        categories: &[String],
        limit: Option<i64>
    ) -> DatabaseResult<Vec<PssEventV2>> {
        let limit = limit.unwrap_or(100);
        let categories_str = categories.join("','");
        
        let mut stmt = conn.prepare(&format!(
            "SELECT * FROM pss_events_v2 
             WHERE match_number = ? AND event_category IN ('{}')
             ORDER BY timestamp DESC LIMIT ?",
            categories_str
        ))?;
        
        let events = stmt.query_map(params![match_number, limit], |row| {
            PssEventV2::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(events)
    }
}
```

### Phase 5: Add New Tauri Commands

**File: `src-tauri/src/tauri_commands.rs`**
```rust
#[tauri::command]
pub async fn get_event_table_events(
    session_id: Option<i64>,
    tournament_id: Option<i64>,
    tournament_day_id: Option<i64>,
    match_number: Option<String>,
    categories: Vec<String>, // ['K', 'P', 'H', 'TH', 'TB', 'R']
    limit: Option<i64>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting event table events with filters: {:?}", {
        session_id, tournament_id, tournament_day_id, match_number, categories, limit
    });
    
    let events = if let Some(session_id) = session_id {
        app.database_plugin().get_events_by_category(session_id, &categories, limit).await?
    } else if let Some(tournament_id) = tournament_id {
        app.database_plugin().get_events_by_tournament(tournament_id, tournament_day_id, &categories, limit).await?
    } else if let Some(match_number) = match_number {
        app.database_plugin().get_events_by_match_number(&match_number, &categories, limit).await?
    } else {
        return Err(TauriError::from(anyhow::anyhow!("No filter criteria provided")));
    };
    
    let events_json: Vec<serde_json::Value> = events
        .into_iter()
        .map(|event| serde_json::json!({
            "id": event.id,
            "event_category": event.event_category,
            "event_code": event.event_code,
            "athlete": event.athlete,
            "round": event.round,
            "time": event.time,
            "timestamp": event.timestamp,
            "description": event.description,
            "match_number": event.match_number,
            "tournament_id": event.tournament_id,
            "tournament_day_id": event.tournament_day_id,
        }))
        .collect();
    
    Ok(serde_json::json!({
        "success": true,
        "events": events_json,
        "count": events_json.len()
    }))
}
```

### Phase 6: Update Frontend Store

**File: `ui/src/stores/databaseEventStore.ts`**
```typescript
interface DatabaseEventStore {
  events: DatabaseEvent[];
  isLoading: boolean;
  lastUpdate: string;
  
  // Filtering
  sessionId: number | null;
  tournamentId: number | null;
  tournamentDayId: number | null;
  matchNumber: string | null;
  eventCategories: string[]; // ['K', 'P', 'H', 'TH', 'TB', 'R']
  
  // Actions
  fetchEvents: () => Promise<void>;
  setFilters: (filters: EventFilters) => void;
  clearFilters: () => void;
}

interface DatabaseEvent {
  id: number;
  event_category: string; // K, P, H, TH, TB, R, O
  event_code: string;
  athlete: string;
  round: number;
  time: string;
  timestamp: string;
  description: string;
  match_number: string | null;
  tournament_id: number | null;
  tournament_day_id: number | null;
}

interface EventFilters {
  sessionId?: number;
  tournamentId?: number;
  tournamentDayId?: number;
  matchNumber?: string;
  eventCategories?: string[];
}
```

### Phase 7: Update Event Table Component

**File: `ui/src/components/molecules/EventTableSection.tsx`**
```typescript
const EventTableSection: React.FC = () => {
  const { 
    events, 
    isLoading, 
    fetchEvents, 
    setFilters,
    subscribeToUpdates 
  } = useDatabaseEventStore();
  
  // Real-time polling for new events
  useEffect(() => {
    const interval = setInterval(() => {
      fetchEvents();
    }, 2000); // Poll every 2 seconds
    
    return () => clearInterval(interval);
  }, [fetchEvents]);
  
  // Subscribe to real-time updates
  useEffect(() => {
    subscribeToUpdates();
    return () => unsubscribeFromUpdates();
  }, []);
  
  // Filter events for display
  const filteredEvents = events.filter(event => 
    ['K', 'P', 'H', 'TH', 'TB', 'R'].includes(event.event_category)
  );
  
  return (
    // ... existing UI with database events
  );
};
```

## Implementation Timeline

### Phase 1: Database Migration (1 hour)
- [ ] Create Migration15
- [ ] Add new fields to pss_events_v2 table
- [ ] Add performance indices
- [ ] Test migration

### Phase 2: Event Processing Logic (2 hours)
- [ ] Add event category mapping function
- [ ] Update event storage logic
- [ ] Add match number extraction
- [ ] Add tournament context tracking

### Phase 3: Database Models & Operations (1 hour)
- [ ] Update PssEventV2 model
- [ ] Add new query functions
- [ ] Add performance optimizations

### Phase 4: Tauri Commands (1 hour)
- [ ] Add new get_event_table_events command
- [ ] Add filtering and pagination
- [ ] Add error handling

### Phase 5: Frontend Integration (2 hours)
- [ ] Create databaseEventStore
- [ ] Update EventTableSection component
- [ ] Add real-time polling
- [ ] Add filtering UI

### Phase 6: Testing & Optimization (1 hour)
- [ ] Test with real PSS data
- [ ] Performance testing
- [ ] Error handling validation

**Total Implementation Time: 8 hours**

## Benefits of This Enhancement

### ✅ Query Performance
- Direct category filtering (no complex parsing)
- Tournament context queries
- Match number queries
- Optimized indices

### ✅ Analytics Capabilities
- Event category statistics
- Tournament performance analysis
- Match-level analytics
- Time-based event analysis

### ✅ Real-time Features
- Fast event table updates
- Efficient filtering
- Scalable architecture

### ✅ Future Extensibility
- Easy to add new categories
- Tournament integration ready
- Analytics foundation built

## Current Architecture Context

### Backend Components
- **UDP Plugin**: `src-tauri/src/plugins/plugin_udp.rs` - Handles PSS event processing
- **Database Plugin**: `src-tauri/src/plugins/plugin_database.rs` - Database operations
- **WebSocket Plugin**: `src-tauri/src/plugins/plugin_websocket.rs` - Real-time communication
- **Tauri Commands**: `src-tauri/src/tauri_commands.rs` - Frontend-backend communication

### Frontend Components
- **Event Table**: `ui/src/components/molecules/EventTableSection.tsx` - Main event display
- **Live Data Store**: `ui/src/stores/liveDataStore.ts` - Current in-memory store
- **PSS Events Hook**: `ui/src/hooks/usePssEvents.ts` - Event listener setup
- **Live Data Events Hook**: `ui/src/hooks/useLiveDataEvents.ts` - WebSocket connection

### Database Schema
- **Main Table**: `pss_events_v2` - Event storage
- **Details Table**: `pss_event_details` - Event-specific data
- **Scores Table**: `pss_scores` - Score tracking
- **Warnings Table**: `pss_warnings` - Warning tracking

## Next Steps

1. **Start with Phase 1**: Database migration to add new fields
2. **Implement Phase 2**: Event processing logic with category mapping
3. **Continue through phases**: Systematic implementation
4. **Test thoroughly**: With real PSS data and various scenarios
5. **Optimize performance**: Based on testing results

## Notes for Future Implementation

- **R (Referee) Events**: User will help mark all referee events
- **Tournament Context**: Ensure tournament_id and tournament_day_id are properly set
- **Match Number Extraction**: Extract from match configuration events
- **Real-time Updates**: Implement proper event broadcasting
- **Error Handling**: Robust error handling for all database operations
- **Performance Monitoring**: Track query performance and optimize as needed

---

**Document Created**: 2025-01-29
**Purpose**: Temporary planning document for Event Table database enhancement
**Status**: Ready for implementation
**Next Action**: Start Phase 1 - Database Migration 