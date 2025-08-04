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

## OBS Integration and Timestamp Calculation

### OBS Timestamp Fields Purpose

The new timestamp fields enable video replay integration with OBS Studio:

1. **rec_timestamp**: 
   - **Calculation**: `pss_event_timestamp - obs_recording_start_timestamp`
   - **Purpose**: Provides relative timestamp for events within the OBS recording
   - **Usage**: Allows precise seeking to event moments in recorded video files

2. **str_timestamp**: 
   - **Calculation**: `pss_event_timestamp - obs_stream_start_timestamp`
   - **Purpose**: Provides relative timestamp for events within the OBS stream
   - **Usage**: Enables live stream synchronization and replay buffer integration

3. **ivr_link**: 
   - **Content**: File path to OBS replay buffer clip
   - **Condition**: Only populated if OBS replay buffer clip exists within ±10 seconds of PSS event timestamp
   - **Purpose**: Direct link to video clip showing the event moment
   - **Usage**: One-click video replay for event analysis

### OBS Integration Requirements

To populate these fields, the system needs:

1. **OBS WebSocket Connection**: Monitor recording/streaming status
2. **Timestamp Tracking**: Store OBS recording and stream start timestamps
3. **Replay Buffer Monitoring**: Track when replay buffer clips are created
4. **Time Synchronization**: Ensure PSS and OBS timestamps are synchronized

### OBS Integration Triggering Rules

#### Basic Requirements for Saving rec_timestamp:
- **OBS_REC WebSocket Connection**: Must be connected
- **OBS_REC Recording Status**: Must be actively recording
- **OBS_REC Video Replay Buffer**: Must be active

#### Trigger 1: New Match Loaded and Ready
**Trigger Event**: PSS event new match loaded and ready (latest event)

**Actions**:
1. **OBS_REC Setup Recording Path**: 
   - Folder: `C:/Users/Current user/Videos/{Tournament Name}/{Current Tournament Day Name}`
   - Example: `C:/Users/Current user/Videos/Albania Open 2025/Day 1`
   - *Note: This can be executed once with tournament day activation*

2. **OBS_REC Setup Recording Name**: 
   - Format: `{matchNumber}_{player1}_{player1_country_IOC}_vs_{player2}_{player2_country_IOC}`
   - Example: `101_John_Smith_USA_vs_Mike_Jones_CAN`

3. **OBS_REC Setup Video Replay Buffer**:
   - Duration: 20 seconds (extracted from Settings)
   - Save Location: `{recording_path}/IVR recordings/`
   - Save Name Format: `{match_number}_{timestamp}`
   - *Note: IVR recordings folder can be created with tournament day activation*

4. **OBS_REC Start Operations**:
   - Start recording
   - Start/enable video replay buffer

5. **OBS_STR Calculate str_timestamp**:
   - Calculate: `event_timestamp - OBS_REC_stream_start_timestamp`
   - Save to str_timestamp field

#### Trigger 2: Challenge/IVR or Replay Button
**Trigger Events**: 
- PSS Event challenge: `ch0;`, `ch1;`, `ch2;`
- REPLAY button clicked

**Actions**:
1. **OBS_REC Save Video Replay Buffer**
2. **Update Event Timestamps**:
   - Find all tracked events in `pss_events_v2` table with timestamp in last 20 seconds
   - Calculate: `timestamp - OBS_REC_recording_start_time`
   - Save as `rec_timestamp` field if empty
3. **Open Video Replay**:
   - Open last saved video replay buffer clip in .mvp player
   - Position at last 10 seconds of the video
4. **Update IVR Links**:
   - Add video file path to `ivr_link` field for all events in last 20 seconds (if empty)
5. **OBS_STR Scene Change**:
   - Change scene to `IVR_SCENE`
6. **IVR Stream Overlay**:
   - Activate starting animation

#### Trigger 3: Challenge Resolution or Video Close
**Trigger Events**:
- PSS Event challenge with parameter win/lost
- .mvp player closed (ESC key pressed)
- Fallback: PSS event representing round continue

**Actions**:
1. **IVR Stream Overlay**:
   - Activate closing animation
2. **OBS_STR Scene Change**:
   - Change scene to `LIVE_SCENE`
3. **OBS_REC Replay Buffer Check**:
   - Check if video replay buffer is active
   - If not active, activate it

#### Additional Requirements:
- **OBS_STR WebSocket Connection**: Must be connected for stream operations

### Implementation Notes

- **Time Format**: All timestamps stored as ISO 8601 strings for consistency
- **Null Values**: Fields are nullable when OBS is not connected or timestamps unavailable
- **Performance**: Indices added for efficient timestamp-based queries
- **Error Handling**: Graceful handling when OBS integration is unavailable

### OBS Integration Implementation Details

#### OBS WebSocket Connections
- **OBS_REC**: Recording instance WebSocket connection
- **OBS_STR**: Streaming instance WebSocket connection
- **Connection Monitoring**: Real-time status tracking for both connections

#### Database-Driven Timestamp Calculation
- **OBS Stream Start Storage**: Store OBS_STR stream start timestamp in database
- **Database Triggers**: Automatically calculate `str_timestamp` when PSS events are saved
- **YouTube Chapter Generation**: Create chapter files from `str_timestamp` data for stream organization

#### Tournament Day Setup (One-time)
- **Recording Path**: Set tournament folder structure on tournament day activation
- **IVR Recordings Folder**: Create subfolder for replay buffer clips
- **Settings Integration**: Extract replay buffer duration from application settings

#### Match Setup Process
1. **Extract Match Data**: Parse player names, countries, match number from PSS events
2. **Generate File Names**: Create standardized naming convention for recordings
3. **Configure OBS**: Set recording path, name, and replay buffer settings
4. **Start Recording**: Begin OBS recording and replay buffer

#### IVR Trigger Processing
1. **Event Detection**: Monitor for challenge events (`ch0;`, `ch1;`, `ch2;`) and replay button clicks
2. **Timestamp Calculation**: Calculate relative timestamps for events in last 20 seconds
3. **Video Processing**: Save replay buffer and open in .mvp player
4. **Database Updates**: Update `rec_timestamp` and `ivr_link` fields for relevant events
5. **Scene Management**: Handle OBS scene transitions and overlay animations

#### Error Handling
- **Connection Failures**: Graceful degradation when OBS connections unavailable
- **File System Errors**: Handle missing folders, permission issues
- **Video Player Issues**: Fallback handling for .mvp player failures
- **Database Errors**: Rollback mechanisms for failed timestamp updates

### Database Trigger Approach for str_timestamp Calculation

#### Overview
Instead of calculating `str_timestamp` in application code, use database triggers to automatically calculate and store the relative timestamp when PSS events are saved. This approach provides:

1. **Automatic Calculation**: No application-level timestamp management needed
2. **Consistency**: All events get `str_timestamp` calculated the same way
3. **Performance**: Database-level calculation is faster than application-level
4. **Reliability**: Triggers ensure no events are missed

#### Stream Interruption Handling

**Problem**: Stream interruptions, laptop restarts, and stream restarts create time offsets that break timestamp calculations.

**Solutions**:

1. **Stream Session Management**: Track multiple stream sessions per tournament day
2. **Time Offset Calculation**: Calculate and apply time offsets between sessions
3. **Automatic Recovery**: Detect stream restarts and adjust timestamps accordingly
4. **Manual Override**: Allow manual timestamp correction for edge cases

#### Implementation Strategy

**1. Enhanced OBS Stream Session Management**
```sql
-- Enhanced table to handle stream interruptions and restarts
CREATE TABLE obs_stream_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    stream_start_timestamp TEXT NOT NULL,  -- ISO 8601 timestamp
    stream_end_timestamp TEXT,             -- NULL until stream ends
    tournament_id INTEGER,
    tournament_day_id INTEGER,
    session_number INTEGER DEFAULT 1,      -- Session number within tournament day (1, 2, 3...)
    is_active BOOLEAN DEFAULT TRUE,        -- Whether this session is currently active
    interruption_reason TEXT,              -- Reason for stream end (restart, crash, manual stop)
    time_offset_seconds INTEGER DEFAULT 0, -- Time offset from previous session
    cumulative_offset_seconds INTEGER DEFAULT 0, -- Total offset from first session
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions(id),
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id),
    FOREIGN KEY (tournament_day_id) REFERENCES tournament_days(id)
);

-- Index for fast lookups
CREATE INDEX idx_obs_stream_sessions_session ON obs_stream_sessions(session_id);
CREATE INDEX idx_obs_stream_sessions_tournament_day ON obs_stream_sessions(tournament_day_id, session_number);
CREATE INDEX idx_obs_stream_sessions_active ON obs_stream_sessions(is_active);
```

**2. Enhanced Database Trigger for str_timestamp Calculation**
```sql
-- Enhanced trigger to handle multiple stream sessions and time offsets
CREATE TRIGGER calculate_str_timestamp_trigger
AFTER INSERT ON pss_events_v2
FOR EACH ROW
WHEN NEW.str_timestamp IS NULL
BEGIN
    UPDATE pss_events_v2 
    SET str_timestamp = (
        SELECT 
            CASE 
                WHEN oss.stream_start_timestamp IS NOT NULL 
                THEN strftime('%H:%M:%S', 
                    ((julianday(NEW.timestamp) - julianday(oss.stream_start_timestamp)) * 24 * 3600) + oss.cumulative_offset_seconds
                )
                ELSE NULL 
            END
        FROM obs_stream_sessions oss
        WHERE oss.tournament_day_id = (
            SELECT tournament_day_id FROM pss_events_v2 WHERE id = NEW.id
        )
        AND oss.is_active = TRUE
        AND oss.stream_start_timestamp IS NOT NULL
        AND NEW.timestamp >= oss.stream_start_timestamp
        AND (oss.stream_end_timestamp IS NULL OR NEW.timestamp <= oss.stream_end_timestamp)
        ORDER BY oss.session_number DESC, oss.created_at DESC
        LIMIT 1
    )
    WHERE id = NEW.id;
END;
```

**3. YouTube Chapter Generation**
```sql
-- Function to generate YouTube chapter data
CREATE VIEW youtube_chapters AS
SELECT 
    session_id,
    match_number,
    str_timestamp,
    event_category,
    description,
    -- Format for YouTube chapters: "00:00:00 Chapter Title"
    str_timestamp || ' ' || 
    CASE 
        WHEN event_category = 'R' THEN 'Referee Decision'
        WHEN event_category = 'K' THEN 'Kick Event'
        WHEN event_category = 'P' THEN 'Punch Point'
        WHEN event_category = 'H' THEN 'Head Point'
        WHEN event_category = 'TH' THEN 'Technical Head Point'
        WHEN event_category = 'TB' THEN 'Technical Body Point'
        ELSE 'Match Event'
    END || ' - ' || COALESCE(description, '') as chapter_line
FROM pss_events_v2
WHERE str_timestamp IS NOT NULL
AND event_category IN ('R', 'K', 'P', 'H', 'TH', 'TB')
AND match_number IS NOT NULL
ORDER BY session_id, str_timestamp;
```

#### YouTube Chapter File Generation

**File Format**: YouTube accepts chapter files in the format:
```
00:00:00 Introduction
00:05:30 Match 101 - John Smith vs Mike Jones
00:12:45 First Point - Head Kick
00:18:20 Referee Decision - Challenge
```

**Generation Process**:
1. **Daily Export**: At tournament day end, export all `str_timestamp` data
2. **Filtering**: Include only significant events (referee decisions, points, challenges)
3. **Formatting**: Convert to YouTube chapter format
4. **File Creation**: Generate `.txt` file for YouTube upload

