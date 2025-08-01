# Hit Level Tracking Implementation

## Overview

This document describes the implementation of hit level tracking functionality in the PSS UDP server. The feature allows tracking of hit level events (`hl1`, `hl2`) and linking them with subsequent point events (`pt1`, `pt2`) for statistical analysis.

## Requirements

The user requested:
> "With every score/point event if it was a kick/punch I want to store also HIT level event. It represents how hard player actually did strike. Sometimes there is a kick/punch that is not strong enough to produce points/scores so I want all that in the event table so later we can make some statistics."

## Implementation Details

### 1. Data Structure

Added a new field to the `UdpServer` struct:
```rust
recent_hit_levels: Arc<Mutex<std::collections::HashMap<u8, Vec<(u8, std::time::SystemTime)>>>>, // athlete -> [(level, timestamp)]
```

This stores:
- **Key**: Athlete number (1 or 2)
- **Value**: Vector of tuples containing (hit_level, timestamp)
- **Limit**: Maximum 10 hit levels per athlete to prevent memory bloat

### 2. Hit Level Tracking Logic

In the `listen_loop_async` function, added tracking logic:

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
            athlete_hit_levels.remove(0);
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

### 3. Enhanced Event Details

Modified the `extract_event_details` function to include hit level information in point events:

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
            
            // Add the highest hit level in the recent window
            if let Some(max_level) = recent_hit_levels.iter().max() {
                details.push(("max_hit_level".to_string(), Some(max_level.to_string()), "u8".to_string()));
            }
            
            // Add the average hit level in the recent window
            let avg_level = recent_hit_levels.iter().sum::<u8>() as f32 / recent_hit_levels.len() as f32;
            details.push(("avg_hit_level".to_string(), Some(format!("{:.1}", avg_level)), "float".to_string()));
        }
    }
    
    Some(details)
}
```

### 4. Database Storage

All hit level events are stored in the database regardless of whether they result in points:

- **Hit Level Events**: Stored as separate events with athlete and level information
- **Point Events**: Enhanced with additional details:
  - `recent_hit_levels`: Comma-separated list of hit levels within 5 seconds
  - `max_hit_level`: Highest hit level in the recent window
  - `avg_hit_level`: Average hit level in the recent window

## Features

### 1. Automatic Tracking
- Every `hl1` and `hl2` event is automatically tracked with timestamp
- Hit levels are associated with the correct athlete (1 or 2)

### 2. Time-Based Filtering
- Only hit levels within the last 5 seconds are linked to point events
- This ensures relevance and prevents linking with old hit data

### 3. Statistical Data
- **Recent Hit Levels**: All hit levels in the time window
- **Maximum Hit Level**: Highest intensity hit
- **Average Hit Level**: Average intensity of recent hits

### 4. Memory Management
- Maximum 10 hit levels per athlete to prevent memory bloat
- Automatic cleanup when new fights start (`FightLoaded`, `FightReady`)

### 5. Comprehensive Storage
- Hit levels without points are still stored in the database
- Enables future statistical analysis of all strikes, not just scoring ones

## Database Schema

The hit level data is stored in the `pss_event_details` table:

```sql
-- For Hit Level Events
INSERT INTO pss_event_details (event_id, detail_key, detail_value, detail_type) VALUES
(123, 'athlete', '1', 'u8'),
(123, 'level', '75', 'u8');

-- For Point Events (enhanced)
INSERT INTO pss_event_details (event_id, detail_key, detail_value, detail_type) VALUES
(124, 'athlete', '1', 'u8'),
(124, 'point_type', '3', 'u8'),
(124, 'recent_hit_levels', '75,85,90', 'String'),
(124, 'max_hit_level', '90', 'u8'),
(124, 'avg_hit_level', '83.3', 'float');
```

## Testing

Created `test_hit_level_tracking.py` to verify the implementation:

1. **Basic Tracking**: Hit levels are tracked and stored
2. **Linking**: Point events include recent hit level data
3. **Weak Hits**: Hit levels without points are still stored
4. **Multiple Hits**: Multiple hit levels in quick succession
5. **Time Window**: Only recent hit levels are linked
6. **Cleanup**: Hit level tracking is cleared for new fights

## Usage Examples

### Scenario 1: Strong Hit with Points
```
hl1;85;  // Athlete 1 hit level 85
pt1;3;   // Athlete 1 scores 3 points (head)
```
**Result**: Point event includes `recent_hit_levels: "85"`, `max_hit_level: "85"`, `avg_hit_level: "85.0"`

### Scenario 2: Multiple Hits with Points
```
hl1;70;  // Athlete 1 hit level 70
hl1;75;  // Athlete 1 hit level 75
hl1;80;  // Athlete 1 hit level 80
pt1;2;   // Athlete 1 scores 2 points (body)
```
**Result**: Point event includes `recent_hit_levels: "70,75,80"`, `max_hit_level: "80"`, `avg_hit_level: "75.0"`

### Scenario 3: Weak Hit without Points
```
hl1;50;  // Athlete 1 hit level 50 (weak hit)
// No point event follows
```
**Result**: Hit level event is stored in database for future statistics

## Benefits

1. **Complete Data**: All strikes are recorded, not just scoring ones
2. **Statistical Analysis**: Enables analysis of hit patterns and effectiveness
3. **Performance Tracking**: Can correlate hit intensity with scoring success
4. **Training Insights**: Coaches can analyze athlete performance patterns
5. **Real-time Linking**: Hit levels are automatically linked to relevant point events

## Future Enhancements

1. **Configurable Time Window**: Make the 5-second window configurable
2. **Hit Level Categories**: Categorize hits by intensity ranges
3. **Performance Metrics**: Calculate strike accuracy and effectiveness
4. **Visual Analytics**: Dashboard for hit level statistics
5. **Export Functionality**: Export hit level data for external analysis

## Conclusion

The hit level tracking implementation successfully addresses the user's requirements by:

- ✅ Storing all hit level events in the database
- ✅ Linking hit levels with point events when they occur together
- ✅ Including hit level data even when no points are scored
- ✅ Providing statistical data (max, average, recent hit levels)
- ✅ Enabling future statistical analysis and insights

The implementation is robust, memory-efficient, and provides a solid foundation for advanced statistical analysis of athlete performance. 