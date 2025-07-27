/**
 * Scoreboard Name Utilities
 * Handles dynamic truncation of player names to prevent overlap with flags
 */

class ScoreboardNameManager {
  constructor() {
    this.maxNameWidth = 620; // 700 - 50 - 30px buffer
    this.flagPosition = 700;
    this.nameStartPosition = 50;
  }

  /**
   * Truncate player name based on length and available space
   * @param {string} fullName - The complete player name
   * @returns {string} - Truncated name
   */
  truncatePlayerName(fullName) {
    if (!fullName) return '';
    
    const nameParts = fullName.trim().split(' ');
    
    // If single name, return as is
    if (nameParts.length === 1) {
      return this.checkWidthAndTruncate(fullName);
    }
    
    // If name is longer than "GASHIM MAGOMEDOV" (16 characters)
    if (fullName.length > 16) {
      // Try first letter + dot + last name
      const firstName = nameParts[0];
      const lastName = nameParts[nameParts.length - 1];
      const abbreviatedName = `${firstName.charAt(0)}. ${lastName}`;
      
      // If still too long, just use last name
      if (abbreviatedName.length > 16) {
        return this.checkWidthAndTruncate(lastName);
      }
      
      return this.checkWidthAndTruncate(abbreviatedName);
    }
    
    // For names shorter than or equal to "GASHIM MAGOMEDOV", return as is
    return this.checkWidthAndTruncate(fullName);
  }

  /**
   * Check if name fits within available width and truncate if necessary
   * @param {string} name - The name to check
   * @returns {string} - Truncated name if needed
   */
  checkWidthAndTruncate(name) {
    // Create a temporary canvas to measure text width
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    ctx.font = 'bold 56px Arial, sans-serif';
    
    const textWidth = ctx.measureText(name).width;
    
    // If text fits within max width, return as is
    if (textWidth <= this.maxNameWidth) {
      return name;
    }
    
    // If text is too wide, truncate with ellipsis
    let truncatedName = name;
    while (ctx.measureText(truncatedName + '...').width > this.maxNameWidth && truncatedName.length > 0) {
      truncatedName = truncatedName.slice(0, -1);
    }
    
    return truncatedName + '...';
  }

  /**
   * Update player name in the SVG overlay
   * @param {string} playerId - 'player1Name' or 'player2Name'
   * @param {string} fullName - The complete player name
   */
  updatePlayerName(playerId, fullName) {
    const nameElement = document.getElementById(playerId);
    if (!nameElement) return;
    
    const truncatedName = this.truncatePlayerName(fullName);
    nameElement.textContent = truncatedName;
    
    // Add data attribute for long names
    if (fullName.length > 16) {
      nameElement.setAttribute('data-long', 'true');
    } else {
      nameElement.removeAttribute('data-long');
    }
  }

  /**
   * Initialize name truncation for both players
   * @param {string} player1Name - First player's full name
   * @param {string} player2Name - Second player's full name
   */
  initializeNames(player1Name, player2Name) {
    this.updatePlayerName('player1Name', player1Name);
    this.updatePlayerName('player2Name', player2Name);
  }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
  module.exports = ScoreboardNameManager;
} else {
  // Browser environment
  window.ScoreboardNameManager = ScoreboardNameManager;
}

// Example usage:
// const nameManager = new ScoreboardNameManager();
// nameManager.initializeNames('PARK TAEJOON', 'GASHIM MAGOMEDOV');
// 
// For very long names:
// nameManager.updatePlayerName('player1Name', 'VERY LONG PLAYER NAME THAT EXCEEDS LIMIT');
// nameManager.updatePlayerName('player2Name', 'ANOTHER VERY LONG PLAYER NAME'); 