**Implementation**:
```rust
// Function to generate YouTube chapter file
async fn generate_youtube_chapters(
    tournament_day_id: i64,
    output_path: &str,
) -> AppResult<()> {
    let chapters = database.get_youtube_chapters(tournament_day_id).await?;
    
    let mut file = std::fs::File::create(output_path)?;
    
    for chapter in chapters {
        writeln!(file, "{}", chapter.chapter_line)?;
    }
    
    Ok(())
}

// Stream interruption handling functions
async fn handle_stream_interruption(
    tournament_day_id: i64,
    reason: &str,
    database: &DatabasePlugin,
) -> AppResult<()> {
    // End current active session
    database.end_active_stream_session(tournament_day_id, reason).await?;
    
    // Calculate time offset from last session
    let time_offset = database.calculate_session_time_offset(tournament_day_id).await?;
    
    // Update cumulative offset for next session
    database.update_cumulative_offset(tournament_day_id, time_offset).await?;
    
    Ok(())
}

async fn start_new_stream_session(
    tournament_day_id: i64,
    database: &DatabasePlugin,
) -> AppResult<i64> {
    // Get next session number
    let session_number = database.get_next_session_number(tournament_day_id).await?;
    
    // Start new session
    let session_id = database.create_new_stream_session(
        tournament_day_id,
        session_number,
        chrono::Utc::now().to_rfc3339()
    ).await?;
    
    Ok(session_id)
}

async fn detect_and_handle_stream_restart(
    tournament_day_id: i64,
    database: &DatabasePlugin,
) -> AppResult<()> {
    // Check if there's a gap in stream sessions
    let last_session = database.get_last_stream_session(tournament_day_id).await?;
    
    if let Some(session) = last_session {
        let time_since_last = chrono::Utc::now() - session.stream_start_timestamp;
        
        // If more than 5 minutes gap, consider it a restart
        if time_since_last.num_minutes() > 5 {
            log::info!("Detected stream restart after {} minutes gap", time_since_last.num_minutes());
            
            // Handle as interruption and start new session
            handle_stream_interruption(tournament_day_id, "auto_restart", database).await?;
            start_new_stream_session(tournament_day_id, database).await?;
        }
    }
    
    Ok(())
}

// Manual timestamp correction
async fn correct_timestamp_offset(
    tournament_day_id: i64,
    session_number: i32,
    offset_seconds: i64,
    database: &DatabasePlugin,
) -> AppResult<()> {
    // Update the specific session's offset
    database.update_session_offset(tournament_day_id, session_number, offset_seconds).await?;
    
    // Recalculate cumulative offsets for all subsequent sessions
    database.recalculate_cumulative_offsets(tournament_day_id).await?;
    
    // Update all affected str_timestamps
    database.recalculate_str_timestamps(tournament_day_id).await?;
    
    Ok(())
}
```

#### Benefits of This Approach

1. **Automatic Calculation**: No manual timestamp management
2. **Stream Organization**: Easy YouTube chapter creation
3. **Data Consistency**: All events have properly calculated timestamps
4. **Performance**: Database-level triggers are efficient
5. **Scalability**: Works for any number of events
6. **Reliability**: Triggers ensure no events are missed
7. **Stream Interruption Resilience**: Handles restarts, crashes, and interruptions automatically
8. **Time Offset Management**: Maintains continuous timestamps across multiple sessions

#### Stream Interruption Handling Strategies

**1. Automatic Detection**
- **Gap Detection**: Monitor for gaps > 5 minutes between sessions
- **Restart Detection**: Detect when OBS stream restarts after interruption
- **Session Tracking**: Maintain session numbers (1, 2, 3...) within tournament day

**2. Time Offset Calculation**
- **Session Offset**: Calculate time difference between session end and restart
- **Cumulative Offset**: Maintain running total of all offsets
- **Timestamp Adjustment**: Apply cumulative offset to all subsequent events

**3. Manual Override**
- **Offset Correction**: Allow manual adjustment of session offsets
- **Timestamp Recalculation**: Recalculate all affected timestamps after correction
- **Session Management**: View and manage all stream sessions

**4. Recovery Scenarios**

**Scenario A: Laptop Restart**
1. Detect gap in stream sessions
2. Calculate time offset from last session
3. Start new session with cumulative offset
4. Continue timestamp calculation with offset applied

**Scenario B: OBS Crash**
1. Detect OBS disconnection
2. End current session with "crash" reason
3. Wait for OBS reconnection
4. Start new session with calculated offset

**Scenario C: Manual Stream Stop/Start**
1. User manually stops stream
2. End current session with "manual_stop" reason
3. User manually starts new stream
4. Start new session with calculated offset

**Scenario D: Network Interruption**
1. Detect network loss
2. End current session with "network_loss" reason
3. Detect network restoration
4. Start new session with calculated offset

#### Implementation Timeline Addition

**Phase 2.5: Database Triggers and YouTube Integration (2 hours)**
- [ ] Create enhanced obs_stream_sessions table with interruption handling
- [ ] Implement enhanced str_timestamp calculation trigger with offsets
- [ ] Create YouTube chapters view
- [ ] Add YouTube chapter file generation function
- [ ] Add stream interruption detection and handling functions
- [ ] Add manual timestamp correction functions
- [ ] Test trigger functionality and interruption scenarios

## OBS Session Management Consolidation

### Unified OBS Sessions Table

**Problem**: Currently, OBS stream sessions and recording sessions are managed separately, leading to:
- Duplicate session tracking logic
- Inconsistent interruption handling
- Complex queries across multiple tables
- Difficulty in correlating recording and streaming sessions

**Solution**: Consolidate into a single `obs_sessions` table with session types and unified management.

#### Enhanced OBS Sessions Schema

```sql
-- Unified OBS sessions table for all session types
CREATE TABLE obs_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,                    -- Links to main session
    session_type TEXT NOT NULL,                     -- 'stream', 'recording', 'replay_buffer'
    obs_connection TEXT NOT NULL,                   -- 'OBS_REC', 'OBS_STR', 'OBS_BOTH'
    start_timestamp TEXT NOT NULL,                  -- ISO 8601 timestamp
    end_timestamp TEXT,                             -- NULL until session ends
    tournament_id INTEGER,
    tournament_day_id INTEGER,
    session_number INTEGER DEFAULT 1,               -- Session number within tournament day (1, 2, 3...)
    is_active BOOLEAN DEFAULT TRUE,                 -- Whether this session is currently active
    interruption_reason TEXT,                       -- Reason for session end (restart, crash, manual stop)
    time_offset_seconds INTEGER DEFAULT 0,          -- Time offset from previous session of same type
    cumulative_offset_seconds INTEGER DEFAULT 0,    -- Total offset from first session of same type
    recording_path TEXT,                            -- For recording sessions: base recording path
    recording_name TEXT,                            -- For recording sessions: current recording name
    stream_key TEXT,                                -- For stream sessions: stream key/URL
    replay_buffer_duration INTEGER DEFAULT 20,      -- For replay buffer: duration in seconds
    replay_buffer_path TEXT,                        -- For replay buffer: save path
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions(id),
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id),
    FOREIGN KEY (tournament_day_id) REFERENCES tournament_days(id)
);

-- Indices for performance and filtering
CREATE INDEX idx_obs_sessions_session ON obs_sessions(session_id);
CREATE INDEX idx_obs_sessions_type ON obs_sessions(session_type);
CREATE INDEX idx_obs_sessions_tournament_day ON obs_sessions(tournament_day_id, session_type, session_number);
CREATE INDEX idx_obs_sessions_active ON obs_sessions(is_active, session_type);
CREATE INDEX idx_obs_sessions_connection ON obs_sessions(obs_connection);
```

#### Session Type Management

**Session Types**:
- **`stream`**: OBS streaming sessions for YouTube chapter generation
- **`recording`**: OBS recording sessions for video file management
- **`replay_buffer`**: OBS replay buffer sessions for IVR clips

**Connection Types**:
- **`OBS_REC`**: Recording connection only
- **`OBS_STR`**: Streaming connection only  
- **`OBS_BOTH`**: Both recording and streaming connections

#### Enhanced Database Trigger for Unified Session Management

```sql
-- Enhanced trigger to handle multiple session types and time offsets
CREATE TRIGGER calculate_str_timestamp_trigger
AFTER INSERT ON pss_events_v2
FOR EACH ROW
WHEN NEW.str_timestamp IS NULL
BEGIN
    UPDATE pss_events_v2 
    SET str_timestamp = (
        SELECT 
            CASE 
                WHEN os.start_timestamp IS NOT NULL 
                THEN strftime('%H:%M:%S', 
                    ((julianday(NEW.timestamp) - julianday(os.start_timestamp)) * 24 * 3600) + os.cumulative_offset_seconds
                )
                ELSE NULL 
            END
        FROM obs_sessions os
        WHERE os.tournament_day_id = (
            SELECT tournament_day_id FROM pss_events_v2 WHERE id = NEW.id
        )
        AND os.session_type = 'stream'
        AND os.is_active = TRUE
        AND os.start_timestamp IS NOT NULL
        AND NEW.timestamp >= os.start_timestamp
        AND (os.end_timestamp IS NULL OR NEW.timestamp <= os.end_timestamp)
        ORDER BY os.session_number DESC, os.created_at DESC
        LIMIT 1
    )
    WHERE id = NEW.id;
END;

-- Trigger for automatic rec_timestamp calculation
CREATE TRIGGER calculate_rec_timestamp_trigger
AFTER INSERT ON pss_events_v2
FOR EACH ROW
WHEN NEW.rec_timestamp IS NULL
BEGIN
    UPDATE pss_events_v2 
    SET rec_timestamp = (
        SELECT 
            CASE 
                WHEN os.start_timestamp IS NOT NULL 
                THEN strftime('%H:%M:%S', 
                    ((julianday(NEW.timestamp) - julianday(os.start_timestamp)) * 24 * 3600) + os.cumulative_offset_seconds
                )
                ELSE NULL 
            END
        FROM obs_sessions os
        WHERE os.tournament_day_id = (
            SELECT tournament_day_id FROM pss_events_v2 WHERE id = NEW.id
        )
        AND os.session_type = 'recording'
        AND os.is_active = TRUE
        AND os.start_timestamp IS NOT NULL
        AND NEW.timestamp >= os.start_timestamp
        AND (os.end_timestamp IS NULL OR NEW.timestamp <= os.end_timestamp)
        ORDER BY os.session_number DESC, os.created_at DESC
        LIMIT 1
    )
    WHERE id = NEW.id;
END;
```

#### Unified Session Management Functions

```rust
// Unified OBS session management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObsSessionType {
    Stream,
    Recording,
    ReplayBuffer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObsConnection {
    Rec,
    Str,
    Both,
}

impl ObsSessionManager {
    // Start new OBS session
    pub async fn start_obs_session(
        &self,
        session_type: ObsSessionType,
        connection: ObsConnection,
        tournament_day_id: i64,
        recording_path: Option<String>,
        recording_name: Option<String>,
        stream_key: Option<String>,
        replay_buffer_duration: Option<i32>,
    ) -> AppResult<i64> {
        let session_number = self.get_next_session_number(tournament_day_id, &session_type).await?;
        
        let session_id = self.create_obs_session(
            session_type,
            connection,
            tournament_day_id,
            session_number,
            recording_path,
            recording_name,
            stream_key,
            replay_buffer_duration,
        ).await?;
        
        Ok(session_id)
    }
    
    // Handle session interruption
    pub async fn handle_session_interruption(
        &self,
        tournament_day_id: i64,
        session_type: ObsSessionType,
        reason: &str,
    ) -> AppResult<()> {
        // End current active session
        self.end_active_session(tournament_day_id, &session_type, reason).await?;
        
        // Calculate time offset from last session
        let time_offset = self.calculate_session_time_offset(tournament_day_id, &session_type).await?;
        
        // Update cumulative offset for next session
        self.update_cumulative_offset(tournament_day_id, &session_type, time_offset).await?;
        
        Ok(())
    }
    
    // Get active session for timestamp calculation
    pub async fn get_active_session(
        &self,
        tournament_day_id: i64,
        session_type: &ObsSessionType,
    ) -> AppResult<Option<ObsSession>> {
        // Query for active session of specified type
        let session = self.database.get_active_obs_session(tournament_day_id, session_type).await?;
        Ok(session)
    }
    
    // Detect and handle session restarts
    pub async fn detect_session_restart(
        &self,
        tournament_day_id: i64,
        session_type: ObsSessionType,
    ) -> AppResult<bool> {
        let last_session = self.get_last_session(tournament_day_id, &session_type).await?;
        
        if let Some(session) = last_session {
            let time_since_last = chrono::Utc::now() - session.start_timestamp;
            
            // If more than 5 minutes gap, consider it a restart
            if time_since_last.num_minutes() > 5 {
                log::info!("Detected {} restart after {} minutes gap", 
                    session_type.to_string(), time_since_last.num_minutes());
                
                // Handle as interruption and start new session
                self.handle_session_interruption(tournament_day_id, session_type.clone(), "auto_restart").await?;
                return Ok(true);
            }
        }
        
        Ok(false)
    }
}
```

#### Enhanced Tauri Commands for Unified Management

