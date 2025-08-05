/**
 * Scoreboard Overlay Utilities
 * Dynamic update functions for taekwondo competition overlays
 */

// Utility function to properly capitalize names (first letter of each word)
function capitalizeName(name) {
  if (!name) return '';
  return name.toLowerCase().split(' ').map(word => 
    word.charAt(0).toUpperCase() + word.slice(1)
  ).join(' ');
}

// Scoreboard Overlay Management Class
class ScoreboardOverlay {
  constructor(svgElement) {
    this.svg = svgElement;
    this.currentTheme = 'default';
    this.transparency = 1.0;
    this.initialize();
  }

  initialize() {
    // Set initial transparency
    this.setTransparency(this.transparency);
    
    // Apply default theme
    this.applyTheme(this.currentTheme);
    
    // Ensure injury section is hidden by default
    this.hideInjurySection();
  }

  // Update player names
  updatePlayerName(player, name) {
    // Map player colors to SVG element IDs
    const elementId = player === 'blue' ? 'player1Name' : 'player2Name';
    const nameElement = this.svg.getElementById(elementId);
    if (nameElement) {
      // Apply proper capitalization (first letter of each word)
      const capitalizedName = capitalizeName(name);
      nameElement.textContent = capitalizedName;
      console.log(`✅ Updated ${player} player name: ${capitalizedName}`);
    } else {
      console.warn(`⚠️ Could not find ${elementId} element`);
    }
  }

  // Update player scores
  updateScore(player, score) {
    // Map player colors to SVG element IDs
    const elementId = player === 'blue' ? 'player1Score' : 'player2Score';
    console.log(`🎯 Updating score for ${player} player, element ID: ${elementId}, score: ${score}`);
    console.log(`🎯 SVG element:`, this.svg);
    
    const scoreElement = this.svg.getElementById(elementId);
    console.log(`🎯 Found score element:`, scoreElement);
    
    if (scoreElement) {
      scoreElement.textContent = score;
      scoreElement.classList.add('score-update');
      setTimeout(() => scoreElement.classList.remove('score-update'), 500);
      console.log(`✅ Updated ${player} player score: ${score}`);
    } else {
      console.warn(`⚠️ Could not find ${elementId} element`);
      console.warn(`⚠️ Available elements with 'Score' in ID:`, Array.from(this.svg.querySelectorAll('[id*="Score"]')).map(el => el.id));
    }
  }

  // Update player countries (flags)
  updateCountry(player, country) {
    // Map player colors to SVG element IDs
    const elementId = player === 'blue' ? 'player1Flag' : 'player2Flag';
    const flagElement = this.svg.getElementById(elementId);
    if (flagElement) {
      // Update the flag image source
      flagElement.setAttribute('href', `../flags/svg/${country}.svg`);
      console.log(`✅ Updated ${player} player country flag: ${country}`);
    } else {
      console.warn(`⚠️ Could not find ${elementId} element`);
    }
  }

  // Update player seeds
  updateSeed(player, seed) {
    const seedElement = this.svg.getElementById(`${player}PlayerSeed`);
    if (seedElement) seedElement.textContent = `(${seed})`;
  }

  // Update penalties and warnings
  updatePenalties(player, penalties, warnings) {
    // Map player colors to SVG element IDs
    const elementId = player === 'blue' ? 'player1Fouls' : 'player2Fouls';
    const penaltiesElement = this.svg.getElementById(elementId);
    if (penaltiesElement) {
      penaltiesElement.textContent = warnings || penalties || 0;
      // Apply pop-out animation
      penaltiesElement.classList.add('update');
      setTimeout(() => penaltiesElement.classList.remove('update'), 500);
      console.log(`✅ Updated ${player} player warnings: ${warnings || penalties || 0}`);
    } else {
      console.warn(`⚠️ Could not find ${elementId} element`);
    }
  }

