# Player Name Truncation System

## Overview
The scoreboard overlay includes an intelligent name truncation system to prevent player names from overlapping with the flag elements positioned at x="700".

## How It Works

### 1. CSS Rules
- **Max Width**: Player names are limited to 620px width (700px flag position - 50px start - 30px buffer)
- **Text Overflow**: Long names are truncated with ellipsis (...)
- **Font Size Adjustment**: Very long names get slightly smaller font (48px instead of 56px)

### 2. JavaScript Logic
The `ScoreboardNameManager` class handles dynamic name truncation:

#### Truncation Rules:
1. **Names ≤ 16 characters** (like "GASHIM MAGOMEDOV"): Display as-is
2. **Names > 16 characters**: Apply abbreviation rules:
   - **First attempt**: First letter + dot + last name (e.g., "J. SMITH")
   - **If still too long**: Just last name
   - **If still too wide**: Truncate with ellipsis

#### Examples:
- `"PARK TAEJOON"` → `"PARK TAEJOON"` (≤16 chars, fits)
- `"GASHIM MAGOMEDOV"` → `"GASHIM MAGOMEDOV"` (16 chars, fits)
- `"ALEXANDER THE GREAT"` → `"A. GREAT"` (first letter + last name)
- `"VERY LONG PLAYER NAME"` → `"NAME"` (just last name)
- `"SUPER DUPER LONG NAME"` → `"NAME..."` (truncated with ellipsis)

## Usage

### Basic Usage
```javascript
// Initialize the name manager
const nameManager = new ScoreboardNameManager();

// Set both player names
nameManager.initializeNames('PARK TAEJOON', 'GASHIM MAGOMEDOV');
```

### Individual Updates
```javascript
// Update individual player names
nameManager.updatePlayerName('player1Name', 'NEW PLAYER NAME');
nameManager.updatePlayerName('player2Name', 'ANOTHER PLAYER');
```

### Integration with Existing Code
```javascript
// In your existing scoreboard update logic
function updateScoreboard(player1Data, player2Data) {
  // Update scores, rounds, etc.
  document.getElementById('player1Score').textContent = player1Data.score;
  document.getElementById('player2Score').textContent = player2Data.score;
  
  // Update names with truncation
  nameManager.updatePlayerName('player1Name', player1Data.fullName);
  nameManager.updatePlayerName('player2Name', player2Data.fullName);
}
```

## Technical Details

### CSS Classes
- `#player1Name, #player2Name`: Base styling with max-width and overflow handling
- `[data-long="true"]`: Applied to names > 16 characters, reduces font size

### JavaScript Methods
- `truncatePlayerName(fullName)`: Main truncation logic
- `checkWidthAndTruncate(name)`: Canvas-based width measurement
- `updatePlayerName(playerId, fullName)`: Updates SVG element with truncated name
- `initializeNames(player1Name, player2Name)`: Sets both player names

### Width Calculation
- **Available space**: 620px (700px flag position - 50px name start - 30px buffer)
- **Font**: Bold 56px Arial (48px for long names)
- **Measurement**: Uses HTML5 Canvas for precise text width calculation

## File Structure
```
ui/public/assets/scoreboard/
├── scoreboard-overlay.svg          # Main SVG with CSS rules
├── scoreboard-name-utils.js        # JavaScript truncation logic
└── README-name-truncation.md       # This documentation
```

## Browser Compatibility
- **Modern browsers**: Full support with Canvas text measurement
- **Fallback**: CSS-only truncation with ellipsis for older browsers
- **Progressive enhancement**: Works without JavaScript, but with limited truncation

## Testing
Test with various name lengths:
- Short: "JOHN DOE"
- Medium: "GASHIM MAGOMEDOV" 
- Long: "ALEXANDER THE GREAT"
- Very Long: "SUPER DUPER LONG PLAYER NAME"
- Single Name: "MESSI"
- Multiple Names: "JOSE MARIA GUTIERREZ" 