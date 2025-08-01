# Enhanced PSS Event System

## Overview

The Enhanced PSS Event System provides robust event processing with status marks, validation, and comprehensive tracking capabilities. This system ensures data quality, enables protocol evolution, and provides detailed insights into event processing.

## Key Features

### 1. Status Mark System

#### Recognition Status Values
- **`recognized`**: Event is fully understood and parsed correctly
- **`unknown`**: Event format is not recognized or validation failed
- **`partial`**: Event partially parsed but some fields unknown
- **`deprecated`**: Event type is no longer used in current protocol

#### Status Tracking
- Automatic status assignment based on parsing and validation results
- Status change history tracking in `pss_event_recognition_history` table
- Ability to update status manually for protocol evolution

### 2. Event Validation

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

### 3. Unknown Event Collection

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

### 4. Enhanced Event Details

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

### 5. Database Schema Enhancements

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

## Implementation Details

### Database Operations

#### PssEventStatusOperations
```rust
// Store event with status
store_pss_event_with_status(conn, event) -> i64

// Update recognition status
update_event_recognition_status(conn, event_id, new_status, changed_by, reason) -> ()

// Store unknown event
store_unknown_event(conn, unknown_event) -> i64

// Get validation rules
get_validation_rules(conn, event_code, protocol_version) -> Vec<PssEventValidationRule>

// Update statistics
update_event_statistics(conn, session_id, event_type_id, status, processing_time) -> ()
```

### Event Processing Flow

1. **Raw Event Reception**: UDP message received
2. **Parsing**: Convert to structured PssEvent
3. **Status Determination**: 
   - Check if event type is recognized
   - Apply validation rules
   - Assign recognition status and confidence
4. **Database Storage**: Store with status and metadata
5. **Statistics Update**: Update processing metrics
6. **Unknown Event Handling**: Store unrecognized events
7. **Event Details**: Extract and store additional details

### Validation Process

1. **Rule Retrieval**: Get validation rules for event type
2. **Rule Application**: Apply each rule to event data
3. **Error Collection**: Gather validation errors and warnings
4. **Status Assignment**: Determine final recognition status
5. **Result Storage**: Store validation results

## Usage Examples

### Basic Event Processing
```rust
// Event is automatically processed with status
let event = PssEvent::Points { athlete: 1, point_type: 3 };
// Results in: recognition_status = "recognized", confidence = 1.0
```

### Invalid Event Handling
```rust
// Invalid point type
let event = PssEvent::Points { athlete: 1, point_type: 6 };
// Results in: recognition_status = "unknown", confidence = 0.3
// Validation error: "Point type must be between 1 and 5"
```

### Unknown Event Collection
```rust
// Unknown event type
let event = PssEvent::Raw("xyz;unknown;event;type;".to_string());
// Results in: recognition_status = "unknown", confidence = 0.0
// Stored in: pss_unknown_events table
```

### Hit Level Tracking
```rust
// Hit level events are tracked
send_udp_message("hl1;75;");
send_udp_message("hl1;85;");
send_udp_message("pt1;3;"); // Point event includes recent hit levels

// Point event details include:
// - recent_hit_levels: "75,85"
// - max_hit_level: "85"
// - avg_hit_level: "80.0"
```

## Database Queries

### Check Event Status Distribution
```sql
SELECT recognition_status, COUNT(*) as count 
FROM pss_events_v2 
GROUP BY recognition_status;
```

### Find Unknown Events
```sql
SELECT raw_data, occurrence_count, first_seen, last_seen 
FROM pss_unknown_events 
ORDER BY occurrence_count DESC;
```

### Check Validation Errors
```sql
SELECT id, raw_data, recognition_status, validation_errors, parser_confidence 
FROM pss_events_v2 
WHERE validation_errors IS NOT NULL;
```

### Analyze Hit Level Tracking
```sql
SELECT e.id, e.raw_data, d.detail_key, d.detail_value 
FROM pss_events_v2 e
JOIN pss_event_details d ON e.id = d.event_id
WHERE e.recognition_status = 'recognized' 
  AND d.detail_key IN ('recent_hit_levels', 'max_hit_level', 'avg_hit_level');
```