  // Update round wins
  updateRoundWins(player, wins) {
    // Map player colors to SVG element IDs
    const elementId = player === 'blue' ? 'player1Rounds' : 'player2Rounds';
    const winsElement = this.svg.getElementById(elementId);
    if (winsElement) {
      winsElement.textContent = wins || 0;
      // Apply pop-out animation
      winsElement.classList.add('update');
      setTimeout(() => winsElement.classList.remove('update'), 500);
      console.log(`✅ Updated ${player} player rounds: ${wins || 0}`);
    } else {
      console.warn(`⚠️ Could not find ${elementId} element`);
    }
  }

  // Update match timer
  updateTimer(minutes, seconds) {
    const timerElement = this.svg.getElementById('matchTimer');
    if (timerElement) {
      timerElement.textContent = `${minutes}:${seconds.toString().padStart(2, '0')}`;
      console.log(`✅ Updated match timer: ${minutes}:${seconds.toString().padStart(2, '0')}`);
    } else {
      console.warn(`⚠️ Could not find matchTimer element`);
    }
  }

  // Update current round
  updateRound(round) {
    const roundElement = this.svg.getElementById('currentRound');
    if (roundElement) {
      roundElement.textContent = this.getOrdinalSuffix(round);
      console.log(`✅ Updated current round: ${this.getOrdinalSuffix(round)}`);
    } else {
      console.warn(`⚠️ Could not find currentRound element`);
    }
  }

  // Update injury time
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

  // Show injury section
  showInjurySection() {
    const injurySection = this.svg.getElementById('injurySection');
    if (injurySection) {
      injurySection.style.display = 'block';
      injurySection.style.opacity = '1';
      console.log('✅ Injury section shown');
    } else {
      console.warn('⚠️ Could not find injurySection element');
    }
  }

  // Hide injury section
  hideInjurySection() {
    const injurySection = this.svg.getElementById('injurySection');
    if (injurySection) {
      injurySection.style.display = 'none';
      injurySection.style.opacity = '0';
      console.log('✅ Injury section hidden');
    } else {
      console.warn('⚠️ Could not find injurySection element');
    }
  }

  // Reset injury time to 0:00
  resetInjuryTime() {
    const injuryElement = this.svg.getElementById('injuryTime');
    if (injuryElement) {
      injuryElement.textContent = '0:00';
      console.log('✅ Injury time reset to 0:00');
    } else {
      console.warn('⚠️ Could not find injuryTime element');
    }
  }

  // Apply new match effect (typewriter animation for new matches)
  applyNewMatchEffect(roundDuration = 120, currentRound = 1) {
    console.log('🎬 Applying new match effect with round duration:', roundDuration, 'and current round:', currentRound);
    
    // Add a subtle animation to indicate new match data
    const elements = [
      this.svg.getElementById('player1Name'),
      this.svg.getElementById('player2Name'),
      this.svg.getElementById('player1Score'),
      this.svg.getElementById('player2Score'),
      this.svg.getElementById('currentRound'),
      this.svg.getElementById('matchTimer')
    ];
    
    elements.forEach((element, index) => {
      if (element) {
        // Add a brief flash effect
        element.style.transition = 'all 0.3s ease';
        element.style.transform = 'scale(1.05)';
        element.style.filter = 'brightness(1.2)';
        
        setTimeout(() => {
          element.style.transform = 'scale(1)';
          element.style.filter = 'brightness(1)';
        }, 300 + (index * 50)); // Stagger the animations
      }
    });
    
    // Reset scores and timer to initial state
    this.updateScore('blue', 0);
    this.updateScore('red', 0);
    this.updateRound(currentRound);
    
    // Use provided round duration instead of hardcoded 2:00
    const minutes = Math.floor(roundDuration / 60);
    const seconds = roundDuration % 60;
    this.updateTimer(minutes, seconds);
    
    this.updatePenalties('blue', null, 0);
    this.updatePenalties('red', null, 0);
    this.updateRoundWins('blue', 0);
    this.updateRoundWins('red', 0);
    
    console.log('✅ New match effect applied');
  }