```rust
#[tauri::command]
pub async fn start_obs_session(
    session_type: String, // "stream", "recording", "replay_buffer"
    connection: String,   // "OBS_REC", "OBS_STR", "OBS_BOTH"
    tournament_day_id: i64,
    recording_path: Option<String>,
    recording_name: Option<String>,
    stream_key: Option<String>,
    replay_buffer_duration: Option<i32>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    let session_type = match session_type.as_str() {
        "stream" => ObsSessionType::Stream,
        "recording" => ObsSessionType::Recording,
        "replay_buffer" => ObsSessionType::ReplayBuffer,
        _ => return Err(TauriError::from(anyhow::anyhow!("Invalid session type"))),
    };
    
    let connection = match connection.as_str() {
        "OBS_REC" => ObsConnection::Rec,
        "OBS_STR" => ObsConnection::Str,
        "OBS_BOTH" => ObsConnection::Both,
        _ => return Err(TauriError::from(anyhow::anyhow!("Invalid connection type"))),
    };
    
    let session_id = app.obs_session_manager()
        .start_obs_session(
            session_type,
            connection,
            tournament_day_id,
            recording_path,
            recording_name,
            stream_key,
            replay_buffer_duration,
        )
        .await?;
    
    Ok(serde_json::json!({
        "success": true,
        "session_id": session_id,
        "message": "OBS session started successfully"
    }))
}

#[tauri::command]
pub async fn handle_obs_session_interruption(
    tournament_day_id: i64,
    session_type: String,
    reason: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    let session_type = match session_type.as_str() {
        "stream" => ObsSessionType::Stream,
        "recording" => ObsSessionType::Recording,
        "replay_buffer" => ObsSessionType::ReplayBuffer,
        _ => return Err(TauriError::from(anyhow::anyhow!("Invalid session type"))),
    };
    
    app.obs_session_manager()
        .handle_session_interruption(tournament_day_id, session_type, &reason)
        .await?;
    
    Ok(serde_json::json!({
        "success": true,
        "message": "Session interruption handled successfully"
    }))
}

#[tauri::command]
pub async fn get_obs_sessions(
    tournament_day_id: i64,
    session_type: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    let session_type_filter = session_type.map(|s| match s.as_str() {
        "stream" => ObsSessionType::Stream,
        "recording" => ObsSessionType::Recording,
        "replay_buffer" => ObsSessionType::ReplayBuffer,
        _ => return Err(TauriError::from(anyhow::anyhow!("Invalid session type"))),
    });
    
    let sessions = app.obs_session_manager()
        .get_sessions(tournament_day_id, session_type_filter.as_ref())
        .await?;
    
    Ok(serde_json::json!({
        "success": true,
        "sessions": sessions
    }))
}
```

#### Benefits of Unified Approach

1. **Simplified Management**: Single table for all OBS session types
2. **Consistent Interruption Handling**: Same logic applies to all session types
3. **Better Filtering**: Easy to query by session type, connection, or tournament context
4. **Unified Time Offset Management**: Consistent handling of time offsets across all session types
5. **Easier Maintenance**: One set of functions for all session management
6. **Better Correlation**: Easy to correlate recording and streaming sessions for the same tournament day
7. **Flexible Session Types**: Easy to add new session types in the future

#### Migration Strategy

1. **Phase 1**: Create new `obs_sessions` table alongside existing `obs_stream_sessions`
2. **Phase 2**: Migrate existing stream session data to new table
3. **Phase 3**: Update triggers and functions to use new table
4. **Phase 4**: Remove old `obs_stream_sessions` table
5. **Phase 5**: Add recording and replay buffer session support

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

-- Add OBS timestamp fields for video replay integration
ALTER TABLE pss_events_v2 ADD COLUMN rec_timestamp TEXT;  -- PSS event timestamp minus OBS recording start timestamp
ALTER TABLE pss_events_v2 ADD COLUMN str_timestamp TEXT;  -- PSS event timestamp minus OBS stream start timestamp
ALTER TABLE pss_events_v2 ADD COLUMN ivr_link TEXT;       -- Path to OBS video replay buffer clip if created ±10 seconds around PSS event timestamp

-- Create unified OBS sessions table for all session types (stream, recording, replay_buffer)
CREATE TABLE obs_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,                    -- Links to main session
    session_type TEXT NOT NULL,                     -- 'stream', 'recording', 'replay_buffer'
    obs_connection TEXT NOT NULL,                   -- 'OBS_REC', 'OBS_STR', 'OBS_BOTH'
    start_timestamp TEXT NOT NULL,                  -- ISO 8601 timestamp
    end_timestamp TEXT,                             -- NULL until session ends
    tournament_id INTEGER,
    tournament_day_id INTEGER,
    session_number INTEGER DEFAULT 1,               -- Session number within tournament day (1, 2, 3...)
    is_active BOOLEAN DEFAULT TRUE,                 -- Whether this session is currently active
    interruption_reason TEXT,                       -- Reason for session end (restart, crash, manual stop)
    time_offset_seconds INTEGER DEFAULT 0,          -- Time offset from previous session of same type
    cumulative_offset_seconds INTEGER DEFAULT 0,    -- Total offset from first session of same type
    recording_path TEXT,                            -- For recording sessions: base recording path
    recording_name TEXT,                            -- For recording sessions: current recording name
    stream_key TEXT,                                -- For stream sessions: stream key/URL
    replay_buffer_duration INTEGER DEFAULT 20,      -- For replay buffer: duration in seconds
    replay_buffer_path TEXT,                        -- For replay buffer: save path
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions(id),
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id),
    FOREIGN KEY (tournament_day_id) REFERENCES tournament_days(id)
);

-- Add indices for performance
CREATE INDEX IF NOT EXISTS idx_pss_events_v2_category ON pss_events_v2(event_category);
CREATE INDEX IF NOT EXISTS idx_pss_events_v2_tournament ON pss_events_v2(tournament_id, tournament_day_id);
CREATE INDEX IF NOT EXISTS idx_pss_events_v2_match_number ON pss_events_v2(match_number);
CREATE INDEX IF NOT EXISTS idx_pss_events_v2_rec_timestamp ON pss_events_v2(rec_timestamp);
CREATE INDEX IF NOT EXISTS idx_pss_events_v2_str_timestamp ON pss_events_v2(str_timestamp);
CREATE INDEX IF NOT EXISTS idx_obs_sessions_session ON obs_sessions(session_id);
CREATE INDEX IF NOT EXISTS idx_obs_sessions_type ON obs_sessions(session_type);
CREATE INDEX IF NOT EXISTS idx_obs_sessions_tournament_day ON obs_sessions(tournament_day_id, session_type, session_number);
CREATE INDEX IF NOT EXISTS idx_obs_sessions_active ON obs_sessions(is_active, session_type);
CREATE INDEX IF NOT EXISTS idx_obs_sessions_connection ON obs_sessions(obs_connection);

-- Create trigger for automatic str_timestamp calculation (stream sessions)
CREATE TRIGGER calculate_str_timestamp_trigger
AFTER INSERT ON pss_events_v2
FOR EACH ROW
WHEN NEW.str_timestamp IS NULL
BEGIN
    UPDATE pss_events_v2 
    SET str_timestamp = (
        SELECT 
            CASE 
                WHEN os.start_timestamp IS NOT NULL 
                THEN strftime('%H:%M:%S', 
                    ((julianday(NEW.timestamp) - julianday(os.start_timestamp)) * 24 * 3600) + os.cumulative_offset_seconds
                )
                ELSE NULL 
            END
        FROM obs_sessions os
        WHERE os.tournament_day_id = (
            SELECT tournament_day_id FROM pss_events_v2 WHERE id = NEW.id
        )
        AND os.session_type = 'stream'
        AND os.is_active = TRUE
        AND os.start_timestamp IS NOT NULL
        AND NEW.timestamp >= os.start_timestamp
        AND (os.end_timestamp IS NULL OR NEW.timestamp <= os.end_timestamp)
        ORDER BY os.session_number DESC, os.created_at DESC
        LIMIT 1
    )
    WHERE id = NEW.id;
END;

-- Create trigger for automatic rec_timestamp calculation (recording sessions)
CREATE TRIGGER calculate_rec_timestamp_trigger
AFTER INSERT ON pss_events_v2
FOR EACH ROW
WHEN NEW.rec_timestamp IS NULL
BEGIN
    UPDATE pss_events_v2 
    SET rec_timestamp = (
        SELECT 
            CASE 
                WHEN os.start_timestamp IS NOT NULL 
                THEN strftime('%H:%M:%S', 
                    ((julianday(NEW.timestamp) - julianday(os.start_timestamp)) * 24 * 3600) + os.cumulative_offset_seconds
                )
                ELSE NULL 
            END
        FROM obs_sessions os
        WHERE os.tournament_day_id = (
            SELECT tournament_day_id FROM pss_events_v2 WHERE id = NEW.id
        )
        AND os.session_type = 'recording'
        AND os.is_active = TRUE
        AND os.start_timestamp IS NOT NULL
        AND NEW.timestamp >= os.start_timestamp
        AND (os.end_timestamp IS NULL OR NEW.timestamp <= os.end_timestamp)
        ORDER BY os.session_number DESC, os.created_at DESC
        LIMIT 1
    )
    WHERE id = NEW.id;
END;

-- Create YouTube chapters view
CREATE VIEW youtube_chapters AS
SELECT 
    session_id,
    match_number,
    str_timestamp,
    event_category,
    description,
    -- Format for YouTube chapters: "00:00:00 Chapter Title"
    str_timestamp || ' ' || 
    CASE 
        WHEN event_category = 'R' THEN 'Referee Decision'
        WHEN event_category = 'K' THEN 'Kick Event'
        WHEN event_category = 'P' THEN 'Punch Point'
        WHEN event_category = 'H' THEN 'Head Point'
        WHEN event_category = 'TH' THEN 'Technical Head Point'
        WHEN event_category = 'TB' THEN 'Technical Body Point'
        ELSE 'Match Event'
    END || ' - ' || COALESCE(description, '') as chapter_line
FROM pss_events_v2
WHERE str_timestamp IS NOT NULL
AND event_category IN ('R', 'K', 'P', 'H', 'TH', 'TB')
AND match_number IS NOT NULL
ORDER BY session_id, str_timestamp;
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
    obs_recording_start: &Arc<Mutex<Option<String>>>,
    obs_stream_start: &Arc<Mutex<Option<String>>>,
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
    
    // Calculate OBS timestamps if available
    let rec_timestamp = if let Some(rec_start) = &*obs_recording_start.lock().await {
        calculate_relative_timestamp(&event.timestamp, rec_start)
    } else {
        None
    };
    
    let str_timestamp = if let Some(str_start) = &*obs_stream_start.lock().await {
        calculate_relative_timestamp(&event.timestamp, str_start)
    } else {
        None
    };
    
    Ok(PssEventV2 {
        // ... existing fields ...
        event_category: event_category.to_string(),
        tournament_id,
        tournament_day_id,
        match_number,
        rec_timestamp,
        str_timestamp,
        ivr_link: None, // Will be set during IVR trigger processing
        // ... rest of fields ...
    })
}

// Helper function to calculate relative timestamps
fn calculate_relative_timestamp(event_timestamp: &str, start_timestamp: &str) -> Option<String> {
    // Implementation to calculate time difference
    // Returns ISO 8601 duration format or None if calculation fails
}
```

3. **Add OBS Integration Functions:**
```rust
// OBS WebSocket connection management
struct ObsConnections {
    rec_connection: Arc<Mutex<Option<ObsWebSocket>>>,
    str_connection: Arc<Mutex<Option<ObsWebSocket>>>,
}

// Tournament day setup
async fn setup_tournament_day_recording(
    tournament_name: &str,
    tournament_day: &str,
    obs_rec: &ObsWebSocket,
) -> AppResult<()> {
    let recording_path = format!(
        "C:/Users/{}/Videos/{}/{}",
        std::env::var("USERNAME").unwrap_or_default(),
        tournament_name,
        tournament_day
    );
    
    // Create IVR recordings subfolder
    let ivr_path = format!("{}/IVR recordings", recording_path);
    std::fs::create_dir_all(&ivr_path)?;
    
    // Set OBS recording path
    obs_rec.set_recording_path(&recording_path).await?;
    
    Ok(())
}

// Match setup
async fn setup_match_recording(
    match_number: &str,
    player1: &str,
    player1_country: &str,
    player2: &str,
    player2_country: &str,
    obs_rec: &ObsWebSocket,
) -> AppResult<()> {
    let recording_name = format!(
        "{}_{}_{}_vs_{}_{}",
        match_number, player1, player1_country, player2, player2_country
    );
    
    // Set recording name and start recording
    obs_rec.set_recording_name(&recording_name).await?;
    obs_rec.start_recording().await?;
    obs_rec.start_replay_buffer().await?;
    
    Ok(())
}

