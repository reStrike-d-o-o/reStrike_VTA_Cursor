# Taekwondo Scoreboard Overlays

High-resolution SVG overlays for taekwondo competition livestreaming at 1080p and 4K resolutions.

## 📁 Current Structure

We standardized overlays by theme and use HTML wrappers for OBS:

### HTML Wrappers
```
ui/public/overlays/
  olympic/
    scoreboard.html  -> /assets/scoreboard/olympic/olympic_scoreboard.svg
    intro.html       -> /assets/scoreboard/olympic/olympic_players_overlay.svg
  modern/
    scoreboard.html  -> /assets/scoreboard/modern/modern_scoreboard.svg
    intro.html       -> /assets/scoreboard/modern/modern_players_overlay.svg
  arcade/
    scoreboard.html  -> /assets/scoreboard/arcade/arcade_scoreboard.svg
    intro.html       -> /assets/scoreboard/arcade/arcade_players_overlay.svg
```

### JavaScript Utilities
- `scoreboard-utils.js` (canonical bindings) and `scoreboard-name-utils.js`

## 🎨 Design Features

### Color Coding (PSS Protocol)
- **Blue Player (Chong)** - Player 1 - Blue gradient backgrounds
- **Red Player (Hong)** - Player 2 - Red gradient backgrounds
- **Professional gradients** with drop shadows for depth
- **High contrast** for readability on any background
- **Multiple color themes**: Default, Olympic, Dark, Bright
- **Adjustable transparency** levels for overlay customization

### Typography
- **Arial font family** for maximum compatibility
- **Bold weights** for scores and important information
- **Scalable text** that remains crisp at any resolution

### Animations
- **Score update animations** with scale effects
- **Timer warning** for last 10 seconds
- **Recording status** with pulse animation
- **Winner reveal** with glow effects
- **Smooth transitions** between overlay states

## 🚀 Usage Instructions

### 1. PSS Drawer Integration

The scoreboard functionality is now fully integrated into the PSS drawer's Scoreboard tab:

1. **Open Advanced Panel** → Click "Advanced" button
2. **Select PSS Drawer** → Click the PSS icon in the sidebar
3. **Go to Scoreboard Tab** → Click the "Scoreboard" tab
4. **Configure Overlays** → Use the comprehensive controls

### 2. Basic Integration

Use the theme wrapper URLs (examples use localhost):
```
http://localhost:3000/overlays/olympic/scoreboard.html
```

### 2. Dynamic Updates

```javascript
// Initialize the scoreboard overlay
const svgElement = document.querySelector('object').contentDocument;
const scoreboard = new ScoreboardOverlay(svgElement);

// Update player names
scoreboard.updatePlayerNames(
  { name: 'KIM JONG-SOO' },  // Blue player
  { name: 'PARK TAE-JOON' }  // Red player
);

// For Player Introduction Overlay - Update with side switching
const playerIntro = new PlayerIntroductionOverlay(svgElement);
playerIntro.updatePlayerDetails(
  { name: 'KIM JONG-SOO', category: 'MEN\'S -58KG', matchType: 'GOLD MEDAL CONTEST' },  // Blue player
  { name: 'PARK TAE-JOON', category: 'MEN\'S -58KG', matchType: 'GOLD MEDAL CONTEST' },  // Red player
  'blueLeft'  // or 'redLeft' to switch sides
);

// Switch player positions
playerIntro.switchSides();

// Or set specific positions
playerIntro.setBlueLeft();  // Blue player on left (default)
playerIntro.setRedLeft();   // Red player on left

// Update scores with animation
scoreboard.updateScores(12, 8);

// Update timer
scoreboard.updateTimer('02:45', 2);

// Update match status
scoreboard.updateMatchStatus('FIGHTING');

// Update penalties and warnings
scoreboard.updatePenalties('blue', 1, 0);
scoreboard.updatePenalties('red', 0, 1);

// Update round wins
scoreboard.updateRoundWins('blue', 1);
scoreboard.updateRoundWins('red', 2);

// Update connection status
scoreboard.updateConnectionStatus(true);

// Update recording status
scoreboard.updateRecordingStatus(true);
```

### 3. PSS Protocol Integration

The overlays automatically listen for PSS events:

```javascript
// PSS events are automatically handled
window.__TAURI__.event.emit('pss-event', {
  type: 'score_update',
  blueScore: 12,
  redScore: 8
});

window.__TAURI__.event.emit('pss-event', {
  type: 'timer_update',
  time: '02:45',
  round: 2
});
```

### 4. Overlay Type Switching

```javascript
// Switch between different overlay types
scoreboard.changeOverlayType('scoreboard');      // Live scoreboard
scoreboard.changeOverlayType('introduction');    // Player introduction
scoreboard.changeOverlayType('winner');          // Winner announcement
scoreboard.changeOverlayType('results');         // Previous results
```

### 5. Player Introduction Side Switching