  // Update combined match info (weight, division, category)
  updateMatchInfo(weight, division, category) {
    const matchInfoElement = this.svg.getElementById('matchInfo');
    if (matchInfoElement) {
      const combinedText = `${weight || ''} ${division || ''} ${category || ''}`.trim();
      matchInfoElement.textContent = combinedText;
      console.log(`✅ Updated match info: ${combinedText}`);
    } else {
      console.warn(`⚠️ Could not find matchInfo element`);
    }
  }

  // Update match category (for backward compatibility)
  updateMatchCategory(category) {
    const matchInfoElement = this.svg.getElementById('matchInfo');
    if (matchInfoElement) {
      // Get current weight and division from the element
      const currentText = matchInfoElement.textContent || '';
      const parts = currentText.split(' ');
      const weight = parts[0] || '';
      const division = parts[1] || '';
      const combinedText = `${weight} ${division} ${category || ''}`.trim();
      matchInfoElement.textContent = combinedText;
      console.log(`✅ Updated match category: ${category}`);
    } else {
      console.warn(`⚠️ Could not find matchInfo element`);
    }
  }

  // Update match type (weight class) - for backward compatibility
  updateMatchType(type) {
    const typeElement = this.svg.getElementById('matchType');
    if (typeElement) {
      typeElement.textContent = type;
      console.log(`✅ Updated match type: ${type}`);
    } else {
      console.warn(`⚠️ Could not find matchType element`);
    }
  }

  // Update match weight (for backward compatibility)
  updateMatchWeight(weight) {
    const matchInfoElement = this.svg.getElementById('matchInfo');
    if (matchInfoElement) {
      // Get current division and category from the element
      const currentText = matchInfoElement.textContent || '';
      const parts = currentText.split(' ');
      const division = parts[1] || '';
      const category = parts.slice(2).join(' ') || '';
      const combinedText = `${weight || ''} ${division} ${category}`.trim();
      matchInfoElement.textContent = combinedText;
      console.log(`✅ Updated match weight: ${weight}`);
    } else {
      console.warn(`⚠️ Could not find matchInfo element`);
    }
  }

  // Update match division (for backward compatibility)
  updateMatchDivision(division) {
    const matchInfoElement = this.svg.getElementById('matchInfo');
    if (matchInfoElement) {
      // Get current weight and category from the element
      const currentText = matchInfoElement.textContent || '';
      const parts = currentText.split(' ');
      const weight = parts[0] || '';
      const category = parts.slice(2).join(' ') || '';
      const combinedText = `${weight} ${division || ''} ${category}`.trim();
      matchInfoElement.textContent = combinedText;
      console.log(`✅ Updated match division: ${division}`);
    } else {
      console.warn(`⚠️ Could not find matchInfo element`);
    }
  }

                    

  // Get ordinal suffix for round numbers
  getOrdinalSuffix(num) {
    const j = num % 10;
    const k = num % 100;
    if (j == 1 && k != 11) {
      return num + "ST";
    }
    if (j == 2 && k != 12) {
      return num + "ND";
    }
    if (j == 3 && k != 13) {
      return num + "RD";
    }
    return num + "TH";
  }

  // Set transparency level
  setTransparency(level) {
    this.transparency = level;
    if (this.svg) {
      this.svg.style.opacity = level.toString();
    }
  }

  // Set color theme
  setTheme(theme) {
    this.currentTheme = theme;
    this.applyTheme(theme);
  }

  // Apply theme colors
  applyTheme(theme) {
    const root = this.svg;
    
    switch (theme) {
      case 'olympic':
        root.style.setProperty('--header-color', '#1B9DA3');
        root.style.setProperty('--accent-color', '#FFD700');
        break;
      case 'dark':
        root.style.setProperty('--header-color', '#111827');
        root.style.setProperty('--accent-color', '#6b7280');
        break;
      case 'bright':
        root.style.setProperty('--header-color', '#3b82f6');
        root.style.setProperty('--accent-color', '#fbbf24');
        break;
      default:
        root.style.setProperty('--header-color', '#1B9DA3');
        root.style.setProperty('--accent-color', '#FFD700');
    }
  }