// IVR trigger processing
async fn process_ivr_trigger(
    event: &PssEvent,
    database: &DatabasePlugin,
    obs_rec: &ObsWebSocket,
    obs_str: &ObsWebSocket,
) -> AppResult<()> {
    // Save replay buffer
    let replay_file = obs_rec.save_replay_buffer().await?;
    
    // Update events in last 20 seconds
    let recent_events = database.get_events_last_20_seconds().await?;
    
    for mut event in recent_events {
        if event.rec_timestamp.is_none() {
            event.rec_timestamp = calculate_rec_timestamp(&event.timestamp).await?;
        }
        if event.ivr_link.is_none() {
            event.ivr_link = Some(replay_file.clone());
        }
        database.update_event(event).await?;
    }
    
    // Open video in .mvp player
    open_mvp_player(&replay_file, 10).await?;
    
    // Change OBS scene
    obs_str.set_scene("IVR_SCENE").await?;
    
    Ok(())
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
    pub rec_timestamp: Option<String>, // PSS event timestamp minus OBS recording start timestamp
    pub str_timestamp: Option<String>, // PSS event timestamp minus OBS stream start timestamp
    pub ivr_link: Option<String>,      // Path to OBS video replay buffer clip if created ±10 seconds around PSS event timestamp
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
    
    // Get events in last 20 seconds for IVR processing
    pub fn get_events_last_20_seconds(
        conn: &Connection,
        session_id: i64
    ) -> DatabaseResult<Vec<PssEventV2>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_events_v2 
             WHERE session_id = ? 
             AND timestamp >= datetime('now', '-20 seconds')
             ORDER BY timestamp DESC"
        )?;
        
        let events = stmt.query_map(params![session_id], |row| {
            PssEventV2::from_row(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(events)
    }
    
    // Update event with OBS timestamp fields
    pub fn update_event_obs_fields(
        conn: &Connection,
        event_id: i64,
        rec_timestamp: Option<String>,
        str_timestamp: Option<String>,
        ivr_link: Option<String>
    ) -> DatabaseResult<()> {
        let mut stmt = conn.prepare(
            "UPDATE pss_events_v2 
             SET rec_timestamp = ?, str_timestamp = ?, ivr_link = ?
             WHERE id = ?"
        )?;
        
        stmt.execute(params![rec_timestamp, str_timestamp, ivr_link, event_id])?;
        
        Ok(())
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
            "rec_timestamp": event.rec_timestamp,
            "str_timestamp": event.str_timestamp,
            "ivr_link": event.ivr_link,
        }))
        .collect();
    
    Ok(serde_json::json!({
        "success": true,
        "events": events_json,
        "count": events_json.len()
    }))
}

#[tauri::command]
pub async fn generate_youtube_chapters(
    tournament_day_id: i64,
    output_path: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Generating YouTube chapters for tournament day: {}", tournament_day_id);
    
    let chapters = app.database_plugin().get_youtube_chapters(tournament_day_id).await?;
    
    // Create the output file
    let mut file = std::fs::File::create(&output_path)
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to create file: {}", e)))?;
    
    // Write chapter lines
    for chapter in chapters {
        writeln!(file, "{}", chapter.chapter_line)
            .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to write to file: {}", e)))?;
    }
    
    Ok(serde_json::json!({
        "success": true,
        "file_path": output_path,
        "chapter_count": chapters.len()
    }))
}

#[tauri::command]
pub async fn start_obs_stream_session(
    session_id: i64,
    tournament_id: Option<i64>,
    tournament_day_id: Option<i64>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Starting OBS stream session for session: {}", session_id);
    
    let stream_start = chrono::Utc::now().to_rfc3339();
    
    app.database_plugin().create_obs_stream_session(
        session_id,
        &stream_start,
        tournament_id,
        tournament_day_id
    ).await?;
    
    Ok(serde_json::json!({
        "success": true,
        "stream_start": stream_start
    }))
}

#[tauri::command]
pub async fn handle_stream_interruption(
    tournament_day_id: i64,
    reason: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Handling stream interruption for tournament day: {}, reason: {}", tournament_day_id, reason);
    
    // End current session and start new one
    app.database_plugin().handle_stream_interruption(tournament_day_id, &reason).await?;
    let new_session_id = app.database_plugin().start_new_stream_session(tournament_day_id).await?;
    
    Ok(serde_json::json!({
        "success": true,
        "new_session_id": new_session_id,
        "message": format!("Stream interruption handled. New session started.")
    }))
}

#[tauri::command]
pub async fn detect_stream_restart(
    tournament_day_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Detecting stream restart for tournament day: {}", tournament_day_id);
    
    let restart_detected = app.database_plugin().detect_and_handle_stream_restart(tournament_day_id).await?;
    
    Ok(serde_json::json!({
        "success": true,
        "restart_detected": restart_detected,
        "message": if restart_detected { "Stream restart detected and handled" } else { "No restart detected" }
    }))
}

#[tauri::command]
pub async fn correct_timestamp_offset(
    tournament_day_id: i64,
    session_number: i32,
    offset_seconds: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Correcting timestamp offset for tournament day: {}, session: {}, offset: {}s", 
               tournament_day_id, session_number, offset_seconds);
    
    app.database_plugin().correct_timestamp_offset(tournament_day_id, session_number, offset_seconds).await?;
    
    Ok(serde_json::json!({
        "success": true,
        "message": format!("Timestamp offset corrected for session {}", session_number)
    }))
}

#[tauri::command]
pub async fn get_stream_sessions(
    tournament_day_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting stream sessions for tournament day: {}", tournament_day_id);
    
    let sessions = app.database_plugin().get_stream_sessions(tournament_day_id).await?;
    
    Ok(serde_json::json!({
        "success": true,
        "sessions": sessions
    }))
}

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
  rec_timestamp: string | null; // PSS event timestamp minus OBS recording start timestamp
  str_timestamp: string | null; // PSS event timestamp minus OBS stream start timestamp
  ivr_link: string | null;      // Path to OBS video replay buffer clip if created ±10 seconds around PSS event timestamp
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

## Performance Optimization Strategy for Real-Time Speed and Resource Efficiency

### Current Performance Analysis

**Identified Bottlenecks:**
1. **UDP Processing**: Unbounded channels and excessive task spawning
2. **Database Operations**: Individual event storage with multiple transactions
3. **WebSocket Broadcasting**: Synchronous JSON serialization for each event
4. **Frontend Rendering**: Frequent re-renders without memoization
5. **Memory Usage**: Unbounded event queues and excessive cloning

### Phase 1: Backend Performance Optimizations (Critical Path)

#### 1.1 UDP Processing Pipeline Optimization

**Current Issues:**
- Unbounded channels causing memory bloat
- Excessive task spawning for each event
- Synchronous database operations blocking UDP processing

**Optimizations:**

```rust
// Replace unbounded channels with bounded channels for backpressure
let (event_tx, event_rx) = mpsc::channel::<PssEvent>(1000); // Bounded capacity

// Implement lock-free event queues using crossbeam
use crossbeam::queue::ArrayQueue;
use std::sync::Arc;

pub struct OptimizedUdpServer {
    event_queue: Arc<ArrayQueue<PssEvent>>,
    batch_processor: Arc<BatchProcessor>,
    // ... other fields
}

// Zero-copy event parsing with minimal allocations
#[derive(Debug, Clone)]
pub struct PssEventRef<'a> {
    pub event_type: &'a str,
    pub data: &'a [u8],
    pub timestamp: u64,
}

impl<'a> PssEventRef<'a> {
    pub fn from_udp_packet(packet: &'a [u8]) -> Option<Self> {
        // Zero-copy parsing without String allocations
        // Use byte slices and references instead of owned Strings
    }
}
```

#### 1.2 Database Optimization Strategy

**Current Issues:**
- Individual INSERT statements for each event
- Multiple database connections and transactions
- Synchronous database operations

**Optimizations:**

```rust
// Batch database operations with connection pooling
use deadpool_sqlite::{Pool, Runtime};

pub struct OptimizedDatabasePlugin {
    pool: Pool,
    batch_size: usize,
    batch_timeout: Duration,
}

impl OptimizedDatabasePlugin {
    // Batch INSERT with prepared statements
    pub async fn batch_insert_events(&self, events: Vec<PssEventV2>) -> AppResult<()> {
        let conn = self.pool.get().await?;
        
        // Use prepared statement for batch inserts
        let mut stmt = conn.prepare(
            "INSERT INTO pss_events_v2 (session_id, event_type_id, timestamp, raw_data, event_category, tournament_id, tournament_day_id, match_number, rec_timestamp, str_timestamp, ivr_link) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        ).await?;
        
        // Batch execute all events
        for event in events {
            stmt.execute(params![
                event.session_id,
                event.event_type_id,
                event.timestamp,
                event.raw_data,
                event.event_category,
                event.tournament_id,
                event.tournament_day_id,
                event.match_number,
                event.rec_timestamp,
                event.str_timestamp,
                event.ivr_link,
            ]).await?;
        }
        
        Ok(())
    }
}
```

#### 1.3 WebSocket Broadcasting Optimization

**Current Issues:**
- JSON serialization for each event
- Synchronous broadcasting to all clients
- No message compression

**Optimizations:**

```rust
// Binary serialization with Protocol Buffers
use prost::Message;

#[derive(Message)]
pub struct OptimizedPssEvent {
    #[prost(uint32, tag = "1")]
    pub event_type: u32,
    #[prost(bytes, tag = "2")]
    pub data: Vec<u8>,
    #[prost(uint64, tag = "3")]
    pub timestamp: u64,
}

// Asynchronous broadcasting with backpressure
impl WebSocketServer {
    pub async fn broadcast_event_optimized(&self, event: &PssEvent) -> AppResult<()> {
        // Serialize once, broadcast to all
        let binary_data = self.serialize_event_binary(event)?;
        
        // Use tokio::spawn for non-blocking broadcast
        let clients = self.clients.clone();
        tokio::spawn(async move {
            let mut failed_clients = Vec::new();
            
            for client in clients.lock().await.iter() {
                if let Err(_) = client.send_binary(&binary_data).await {
                    failed_clients.push(client.id.clone());
                }
            }
            
            // Remove failed clients
            if !failed_clients.is_empty() {
                // Cleanup failed clients
            }
        });
        
        Ok(())
    }
}
```

### Phase 2: Frontend Performance Optimizations

#### 2.1 React Component Optimization

**Current Issues:**
- Frequent re-renders without memoization
- Large event arrays in memory
- No virtualization for long lists

**Optimizations:**

```typescript
// Memoized event components with React.memo
const EventItem = React.memo<{ event: PssEventData }>(({ event }) => {
  return (
    <div className="event-item">
      <span className="event-type">{event.eventType}</span>
      <span className="event-time">{event.time}</span>
    </div>
  );
}, (prevProps, nextProps) => {
  // Custom comparison for optimal re-rendering
  return prevProps.event.id === nextProps.event.id &&
         prevProps.event.timestamp === nextProps.event.timestamp;
});

// Virtualized event list for performance
import { FixedSizeList as List } from 'react-window';

const EventTableSection: React.FC = () => {
  const { events } = useDatabaseEventStore();
  
  return (
    <List
      height={400}
      itemCount={events.length}
      itemSize={35}
      width="100%"
    >
      {({ index, style }) => (
        <EventItem 
          style={style} 
          event={events[index]} 
        />
      )}
    </List>
  );
};
```

#### 2.2 State Management Optimization

**Current Issues:**
- Zustand store updates causing full re-renders
- No selective subscriptions
- Large state objects

**Optimizations:**

```typescript
// Selective subscriptions with Zustand
export const useDatabaseEventStore = create<DatabaseEventState>()(
  subscribeWithSelector((set, get) => ({
    events: [],
    filters: {},
    
    // Selective updates
    addEvent: (event: DatabaseEvent) => {
      set((state) => ({
        events: [event, ...state.events.slice(0, 999)] // Keep only last 1000 events
      }));
    },
    
    // Batch updates
    addEventsBatch: (events: DatabaseEvent[]) => {
      set((state) => ({
        events: [...events, ...state.events].slice(0, 999)
      }));
    },
  }))
);

// Selective subscriptions
export const useEventCount = () => 
  useDatabaseEventStore((state) => state.events.length);

export const useFilteredEvents = (filter: EventFilter) => 
  useDatabaseEventStore((state) => 
    state.events.filter(event => filter(event))
  );
```

#### 2.3 Real-Time Update Optimization

**Current Issues:**
- Polling every 2 seconds
- No throttling or debouncing
- Full data fetches

**Optimizations:**

```typescript
// WebSocket with binary messages and throttling
class OptimizedWebSocket {
  private ws: WebSocket | null = null;
  private messageQueue: any[] = [];
  private throttleTimer: NodeJS.Timeout | null = null;
  
  constructor(private url: string, private onMessage: (data: any) => void) {}
  
  connect() {
    this.ws = new WebSocket(this.url);
    this.ws.binaryType = 'arraybuffer';
    
    this.ws.onmessage = (event) => {
      // Parse binary message
      const data = this.parseBinaryMessage(event.data);
      
      // Throttle updates to prevent UI blocking
      this.messageQueue.push(data);
      
      if (!this.throttleTimer) {
        this.throttleTimer = setTimeout(() => {
          this.processMessageQueue();
        }, 16); // ~60fps
      }
    };
  }
  
  private processMessageQueue() {
    this.throttleTimer = null;
    const batch = this.messageQueue.splice(0);
    
    // Batch update state
    this.onMessage(batch);
  }
}
```

### Phase 3: Memory and Resource Optimization

#### 3.1 Memory Management

**Current Issues:**
- Unbounded event queues
- Excessive cloning and allocations
- No memory limits

**Optimizations:**