### Performance Metrics
```sql
SELECT recognition_status, 
       AVG(processing_time_ms) as avg_time,
       MIN(processing_time_ms) as min_time,
       MAX(processing_time_ms) as max_time
FROM pss_events_v2 
GROUP BY recognition_status;
```

## Testing

### Test Script
Use `test_enhanced_pss_system.py` to verify:
- Status mark assignment
- Event validation
- Unknown event collection
- Hit level tracking
- Database storage

### Test Categories
1. **Valid Events**: Should be marked as "recognized"
2. **Invalid Events**: Should be marked as "unknown" with validation errors
3. **Unknown Events**: Should be collected in unknown_events table
4. **Hit Level Tracking**: Should link hit levels to point events
5. **Edge Cases**: Boundary value testing
6. **Performance**: Processing time tracking

## Benefits

### For Users
- **Better Error Reporting**: Detailed validation error messages
- **Data Quality**: Automatic validation and status tracking
- **Future Compatibility**: Unknown event collection for protocol updates
- **Performance Insights**: Processing time and confidence metrics

### For Developers
- **Robust Parsing**: Graceful handling of malformed events
- **Protocol Evolution**: Easy addition of new event types
- **Debugging**: Comprehensive error tracking and unknown event analysis
- **Performance Monitoring**: Detailed processing metrics

### For Data Analysis
- **Complete Event History**: All events stored regardless of recognition status
- **Quality Metrics**: Confidence scores and validation results
- **Performance Insights**: Processing time analysis
- **Statistical Analysis**: Hit level trends and patterns

## Configuration

### Validation Rules
Validation rules are stored in the database and can be modified:
```sql
-- Add new validation rule
INSERT INTO pss_event_validation_rules 
(event_code, protocol_version, rule_name, rule_type, rule_definition, error_message)
VALUES ('pt1', '2.3', 'custom_range', 'range', '1-10', 'Custom point type range');
```

### Protocol Version
The system supports multiple protocol versions:
- Current: PSS v2.3
- Future versions can be added with new validation rules
- Backward compatibility maintained

## Migration

### Database Migration
The enhanced system includes Migration 8 which:
- Adds new tables for status tracking
- Adds new fields to existing tables
- Populates validation rules
- Creates performance indexes

### Data Migration
Existing events are automatically assigned:
- `recognition_status = "recognized"`
- `protocol_version = "2.3"`
- `parser_confidence = 1.0`

## Troubleshooting

### Common Issues

#### High Unknown Event Count
- Check for protocol changes
- Review validation rules
- Analyze unknown event patterns

#### Validation Errors
- Verify event format against PSS specification
- Check validation rule definitions
- Review error messages for specific issues

#### Performance Issues
- Monitor processing times
- Check database indexes
- Review event statistics

### Debugging Tools

#### Unknown Event Analysis
```sql
-- Find most common unknown events
SELECT raw_data, occurrence_count 
FROM pss_unknown_events 
ORDER BY occurrence_count DESC 
LIMIT 10;
```

#### Validation Error Analysis
```sql
-- Find events with validation errors
SELECT raw_data, validation_errors 
FROM pss_events_v2 
WHERE validation_errors IS NOT NULL;
```

#### Performance Analysis
```sql
-- Check processing performance
SELECT recognition_status, 
       AVG(processing_time_ms) as avg_time,
       COUNT(*) as event_count
FROM pss_events_v2 
GROUP BY recognition_status;
```

## Future Enhancements

### Planned Features
- **Machine Learning**: Automatic unknown event classification
- **Protocol Versioning**: Support for multiple protocol versions
- **Real-time Analytics**: Live event processing metrics
- **Advanced Validation**: Custom validation rule engine
- **Event Correlation**: Cross-event validation and analysis

### Extensibility
The system is designed for easy extension:
- New event types can be added
- Validation rules can be modified
- Status tracking can be enhanced
- Unknown event analysis can be improved

## Conclusion

The Enhanced PSS Event System provides a robust, scalable, and future-proof solution for PSS event processing. With comprehensive status tracking, validation, and unknown event collection, it ensures data quality while enabling protocol evolution and providing detailed insights into event processing performance. 