  // Change overlay type
  changeOverlayType(type) {
    // This would be handled by loading a different SVG file
    console.log(`Changing overlay type to: ${type}`);
  }

  // Show/hide sections
  toggleSection(sectionId, visible) {
    const section = this.svg.getElementById(sectionId);
    if (section) {
      section.style.display = visible ? 'block' : 'none';
    }
  }

  // Add animation to section
  addSectionAnimation(sectionId, animationClass) {
    const section = this.svg.getElementById(sectionId);
    if (section) {
      section.classList.add(animationClass);
    }
  }

  // Remove animation from section
  removeSectionAnimation(sectionId, animationClass) {
    const section = this.svg.getElementById(sectionId);
    if (section) {
      section.classList.remove(animationClass);
    }
  }
}

// Player Introduction Overlay Class
class PlayerIntroductionOverlay extends ScoreboardOverlay {
  constructor(svgElement) {
    super(svgElement);
    this.initialize();
  }

  initialize() {
    // Set initial transparency
    this.setTransparency(this.transparency);
    
    // Apply default theme
    this.applyTheme(this.currentTheme);
    
    console.log('✅ Player Introduction Overlay initialized');
  }

  // Update Player 1 (Blue) information
  updatePlayer1(name, country) {
    this.updatePlayer1Name(name);
    this.updatePlayer1Flag(country);
  }

  // Update Player 2 (Red) information
  updatePlayer2(name, country) {
    this.updatePlayer2Name(name);
    this.updatePlayer2Flag(country);
  }

  // Update Player 1 name in the VS string
  updatePlayer1Name(name) {
    const nameElement = this.svg.getElementById('playerVSString');
    if (nameElement) {
      const currentText = nameElement.textContent;
      const parts = currentText.split(' VS ');
      if (parts.length === 2) {
        const newText = `${capitalizeName(name)} VS ${parts[1]}`;
        nameElement.textContent = newText;
      } else {
        const newText = `${capitalizeName(name)} VS Gashim Magomedov`;
        nameElement.textContent = newText;
      }
      
      console.log(`✅ Updated Player 1 name: ${capitalizeName(name)}`);
    }
  }

  // Update Player 2 name in the VS string
  updatePlayer2Name(name) {
    const nameElement = this.svg.getElementById('playerVSString');
    if (nameElement) {
      const currentText = nameElement.textContent;
      const parts = currentText.split(' VS ');
      if (parts.length === 2) {
        const newText = `${parts[0]} VS ${capitalizeName(name)}`;
        nameElement.textContent = newText;
      } else {
        const newText = `Park Taejoon VS ${capitalizeName(name)}`;
        nameElement.textContent = newText;
      }
      
      console.log(`✅ Updated Player 2 name: ${capitalizeName(name)}`);
    }
  }

  // Update Player 1 flag
  updatePlayer1Flag(countryCode) {
    const flagElement = this.svg.getElementById('leftPlayerFlag');
    if (flagElement) {
      flagElement.setAttribute('href', `../flags/svg/${countryCode}.svg`);
      
             // Adjust glass effect rectangle after flag loads
       const adjustLeftFlag = () => {
         // Get the actual rendered width of the flag
         const flagRect = flagElement.getBoundingClientRect();
         const flagWidth = flagRect.width;
         
         if (flagWidth > 0) {
           // Update the glass effect rectangle width
           const glassRect = this.svg.getElementById('leftPlayerFlagGlass');
           if (glassRect) {
             glassRect.setAttribute('width', flagWidth.toString());
           }
           
           console.log(`✅ Updated Player 1 flag glass effect: width=${flagWidth}`);
         } else {
           // If width is not available yet, try again after a short delay
           setTimeout(adjustLeftFlag, 100);
         }
       };
      
      flagElement.onload = adjustLeftFlag;
      setTimeout(adjustLeftFlag, 50);
      
      console.log(`✅ Updated Player 1 flag: ${countryCode}`);
    }
  }

