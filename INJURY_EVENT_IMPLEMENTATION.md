# INJURY Event Implementation

## Overview

This document describes the implementation of INJURY events from the PSS (Protocol Standard Scoreboard) protocol in the reStrike VTA application. The INJURY event allows the scoreboard overlay to show/hide injury time sections and update injury time values.

## PSS Protocol Specification

According to the PSS protocol (v2.3), the INJURY event has the following structure:

```
# INJURY
# Stream broadcasted when injury time is running.

MAIN_STREAMS:
  ij1;  Main stream for athlete 1
  ij2;  Main stream for athlete 2
  ij0;  Main stream for unidentified athlete

REQUIRED_ARGUMENTS:
  1:23;  Clock of the injury time

OPTIONAL_ARGUMENTS:
  show;   When showing the injury time
  hide;   When hiding the injury time
  reset;  When resetting the injury time

EXAMPLES:
  ij1;1:23;show;
  ij1;1:22;
  ij1;1:21;hide;
  ij2;0:45;show;
  ij2;0:44;
  ij2;0:44;hide;
  ij0;1:42;
```

## Implementation Details

### 1. Scoreboard Overlay HTML (`ui/public/scoreboard-overlay.html`)

#### Event Handler
- Added `injury` case to the main event handler switch statement
- Created `handleInjuryEvent(event)` function to process injury events

#### Event Processing
The `handleInjuryEvent` function:
- Parses time in format "m:ss" or "ss"
- Updates injury time display using `scoreboardInstance.updateInjuryTime(minutes, seconds)`
- Handles show/hide/reset actions:
  - `show`: Calls `scoreboardInstance.showInjurySection()`
  - `hide`: Calls `scoreboardInstance.hideInjurySection()`
  - `reset`: Calls `scoreboardInstance.resetInjuryTime()`

#### Raw Event Parsing
Added parsing for raw PSS injury messages:
- Detects `ij1;`, `ij2;`, or `ij0;` prefixes
- Extracts time and action from the message
- Converts to structured injury event

### 2. Scoreboard Utils (`ui/public/assets/scoreboard/scoreboard-utils.js`)

#### Existing Method
- `updateInjuryTime(minutes, seconds)`: Updates the injury time display

#### New Methods Added
- `showInjurySection()`: Shows the injury section by setting display to 'block' and opacity to '1'
- `hideInjurySection()`: Hides the injury section by setting display to 'none' and opacity to '0'
- `resetInjuryTime()`: Resets injury time to '0:00'

#### Initialization
- The `initialize()` method calls `hideInjurySection()` to ensure the injury section is hidden by default when the scoreboard loads

### 3. SVG Structure (`ui/public/assets/scoreboard/scoreboard-overlay.svg`)

The injury section is present in the SVG with default hidden state:
```xml
<!-- Injury Time Section -->
<g id="injurySection" display="none">
  <polygon points="360,0 520,0 520,80 360,80" fill="url(#yellowGradient)" filter="url(#dropShadow)"/>
  <polygon points="360,0 520,0 520,80 360,80" fill="url(#glassGradient)" opacity="0.5"/>
  <polygon points="360,0 520,0 520,40 360,40" fill="url(#glassHighlight)" opacity="0.7" filter="url(#glassReflection)"/>
  <text x="440" y="60" text-anchor="middle" font-family="Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif" font-size="64" font-weight="600" fill="black" id="injuryTime" filter="url(#emboss3D)">0:00</text>
</g>
```

**Default State**: The injury section is hidden by default with `display="none"` attribute.

## Testing

### Test Page (`ui/public/test-scoreboard-fixes.html`)

Added test buttons for injury functionality:
- **Test Injury Event**: Sends injury event with time update
- **Test Injury Show**: Sends injury event with 'show' action
- **Test Injury Hide**: Sends injury event with 'hide' action  
- **Test Injury Reset**: Sends injury event with 'reset' action
- **Test Default Hidden State**: Opens scoreboard overlay and verifies injury section is hidden by default

### Raw Event Testing
Added injury raw events to the `testRawEvents()` function:
- `ij1;1:23;show;` - Show injury time for athlete 1
- `ij2;0:45;` - Update injury time for athlete 2
- `ij1;1:22;hide;` - Hide injury time for athlete 1

## Usage Examples

### WebSocket Event
```javascript
const injuryEvent = {
    type: 'injury',
    time: '1:23',
    action: 'show'
};
```

### Raw PSS Message
```
ij1;1:23;show;
ij2;0:45;
ij1;1:22;hide;
```

## Event Flow

1. **PSS Protocol**: Sends injury event via UDP to the application
2. **Rust Backend**: Receives and processes the PSS event
3. **WebSocket Server**: Broadcasts the injury event to connected clients
4. **Scoreboard Overlay**: Receives the event and updates the display
5. **SVG Update**: Injury section visibility and time are updated

## Error Handling

- Console warnings if injury section elements are not found
- Graceful fallback if WebSocket connection is unavailable
- Error logging for malformed injury events

## Future Enhancements

- Add visual indicators for which athlete the injury time belongs to
- Implement injury time countdown animations
- Add sound alerts for injury time events
- Support for multiple concurrent injury times 