```rust
// Memory-bounded event queues
use std::collections::VecDeque;

pub struct BoundedEventQueue {
    queue: VecDeque<PssEvent>,
    max_size: usize,
    max_memory_mb: usize,
}

impl BoundedEventQueue {
    pub fn new(max_size: usize, max_memory_mb: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(max_size),
            max_size,
            max_memory_mb,
        }
    }
    
    pub fn push(&mut self, event: PssEvent) {
        if self.queue.len() >= self.max_size {
            self.queue.pop_back(); // Remove oldest event
        }
        self.queue.push_front(event);
    }
}

// Object pooling for frequently allocated objects
use std::sync::Arc;
use parking_lot::Mutex;

pub struct ObjectPool<T> {
    pool: Arc<Mutex<Vec<T>>>,
    create_fn: Box<dyn Fn() -> T + Send + Sync>,
}

impl<T> ObjectPool<T> {
    pub fn acquire(&self) -> T {
        if let Some(obj) = self.pool.lock().pop() {
            obj
        } else {
            (self.create_fn)()
        }
    }
    
    pub fn release(&self, obj: T) {
        self.pool.lock().push(obj);
    }
}
```

#### 3.2 CPU Optimization

**Current Issues:**
- CPU-bound operations in async tasks
- No work stealing
- Inefficient parsing

**Optimizations:**

```rust
// CPU-bound work offloading
impl UdpServer {
    async fn process_event_cpu_intensive(&self, event: PssEvent) -> AppResult<()> {
        // Offload CPU-intensive work to blocking thread pool
        let result = tokio::task::spawn_blocking(move || {
            // CPU-intensive processing here
            process_event_heavy_computation(event)
        }).await?;
        
        Ok(result)
    }
}

// SIMD-optimized parsing for high-frequency events
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

fn parse_pss_message_simd(message: &[u8]) -> Option<PssEvent> {
    // Use SIMD instructions for fast string parsing
    // This can be 4-8x faster than standard string operations
}
```

### Phase 4: Network and I/O Optimization

#### 4.1 UDP Socket Optimization

**Current Issues:**
- Small receive buffers
- No socket options optimization
- Synchronous socket operations

**Optimizations:**

```rust
// Optimized UDP socket configuration
impl UdpServer {
    async fn create_optimized_socket(&self, port: u16) -> AppResult<UdpSocket> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).await?;
        
        // Increase receive buffer size
        socket.set_recv_buffer_size(1024 * 1024)?; // 1MB buffer
        
        // Set socket options for high-performance
        socket.set_nodelay(true)?;
        socket.set_keepalive(true)?;
        
        // Enable kernel-level optimizations
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::io::AsRawFd;
            let fd = socket.as_raw_fd();
            
            // Set SO_REUSEPORT for load balancing
            unsafe {
                let optval: libc::c_int = 1;
                libc::setsockopt(
                    fd,
                    libc::SOL_SOCKET,
                    libc::SO_REUSEPORT,
                    &optval as *const _ as *const libc::c_void,
                    std::mem::size_of_val(&optval) as libc::socklen_t,
                );
            }
        }
        
        Ok(socket)
    }
}
```

#### 4.2 Database Connection Pooling

**Current Issues:**
- Single database connection
- No connection pooling
- Synchronous database operations

**Optimizations:**

```rust
// Database connection pooling with async operations
use deadpool_sqlite::{Pool, Runtime};

pub struct OptimizedDatabasePlugin {
    pool: Pool,
    prepared_statements: Arc<Mutex<HashMap<String, PreparedStatement>>>,
}

impl OptimizedDatabasePlugin {
    pub async fn new(database_path: &str) -> AppResult<Self> {
        let pool = Pool::builder(
            deadpool_sqlite::Manager::new(database_path)
        )
        .max_size(20) // Pool size based on CPU cores
        .build()
        .map_err(|e| AppError::ConfigError(e.to_string()))?;
        
        Ok(Self {
            pool,
            prepared_statements: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    // Prepared statement caching
    pub async fn get_prepared_statement(&self, sql: &str) -> AppResult<PreparedStatement> {
        if let Some(stmt) = self.prepared_statements.lock().get(sql) {
            return Ok(stmt.clone());
        }
        
        let conn = self.pool.get().await?;
        let stmt = conn.prepare(sql).await?;
        
        self.prepared_statements.lock().insert(sql.to_string(), stmt.clone());
        Ok(stmt)
    }
}
```

### Phase 5: Monitoring and Profiling

#### 5.1 Performance Metrics

```rust
// Real-time performance monitoring
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub events_per_second: f64,
    pub average_latency_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub database_operations_per_second: f64,
    pub websocket_messages_per_second: f64,
}

impl PerformanceMonitor {
    pub fn track_event_processing(&self, start_time: Instant) {
        let duration = start_time.elapsed();
        self.update_metrics(duration);
    }
    
    pub fn get_metrics(&self) -> PerformanceMetrics {
        // Return current performance metrics
    }
}
```

#### 5.2 Resource Usage Monitoring

```rust
// Memory and CPU monitoring
impl UdpServer {
    pub fn monitor_resources(&self) {
        let memory_usage = self.get_memory_usage();
        let cpu_usage = self.get_cpu_usage();
        
        if memory_usage > 80.0 || cpu_usage > 90.0 {
            log::warn!("High resource usage detected: Memory: {}%, CPU: {}%", 
                      memory_usage, cpu_usage);
            
            // Implement backpressure or throttling
            self.enable_backpressure();
        }
    }
}
```

### Implementation Priority

1. **Critical (Week 1)**: UDP processing pipeline and database batching
2. **High (Week 2)**: WebSocket optimization and frontend memoization
3. **Medium (Week 3)**: Memory management and object pooling
4. **Low (Week 4)**: Advanced optimizations and monitoring

### Expected Performance Improvements

- **Latency**: 50-80% reduction in event processing time
- **Throughput**: 3-5x increase in events per second
- **Memory Usage**: 60-70% reduction in memory footprint
- **CPU Usage**: 40-60% reduction in CPU utilization
- **Real-time Responsiveness**: Sub-10ms event-to-UI latency

### Resource Usage Targets

- **Memory**: < 100MB for typical tournament day
- **CPU**: < 20% average usage during peak events
- **Network**: < 1MB/s for WebSocket traffic
- **Database**: < 1000 operations per second

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
- OBS video replay integration ready
- YouTube chapter generation ready
- Automated stream organization
- Stream Interruption Resilience
- Time Offset Management
- Unified OBS Session Management
- Flexible Session Type Support

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
- **OBS Integration**: Ensure OBS WebSocket connection is established before calculating timestamps
- **Timestamp Synchronization**: Verify PSS and OBS timestamps are properly synchronized
- **Replay Buffer**: Monitor OBS replay buffer events to capture video clips within ±10 seconds of PSS events
- **OBS WebSocket Connections**: Maintain separate connections for OBS_REC (recording) and OBS_STR (streaming)
- **Tournament Day Setup**: Execute recording path setup once per tournament day activation
- **IVR Trigger Detection**: Monitor for challenge events (`ch0;`, `ch1;`, `ch2;`) and replay button clicks
- **Video Player Integration**: Ensure .mvp player can be controlled programmatically for video positioning
- **Scene Management**: Handle OBS scene transitions between LIVE_SCENE and IVR_SCENE
- **File System Operations**: Create tournament folder structure and handle IVR recordings subfolder
- **Error Recovery**: Implement fallback mechanisms for OBS connection failures and video player issues
- **Unified Session Management**: Use single obs_sessions table for all session types (stream, recording, replay_buffer)
- **Session Type Filtering**: Implement proper filtering by session type and connection type
- **Interruption Handling**: Apply consistent interruption handling logic across all session types
- **Time Offset Management**: Maintain cumulative offsets for accurate timestamp calculations across session restarts

---

**Document Created**: 2025-01-29
**Purpose**: Temporary planning document for Event Table database enhancement
**Status**: Ready for implementation
**Next Action**: Start Phase 1 - Database Migration 

---

## 🏗️ Master/Slave Architecture & Central Management System

### **System Overview**

The Master/Slave architecture transforms the reStrike VTA system into a comprehensive tournament management platform with centralized control, monitoring, and coordination capabilities.

#### **Architecture Components**
```
┌─────────────────────────────────────────────────────────────────┐
│                        MASTER NODE                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │   Central DB    │  │  YT Manager     │  │  IVR Central    │  │
│  │   (SQLite)      │  │   Controller    │  │     Desk        │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
│           │                     │                     │          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │  Remote Control │  │  Scene Manager  │  │  Health Monitor │  │
│  │     Drawer      │  │   (Bulk/Indiv)  │  │   (All Slaves)  │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │   Network Layer   │
                    │  (Auto Discovery) │
                    └─────────┬─────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
┌───────▼────────┐  ┌─────────▼─────────┐  ┌───────▼────────┐
│   SLAVE NODE 1 │  │   SLAVE NODE 2    │  │   SLAVE NODE N │
│  ┌───────────┐ │  │  ┌───────────┐    │  │  ┌───────────┐ │
│  │ OBS_STR   │ │  │  │ OBS_STR   │    │  │  │ OBS_STR   │ │
│  │ OBS_REC   │ │  │  │ OBS_REC   │    │  │  │ OBS_REC   │ │
│  │ IVR Work  │ │  │  │ IVR Work  │    │  │  │ IVR Work  │ │
│  │ Station   │ │  │  │ Station   │    │  │  │ Station   │ │
│  └───────────┘ │  │  └───────────┘    │  │  └───────────┘ │
└────────────────┘  └───────────────────┘  └────────────────┘
```

### **Master Node Features**

#### **1. Central Database Management**
```sql
-- Central database schema for master node
CREATE TABLE central_tournaments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tournament_name TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT,
    status TEXT DEFAULT 'active', -- 'preparing', 'active', 'finishing', 'completed'
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE central_slave_nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id TEXT UNIQUE NOT NULL,           -- Unique identifier for each slave
    hostname TEXT NOT NULL,
    ip_address TEXT NOT NULL,
    last_seen TEXT DEFAULT CURRENT_TIMESTAMP,
    status TEXT DEFAULT 'online',           -- 'online', 'offline', 'error'
    obs_rec_status TEXT DEFAULT 'disconnected',
    obs_str_status TEXT DEFAULT 'disconnected',
    current_scene TEXT,
    recording_status TEXT DEFAULT 'stopped',
    stream_status TEXT DEFAULT 'stopped',
    disk_usage_percent REAL DEFAULT 0.0,
    cpu_usage_percent REAL DEFAULT 0.0,
    memory_usage_percent REAL DEFAULT 0.0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE central_obs_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slave_node_id INTEGER NOT NULL,
    session_type TEXT NOT NULL,             -- 'stream', 'recording', 'replay_buffer'
    start_timestamp TEXT NOT NULL,
    end_timestamp TEXT,
    tournament_id INTEGER,
    tournament_day_id INTEGER,
    recording_path TEXT,
    recording_name TEXT,
    stream_key TEXT,
    status TEXT DEFAULT 'active',           -- 'active', 'stopped', 'error'
    health_status TEXT DEFAULT 'healthy',   -- 'healthy', 'warning', 'error'
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (slave_node_id) REFERENCES central_slave_nodes(id),
    FOREIGN KEY (tournament_id) REFERENCES central_tournaments(id)
);

CREATE TABLE central_ivr_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slave_node_id INTEGER NOT NULL,
    event_id TEXT NOT NULL,                 -- Original PSS event ID from slave
    event_timestamp TEXT NOT NULL,
    event_category TEXT NOT NULL,
    match_number TEXT,
    player1_name TEXT,
    player2_name TEXT,
    rec_timestamp TEXT,
    str_timestamp TEXT,
    ivr_link TEXT,                          -- Path to video clip
    escalation_status TEXT DEFAULT 'pending', -- 'pending', 'reviewed', 'resolved'
    escalation_notes TEXT,
    reviewed_by TEXT,
    reviewed_at TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (slave_node_id) REFERENCES central_slave_nodes(id)
);

CREATE TABLE central_yt_streams (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tournament_id INTEGER NOT NULL,
    stream_title TEXT NOT NULL,
    stream_key TEXT NOT NULL,
    yt_channel_id TEXT NOT NULL,
    status TEXT DEFAULT 'offline',          -- 'offline', 'live', 'ended'
    health_status TEXT DEFAULT 'healthy',   -- 'healthy', 'warning', 'error'
    viewer_count INTEGER DEFAULT 0,
    chat_enabled BOOLEAN DEFAULT TRUE,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (tournament_id) REFERENCES central_tournaments(id)
);

CREATE TABLE central_announcements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    announcement_type TEXT NOT NULL,        -- 'chat', 'broadcast', 'system'
    target_nodes TEXT,                      -- JSON array of node IDs or 'all'
    message TEXT NOT NULL,
    priority TEXT DEFAULT 'normal',         -- 'low', 'normal', 'high', 'urgent'
    status TEXT DEFAULT 'pending',          -- 'pending', 'sent', 'failed'
    sent_at TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

#### **2. Remote Control Drawer**
```typescript
// Master node remote control interface
interface MasterRemoteControl {
  // Bulk operations across all slaves
  muteAllStreams(): Promise<void>;
  unmuteAllStreams(): Promise<void>;
  startAllRecordings(): Promise<void>;
  stopAllRecordings(): Promise<void>;
  changeAllScenes(sceneName: string): Promise<void>;
  