  // Update Player 2 flag
  updatePlayer2Flag(countryCode) {
    const flagElement = this.svg.getElementById('rightPlayerFlag');
    if (flagElement) {
      flagElement.setAttribute('href', `../flags/svg/${countryCode}.svg`);
      
             // Dynamically adjust position after flag loads to ensure 20px right padding
       const adjustFlagPosition = () => {
         // Get the actual rendered width of the flag
         const flagRect = flagElement.getBoundingClientRect();
         const flagWidth = flagRect.width;
         
         if (flagWidth > 0) {
           // Calculate new x position to ensure 20px right padding
           // Red rectangle starts at x=1660, so flag should end at x=1640
           const newX = 1640 - flagWidth;
           
           // Update flag position
           flagElement.setAttribute('x', newX.toString());
           
           // Also update the glass effect rectangle position and width
           const glassRect = this.svg.getElementById('rightPlayerFlagGlass');
           if (glassRect) {
             glassRect.setAttribute('x', newX.toString());
             glassRect.setAttribute('width', flagWidth.toString());
           }
           
           console.log(`✅ Updated Player 2 flag position: x=${newX}, width=${flagWidth}`);
         } else {
           // If width is not available yet, try again after a short delay
           setTimeout(adjustFlagPosition, 100);
         }
       };
      
      // Wait for the flag to load and then adjust position
      flagElement.onload = adjustFlagPosition;
      
      // Also try immediately in case the flag is already loaded
      setTimeout(adjustFlagPosition, 50);
      
      console.log(`✅ Updated Player 2 flag: ${countryCode}`);
    }
  }

  // Apply announcement effect
  applyAnnouncementEffect() {
    const announcementSection = this.svg.getElementById('announcementSection');
    if (announcementSection) {
      announcementSection.classList.add('announcement-fade-in');
      console.log('✅ Applied announcement effect');
    }
  }

  // Update VS string directly
  updateVSString(player1Name, player2Name) {
    const nameElement = this.svg.getElementById('playerVSString');
    if (nameElement) {
      const newText = `${capitalizeName(player1Name)} VS ${capitalizeName(player2Name)}`;
      nameElement.textContent = newText;
      
      console.log(`✅ Updated VS string: ${capitalizeName(player1Name)} VS ${capitalizeName(player2Name)}`);
    }
  }





  updateElement(id, value) {
    const element = this.svg.getElementById(id);
    if (element) element.textContent = value;
  }
}

// Winner Announcement Overlay Class
class WinnerAnnouncementOverlay extends ScoreboardOverlay {
  updateWinner(name, country, seed, score) {
    this.updateElement('winnerName', name);
    this.updateElement('winnerCountry', country);
    this.updateElement('winnerSeed', `(${seed})`);
    this.updateElement('finalScore', score);
  }

  updateMatchDetails(category, type, number) {
    this.updateElement('matchWeight', category);
    this.updateElement('matchCategory', type);
    this.updateElement('matchNumber', `MATCH #${number}`);
  }

  updateElement(id, value) {
    const element = this.svg.getElementById(id);
    if (element) element.textContent = value;
  }
}

// Previous Results Overlay Class
class PreviousResultsOverlay extends ScoreboardOverlay {
  updatePlayerInfo(name, country, seed, wins, losses, winRate) {
    this.updateElement('playerName', name);
    this.updateElement('playerName2', name);
    this.updateElement('playerCountry', country);
    this.updateElement('playerSeed', `(${seed})`);
    this.updateElement('totalWins', wins);
    this.updateElement('totalLosses', losses);
    this.updateElement('winRate', `${winRate}%`);
  }

