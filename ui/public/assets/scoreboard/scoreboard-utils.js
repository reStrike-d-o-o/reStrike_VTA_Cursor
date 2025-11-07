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

    // Ensure we have a resolvable match info element for overlays lacking explicit id
    this.ensureMatchInfoElement();

    // Ensure injury section is hidden by default
    this.hideInjurySection();
  }

  // Try to locate the match info <text> element and assign id="matchInfo" if missing
  ensureMatchInfoElement() {
    if (this.getSvgElement('matchInfo')) return;
    try {
      const candidates = Array.from(this.svg.querySelectorAll('text'));
      // Heuristic: two or more tspans and initially contains kg/Under/Men/Women
      const target = candidates.find(el => {
        const tspans = el.querySelectorAll('tspan');
        if (tspans.length < 1) return false;
        const content = (el.textContent || '').toLowerCase();
        return /kg|under|men|women|\+|−|-/.test(content);
      });
      if (target && !target.id) {
        target.id = 'matchInfo';
      }
    } catch (_) { /* noop */ }
  }

  // Internal: try multiple candidate IDs and return the first element found
  getSvgElement(id) {
    return this.svg.getElementById(id);
  }

  // Resolve the first existing element from a list of candidate IDs
  getSvgElementAny(ids) {
    const list = Array.isArray(ids) ? ids : [ids];
    for (const cand of list) {
      if (!cand) continue;
      const el = this.svg.getElementById(cand);
      if (el) return el;
    }
    return null;
  }

  // Normalize IOC flag codes to uppercase trimmed format
  normalizeFlagCode(country) {
    return (country || '').toString().trim().toUpperCase();
  }

  // Ensure we have an <image> element sized to the provided flag container
  ensureFlagImage(flagElement) {
    if (!flagElement) return null;
    const ns = 'http://www.w3.org/2000/svg';
    const tag = (flagElement.tagName || '').toLowerCase();
    if (tag === 'image') {
      flagElement.setAttribute('data-flag', 'true');
      if (!flagElement.getAttribute('preserveAspectRatio')) {
        flagElement.setAttribute('preserveAspectRatio', 'xMidYMid meet');
      }
      flagElement.style.pointerEvents = 'none';
      return flagElement;
    }

    let imageEl = flagElement.querySelector('image[data-flag="true"]');
    if (!imageEl) {
      imageEl = document.createElementNS(ns, 'image');
      imageEl.setAttribute('data-flag', 'true');
      flagElement.appendChild(imageEl);
    }

    let x = 0;
    let y = 0;
    let width = 60;
    let height = 40;

    const rect = flagElement.querySelector('rect');
    if (rect) {
      x = parseFloat(rect.getAttribute('x')) || x;
      y = parseFloat(rect.getAttribute('y')) || y;
      width = parseFloat(rect.getAttribute('width')) || width;
      height = parseFloat(rect.getAttribute('height')) || height;
    } else if (typeof flagElement.getBBox === 'function') {
      try {
        const bb = flagElement.getBBox();
        if (bb) {
          x = Number.isFinite(bb.x) ? bb.x : x;
          y = Number.isFinite(bb.y) ? bb.y : y;
          width = Number.isFinite(bb.width) && bb.width > 0 ? bb.width : width;
          height = Number.isFinite(bb.height) && bb.height > 0 ? bb.height : height;
        }
      } catch (_) { /* ignore */ }
    }

    imageEl.setAttribute('x', String(x));
    imageEl.setAttribute('y', String(y));
    imageEl.setAttribute('width', String(Math.max(1, width)));
    imageEl.setAttribute('height', String(Math.max(1, height)));
    imageEl.setAttribute('preserveAspectRatio', 'xMidYMid meet');
    imageEl.style.pointerEvents = 'none';
    return imageEl;
  }

  // Apply flag asset to a specific element (group or image) and return the image node
  setFlagForElement(flagElement, country) {
    if (!flagElement) return null;
    const imageEl = this.ensureFlagImage(flagElement);
    if (!imageEl) return null;

    const code = this.normalizeFlagCode(country);
    if (!code) {
      imageEl.removeAttribute('href');
      try { imageEl.removeAttributeNS('http://www.w3.org/1999/xlink', 'href'); } catch (_) { /* ignore */ }
      imageEl.style.display = 'none';
      if (flagElement !== imageEl) flagElement.style.display = 'none';
      return imageEl;
    }

    const url = `/assets/flags/svg/${code}.svg`;
    imageEl.setAttribute('href', url);
    imageEl.setAttributeNS('http://www.w3.org/1999/xlink', 'xlink:href', url);
    imageEl.style.display = 'block';
    if (flagElement !== imageEl) flagElement.style.display = 'block';
    imageEl.dataset.flagCode = code;
    return imageEl;
  }

  // Convenience: set flag using candidate IDs, returning the image element when successful
  setFlagForElementCandidates(ids, country) {
    const flagElement = this.getSvgElementAny(ids);
    if (!flagElement) return null;
    return this.setFlagForElement(flagElement, country);
  }

  // Compute current rendered width for layout adjustments (falls back to width attribute)
  getFlagDisplayWidth(element) {
    if (!element) return 0;
    try {
      if (typeof element.getBBox === 'function') {
        const bbox = element.getBBox();
        if (bbox && Number.isFinite(bbox.width) && bbox.width > 0) {
          return bbox.width;
        }
      }
    } catch (_) { /* ignore */ }
    if (typeof element.getBoundingClientRect === 'function') {
      const rect = element.getBoundingClientRect();
      if (rect && Number.isFinite(rect.width) && rect.width > 0) {
        return rect.width;
      }
    }
    const attrWidth = parseFloat(element.getAttribute('width') || '');
    return Number.isFinite(attrWidth) && attrWidth > 0 ? attrWidth : 0;
  }

  // Attach a one-time load handler for dynamically swapped SVG images
  onSvgImageLoad(imageEl, handler) {
    if (!imageEl || typeof handler !== 'function') return;
    try {
      imageEl.addEventListener('load', handler, { once: true });
    } catch (_) {
      imageEl.addEventListener('load', handler);
    }
  }

  // If element is a group, set text of its first <text> child, else set its own text
  setTextForElementOrGroup(el, value) {
    if (!el) return;
    if (el.tagName && el.tagName.toLowerCase() === 'g') {
      const textChild = el.querySelector('text');
      if (textChild) {
        this.setTextForElementOrGroup(textChild, value);
        return;
      }
      el.textContent = value;
      return;
    }
    if (el.tagName && el.tagName.toLowerCase() === 'text') {
      const tspans = el.querySelectorAll('tspan');
      if (tspans.length > 0) {
        tspans[0].textContent = value;
        for (let i = 1; i < tspans.length; i += 1) {
          const span = tspans[i];
          if (span) span.textContent = '';
        }
        return;
      }
    }
    el.textContent = value;
  }

  expectsGamJeomFractionFormat() {
    if (this._expectsGamJeomFraction !== undefined) {
      return this._expectsGamJeomFraction;
    }
    const hasModernGamJeomIds = Boolean(this.getSvgElementAny(['modern_gam_jeom_player1', 'modern_gam_jeom_player2']));
    this._expectsGamJeomFraction = hasModernGamJeomIds;
    return this._expectsGamJeomFraction;
  }

  // Update player names
  updatePlayerName(player, name) {
    // Map player colors to SVG element IDs (support legacy and new schemas)
    const candidateIds = player === 'blue'
      ? ['modern_player1Name', 'modern_player1_name', 'athlete1Name', 'player1Name']
      : ['modern_player2Name', 'modern_player2_name', 'athlete2Name', 'player2Name'];
    const nameElement = this.getSvgElementAny(candidateIds);
    if (nameElement) {
      const raw = (name == null ? '' : String(name));
      // Preserve case for IOC/country codes (2-4 uppercase letters/digits)
      const display = /^[A-Z0-9]{2,4}$/.test(raw.trim()) ? raw.trim().toUpperCase() : capitalizeName(raw);
      this.setTextForElementOrGroup(nameElement, display);
      console.log(`✅ Updated ${player} player name: ${display}`);
    } else {
      console.warn(`⚠️ Could not find name element for ${player} (tried: ${candidateIds.join(', ')})`);
    }
  }

  // Preferred API using "athlete" terminology (backward compatible)
  updateAthleteName(side, name) { this.updatePlayerName(side, name); }

  // Update player scores
  updateScore(player, score) {
    // Map player colors to SVG element IDs (support legacy and new schemas)
    const candidateIds = player === 'blue'
      ? ['modern_player1Score', 'player1Score', 'athlete1Score']
      : ['modern_player2Score', 'player2Score', 'athlete2Score'];
    const elementId = candidateIds[1];
    console.log(`🎯 Updating score for ${player} player, id: ${elementId}, score: ${score}`);
    console.log(`🎯 SVG element:`, this.svg);
    const scoreElement = this.getSvgElementAny(candidateIds);
    console.log(`🎯 Found score element:`, scoreElement);

    if (scoreElement) {
      this.setTextForElementOrGroup(scoreElement, score);
      scoreElement.classList.add('score-update');
      setTimeout(() => scoreElement.classList.remove('score-update'), 500);
      console.log(`✅ Updated ${player} player score: ${score}`);
    } else {
      console.warn(`⚠️ Could not find score element for ${player} (tried: ${candidateIds.join(', ')})`);
      console.warn(`⚠️ Available elements with 'Score' in ID:`, Array.from(this.svg.querySelectorAll('[id*="Score"]')).map(el => el.id));
    }
  }

  updateAthleteScore(side, score) { this.updateScore(side, score); }

  // Update player countries (flags)
  updateCountry(player, country) {
    const code = this.normalizeFlagCode(country);
    const imageEl = this.setFlagForElementCandidates(
      player === 'blue'
        ? ['modern_player1Flag', 'modern_player1_flag', 'athlete1Flag', 'player1Flag', 'flag1', 'flag1_x5F_placeholder', 'leftPlayerFlag']
        : ['modern_player2Flag', 'modern_player2_flag', 'athlete2Flag', 'player2Flag', 'flag2', 'flag2_x5F_placeholder', 'rightPlayerFlag'],
      code
    );

    if (imageEl) {
      console.log(`✅ Updated ${player} player country flag: ${code}`);
    } else {
      console.warn(`⚠️ Could not find flag element for ${player}`);
    }

    const labelCandidates = player === 'blue'
      ? ['modern_player1IOC', 'modern_player1_ioc', 'modern_player1Country', 'player1Country', 'athlete1Country']
      : ['modern_player2IOC', 'modern_player2_ioc', 'modern_player2Country', 'player2Country', 'athlete2Country'];
    const labelElement = this.getSvgElementAny(labelCandidates);
    if (labelElement) {
      this.setTextForElementOrGroup(labelElement, code);
    }
  }

  updateAthleteFlag(side, country) {
    // Normalize to IOC uppercase
    const code = (country || '').toString().trim().toUpperCase();
    this.updateCountry(side, code);
  }

  // Update player seeds
  updateSeed(player, seed) {
    const seedElement = this.svg.getElementById(`${player}PlayerSeed`);
    if (!seedElement) return;
    const value = (seed == null || seed === '') ? '' : `(${seed})`;
    this.setTextForElementOrGroup(seedElement, value);
  }

  // Update penalties and warnings
  updatePenalties(player, penalties, warnings) {
    // Map player colors to SVG element IDs (support legacy and new schemas)
    const penaltyIdCandidates = player === 'blue'
      ? ['modern_player1Fouls', 'modern_gam_jeom_player1', 'athlete1Warnings', 'player1Fouls']
      : ['modern_player2Fouls', 'modern_gam_jeom_player2', 'athlete2Warnings', 'player2Fouls'];
    const penaltiesElement = this.getSvgElementAny(penaltyIdCandidates);
    if (penaltiesElement) {
      const rawValue = (warnings != null ? warnings : penalties);
      let numerator = 0;
      if (rawValue != null) {
        if (typeof rawValue === 'number' && Number.isFinite(rawValue)) {
          numerator = rawValue;
        } else {
          const textValue = String(rawValue).trim();
          const match = textValue.match(/^(\d+)/);
          if (match) numerator = parseInt(match[1], 10);
        }
      }
      if (!Number.isFinite(numerator)) numerator = 0;
      numerator = Math.max(0, Math.min(5, Math.floor(numerator)));
      const showFraction = this.expectsGamJeomFractionFormat();
      const displayValue = showFraction ? `${numerator}/5` : String(numerator);
      // Update text content robustly (handles <text><tspan>..</tspan></text>)
      const tspan = penaltiesElement.querySelector('tspan');
      if (tspan) tspan.textContent = displayValue;
      else this.setTextForElementOrGroup(penaltiesElement, displayValue);
      // Also update by strict IDs to avoid selector drift
      try {
        const strictIds = player === 'blue'
          ? ['modern_player1Fouls', 'player1Fouls']
          : ['modern_player2Fouls', 'player2Fouls'];
        for (const strictId of strictIds) {
          const strictEl = this.svg.getElementById(strictId);
          if (!strictEl) continue;
          this.setTextForElementOrGroup(strictEl, displayValue);
          strictEl.style.display = 'block';
        }
      } catch (_) { /* noop */ }
      // Hide/show warnings background tile and number when zero
      const sideGroup = this.getSvgElementAny(player === 'blue' ? ['player1_x5F_blue', 'athlete1Group'] : ['player2_x5F_red', 'athlete2Group']);
      if (sideGroup) {
        const warnBg = sideGroup.querySelector('rect.cls-36');
        if (warnBg) warnBg.style.display = 'block';
      }
      penaltiesElement.style.display = 'block';
      // Apply pop-out animation
      penaltiesElement.classList.add('update');
      setTimeout(() => penaltiesElement.classList.remove('update'), 500);
      console.log(`✅ Updated ${player} player warnings: ${displayValue}`);
    } else {
      console.warn(`⚠️ Could not find penalty element for ${player} (tried: ${penaltyIdCandidates.join(', ')})`);
    }
  }

  updateAthleteWarnings(side, value) { this.updatePenalties(side, null, value); }

  // Update round wins
  updateRoundWins(player, wins) {
    // Map player colors to SVG element IDs (support legacy and new schemas)
    const roundIdCandidates = player === 'blue'
      ? ['modern_player1Rounds', 'athlete1Rounds', 'player1Rounds']
      : ['modern_player2Rounds', 'athlete2Rounds', 'player2Rounds'];
    const winsElement = this.getSvgElementAny(roundIdCandidates);
    if (winsElement) {
      this.setTextForElementOrGroup(winsElement, wins || 0);
      // Apply pop-out animation
      winsElement.classList.add('update');
      setTimeout(() => winsElement.classList.remove('update'), 500);
      console.log(`✅ Updated ${player} player rounds: ${wins || 0}`);
    } else {
      console.warn(`⚠️ Could not find rounds element for ${player} (tried: ${roundIdCandidates.join(', ')})`);
    }
  }

  updateAthleteRounds(side, wins) { this.updateRoundWins(side, wins); }

  // Update match timer
  updateTimer(minutes, seconds) {
    const timerElement = this.getSvgElementAny(['modern_match_time', 'matchTimer', 'time']);
    if (timerElement) {
      this.setTextForElementOrGroup(timerElement, `${minutes}:${seconds.toString().padStart(2, '0')}`);
      console.log(`✅ Updated match timer: ${minutes}:${seconds.toString().padStart(2, '0')}`);
    } else {
      console.warn(`⚠️ Could not find match timer element (tried: modern_match_time, matchTimer, time)`);
    }
  }

  // Update current round
  updateRound(round) {
    const roundElement = this.getSvgElementAny(['modern_round_number', 'currentRound']);
    if (roundElement) {
      const isModern = roundElement.id === 'modern_round_number';
      const value = isModern ? `ROUND ${round}` : this.getOrdinalSuffix(round);
      this.setTextForElementOrGroup(roundElement, value);
      console.log(`✅ Updated current round: ${value}`);
    } else {
      console.warn(`⚠️ Could not find round element (tried: modern_round_number, currentRound)`);
    }
  }

  // Update injury time
  updateInjuryTime(time) {
    const injuryElement = this.getSvgElementAny(['modern_injury_time', 'injuryTime', 'injury_x5F_time']);
    if (injuryElement) {
      // Handle both string format ("1:00") and separate parameters (minutes, seconds)
      if (typeof time === 'string') {
        this.setTextForElementOrGroup(injuryElement, time);
        console.log(`✅ Updated injury time: ${time}`);
      } else {
        // Fallback for separate minutes/seconds parameters
        const minutes = arguments[0] || 0;
        const seconds = arguments[1] || 0;
        this.setTextForElementOrGroup(injuryElement, `${minutes}:${seconds.toString().padStart(2, '0')}`);
        console.log(`✅ Updated injury time: ${minutes}:${seconds.toString().padStart(2, '0')}`);
      }
    } else {
      console.warn(`⚠️ Could not find injury time element (tried: modern_injury_time, injuryTime, injury_x5F_time)`);
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
    const injuryElement = this.getSvgElementAny(['modern_injury_time', 'injuryTime', 'injury_x5F_time']);
    if (injuryElement) {
      this.setTextForElementOrGroup(injuryElement, '0:00');
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
      this.getSvgElementAny(['modern_player1Name', 'player1Name', 'athlete1Name']),
      this.getSvgElementAny(['modern_player2Name', 'player2Name', 'athlete2Name']),
      this.getSvgElementAny(['modern_player1Score', 'player1Score', 'athlete1Score']),
      this.getSvgElementAny(['modern_player2Score', 'player2Score', 'athlete2Score']),
      this.getSvgElementAny(['modern_round_number', 'currentRound']),
      this.getSvgElementAny(['modern_match_time', 'matchTimer', 'time'])
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

  // Abbreviate long competition phases
  abbreviateCategory(categoryRaw) {
    if (!categoryRaw) return '';
    const c = String(categoryRaw).trim().toLowerCase().replace(/[-_]+/g, ' ').replace(/\s+/g, ' ');
    if (/(bronze.*contest|bronze)/.test(c)) return 'Bronze'; // or 'BMD'
    if (/finals?$/.test(c) || c === 'final') return 'F';
    if (/(semi\s*finals?|semifinal)/.test(c)) return 'SF';
    if (/(quarter\s*finals?|quarterfinal)/.test(c)) return 'QF';
    // Generic round-of detection: "round of N", "roN", or "rN"
    const roundMatch = c.match(/(?:round of\s*(\d+))|(?:r(?:o)?\s*(\d+))/);
    if (roundMatch) {
      const n = roundMatch[1] || roundMatch[2];
      return `R of ${n}`;
    }
    if (/prelim|preliminary/.test(c)) return 'Prelim';
    return categoryRaw; // default unchanged
  }

  // Aggressive abbreviations used when text would overflow
  abbreviateCategoryStrict(categoryRaw) {
    if (!categoryRaw) return '';
    const c = String(categoryRaw).trim().toLowerCase();
    if (/(bronze.*contest|bronze)/.test(c)) return 'BMD';
    if (/finals?$/.test(c) || c === 'final') return 'F';
    if (/(semi\s*finals?|semifinal)/.test(c)) return 'SF';
    if (/(quarter\s*finals?|quarterfinal)/.test(c)) return 'QF';
    // Generic round-of detection: "round of N", "roN", or "rN"
    const roundMatch = c.match(/(?:round of\s*(\d+))|(?:r(?:o)?\s*(\d+))/);
    if (roundMatch) {
      const n = roundMatch[1] || roundMatch[2];
      return `R of ${n}`;
    }
    if (/repechage/.test(c)) return 'REP';
    return categoryRaw;
  }

  // Abbreviate common division labels (e.g., "Under 21" -> "U21")
  abbreviateDivision(divisionRaw) {
    if (!divisionRaw) return '';
    const d = String(divisionRaw).trim();
    // Normalize to lower for checks
    const dl = d.toLowerCase().replace(/[-_]+/g, ' ').replace(/\s+/g, ' ');
    const sanitized = dl.replace(/\./g, '');
    // Under N patterns
    let m = dl.match(/^under\s*(\d{1,2})$/);
    if (!m) m = dl.match(/^u\s*-?\s*(\d{1,2})$/);
    if (m) return `U${m[1]}`;
    // Explicit mappings per user rules
    if (/^sen(?:ior)?s?$/.test(sanitized)) return '';
    if (/^(junior|juniors|u18|under 18|u 18|u-18)$/.test(dl)) return 'JUN';
    if (/^(cadet|cadets|u15|under 15|u 15|u-15)$/.test(dl)) return 'CAD';
    if (/^masters?$/.test(dl) || /^veterans?$/.test(dl)) return 'MAS';
    if (/^(kids?|children)$/.test(dl)) return 'KID';
    return d;
  }

  // Normalize weight label formatting (e.g., "M -78kg" -> "M-78kg", "W +73kg" -> "W+73kg")
  normalizeWeightLabel(weightRaw) {
    if (!weightRaw) return '';
    let s = String(weightRaw).trim().replace(/\s+/g, ' ');
    // Ensure "kg" has no preceding space
    s = s.replace(/\s*kg\b/i, 'kg');
    // Remove spaces between gender and sign, and between sign and number
    s = s.replace(/^([mMwW])\s*([+-])\s*/, (match, g, sign) => `${g.toUpperCase()}${sign}`);
    s = s.replace(/([+-])\s*(\d)/, '$1$2');
    return s;
  }

  // Derive modern gender label ("Men's"/"Women's") and stripped weight value
  deriveGenderWeightLabels(weightSource, options = {}) {
    const toAscii = (value) => String(value ?? '').replace(/[’‘]/g, "'").trim();
    const rawWeight = toAscii(weightSource);
    const normalizedWeight = toAscii(options.normalizedWeight ?? rawWeight);
    const divisionText = toAscii(options.division);
    const categoryText = toAscii(options.category);

    const genderSources = [rawWeight, divisionText, categoryText, normalizedWeight];
    let genderWord = '';
    for (const src of genderSources) {
      if (!src) continue;
      const lower = src.toLowerCase();
      if (/\bwomen(?:'s)?\b/.test(lower) || /\bfemale\b/.test(lower) || /^w\b/.test(lower)) {
        genderWord = "Women's";
        break;
      }
      if (/\bmen(?:'s)?\b/.test(lower) || /\bmale\b/.test(lower) || /^m\b/.test(lower)) {
        genderWord = "Men's";
        break;
      }
    }
    if (!genderWord && rawWeight) {
      if (/^w\b/i.test(rawWeight)) genderWord = "Women's";
      else if (/^m\b/i.test(rawWeight)) genderWord = "Men's";
    }

    const weightCandidates = [rawWeight, normalizedWeight, divisionText, categoryText];
    let weightLabel = '';
    let detectedSign = '';
    let numericPortion = '';
    for (const candidate of weightCandidates) {
      if (!candidate) continue;
      const signMatch = candidate.match(/([+-])\s*(\d+(?:\.\d+)?)/);
      if (signMatch) {
        detectedSign = signMatch[1];
        numericPortion = signMatch[2];
        break;
      }
      const numericMatch = candidate.match(/(\d+(?:\.\d+)?)/);
      if (numericMatch && !numericPortion) {
        numericPortion = numericMatch[1];
      }
    }

    if (!numericPortion && normalizedWeight) {
      const cleaned = this.normalizeWeightLabel(normalizedWeight);
      const numericMatch = cleaned.match(/(\d+(?:\.\d+)?)/);
      if (numericMatch) numericPortion = numericMatch[1];
      const signMatch = cleaned.match(/^([+-])/);
      if (signMatch) detectedSign = signMatch[1];
    }

    if (!detectedSign) {
      const loweredSources = genderSources.map((src) => src.toLowerCase());
      if (loweredSources.some((src) => /\bover\b/.test(src) || /\bplus\b/.test(src))) {
        detectedSign = '+';
      } else if (loweredSources.some((src) => /\bunder\b/.test(src) || /\bminus\b/.test(src))) {
        detectedSign = '-';
      }
    }

    if (numericPortion) {
      // Drop trailing .0
      if (/\.0+$/.test(numericPortion)) {
        numericPortion = numericPortion.replace(/\.0+$/, '');
      }
      weightLabel = `${detectedSign || ''}${numericPortion}`;
    }

    const genderLabel = genderWord || '';
    return {
      genderLabel,
      weightLabel
    };
  }

  // Update combined match info (weight, division, category)
  updateMatchInfo(weight, division, category) {
    // Normalize/abbreviate
    const shortCategory = this.abbreviateCategory(category);
    const segWeight = this.normalizeWeightLabel((weight || '').trim());
    const segDivision = this.abbreviateDivision((division || '').trim());
    const segCategory = (shortCategory || '').trim();

    // Build display with separators
    const leftSegment = [segWeight, segDivision].filter(Boolean).join(' | ');
    const rightSegment = segCategory ? ` | ${segCategory}` : '';

    const rawWeight = (weight || '').trim();
    const { genderLabel, weightLabel } = this.deriveGenderWeightLabels(rawWeight, {
      normalizedWeight: segWeight,
      division,
      category
    });

    const modernGender = this.getSvgElementAny(['modern_gender']);
    const modernWeight = this.getSvgElementAny(['modern_weight']);
    if (modernGender) this.setTextForElementOrGroup(modernGender, genderLabel || '');
    if (modernWeight) this.setTextForElementOrGroup(modernWeight, weightLabel || '');

    const matchInfoElement = this.getSvgElementAny(['matchInfo']);
    if (!matchInfoElement) { console.warn('⚠️ Could not find matchInfo element'); return; }

    const tspans = matchInfoElement.querySelectorAll('tspan');
    if (tspans.length >= 2) {
      const normalizedRight = leftSegment ? rightSegment : (segCategory || '');
      tspans[0].textContent = leftSegment || '';
      tspans[1].textContent = normalizedRight;
      const cls0 = tspans[0].getAttribute('class');
      if (cls0) { tspans[1].setAttribute('class', cls0); }
    } else if (tspans.length === 1) {
      tspans[0].textContent = `${leftSegment}${rightSegment}`.trim();
    } else {
      this.setTextForElementOrGroup(matchInfoElement, `${leftSegment}${rightSegment}`.trim());
    }
    console.log('✅ Updated match info');
  }

  // Update match number (strip leading zeros)
  updateMatchNumber(num) {
    const raw = (num == null) ? '' : String(num).trim();
    const normalized = raw.replace(/^0+/, '') || '0';
    const target = this.getSvgElementAny(['modern_match_number', 'matchNumber', 'match']);
    if (target) {
      const displayValue = target.id === 'modern_match_number' ? raw : normalized;
      this.setTextForElementOrGroup(target, displayValue);
    }
  }

  // Injury helpers
  setInjuryTime(minutes, seconds) {
    const t = this.getSvgElementAny(['modern_injury_time', 'injuryTime', 'injury_x5F_time']);
    if (t) this.setTextForElementOrGroup(t, `${minutes}:${String(seconds).padStart(2, '0')}`);
  }
  setInjuryVisible(visible) {
    const t = this.getSvgElementAny(['modern_injury_time', 'injuryTime', 'injury_x5F_time']);
    const bg = this.getSvgElementAny(['modern_injury_bg', 'injury_x5F_time_x5F_bg', 'injuryBg']);
    if (t) t.style.display = visible ? 'block' : 'none';
    if (bg) bg.style.display = visible ? 'block' : 'none';
    // Toggle logo positions only by visibility (no geometry changes)
    const logoPos1 = this.getSvgElementAny(['logo_x5F_position1']);
    const logoPos2 = this.getSvgElementAny(['logo_x5F_position2']);
    if (logoPos1) logoPos1.style.display = visible ? 'block' : 'none';
    if (logoPos2) logoPos2.style.display = visible ? 'none' : 'block';
  }

  // Update match category (for backward compatibility)
  updateMatchCategory(category) {
    const matchInfoElement = this.getSvgElement('matchInfo');
    if (matchInfoElement) {
      // Get current weight and division from the element
      const currentText = matchInfoElement.textContent || '';
      const parts = currentText.split(' ');
      const weight = parts[0] || '';
      const division = parts[1] || '';
      const combinedText = `${weight} ${division} ${category || ''}`.trim();
      this.setTextForElementOrGroup(matchInfoElement, combinedText);
      console.log(`✅ Updated match category: ${category}`);
    } else {
      console.warn(`⚠️ Could not find matchInfo element`);
    }
  }

  // Update match type (weight class) - for backward compatibility
  updateMatchType(type) {
    const typeElement = this.getSvgElement('matchType');
    if (typeElement) {
      this.setTextForElementOrGroup(typeElement, type);
      console.log(`✅ Updated match type: ${type}`);
    } else {
      console.warn(`⚠️ Could not find matchType element`);
    }
  }

  // Update match weight (for backward compatibility)
  updateMatchWeight(weight) {
    const matchInfoElement = this.getSvgElement('matchInfo');
    if (matchInfoElement) {
      // Get current division and category from the element
      const currentText = matchInfoElement.textContent || '';
      const parts = currentText.split(' ');
      const division = parts[1] || '';
      const category = parts.slice(2).join(' ') || '';
      const combinedText = `${weight || ''} ${division} ${category}`.trim();
      this.setTextForElementOrGroup(matchInfoElement, combinedText);
      console.log(`✅ Updated match weight: ${weight}`);
    } else {
      console.warn(`⚠️ Could not find matchInfo element`);
    }
  }

  // Update match division (for backward compatibility)
  updateMatchDivision(division) {
    const matchInfoElement = this.getSvgElementAny(['matchInfo', 'tournament_x5F_name']);
    if (matchInfoElement) {
      // Get current weight and category from the element
      const currentText = matchInfoElement.textContent || '';
      const parts = currentText.split(' ');
      const weight = parts[0] || '';
      const category = parts.slice(2).join(' ') || '';
      const combinedText = `${weight} ${division || ''} ${category}`.trim();
      this.setTextForElementOrGroup(matchInfoElement, combinedText);
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
      return num + "st";
    }
    if (j == 2 && k != 12) {
      return num + "nd";
    }
    if (j == 3 && k != 13) {
      return num + "rd";
    }
    return num + "th";
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
    // Prefer VS string, otherwise fall back to per-side name element (new schema)
    const nameElement = this.getSvgElement(['playerVSString', 'player1_x5F_name']);
    if (nameElement) {
      const currentText = nameElement.textContent;
      if (nameElement.id === 'playerVSString') {
        const parts = currentText.split(' VS ');
        if (parts.length === 2) {
          const newText = `${capitalizeName(name)} VS ${parts[1]}`;
          nameElement.textContent = newText;
        } else {
          const newText = `${capitalizeName(name)} VS Gashim Magomedov`;
          nameElement.textContent = newText;
        }
      } else {
        nameElement.textContent = capitalizeName(name);
      }

      console.log(`✅ Updated Player 1 name: ${capitalizeName(name)}`);
    }
  }

  // Update Player 2 name in the VS string
  updatePlayer2Name(name) {
    const nameElement = this.getSvgElement(['playerVSString', 'player2_x5F_name']);
    if (nameElement) {
      const currentText = nameElement.textContent;
      if (nameElement.id === 'playerVSString') {
        const parts = currentText.split(' VS ');
        if (parts.length === 2) {
          const newText = `${parts[0]} VS ${capitalizeName(name)}`;
          nameElement.textContent = newText;
        } else {
          const newText = `Park Taejoon VS ${capitalizeName(name)}`;
          nameElement.textContent = newText;
        }
      } else {
        nameElement.textContent = capitalizeName(name);
      }

      console.log(`✅ Updated Player 2 name: ${capitalizeName(name)}`);
    }
  }

  // Update Player 1 flag
  updatePlayer1Flag(countryCode) {
    const imageEl = this.setFlagForElementCandidates(
      ['modern_player1Flag', 'leftPlayerFlag', 'flag1', 'flag1_x5F_placeholder'],
      countryCode
    );
    if (!imageEl) return;

    const adjustLeftFlag = () => {
      const flagWidth = this.getFlagDisplayWidth(imageEl);
      if (flagWidth <= 0) {
        setTimeout(adjustLeftFlag, 100);
        return;
      }
      const glassRect = this.getSvgElementAny(['modern_player1FlagGlass', 'leftPlayerFlagGlass']);
      if (glassRect) {
        glassRect.setAttribute('width', flagWidth.toString());
      }
      console.log(`✅ Updated Player 1 flag glass effect: width=${flagWidth}`);
    };

    this.onSvgImageLoad(imageEl, adjustLeftFlag);
    setTimeout(adjustLeftFlag, 75);

    console.log(`✅ Updated Player 1 flag: ${this.normalizeFlagCode(countryCode)}`);
  }

  // Update Player 2 flag
  updatePlayer2Flag(countryCode) {
    const imageEl = this.setFlagForElementCandidates(
      ['modern_player2Flag', 'rightPlayerFlag', 'flag2', 'flag2_x5F_placeholder'],
      countryCode
    );
    if (!imageEl) return;

    const adjustFlagPosition = () => {
      const flagWidth = this.getFlagDisplayWidth(imageEl);
      if (flagWidth <= 0) {
        setTimeout(adjustFlagPosition, 100);
        return;
      }

      const container = this.getSvgElementAny([
        'modern_player2Flag',
        'rightPlayerFlag',
        'flag2',
        'flag2_x5F_placeholder'
      ]);

      let anchorRight = Number.parseFloat(imageEl.dataset.anchorRight || '');
      let leftBoundary = Number.NaN;

      if (container) {
        const rect = container.querySelector('rect');
        if (rect) {
          const rectX = Number.parseFloat(rect.getAttribute('x') || '');
          const rectWidth = Number.parseFloat(rect.getAttribute('width') || '');
          if (Number.isFinite(rectX) && Number.isFinite(rectWidth)) {
            leftBoundary = rectX;
            anchorRight = rectX + rectWidth;
          }
        }
      }

      if (!Number.isFinite(anchorRight)) {
        const initialX = Number.parseFloat(imageEl.getAttribute('x') || '');
        const initialWidth = Number.parseFloat(imageEl.getAttribute('width') || '');
        const inferredRight = Number.isFinite(initialX) && Number.isFinite(initialWidth)
          ? initialX + initialWidth
          : Number.NaN;
        anchorRight = Number.isFinite(inferredRight) ? inferredRight : 1640;
      }

      imageEl.dataset.anchorRight = String(anchorRight);

      const computedLeft = anchorRight - flagWidth;
      const targetX = Number.isFinite(leftBoundary)
        ? Math.max(leftBoundary, computedLeft)
        : computedLeft;
      imageEl.setAttribute('x', targetX.toString());

      const glassRect = this.getSvgElementAny(['modern_player2FlagGlass', 'rightPlayerFlagGlass']);
      if (glassRect) {
        glassRect.setAttribute('x', targetX.toString());
        glassRect.setAttribute('width', flagWidth.toString());
      }

      console.log(`✅ Updated Player 2 flag position: x=${targetX}, width=${flagWidth}`);
    };

    this.onSvgImageLoad(imageEl, adjustFlagPosition);
    setTimeout(adjustFlagPosition, 75);

    console.log(`✅ Updated Player 2 flag: ${this.normalizeFlagCode(countryCode)}`);
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
    if (name != null) {
      this.updateElementAny(['winnerName', 'player_x5F_name', 'modern_winner_name'], name);
    }
    if (country != null) {
      const formatted = this.normalizeFlagCode(country);
      this.updateElementAny(['winnerCountry', 'country_x5F_name', 'modern_winner_ioc'], formatted);
      this.setFlagForElementCandidates(['modern_winner_flag', 'flag', 'flag_x5F_placeholder'], formatted);
    }
    if (seed != null && seed !== '') {
      this.updateElementAny(['winnerSeed', 'modern_winner_seed'], `(${seed})`);
    }
    if (score != null) {
      this.updateElementAny(['finalScore', 'modern_final_score'], score);
    }
  }

  updateMatchDetails(category, type, number) {
    if (category != null) {
      const rawCategory = String(category).trim();
      const { genderLabel, weightLabel } = this.deriveGenderWeightLabels(rawCategory, {
        normalizedWeight: rawCategory
      });
      this.updateElementAny(['matchWeight'], rawCategory);
      this.updateElementAny(['modern_gender'], genderLabel || '');
      this.updateElementAny(['modern_weight'], weightLabel || '');
    }
    if (type != null) {
      this.updateElementAny(['matchCategory'], type);
      this.updateElementAny(['modern_phase'], type);
    }
    if (number != null) {
      const rawNumber = String(number).trim();
      this.updateElementAny(['matchNumber'], `MATCH #${rawNumber}`);
      this.updateElementAny(['modern_match_number'], rawNumber);
    }
  }

  updateElementAny(ids, value) {
    const candidates = Array.isArray(ids) ? ids : [ids];
    const element = this.getSvgElementAny(candidates);
    if (!element) return;
    this.setTextForElementOrGroup(element, value);
  }

  updateElement(id, value) {
    this.updateElementAny([id], value);
  }
}

// Previous Results Overlay Class
class PreviousResultsOverlay extends ScoreboardOverlay {
  updatePlayerInfo(name, country, seed, wins, losses, winRate) {
    if (name != null) {
      this.updateElement(['modern_player_name', 'playerName', 'playerName2'], name);
      this.updateElement(['playerName2', 'playerSecondaryName'], name);
    }

    if (country != null) {
      const code = this.normalizeFlagCode(country);
      this.updateElement(['modern_player_ioc', 'playerCountry'], code);
      this.setFlagForElementCandidates(['flag1', 'flag1_x5F_placeholder', 'playerFlag'], code);
    }

    if (seed != null) {
      const seedValue = seed === '' ? '' : `(${seed})`;
      this.updateElement(['playerSeed', 'modern_player_seed'], seedValue);
    }

    if (wins != null) this.updateElement(['totalWins', 'modern_total_wins'], wins);
    if (losses != null) this.updateElement(['totalLosses', 'modern_total_losses'], losses);
    if (winRate != null) {
      const rateValue = winRate === '' ? '' : `${winRate}%`;
      this.updateElement(['winRate', 'modern_win_rate'], rateValue);
    }
  }

  updateMatchResult(matchNumber, opponent, result, score, winType) {
    const index = Number.parseInt(matchNumber, 10);
    if (!Number.isFinite(index) || index < 1) return;

    const opponentInfo = this.parseOpponent(opponent);
    const normalizedResult = (result ?? '').toString().trim();
    const normalizedScore = score == null ? '' : String(score).trim();
    const normalizedWinType = winType == null ? normalizedResult : String(winType).trim();

    const resultElement = this.getSvgElementAny([
      `modern_result${index}`,
      `match${index}Result`,
      `result${index}`
    ]);
    if (resultElement) {
      const displayResult = normalizedResult.toUpperCase();
      const shortResult = displayResult.length > 1 ? displayResult[0] : displayResult;
      this.setTextForElementOrGroup(resultElement, shortResult);
      if (displayResult) {
        const isWin = displayResult.startsWith('W');
        resultElement.setAttribute('fill', isWin ? '#10b981' : '#ef4444');
      }
    }

    this.updateElement([
      `modern_win_type${index}`,
      `winType${index}`
    ], normalizedWinType);

    this.updateElement([
      `modern_score${index}`,
      `match${index}Score`,
      `score${index}`
    ], normalizedScore);

    if (opponentInfo.name || opponentInfo.display) {
      this.updateElement([
        `modern_opponent${index}_name`,
        `opponent${index}Name`
      ], opponentInfo.name || opponentInfo.display);
    }

    this.updateElement([
      `modern_opponent${index}_ioc`,
      `opponent${index}Country`
    ], opponentInfo.code);

    const flagIndex = Math.max(2, Math.min(4, index + 1));
    this.setFlagForElementCandidates([
      `flag${flagIndex}`,
      `flag${flagIndex}_x5F_placeholder`,
      `opponent${index}Flag`
    ], opponentInfo.code);
  }

  updateTournamentInfo(tournament, weightClass) {
    if (tournament != null) {
      this.updateElement(['modern_tournament_name', 'tournamentName'], tournament);
    }
    if (weightClass != null) {
      this.updateElement(['modern_weight_class', 'weightClass'], weightClass);
    }
  }

  parseOpponent(opponent) {
    if (opponent == null) {
      return { code: '', name: '', display: '' };
    }

    if (typeof opponent === 'object') {
      const code = this.normalizeFlagCode(opponent.ioc || opponent.code || opponent.country || '');
      const name = opponent.name || opponent.fullName || opponent.displayName || '';
      const display = opponent.display || [code, name].filter(Boolean).join(code && name ? ' - ' : '') || name || code;
      return { code, name, display };
    }

    const raw = String(opponent).trim();
    let code = '';
    let name = '';
    let remainder = raw;

    const hyphenMatch = raw.match(/^\s*([A-Z]{2,3})\s*[-–|]\s*(.+)$/);
    if (hyphenMatch) {
      code = hyphenMatch[1];
      remainder = hyphenMatch[2];
    } else {
      const parenMatch = raw.match(/^\s*(.+?)\s*\(([A-Z]{2,3})\)\s*$/);
      if (parenMatch) {
        remainder = parenMatch[1];
        code = parenMatch[2];
      } else if (/^[A-Z]{2,3}$/.test(raw)) {
        code = raw;
        remainder = '';
      }
    }

    name = remainder.trim();
    if (!name && code) {
      name = raw.replace(code, '').replace(/^[-–|]+/, '').trim();
    }

    return {
      code: this.normalizeFlagCode(code),
      name,
      display: raw
    };
  }

  updateElement(ids, value) {
    const candidates = Array.isArray(ids) ? ids : [ids];
    const element = this.getSvgElementAny(candidates);
    if (!element) return;
    this.setTextForElementOrGroup(element, value == null ? '' : String(value));
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