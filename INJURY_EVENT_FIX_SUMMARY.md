# Injury Event Fix Summary

## Problem
The injury time section was not showing up because injury events were being received with `type: 'other'` instead of `type: 'injury'`. This caused them to fall into the default case of the `handlePssEvent` switch statement and be unhandled.

## Root Cause
From the console logs provided by the user:
```
📡 Received PSS event: {
  description: 'Event: Injury { athlete: 0, time: "01:00", action: Some("show") }',
  event: 'Injury { athlete: 0, time: "01:00", action: Some("show") }',
  type: 'other'
}
📝 Unhandled event type: other
```

The injury data was embedded within the `description` or `event` string, but the event type was incorrectly set to `'other'`.

## Solution Implemented

### 1. Modified Event Handler in `scoreboard-overlay.html`
- Updated the default case in `handlePssEvent` to detect injury events disguised as `'other'` type
- Added `handleInjuryEventFromOther` function to parse injury data from the description/event string

### 2. Added Missing Injury Event Handlers
- Created `handleInjuryEvent` function for properly typed injury events
- Created `handleInjuryEventFromOther` function to parse injury data from string format
- Added regex parsing to extract `athlete`, `time`, and `action` from injury strings

### 3. Fixed `updateInjuryTime` Method in `scoreboard-utils.js`
- Modified to handle both string format (`"1:00"`) and separate parameters (`minutes`, `seconds`)
- Added proper logging for debugging

### 4. Added Raw Injury Event Parsing
- Added parsing for raw PSS injury messages (`ij1;`, `ij2;`, `ij0;`) in `handleRawEvent`
- Supports both time updates and show/hide actions

### 5. Enhanced Testing
- Added `testInjuryFromOtherType()` function to test the new parsing
- Added button in test page to trigger the test

## Code Changes

### `ui/public/scoreboard-overlay.html`
```javascript
// Modified default case
default:
    // Check if this is an injury event disguised as 'other' type
    if (event.type === 'other' && (event.description?.includes('Injury') || event.event?.includes('Injury'))) {
        console.log('🩹 Parsing injury event from other type:', event);
        handleInjuryEventFromOther(event);
    } else {
        console.log('📝 Unhandled event type:', event.type);
    }

// Added injury event handlers
function handleInjuryEvent(event) {
    console.log('🩹 Injury event:', event);
    
    if (event.time !== undefined) {
        scoreboardInstance.updateInjuryTime(event.time);
    }
    
    if (event.action) {
        if (event.action === 'show') {
            scoreboardInstance.showInjurySection();
        } else if (event.action === 'hide') {
            scoreboardInstance.hideInjurySection();
        } else if (event.action === 'reset') {
            scoreboardInstance.resetInjuryTime();
        }
    }
}

function handleInjuryEventFromOther(event) {
    console.log('🩹 Parsing injury event from other type:', event);
    
    // Extract injury data from description or event string
    const injuryString = event.description || event.event || '';
    
    // Parse the injury data using regex
    const injuryMatch = injuryString.match(/Injury\s*\{\s*athlete:\s*(\d+),\s*time:\s*"([^"]+)",\s*action:\s*(Some\("([^"]+)"\)|None)\s*\}/);
    
    if (injuryMatch) {
        const athlete = parseInt(injuryMatch[1]);
        const time = injuryMatch[2];
        const actionRaw = injuryMatch[3];
        const action = injuryMatch[4] || null; // Extract from Some("action") or null
        
        console.log('🩹 Parsed injury data:', { athlete, time, action });
        
        // Update injury time
        if (time) {
            scoreboardInstance.updateInjuryTime(time);
        }
        
        // Handle action
        if (action === 'show') {
            scoreboardInstance.showInjurySection();
            console.log('✅ Showing injury section');
        } else if (action === 'hide') {
            scoreboardInstance.hideInjurySection();
            console.log('✅ Hiding injury section');
        } else if (action === 'reset') {
            scoreboardInstance.resetInjuryTime();
            console.log('✅ Resetting injury time');
        } else if (action === null) {
            // Just update the time without changing visibility
            console.log('✅ Updated injury time only');
        }
    } else {
        console.warn('⚠️ Could not parse injury data from string:', injuryString);
    }
}
```

### `ui/public/assets/scoreboard/scoreboard-utils.js`
```javascript
// Fixed updateInjuryTime method
updateInjuryTime(time) {
    const injuryElement = this.svg.getElementById('injuryTime');
    if (injuryElement) {
        // Handle both string format ("1:00") and separate parameters (minutes, seconds)
        if (typeof time === 'string') {
            injuryElement.textContent = time;
            console.log(`✅ Updated injury time: ${time}`);
        } else {
            // Fallback for separate minutes/seconds parameters
            const minutes = arguments[0] || 0;
            const seconds = arguments[1] || 0;
            injuryElement.textContent = `${minutes}:${seconds.toString().padStart(2, '0')}`;
            console.log(`✅ Updated injury time: ${minutes}:${seconds.toString().padStart(2, '0')}`);
        }
    } else {
        console.warn(`⚠️ Could not find injuryTime element`);
    }
}
```

## Testing
1. Open `ui/public/test-scoreboard-fixes.html`
2. Click "Test Injury From Other Type" button
3. Check the scoreboard overlay to see if the injury section appears with time "01:00"
4. Check console logs for parsing confirmation

## Expected Behavior
- Injury events with `type: 'other'` will now be properly parsed
- The injury section will show/hide based on the `action` field
- Injury time will update correctly
- Console logs will show successful parsing and updates

## Backend Note
This is a frontend workaround for the backend issue where injury events are being sent with `type: 'other'` instead of `type: 'injury'`. The root cause should be fixed in the backend PSS event processing to send the correct event type. 