  updateMatchResult(matchNumber, opponent, result, score) {
    const resultElement = this.svg.getElementById(`match${matchNumber}Result`);
    if (resultElement) {
      resultElement.textContent = `${result} ${score}`;
      resultElement.setAttribute('fill', result === 'WIN' ? '#10b981' : '#ef4444');
    }
  }

  updateTournamentInfo(tournament, weightClass) {
    this.updateElement('tournamentName', tournament);
    this.updateElement('weightClass', weightClass);
  }

  updateElement(id, value) {
    const element = this.svg.getElementById(id);
    if (element) element.textContent = value;
  }
}

// Victory Ceremony Overlay Class
class VictoryCeremonyOverlay extends ScoreboardOverlay {
  updateCeremony(medalists, eventInfo) {
    // Update event information
    this.updateElement('eventCategory', eventInfo.category);
    this.updateElement('eventType', eventInfo.type);
    this.updateElement('ceremonyTitle', eventInfo.title);
    this.updateElement('ceremonyDate', eventInfo.date);
    
    // Update gold medalist
    if (medalists.gold) {
      this.updateElement('goldPlayerCountry', medalists.gold.country);
      this.updateElement('goldPlayerName', medalists.gold.name);
      this.updateElement('goldPlayerSeed', `(${medalists.gold.seed})`);
      this.updateElement('goldPlayerScore', `Final Score: ${medalists.gold.score}`);
    }
    
    // Update silver medalist
    if (medalists.silver) {
      this.updateElement('silverPlayerCountry', medalists.silver.country);
      this.updateElement('silverPlayerName', medalists.silver.name);
      this.updateElement('silverPlayerSeed', `(${medalists.silver.seed})`);
      this.updateElement('silverPlayerScore', `Final Score: ${medalists.silver.score}`);
    }
    
    // Update bronze medalists
    if (medalists.bronze1) {
      this.updateElement('bronze1PlayerCountry', medalists.bronze1.country);
      this.updateElement('bronze1PlayerName', medalists.bronze1.name);
      this.updateElement('bronze1PlayerSeed', `(${medalists.bronze1.seed})`);
      this.updateElement('bronze1PlayerScore', `Final Score: ${medalists.bronze1.score}`);
    }
    
    if (medalists.bronze2) {
      this.updateElement('bronze2PlayerCountry', medalists.bronze2.country);
      this.updateElement('bronze2PlayerName', medalists.bronze2.name);
      this.updateElement('bronze2PlayerSeed', `(${medalists.bronze2.seed})`);
      this.updateElement('bronze2PlayerScore', `Final Score: ${medalists.bronze2.score}`);
    }
    
    // Add ceremony animations
    this.addCeremonyAnimations();
  }

  addCeremonyAnimations() {
    // Add animation classes to medal sections
    const goldSection = this.svg.getElementById('goldSection');
    const silverSection = this.svg.getElementById('silverSection');
    const bronze1Section = this.svg.getElementById('bronze1Section');
    const bronze2Section = this.svg.getElementById('bronze2Section');
    
    if (goldSection) goldSection.classList.add('gold-section');
    if (silverSection) silverSection.classList.add('silver-section');
    if (bronze1Section) bronze1Section.classList.add('bronze-section');
    if (bronze2Section) bronze2Section.classList.add('bronze-section');
  }

  updateElement(id, value) {
    const element = this.svg.getElementById(id);
    if (element) element.textContent = value;
  }
}

// Export classes for use in other modules
if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    ScoreboardOverlay,
    PlayerIntroductionOverlay,
    WinnerAnnouncementOverlay,
    PreviousResultsOverlay,
    VictoryCeremonyOverlay
  };
} else {
  window.ScoreboardOverlay = ScoreboardOverlay;
  window.PlayerIntroductionOverlay = PlayerIntroductionOverlay;
  window.WinnerAnnouncementOverlay = WinnerAnnouncementOverlay;
  window.PreviousResultsOverlay = PreviousResultsOverlay;
  window.VictoryCeremonyOverlay = VictoryCeremonyOverlay;
} 