  // Individual slave control
  controlSlave(slaveId: string, action: SlaveAction): Promise<void>;
  getSlaveStatus(slaveId: string): Promise<SlaveStatus>;
  
  // Tournament management
  activateTournamentDay(tournamentId: string, dayNumber: number): Promise<void>;
  prepareTournamentFolders(tournamentId: string): Promise<void>;
  finishTournamentDay(tournamentId: string, dayNumber: number): Promise<void>;
  
  // Health monitoring
  getSystemHealth(): Promise<SystemHealthReport>;
  getSlaveHealth(slaveId: string): Promise<SlaveHealthReport>;
}

interface SlaveAction {
  type: 'mute' | 'unmute' | 'start_recording' | 'stop_recording' | 'change_scene' | 'restart_obs';
  parameters?: Record<string, any>;
}

interface SlaveStatus {
  nodeId: string;
  hostname: string;
  ipAddress: string;
  status: 'online' | 'offline' | 'error';
  obsRecStatus: string;
  obsStrStatus: string;
  currentScene: string;
  recordingStatus: string;
  streamStatus: string;
  diskUsage: number;
  cpuUsage: number;
  memoryUsage: number;
  lastSeen: string;
}
```

#### **3. YT Manager Controller**
```typescript
// YouTube stream management and control
interface YTManagerController {
  // Stream management
  createLiveStream(tournamentId: string, title: string): Promise<YTStreamInfo>;
  startLiveStream(streamId: string): Promise<void>;
  endLiveStream(streamId: string): Promise<void>;
  updateStreamTitle(streamId: string, title: string): Promise<void>;
  
  // Chat management
  sendChatMessage(streamId: string, message: string): Promise<void>;
  moderateChat(streamId: string, action: ChatModerationAction): Promise<void>;
  getChatMessages(streamId: string, limit?: number): Promise<ChatMessage[]>;
  
  // Chapter management
  uploadChapters(streamId: string, chapters: YTChapter[]): Promise<void>;
  updateChapters(streamId: string, chapters: YTChapter[]): Promise<void>;
  
  // Health monitoring
  getStreamHealth(streamId: string): Promise<YTStreamHealth>;
  getChannelAnalytics(channelId: string): Promise<YTChannelAnalytics>;
}

interface YTStreamInfo {
  streamId: string;
  title: string;
  streamKey: string;
  rtmpUrl: string;
  status: 'offline' | 'live' | 'ended';
  viewerCount: number;
  healthStatus: 'healthy' | 'warning' | 'error';
}

interface YTChapter {
  timestamp: string; // HH:MM:SS format
  title: string;
  description?: string;
}

interface ChatModerationAction {
  type: 'delete' | 'timeout' | 'ban';
  userId: string;
  duration?: number; // seconds for timeout
  reason?: string;
}
```

#### **4. IVR Central Desk**
```typescript
// Central IVR review and management
interface IVRCentralDesk {
  // IVR event review
  getEscalatedEvents(filters?: IVREventFilters): Promise<IVREvent[]>;
  reviewEvent(eventId: string, review: IVREventReview): Promise<void>;
  bulkReviewEvents(eventIds: string[], review: IVREventReview): Promise<void>;
  
  // Video management
  downloadSlaveVideo(slaveId: string, videoPath: string): Promise<string>;
  uploadVideoToYT(videoPath: string, title: string, description?: string): Promise<string>;
  cleanupOldVideos(olderThanDays: number): Promise<number>;
  
  // Report generation
  generateIVRReport(tournamentId: string, dateRange?: DateRange): Promise<IVRReport>;
  generateTournamentReport(tournamentId: string): Promise<TournamentReport>;
  
  // Remote support
  sendMessageToSlave(slaveId: string, message: string): Promise<void>;
  broadcastMessage(message: string, targetSlaves?: string[]): Promise<void>;
  getSlaveLogs(slaveId: string, logType: 'obs' | 'system' | 'application'): Promise<string[]>;
}

interface IVREventFilters {
  tournamentId?: string;
  dateRange?: DateRange;
  escalationStatus?: 'pending' | 'reviewed' | 'resolved';
  eventCategory?: string[];
  slaveNodeId?: string;
}

interface IVREventReview {
  status: 'approved' | 'rejected' | 'needs_review';
  notes?: string;
  reviewer: string;
  action?: 'upload_to_yt' | 'delete' | 'archive';
}

interface IVRReport {
  totalEvents: number;
  escalatedEvents: number;
  resolvedEvents: number;
  averageResponseTime: number;
  topEventCategories: Array<{category: string, count: number}>;
  slavePerformance: Array<{slaveId: string, eventsProcessed: number, avgProcessingTime: number}>;
}
```

### **Slave Node Features**

#### **1. Auto-Discovery and Registration**
```rust
// Slave node auto-discovery implementation
pub struct SlaveDiscovery {
    node_id: String,
    hostname: String,
    master_endpoints: Vec<String>,
    discovery_interval: Duration,
    registration_retry_interval: Duration,
}

impl SlaveDiscovery {
    pub async fn start_discovery(&self) -> AppResult<()> {
        loop {
            // Try to discover master nodes on local network
            if let Some(master_endpoint) = self.discover_master().await? {
                // Register with master
                if self.register_with_master(&master_endpoint).await? {
                    info!("Successfully registered with master: {}", master_endpoint);
                    break;
                }
            }
            
            // Wait before retry
            tokio::time::sleep(self.discovery_interval).await;
        }
        
        // Start heartbeat to maintain connection
        self.start_heartbeat().await?;
        
        Ok(())
    }
    
    async fn discover_master(&self) -> AppResult<Option<String>> {
        // Use mDNS/Bonjour for local network discovery
        // Fallback to UDP broadcast for discovery
        // Return first responding master endpoint
        
        // Implementation would use tokio-dns or similar
        Ok(None) // Placeholder
    }
    
    async fn register_with_master(&self, master_endpoint: &str) -> AppResult<bool> {
        let registration_data = SlaveRegistration {
            node_id: self.node_id.clone(),
            hostname: self.hostname.clone(),
            ip_address: self.get_local_ip().await?,
            obs_rec_status: self.get_obs_rec_status().await?,
            obs_str_status: self.get_obs_str_status().await?,
            current_scene: self.get_current_scene().await?,
            recording_status: self.get_recording_status().await?,
            stream_status: self.get_stream_status().await?,
        };
        
        // Send registration to master
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/api/slaves/register", master_endpoint))
            .json(&registration_data)
            .send()
            .await?;
            
        Ok(response.status().is_success())
    }
    