```javascript
// Initialize player introduction overlay
const playerIntro = new PlayerIntroductionOverlay(svgElement);

// Set players with specific layout
playerIntro.updatePlayerDetails(bluePlayer, redPlayer, 'blueLeft');  // Default: Blue left, Red right
playerIntro.updatePlayerDetails(bluePlayer, redPlayer, 'redLeft');   // Red left, Blue right

// Switch sides dynamically
playerIntro.switchSides();  // Toggle between layouts

// Set specific positions
playerIntro.setBlueLeft();  // Blue player on left
playerIntro.setRedLeft();   // Red player on left
```

### 6. Victory Ceremony Overlay

```javascript
// Initialize victory ceremony overlay
const victoryCeremony = new VictoryCeremonyOverlay(svgElement);

// Update ceremony with medalists
victoryCeremony.updateCeremony({
  gold: { country: 'KOR', name: 'PARK TAEJOON', seed: 5, score: '2-0' },
  silver: { country: 'AZE', name: 'GASHIM MAGOMEDOV', seed: 10, score: '0-2' },
  bronze1: { country: 'FRA', name: 'CYRIAN RAVET', seed: 4, score: '2-1' },
  bronze2: { country: 'TUN', name: 'MOHAMED KHALIL', seed: 1, score: '1-2' }
}, {
  category: 'MEN\'S -58KG',
  type: 'VICTORY CEREMONY',
  title: 'OLYMPIC GAMES - TOKYO 2024',
  date: 'July 25, 2024'
});
```

### 7. Color Themes and Transparency

```javascript
// Set color theme
scoreboard.setTheme('olympic');  // 'default', 'olympic', 'dark', 'bright'

// Set transparency level (0.1 to 1.0)
scoreboard.setTransparency(0.8);

// Toggle between full names and country codes
// This is handled in the PSS drawer interface
```

## 📺 Streaming Software Integration

### OBS Studio
1. Add **Browser Source**
2. Set URL to your HTML file containing the overlay
3. Set width to **1920** and height to **1080**
4. Enable **Refresh browser when scene becomes active**

### Streamlabs OBS
1. Add **Browser Source**
2. Set URL to your HTML file
3. Set dimensions to **1920x1080**
4. Enable **Shutdown source when not visible**

### XSplit Broadcaster
1. Add **Web Page** source
2. Set URL to your HTML file
3. Set dimensions to **1920x1080**

## 🎯 Resolution Support

### 1080p (1920x1080)
- **Native resolution** - Perfect quality
- **Recommended** for most streaming setups

### 4K (3840x2160)
- **Scalable SVG** - Maintains crisp quality
- **Double the dimensions** for ultra-high resolution

### Custom Resolutions
- **ViewBox scaling** - Automatically adapts
- **Maintains aspect ratio** at any size

## 🔧 Customization

### Colors
Edit the gradient definitions in the SVG files:

```xml
<linearGradient id="bluePlayerGradient">
  <stop offset="0%" style="stop-color:#1e40af"/>
  <stop offset="100%" style="stop-color:#3b82f6"/>
</linearGradient>
```

### Fonts
Change the font-family attribute:

```xml
<text font-family="Arial, sans-serif">Player Name</text>
```

### Animations
Modify CSS animations in the `<style>` section:

```css
@keyframes scoreUpdate {
  0% { transform: scale(1); }
  50% { transform: scale(1.1); }
  100% { transform: scale(1); }
}
```

## 📊 Event Types Supported

### Scoreboard Events
- Score updates
- Timer changes
- Round progression
- Match status changes
- Penalty/warning updates
- Round win indicators

### Player Introduction Events
- Player name updates
- Match category information
- Flag integration (placeholder)
- Side switching (blue/red player positions)

### Victory Ceremony Events
- 4-player medal ceremony display
- Gold, Silver, and Bronze medalists
- Event information and ceremony details
- Medal reveal animations

### Winner Announcement Events
- Winner name display
- Final score presentation
- Round-by-round results
- Victory method indication

### Previous Results Events
- Match history display
- Opponent information
- Win/loss statistics
- Tournament progression

## 🔗 Integration with reStrike VTA

These overlays are designed to work seamlessly with the reStrike VTA system:

1. **PSS Protocol Integration** - Automatic blue/red player detection
2. **Real-time Updates** - Live score and event updates
3. **Flag System** - Integration with existing flag assets
4. **Event System** - Compatible with existing event handling

## 📝 Notes

- **No Olympic branding** - Clean, generic design
- **Full player names** - Dynamically changeable
- **Blue/Red color coding** - Based on PSS protocol (Chong/Hong)
- **High resolution** - Optimized for 1080p and 4K streaming
- **Professional appearance** - Suitable for competition broadcasting

## 🆘 Troubleshooting

### Overlay not displaying
- Check file paths are correct
- Verify SVG file is accessible
- Ensure streaming software supports SVG

### Updates not working
- Check JavaScript console for errors
- Verify event listeners are properly set up
- Ensure Tauri integration is working

### Performance issues
- Reduce animation complexity if needed
- Check for memory leaks in event listeners
- Monitor CPU usage during streaming

## 📞 Support

For technical support or customization requests, refer to the main project documentation or create an issue in the project repository. 