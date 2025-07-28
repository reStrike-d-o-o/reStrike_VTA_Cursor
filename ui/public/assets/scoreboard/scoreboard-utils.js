/**
 * Scoreboard Overlay Utilities
 * Dynamic update functions for taekwondo competition overlays
 */

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
      this.applyTypewriterEffect(nameElement, name);
      console.log(`✅ Updated ${player} player name: ${name}`);
    } else {
      console.warn(`⚠️ Could not find ${elementId} element`);
    }
  }

  // Update player scores
  updateScore(player, score) {
    // Map player colors to SVG element IDs
    const elementId = player === 'blue' ? 'player1Score' : 'player2Score';
    const scoreElement = this.svg.getElementById(elementId);
    if (scoreElement) {
      scoreElement.textContent = score;
      scoreElement.classList.add('score-update');
      setTimeout(() => scoreElement.classList.remove('score-update'), 500);
      console.log(`✅ Updated ${player} player score: ${score}`);
    } else {
      console.warn(`⚠️ Could not find ${elementId} element`);
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

  // Update combined match info (weight, division, category)
  updateMatchInfo(weight, division, category) {
    const matchInfoElement = this.svg.getElementById('matchInfo');
    if (matchInfoElement) {
      const combinedText = `${weight || ''} ${division || ''} ${category || ''}`.trim();
      this.applyTypewriterEffect(matchInfoElement, combinedText);
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
      this.applyTypewriterEffect(matchInfoElement, combinedText);
      console.log(`✅ Updated match category: ${category}`);
    } else {
      console.warn(`⚠️ Could not find matchInfo element`);
    }
  }

  // Update match type (weight class) - for backward compatibility
  updateMatchType(type) {
    const typeElement = this.svg.getElementById('matchType');
    if (typeElement) {
      this.applyTypewriterEffect(typeElement, type);
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
      this.applyTypewriterEffect(matchInfoElement, combinedText);
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
      this.applyTypewriterEffect(matchInfoElement, combinedText);
      console.log(`✅ Updated match division: ${division}`);
    } else {
      console.warn(`⚠️ Could not find matchInfo element`);
    }
  }

  // Apply typewriter effect to text element
  applyTypewriterEffect(element, text) {
    if (!element) return;
    
    // Clear any existing animations
    element.classList.remove('typewriter', 'typewriter-complete', 'fade-in');
    
    // Set the text content
    element.textContent = text;
    
    // Apply typewriter effect
    element.classList.add('typewriter');
    
    // After typewriter animation completes, add fade-in effect
    setTimeout(() => {
      element.classList.remove('typewriter');
      element.classList.add('typewriter-complete');
    }, 1500);
    
    // Remove all animation classes after completion
    setTimeout(() => {
      element.classList.remove('typewriter-complete', 'fade-in');
    }, 2000);
  }

  // Apply new match effect to all text elements
  applyNewMatchEffect() {
    console.log('🎬 Applying new match typewriter effect...');
    
    // Get all text elements that should have the effect
    const textElements = [
      { id: 'player1Name', text: this.svg.getElementById('player1Name')?.textContent || '' },
      { id: 'player2Name', text: this.svg.getElementById('player2Name')?.textContent || '' },
      { id: 'matchInfo', text: this.svg.getElementById('matchInfo')?.textContent || '' },
      { id: 'matchNumber', text: this.svg.getElementById('matchNumber')?.textContent || '' }
    ];
    
    // Apply staggered typewriter effects
    textElements.forEach((item, index) => {
      const element = this.svg.getElementById(item.id);
      if (element && item.text) {
        setTimeout(() => {
          this.applyTypewriterEffect(element, item.text);
        }, index * 300); // Stagger by 300ms
      }
    });
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
  setBlueLeft() {
    // Update left player to blue
    const leftSection = this.svg.getElementById('leftPlayerSection');
    const rightSection = this.svg.getElementById('rightPlayerSection');
    
    if (leftSection && rightSection) {
      // Update color marks
      const leftMark = leftSection.querySelector('rect[fill="#0066CC"]');
      const rightMark = rightSection.querySelector('rect[fill="#CC0000"]');
      
      if (leftMark && rightMark) {
        leftMark.setAttribute('fill', '#0066CC');
        rightMark.setAttribute('fill', '#CC0000');
      }
    }
  }

  setRedLeft() {
    // Update left player to red
    const leftSection = this.svg.getElementById('leftPlayerSection');
    const rightSection = this.svg.getElementById('rightPlayerSection');
    
    if (leftSection && rightSection) {
      // Update color marks
      const leftMark = leftSection.querySelector('rect[fill="#0066CC"]');
      const rightMark = rightSection.querySelector('rect[fill="#CC0000"]');
      
      if (leftMark && rightMark) {
        leftMark.setAttribute('fill', '#CC0000');
        rightMark.setAttribute('fill', '#0066CC');
      }
    }
  }

  updateLeftPlayer(name, country, seed) {
    this.updateElement('leftPlayerName', name);
    this.updateElement('leftPlayerCountry', country);
    this.updateElement('leftPlayerSeed', `(${seed})`);
  }

  updateRightPlayer(name, country, seed) {
    this.updateElement('rightPlayerName', name);
    this.updateElement('rightPlayerCountry', country);
    this.updateElement('rightPlayerSeed', `(${seed})`);
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