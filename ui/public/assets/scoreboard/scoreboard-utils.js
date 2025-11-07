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

  // If element is a group, set text of its first <text> child, else set its own text
  setTextForElementOrGroup(el, value) {
    if (!el) return;
    if (el.tagName && el.tagName.toLowerCase() === 'g') {
      const textChild = el.querySelector('text');
      if (textChild) { textChild.textContent = value; return; }
    }
    el.textContent = value;
  }

  // Update player names
  updatePlayerName(player, name) {
    // Map player colors to SVG element IDs (support legacy and new schemas)
    const candidateIds = player === 'blue'
      ? ['modern_player1Name', 'athlete1Name', 'player1Name']
      : ['modern_player2Name', 'athlete2Name', 'player2Name'];
    const nameElement = this.getSvgElementAny(candidateIds);
    if (nameElement) {
      const raw = (name == null ? '' : String(name));
      // Preserve case for IOC/country codes (2-4 uppercase letters/digits)
      const display = /^[A-Z0-9]{2,4}$/.test(raw.trim()) ? raw.trim().toUpperCase() : capitalizeName(raw);
      nameElement.textContent = display;
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
    // Map player colors to SVG element IDs (support legacy and new schemas)
    const flagElement = this.getSvgElementAny(
      player === 'blue'
        ? ['modern_player1Flag', 'athlete1Flag', 'player1Flag']
        : ['modern_player2Flag', 'athlete2Flag', 'player2Flag']
    );
    if (flagElement) {
      // Add or update a single image sized to the first inner rect (stable frame)
      try {
        const r = flagElement.querySelector('rect');
        let x = 0, y = 0, w = 60, h = 40;
        if (r) {
          x = parseFloat(r.getAttribute('x')) || 0;
          y = parseFloat(r.getAttribute('y')) || 0;
          w = parseFloat(r.getAttribute('width')) || 60;
          h = parseFloat(r.getAttribute('height')) || 40;
        } else {
          const bb = flagElement.getBBox();
          x = bb.x; y = bb.y; w = bb.width; h = bb.height;
        }
        let imageEl = flagElement.querySelector('image[data-flag="true"]');
        if (!imageEl) {
          imageEl = document.createElementNS('http://www.w3.org/2000/svg', 'image');
          imageEl.setAttribute('data-flag', 'true');
          flagElement.appendChild(imageEl);
        }
        imageEl.setAttribute('x', String(x));
        imageEl.setAttribute('y', String(y));
        imageEl.setAttribute('width', String(Math.max(1, w)));
        imageEl.setAttribute('height', String(Math.max(1, h)));
        imageEl.setAttribute('preserveAspectRatio', 'xMidYMid meet');
        imageEl.style.pointerEvents = 'none';
        const url = `/assets/flags/svg/${country}.svg`;
        imageEl.setAttribute('href', url);
        imageEl.setAttributeNS('http://www.w3.org/1999/xlink', 'xlink:href', url);
      } catch (_) {
        // ignore
      }
      console.log(`✅ Updated ${player} player country flag: ${country}`);
    } else {
      console.warn(`⚠️ Could not find flag element for ${player}`);
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
    if (seedElement) seedElement.textContent = `(${seed})`;
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
      const displayValue = `${numerator}/5`;
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
          const ts = strictEl.querySelector('tspan');
          if (ts) ts.textContent = displayValue;
          else strictEl.textContent = displayValue;
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
      winsElement.textContent = wins || 0;
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
      timerElement.textContent = `${minutes}:${seconds.toString().padStart(2, '0')}`;
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
    // Under N patterns
    let m = dl.match(/^under\s*(\d{1,2})$/);
    if (!m) m = dl.match(/^u\s*-?\s*(\d{1,2})$/);
    if (m) return `U${m[1]}`;
    // Explicit mappings per user rules
    if (/^senior[s]?$/.test(dl)) return 'SEN';
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

  // Derive modern gender label ("Men's under"/"Women's over") and stripped weight value
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

    let rangeWord = '';
    if (/\+/.test(rawWeight)) rangeWord = 'over';
    else if (/-/.test(rawWeight)) rangeWord = 'under';
    else {
      for (const src of genderSources) {
        if (!src) continue;
        const lower = src.toLowerCase();
        if (/\bover\b/.test(lower)) { rangeWord = 'over'; break; }
        if (/\bunder\b/.test(lower)) { rangeWord = 'under'; break; }
      }
    }
    if (!rangeWord && genderWord) rangeWord = 'under';

    const weightCandidates = [rawWeight, normalizedWeight, divisionText, categoryText];
    let weightLabel = '';
    for (const candidate of weightCandidates) {
      if (!candidate) continue;
      const match = candidate.match(/([+-]?\s*\d+(?:\.\d+)?\s*(?:kg|kgs|kilograms|lb|lbs|pounds))/i);
      if (match) {
        weightLabel = match[0].replace(/\s+/g, ' ').trim();
        break;
      }
    }
    if (!weightLabel && normalizedWeight) {
      const cleaned = this.normalizeWeightLabel(normalizedWeight);
      weightLabel = cleaned.replace(/^[MW]\s*/i, '').replace(/^[MW](?=[+-])/, '').trim();
    }
    weightLabel = weightLabel.replace(/[’‘]/g, "'");

    const genderLabel = genderWord ? `${genderWord}${rangeWord ? ` ${rangeWord}` : ''}` : '';
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
    if (modernGender) modernGender.textContent = genderLabel || '';
    if (modernWeight) modernWeight.textContent = weightLabel ? ` ${weightLabel}` : '';

    const matchInfoElement = this.getSvgElementAny(['matchInfo']);
    if (!matchInfoElement) { console.warn('⚠️ Could not find matchInfo element'); return; }

    const tspans = matchInfoElement.querySelectorAll('tspan');
    if (tspans.length >= 2) {
      // Update texts
      tspans[0].textContent = leftSegment || '';
      tspans[1].textContent = rightSegment || '';

      // Ensure consistent styling (bold) across both tspans
      const cls0 = tspans[0].getAttribute('class');
      if (cls0) { tspans[1].setAttribute('class', cls0); }

      // Recalculate x for the second tspan to avoid overlap
      try {
        // Ensure rendering applies before measuring
        const baseX = parseFloat(tspans[0].getAttribute('x') || '0');
        // Force layout
        const bbox = tspans[0].getBBox();
        const measuredWidth = bbox ? bbox.width : 0;
        const padding = 0; // rely on leading space in second tspan (" | ...")
        const newX = isNaN(baseX) ? (bbox.x + measuredWidth + padding) : (baseX + measuredWidth + padding);
        tspans[1].setAttribute('x', String(newX));
        // Keep same y as first unless explicitly defined
        if (!tspans[1].getAttribute('y') && tspans[0].getAttribute('y')) {
          tspans[1].setAttribute('y', tspans[0].getAttribute('y'));
        }

        // Ensure combined text does not overflow its background
        const bg = this.svg.getElementById('tournament_x5F_name_x5F_bg') || this.svg.getElementById('tournamentNameBg') || this.svg.getElementById('tournament_bg') || this.svg.getElementById('BG');
        if (bg) {
          const bgBox = bg.getBBox();
          const span2Box = tspans[1].getBBox();
          const rightEdge = span2Box.x + span2Box.width;
          const allowedRight = bgBox.x + bgBox.width - 2; // small padding

          if (rightEdge > allowedRight && segCategory) {
            // Try stricter abbreviation for category (e.g., Bronze -> BMD)
            const strict = this.abbreviateCategoryStrict(segCategory);
            const newRightText = ` | ${strict}`;
            tspans[1].textContent = newRightText;
            if (cls0) { tspans[1].setAttribute('class', cls0); }
            // Re-measure after update
            const span2Box2 = tspans[1].getBBox();
            if (span2Box2.x + span2Box2.width > allowedRight) {
              // Remove separator
              const tighter = strict;
              tspans[1].textContent = ` ${tighter}`;
              if (cls0) { tspans[1].setAttribute('class', cls0); }
              const span2Box3 = tspans[1].getBBox();
              if (span2Box3.x + span2Box3.width > allowedRight) {
                // Drop category entirely as last resort
                tspans[1].textContent = '';
              }
            }
          }
        }
      } catch (_) {
        // Fallback: leave original positioning
      }
    } else if (tspans.length === 1) {
      tspans[0].textContent = `${leftSegment}${rightSegment}`.trim();
    } else {
      matchInfoElement.textContent = `${leftSegment}${rightSegment}`.trim();
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
      try {
        // Center the match number within its background rect if available
        const bg = this.getSvgElementAny(['match_x5F_bg', 'match_bg']);
        if (bg && bg.getBBox) {
          const bb = bg.getBBox();
          const centerX = bb.x + (bb.width / 2);
          // Extract current translateY from transform
          const tr = target.getAttribute('transform');
          let y = 0;
          if (tr && /translate\(([^,\s]+)[,\s]+([^)]+)\)/.test(tr)) {
            y = parseFloat(RegExp.$2) || 0;
          }
          target.setAttribute('text-anchor', 'middle');
          target.setAttribute('transform', `translate(${centerX} ${y})`);
        }
      } catch (_) { /* ignore alignment errors */ }
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
      matchInfoElement.textContent = combinedText;
      console.log(`✅ Updated match category: ${category}`);
    } else {
      console.warn(`⚠️ Could not find matchInfo element`);
    }
  }

  // Update match type (weight class) - for backward compatibility
  updateMatchType(type) {
    const typeElement = this.getSvgElement('matchType');
    if (typeElement) {
      typeElement.textContent = type;
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
      matchInfoElement.textContent = combinedText;
      console.log(`✅ Updated match weight: ${weight}`);
    } else {
      console.warn(`⚠️ Could not find matchInfo element`);
    }
  }

  // Update match division (for backward compatibility)
  updateMatchDivision(division) {
    const matchInfoElement = this.getSvgElement(['matchInfo', 'tournament_x5F_name']);
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
    const flagElement = this.getSvgElementAny(['modern_player1Flag', 'leftPlayerFlag']);
    if (flagElement) {
      flagElement.setAttribute('href', `/assets/flags/svg/${countryCode}.svg`);

      // Adjust glass effect rectangle after flag loads
      const adjustLeftFlag = () => {
        // Get the actual rendered width of the flag
        const flagRect = flagElement.getBoundingClientRect();
        const flagWidth = flagRect.width;

        if (flagWidth > 0) {
          // Update the glass effect rectangle width
          const glassRect = this.getSvgElementAny(['modern_player1FlagGlass', 'leftPlayerFlagGlass']);
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
    const flagElement = this.getSvgElementAny(['modern_player2Flag', 'rightPlayerFlag']);
    if (flagElement) {
      flagElement.setAttribute('href', `/assets/flags/svg/${countryCode}.svg`);

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
          const glassRect = this.getSvgElementAny(['modern_player2FlagGlass', 'rightPlayerFlagGlass']);
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
    if (name != null) {
      this.updateElementAny(['winnerName', 'player_x5F_name', 'modern_winner_name'], name);
    }
    if (country != null) {
      const formatted = String(country).trim().toUpperCase();
      this.updateElementAny(['winnerCountry', 'country_x5F_name', 'modern_winner_ioc'], formatted);
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
      this.updateElementAny(['modern_weight'], weightLabel ? ` ${weightLabel}` : '');
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