    async fn start_heartbeat(&self) -> AppResult<()> {
        loop {
            // Send heartbeat to master with current status
            let heartbeat_data = SlaveHeartbeat {
                node_id: self.node_id.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                obs_rec_status: self.get_obs_rec_status().await?,
                obs_str_status: self.get_obs_str_status().await?,
                current_scene: self.get_current_scene().await?,
                recording_status: self.get_recording_status().await?,
                stream_status: self.get_stream_status().await?,
                disk_usage: self.get_disk_usage().await?,
                cpu_usage: self.get_cpu_usage().await?,
                memory_usage: self.get_memory_usage().await?,
            };
            
            // Send heartbeat (implementation details)
            
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    }
}
```

#### **2. Master Command Processing**
```rust
// Slave command processing for master control
pub struct MasterCommandProcessor {
    obs_controller: Arc<ObsController>,
    event_processor: Arc<EventProcessor>,
    command_queue: Arc<Mutex<VecDeque<MasterCommand>>>,
}

impl MasterCommandProcessor {
    pub async fn process_master_commands(&self) -> AppResult<()> {
        loop {
            // Check for new commands from master
            if let Some(command) = self.command_queue.lock().await.pop_front() {
                match command {
                    MasterCommand::MuteStream => {
                        self.obs_controller.mute_stream().await?;
                    }
                    MasterCommand::UnmuteStream => {
                        self.obs_controller.unmute_stream().await?;
                    }
                    MasterCommand::StartRecording => {
                        self.obs_controller.start_recording().await?;
                    }
                    MasterCommand::StopRecording => {
                        self.obs_controller.stop_recording().await?;
                    }
                    MasterCommand::ChangeScene { scene_name } => {
                        self.obs_controller.set_current_scene(&scene_name).await?;
                    }
                    MasterCommand::RestartObs => {
                        self.obs_controller.restart_obs().await?;
                    }
                    MasterCommand::SendMessage { message } => {
                        self.display_message(&message).await?;
                    }
                }
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    async fn display_message(&self, message: &str) -> AppResult<()> {
        // Display message on slave UI
        // Could be toast notification, overlay, etc.
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MasterCommand {
    MuteStream,
    UnmuteStream,
    StartRecording,
    StopRecording,
    ChangeScene { scene_name: String },
    RestartObs,
    SendMessage { message: String },
}
```

#### **3. Data Synchronization**
```rust
// Data sync from slave to master
pub struct DataSyncManager {
    master_endpoint: String,
    sync_interval: Duration,
    event_buffer: Arc<Mutex<Vec<PssEventV2>>>,
    ivr_buffer: Arc<Mutex<Vec<IVREvent>>>,
}

impl DataSyncManager {
    pub async fn start_sync(&self) -> AppResult<()> {
        loop {
            // Sync PSS events
            self.sync_pss_events().await?;
            
            // Sync IVR events
            self.sync_ivr_events().await?;
            
            // Sync health status
            self.sync_health_status().await?;
            
            tokio::time::sleep(self.sync_interval).await;
        }
    }
    
    async fn sync_pss_events(&self) -> AppResult<()> {
        let events = {
            let mut buffer = self.event_buffer.lock().await;
            buffer.drain(..).collect::<Vec<_>>()
        };
        
        if !events.is_empty() {
            let client = reqwest::Client::new();
            let response = client
                .post(&format!("{}/api/events/bulk", self.master_endpoint))
                .json(&events)
                .send()
                .await?;
                
            if !response.status().is_success() {
                warn!("Failed to sync PSS events to master");
            }
        }
        
        Ok(())
    }
    
    async fn sync_ivr_events(&self) -> AppResult<()> {
        let events = {
            let mut buffer = self.ivr_buffer.lock().await;
            buffer.drain(..).collect::<Vec<_>>()
        };
        
        if !events.is_empty() {
            let client = reqwest::Client::new();
            let response = client
                .post(&format!("{}/api/ivr-events/bulk", self.master_endpoint))
                .json(&events)
                .send()
                .await?;
                
            if !response.status().is_success() {
                warn!("Failed to sync IVR events to master");
            }
        }
        
        Ok(())
    }
}
```

### **Network Discovery and Communication**

#### **1. Auto-Discovery Protocol**
```rust
// Network discovery using UDP broadcast and mDNS
pub struct NetworkDiscovery {
    discovery_port: u16,
    service_name: String,
    service_type: String,
}

impl NetworkDiscovery {
    pub async fn discover_masters(&self) -> AppResult<Vec<String>> {
        let mut master_endpoints = Vec::new();
        
        // Method 1: UDP broadcast discovery
        if let Ok(endpoints) = self.udp_broadcast_discovery().await {
            master_endpoints.extend(endpoints);
        }
        
        // Method 2: mDNS/Bonjour discovery
        if let Ok(endpoints) = self.mdns_discovery().await {
            master_endpoints.extend(endpoints);
        }
        
        // Method 3: Pre-configured endpoints
        master_endpoints.extend(self.get_configured_endpoints());
        
        Ok(master_endpoints)
    }
    
    async fn udp_broadcast_discovery(&self) -> AppResult<Vec<String>> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.set_broadcast(true)?;
        
        let discovery_message = serde_json::json!({
            "type": "discovery_request",
            "service": "restrike_vta_master",
            "version": "1.0"
        });
        
        let message_bytes = serde_json::to_vec(&discovery_message)?;
        
        // Send broadcast
        socket.send_to(&message_bytes, "255.255.255.255:8080").await?;
        
        // Listen for responses
        let mut buffer = [0; 1024];
        let mut endpoints = Vec::new();
        
        // Set timeout for discovery
        let timeout = tokio::time::sleep(Duration::from_secs(5));
        let mut response_future = Box::pin(async {
            loop {
                if let Ok((len, addr)) = socket.recv_from(&mut buffer).await {
                    if let Ok(response) = serde_json::from_slice::<DiscoveryResponse>(&buffer[..len]) {
                        if response.service == "restrike_vta_master" {
                            endpoints.push(format!("http://{}:{}", addr.ip(), response.port));
                        }
                    }
                }
            }
        });
        
        tokio::select! {
            _ = &mut response_future => {},
            _ = timeout => {},
        }
        
        Ok(endpoints)
    }
    
    async fn mdns_discovery(&self) -> AppResult<Vec<String>> {
        // Use mdns crate for mDNS discovery
        // This would discover services advertised via mDNS/Bonjour
        Ok(Vec::new()) // Placeholder
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DiscoveryResponse {
    service: String,
    port: u16,
    version: String,
}
```

#### **2. Shared Folder Management**
```rust
// Shared folder management for recordings
pub struct SharedFolderManager {
    local_recording_path: PathBuf,
    shared_folder_path: PathBuf,
    sync_interval: Duration,
}

impl SharedFolderManager {
    pub async fn start_folder_sync(&self) -> AppResult<()> {
        loop {
            // Sync new recordings to shared folder
            self.sync_recordings_to_shared().await?;
            
            // Clean up old recordings
            self.cleanup_old_recordings().await?;
            
            tokio::time::sleep(self.sync_interval).await;
        }
    }
    
    async fn sync_recordings_to_shared(&self) -> AppResult<()> {
        let local_files = tokio::fs::read_dir(&self.local_recording_path).await?;
        
        for entry in local_files {
            let entry = entry?;
            let file_path = entry.path();
            
            if file_path.is_file() && self.is_recording_file(&file_path) {
                let shared_path = self.shared_folder_path.join(file_path.file_name().unwrap());
                
                if !shared_path.exists() {
                    // Copy file to shared folder
                    tokio::fs::copy(&file_path, &shared_path).await?;
                    info!("Synced recording to shared folder: {:?}", shared_path);
                }
            }
        }
        
        Ok(())
    }
    
    fn is_recording_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            matches!(extension.to_str(), Some("mp4") | Some("mkv") | Some("avi"))
        } else {
            false
        }
    }
    
    async fn cleanup_old_recordings(&self) -> AppResult<()> {
        // Remove recordings older than configured days
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(7);
        
        let local_files = tokio::fs::read_dir(&self.local_recording_path).await?;
        
        for entry in local_files {
            let entry = entry?;
            let file_path = entry.path();
            
            if let Ok(metadata) = file_path.metadata() {
                if let Ok(created) = metadata.created() {
                    let created_date = chrono::DateTime::from(created);
                    if created_date < cutoff_date {
                        tokio::fs::remove_file(&file_path).await?;
                        info!("Cleaned up old recording: {:?}", file_path);
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

### **Performance Considerations**

#### **1. Minimal Slave Impact**
```rust
// Performance-optimized slave operations
pub struct OptimizedSlaveOperations {
    // Use bounded channels to prevent memory overflow
    command_queue: Arc<Mutex<VecDeque<MasterCommand>>>,
    event_buffer: Arc<Mutex<Vec<PssEventV2>>>,
    
    // Batch operations to reduce network overhead
    sync_batch_size: usize,
    sync_interval: Duration,
    
    // Async operations to prevent blocking
    sync_handle: Option<JoinHandle<()>>,
    heartbeat_handle: Option<JoinHandle<()>>,
}

impl OptimizedSlaveOperations {
    pub fn new() -> Self {
        Self {
            command_queue: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            event_buffer: Arc::new(Mutex::new(Vec::with_capacity(1000))),
            sync_batch_size: 50,
            sync_interval: Duration::from_secs(30),
            sync_handle: None,
            heartbeat_handle: None,
        }
    }
    
    pub async fn start_background_operations(&mut self) -> AppResult<()> {
        // Start sync in background
        let sync_handle = tokio::spawn(self.start_sync_loop());
        self.sync_handle = Some(sync_handle);
        
        // Start heartbeat in background
        let heartbeat_handle = tokio::spawn(self.start_heartbeat_loop());
        self.heartbeat_handle = Some(heartbeat_handle);
        
        Ok(())
    }
    
    async fn start_sync_loop(self) -> AppResult<()> {
        loop {
            // Only sync if there are events to sync
            let event_count = self.event_buffer.lock().await.len();
            if event_count >= self.sync_batch_size {
                self.sync_events().await?;
            }
            
            tokio::time::sleep(self.sync_interval).await;
        }
    }
    
    async fn start_heartbeat_loop(self) -> AppResult<()> {
        loop {
            // Send lightweight heartbeat
            self.send_heartbeat().await?;
            
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    }
}
```

#### **2. Efficient Data Transfer**
```rust
// Efficient data serialization and compression
pub struct EfficientDataTransfer {
    compression_enabled: bool,
    batch_size: usize,
}

impl EfficientDataTransfer {
    pub async fn send_events_batch(&self, events: Vec<PssEventV2>) -> AppResult<()> {
        // Serialize to binary format (more efficient than JSON)
        let serialized = bincode::serialize(&events)?;
        
        let data = if self.compression_enabled {
            // Compress data to reduce network overhead
            let mut compressed = Vec::new();
            let mut encoder = flate2::write::GzEncoder::new(&mut compressed, flate2::Compression::default());
            encoder.write_all(&serialized)?;
            encoder.finish()?;
            compressed
        } else {
            serialized
        };
        
        // Send compressed/binary data
        self.send_data(&data).await?;
        
        Ok(())
    }
}
```

### **Implementation Timeline**

#### **Phase 1: Core Infrastructure (Week 1-2)**
1. **Master Database Schema** (2 days)
   - Implement central database tables
   - Create data synchronization logic
   - Add health monitoring tables

2. **Network Discovery** (2 days)
   - Implement UDP broadcast discovery
   - Add mDNS/Bonjour support
   - Create auto-registration system

3. **Basic Master/Slave Communication** (3 days)
   - Implement command processing
   - Add heartbeat system
   - Create status reporting

#### **Phase 2: Master Control Features (Week 3-4)**
4. **Remote Control Drawer** (3 days)
   - Create bulk operations interface
   - Implement individual slave control
   - Add tournament management features

5. **Health Monitoring** (2 days)
   - Implement system health dashboard
   - Add performance metrics collection
   - Create alerting system

#### **Phase 3: Advanced Features (Week 5-6)**
6. **YT Manager** (4 days)
   - Implement YouTube API integration
   - Add stream management
   - Create chat moderation tools

7. **IVR Central Desk** (3 days)
   - Implement event review system
   - Add video management
   - Create reporting tools

#### **Phase 4: Optimization & Testing (Week 7-8)**
8. **Performance Optimization** (3 days)
   - Optimize data transfer
   - Implement caching strategies
   - Add compression

9. **Testing & Documentation** (2 days)
   - Comprehensive testing
   - Documentation updates
   - Deployment guides

### **Benefits**

#### **Centralized Control**
- **Bulk Operations**: Control all slaves simultaneously
- **Real-time Monitoring**: Monitor all systems from one location
- **Automated Management**: Reduce manual intervention

#### **Scalability**
- **Dynamic Discovery**: Automatically find and connect to slaves
- **Load Distribution**: Distribute work across multiple slaves
- **Fault Tolerance**: Continue operation if individual slaves fail

#### **Efficiency**
- **Shared Resources**: Centralized storage and processing
- **Automated Workflows**: Streamlined tournament management
- **Reduced Overhead**: Minimal impact on slave performance

#### **Advanced Features**
- **YouTube Integration**: Direct stream and chat management
- **IVR Review**: Centralized video review and escalation
- **Reporting**: Comprehensive tournament and performance reports

---

## 📞 Support and Maintenance
```

## Centralized Current Matches View for YT Manager

### Overview
The YT Manager on the Master node will feature a centralized overlay showing real-time match data from all connected slaves. This provides YouTube viewers with a single overview of all ongoing matches across the tournament.

### Architecture Design

#### Current WebSocket Infrastructure
- **Backend WebSocket Server**: `src-tauri/src/plugins/plugin_websocket.rs` (port 3001)
- **Overlay Clients**: HTML files connect directly to backend WebSocket
- **Data Flow**: PSS events → Backend → WebSocket broadcast → All clients

#### Master Implementation Strategy
- **Master as WebSocket Client**: Master connects to each slave's WebSocket server as a client
- **One Connection Per Slave**: Minimal resource impact on slaves
- **Real-time Data**: Direct access to live PSS events without polling
- **Consistent Data**: Same data format as local overlays

### Technical Implementation

#### 1. Master WebSocket Client Infrastructure

```rust
// src-tauri/src/plugins/plugin_master_websocket.rs
use tokio_tungstenite::{connect_async, WebSocketStream};
use tokio::net::TcpStream;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct SlaveWebSocketClient {
    pub slave_id: String,
    pub slave_name: String,
    pub slave_ip: String,
    pub connection: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    pub is_connected: bool,
    pub last_heartbeat: chrono::DateTime<Utc>,
    pub current_match_data: Option<CurrentMatchData>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CurrentMatchData {
    pub match_number: String,
    pub tournament_day: String,
    pub athlete1_name: String,
    pub athlete1_country: String,
    pub athlete2_name: String,
    pub athlete2_country: String,
    pub current_round: u8,
    pub current_time: String,
    pub athlete1_score: u8,
    pub athlete2_score: u8,
    pub athlete1_rounds_won: u8,
    pub athlete2_rounds_won: u8,
    pub match_status: String, // "active", "break", "finished"
    pub last_update: chrono::DateTime<Utc>,
}

pub struct MasterWebSocketManager {
    slave_clients: Arc<Mutex<HashMap<String, SlaveWebSocketClient>>>,
    central_match_data: Arc<Mutex<HashMap<String, CurrentMatchData>>>,
    event_tx: mpsc::UnboundedSender<MasterWebSocketEvent>,
}

#[derive(Debug, Clone)]
pub enum MasterWebSocketEvent {
    SlaveConnected { slave_id: String, slave_name: String },
    SlaveDisconnected { slave_id: String },
    MatchDataUpdated { slave_id: String, match_data: CurrentMatchData },
    SlaveHeartbeat { slave_id: String },
}
```

#### 2. Master WebSocket Client Connection Logic

```rust
impl MasterWebSocketManager {
    pub async fn connect_to_slave(&self, slave_ip: String, slave_id: String, slave_name: String) -> AppResult<()> {
        let ws_url = format!("ws://{}:3001", slave_ip);
        
        let (ws_stream, _) = connect_async(&ws_url).await
            .map_err(|e| AppError::ConfigError(format!("Failed to connect to slave {}: {}", slave_id, e)))?;
        
        let mut client = SlaveWebSocketClient {
            slave_id: slave_id.clone(),
            slave_name: slave_name.clone(),
            slave_ip: slave_ip.clone(),
            connection: Some(ws_stream),
            is_connected: true,
            last_heartbeat: Utc::now(),
            current_match_data: None,
        };
        
        // Store client
        self.slave_clients.lock().unwrap().insert(slave_id.clone(), client);
        
        // Notify event system
        let _ = self.event_tx.send(MasterWebSocketEvent::SlaveConnected {
            slave_id: slave_id.clone(),
            slave_name: slave_name.clone(),
        });
        
        log::info!("🔗 Master connected to slave {} ({}) at {}", slave_name, slave_id, slave_ip);
        Ok(())
    }
    
    pub async fn handle_slave_messages(&self, slave_id: String) -> AppResult<()> {
        let mut client = self.slave_clients.lock().unwrap()
            .get_mut(&slave_id)
            .ok_or_else(|| AppError::ConfigError("Slave client not found".to_string()))?;
        
        if let Some(ws_stream) = &mut client.connection {
            let (mut write, mut read) = ws_stream.split();
            
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(data) = serde_json::from_str::<Value>(&text) {
                            self.process_slave_event(&slave_id, data).await?;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        log::info!("🔌 Slave {} disconnected", slave_id);
                        break;
                    }
                    Err(e) => {
                        log::error!("❌ WebSocket error from slave {}: {}", slave_id, e);
                        break;
                    }
                    _ => {}
                }
            }
        }
        
        // Handle disconnection
        client.is_connected = false;
        client.connection = None;
        
        let _ = self.event_tx.send(MasterWebSocketEvent::SlaveDisconnected {
            slave_id: slave_id.clone(),
        });
        
        Ok(())
    }
    
    async fn process_slave_event(&self, slave_id: &str, data: Value) -> AppResult<()> {
        if let Some(event_data) = data.get("data") {
            if let Some(event_type) = event_data.get("type").and_then(|v| v.as_str()) {
                match event_type {
                    "athletes" => self.update_match_data_from_athletes(slave_id, event_data).await?,
                    "current_scores" => self.update_match_data_from_scores(slave_id, event_data).await?,
                    "round" => self.update_match_data_from_round(slave_id, event_data).await?,
                    "clock" => self.update_match_data_from_clock(slave_id, event_data).await?,
                    "match_config" => self.update_match_data_from_config(slave_id, event_data).await?,
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
```

#### 3. Centralized Match Data Processing

```rust
impl MasterWebSocketManager {
    async fn update_match_data_from_athletes(&self, slave_id: &str, event_data: &Value) -> AppResult<()> {
        let mut central_data = self.central_match_data.lock().unwrap();
        
        let match_data = central_data.entry(slave_id.to_string()).or_insert_with(|| CurrentMatchData {
            match_number: "".to_string(),
            tournament_day: "".to_string(),
            athlete1_name: "".to_string(),
            athlete1_country: "".to_string(),
            athlete2_name: "".to_string(),
            athlete2_country: "".to_string(),
            current_round: 1,
            current_time: "0:00".to_string(),
            athlete1_score: 0,
            athlete2_score: 0,
            athlete1_rounds_won: 0,
            athlete2_rounds_won: 0,
            match_status: "active".to_string(),
            last_update: Utc::now(),
        });
        
        // Update athlete information
        if let Some(athlete1_short) = event_data.get("athlete1_short").and_then(|v| v.as_str()) {
            match_data.athlete1_name = athlete1_short.to_string();
        }
        if let Some(athlete1_country) = event_data.get("athlete1_country").and_then(|v| v.as_str()) {
            match_data.athlete1_country = athlete1_country.to_string();
        }
        if let Some(athlete2_short) = event_data.get("athlete2_short").and_then(|v| v.as_str()) {
            match_data.athlete2_name = athlete2_short.to_string();
        }
        if let Some(athlete2_country) = event_data.get("athlete2_country").and_then(|v| v.as_str()) {
            match_data.athlete2_country = athlete2_country.to_string();
        }
        
        match_data.last_update = Utc::now();
        
        // Notify event system
        let _ = self.event_tx.send(MasterWebSocketEvent::MatchDataUpdated {
            slave_id: slave_id.to_string(),
            match_data: match_data.clone(),
        });
        
        Ok(())
    }
    
    async fn update_match_data_from_scores(&self, slave_id: &str, event_data: &Value) -> AppResult<()> {
        let mut central_data = self.central_match_data.lock().unwrap();
        
        if let Some(match_data) = central_data.get_mut(slave_id) {
            if let Some(athlete1_score) = event_data.get("athlete1_score").and_then(|v| v.as_u64()) {
                match_data.athlete1_score = athlete1_score as u8;
            }
            if let Some(athlete2_score) = event_data.get("athlete2_score").and_then(|v| v.as_u64()) {
                match_data.athlete2_score = athlete2_score as u8;
            }
            
            match_data.last_update = Utc::now();
            
            // Notify event system
            let _ = self.event_tx.send(MasterWebSocketEvent::MatchDataUpdated {
                slave_id: slave_id.to_string(),
                match_data: match_data.clone(),
            });
        }
        
        Ok(())
    }
    
    async fn update_match_data_from_clock(&self, slave_id: &str, event_data: &Value) -> AppResult<()> {
        let mut central_data = self.central_match_data.lock().unwrap();
        
        if let Some(match_data) = central_data.get_mut(slave_id) {
            if let Some(time) = event_data.get("time").and_then(|v| v.as_str()) {
                match_data.current_time = time.to_string();
            }
            if let Some(action) = event_data.get("action").and_then(|v| v.as_str()) {
                match_data.match_status = match action {
                    "start" => "active".to_string(),
                    "stop" => "break".to_string(),
                    "reset" => "break".to_string(),
                    _ => "active".to_string(),
                };
            }
            
            match_data.last_update = Utc::now();
            
            // Notify event system
            let _ = self.event_tx.send(MasterWebSocketEvent::MatchDataUpdated {
                slave_id: slave_id.to_string(),
                match_data: match_data.clone(),
            });
        }
        
        Ok(())
    }
}
```

#### 4. YT Manager Centralized Overlay Component

```typescript
// ui/src/components/organisms/YtManagerCentralizedOverlay.tsx
import React, { useState, useEffect, useMemo } from 'react';
import { useMasterWebSocketStore } from '../../stores/masterWebSocketStore';

interface MatchCardProps {
  slaveId: string;
  slaveName: string;
  matchData: CurrentMatchData;
  isActive: boolean;
}

const MatchCard: React.FC<MatchCardProps> = React.memo(({ slaveId, slaveName, matchData, isActive }) => {
  const timeSinceUpdate = useMemo(() => {
    const now = new Date();
    const lastUpdate = new Date(matchData.last_update);
    return Math.floor((now.getTime() - lastUpdate.getTime()) / 1000);
  }, [matchData.last_update]);

  const isStale = timeSinceUpdate > 30; // 30 seconds threshold

  return (
    <div className={`
      match-card 
      ${isActive ? 'active' : 'inactive'} 
      ${isStale ? 'stale' : ''}
      ${matchData.match_status === 'finished' ? 'finished' : ''}
    `}>
      <div className="match-header">
        <div className="slave-name">{slaveName}</div>
        <div className="match-number">#{matchData.match_number}</div>
        <div className="status-indicator">
          {isActive ? '🟢' : '🔴'} {matchData.match_status}
        </div>
      </div>
      
      <div className="match-content">
        <div className="athletes">
          <div className="athlete athlete1">
            <div className="name">{matchData.athlete1_name}</div>
            <div className="country">{matchData.athlete1_country}</div>
            <div className="score">{matchData.athlete1_score}</div>
          </div>
          
          <div className="vs">VS</div>
          
          <div className="athlete athlete2">
            <div className="name">{matchData.athlete2_name}</div>
            <div className="country">{matchData.athlete2_country}</div>
            <div className="score">{matchData.athlete2_score}</div>
          </div>
        </div>
        
        <div className="match-details">
          <div className="round">R{matchData.current_round}</div>
          <div className="time">{matchData.current_time}</div>
          <div className="rounds-won">
            {matchData.athlete1_rounds_won} - {matchData.athlete2_rounds_won}
          </div>
        </div>
      </div>
      
      {isStale && (
        <div className="stale-warning">
          ⚠️ No updates for {timeSinceUpdate}s
        </div>
      )}
    </div>
  );
});

export const YtManagerCentralizedOverlay: React.FC = () => {
  const { connectedSlaves, matchData, connectionStatus } = useMasterWebSocketStore();
  
  const activeMatches = useMemo(() => {
    return Object.entries(matchData)
      .filter(([slaveId, data]) => {
        const isConnected = connectedSlaves.some(slave => slave.id === slaveId && slave.isConnected);
        const hasValidData = data.athlete1_name && data.athlete2_name;
        return isConnected && hasValidData;
      })
      .sort(([, a], [, b]) => {
        // Sort by match status: active > break > finished
        const statusOrder = { active: 3, break: 2, finished: 1 };
        return (statusOrder[b.match_status as keyof typeof statusOrder] || 0) - 
               (statusOrder[a.match_status as keyof typeof statusOrder] || 0);
      });
  }, [connectedSlaves, matchData]);

  return (
    <div className="yt-manager-centralized-overlay">
      <div className="overlay-header">
        <h2>🏆 Tournament Live Matches</h2>
        <div className="connection-status">
          {connectionStatus.connected ? '🟢 Connected' : '🔴 Disconnected'}
          <span className="slave-count">({connectedSlaves.length} slaves)</span>
        </div>
      </div>
      
      <div className="matches-grid">
        {activeMatches.length > 0 ? (
          activeMatches.map(([slaveId, matchData]) => {
            const slave = connectedSlaves.find(s => s.id === slaveId);
            return (
              <MatchCard
                key={slaveId}
                slaveId={slaveId}
                slaveName={slave?.name || 'Unknown Slave'}
                matchData={matchData}
                isActive={slave?.isConnected || false}
              />
            );
          })
        ) : (
          <div className="no-matches">
            <div className="no-matches-icon">🏟️</div>
            <div className="no-matches-text">No active matches</div>
            <div className="no-matches-subtext">Waiting for match data from slaves...</div>
          </div>
        )}
      </div>
      
      <div className="overlay-footer">
        <div className="last-update">
          Last update: {new Date().toLocaleTimeString()}
        </div>
        <div className="total-matches">
          {activeMatches.length} active match{activeMatches.length !== 1 ? 'es' : ''}
        </div>
      </div>
    </div>
  );
};
```

#### 5. Master WebSocket Store

```typescript
// ui/src/stores/masterWebSocketStore.ts
import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';

interface ConnectedSlave {
  id: string;
  name: string;
  ip: string;
  isConnected: boolean;
  lastHeartbeat: Date;
}

interface CurrentMatchData {
  matchNumber: string;
  tournamentDay: string;
  athlete1Name: string;
  athlete1Country: string;
  athlete2Name: string;
  athlete2Country: string;
  currentRound: number;
  currentTime: string;
  athlete1Score: number;
  athlete2Score: number;
  athlete1RoundsWon: number;
  athlete2RoundsWon: number;
  matchStatus: 'active' | 'break' | 'finished';
  lastUpdate: Date;
}

interface MasterWebSocketState {
  connectedSlaves: ConnectedSlave[];
  matchData: Record<string, CurrentMatchData>;
  connectionStatus: {
    connected: boolean;
    totalSlaves: number;
    activeSlaves: number;
  };
  error: string | null;
}

interface MasterWebSocketActions {
  addSlave: (slave: ConnectedSlave) => void;
  removeSlave: (slaveId: string) => void;
  updateSlaveStatus: (slaveId: string, isConnected: boolean) => void;
  updateMatchData: (slaveId: string, matchData: CurrentMatchData) => void;
  clearMatchData: (slaveId: string) => void;
  setError: (error: string | null) => void;
  clearError: () => void;
}

export const useMasterWebSocketStore = create<MasterWebSocketState & MasterWebSocketActions>()(
  subscribeWithSelector((set, get) => ({
    connectedSlaves: [],
    matchData: {},
    connectionStatus: {
      connected: false,
      totalSlaves: 0,
      activeSlaves: 0,
    },
    error: null,

    addSlave: (slave) => set((state) => ({
      connectedSlaves: [...state.connectedSlaves.filter(s => s.id !== slave.id), slave],
      connectionStatus: {
        ...state.connectionStatus,
        totalSlaves: state.connectedSlaves.length + 1,
        activeSlaves: state.connectedSlaves.filter(s => s.isConnected).length + (slave.isConnected ? 1 : 0),
      },
    })),

    removeSlave: (slaveId) => set((state) => {
      const slave = state.connectedSlaves.find(s => s.id === slaveId);
      const wasConnected = slave?.isConnected || false;
      
      return {
        connectedSlaves: state.connectedSlaves.filter(s => s.id !== slaveId),
        matchData: Object.fromEntries(
          Object.entries(state.matchData).filter(([id]) => id !== slaveId)
        ),
        connectionStatus: {
          ...state.connectionStatus,
          totalSlaves: state.connectedSlaves.length - 1,
          activeSlaves: state.connectionStatus.activeSlaves - (wasConnected ? 1 : 0),
        },
      };
    }),

    updateSlaveStatus: (slaveId, isConnected) => set((state) => {
      const updatedSlaves = state.connectedSlaves.map(slave =>
        slave.id === slaveId ? { ...slave, isConnected } : slave
      );
      
      const activeSlaves = updatedSlaves.filter(s => s.isConnected).length;
      
      return {
        connectedSlaves: updatedSlaves,
        connectionStatus: {
          ...state.connectionStatus,
          activeSlaves,
          connected: activeSlaves > 0,
        },
      };
    }),

    updateMatchData: (slaveId, matchData) => set((state) => ({
      matchData: {
        ...state.matchData,
        [slaveId]: matchData,
      },
    })),

    clearMatchData: (slaveId) => set((state) => ({
      matchData: Object.fromEntries(
        Object.entries(state.matchData).filter(([id]) => id !== slaveId)
      ),
    })),

    setError: (error) => set({ error }),
    clearError: () => set({ error: null }),
  }))
);
```

### Implementation Benefits

1. **Real-time Updates**: Direct WebSocket connection ensures minimal latency
2. **Minimal Slave Impact**: One additional WebSocket client per slave (negligible load)
3. **Scalable**: Can handle 10+ slaves without performance issues
4. **Fault Tolerant**: Automatic reconnection and stale data detection
5. **Consistent Data**: Same data format as local overlays
6. **Visual Feedback**: Clear status indicators for connection and data freshness

### Performance Considerations

- **WebSocket Messages**: ~1-5 per second per slave during active matches
- **Memory Usage**: ~1KB per slave for match data storage
- **CPU Impact**: Minimal - only JSON parsing and state updates
- **Network**: ~5-25KB/s total for 5 slaves (very low bandwidth)

### Next Steps

1. **Phase 1**: Implement Master WebSocket client infrastructure
2. **Phase 2**: Create centralized match data processing
3. **Phase 3**: Build YT Manager overlay component
4. **Phase 4**: Integrate with Master/Slave discovery system
5. **Phase 5**: Add advanced features (filtering, sorting, alerts)

This implementation provides a robust, scalable solution that leverages existing infrastructure while providing real-time tournament overview for YouTube viewers.