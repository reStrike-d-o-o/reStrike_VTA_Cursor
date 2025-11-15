(function () {
  const scoreboardCore = new window.ScoreboardCore();
  let scoreboardInstance = null;

  function setConnectionStatus(connected) {
    const indicator = document.getElementById('connection-status');
    const pssStatusElement = document.getElementById('pss-status');
    if (indicator) indicator.classList.toggle('connected', Boolean(connected));
    if (pssStatusElement) {
      pssStatusElement.textContent = connected ? 'Connected' : 'Disconnected';
      pssStatusElement.style.color = connected ? '#10b981' : '#ef4444';
    }
  }

  function initOverlay() {
    if (scoreboardInstance) return;
    const obj = document.getElementById('scoreboard-svg');
    const doc = obj?.contentDocument || obj?.contentWindow?.document;
    const svg = doc?.querySelector('svg');
    if (!svg || typeof ScoreboardOverlay === 'undefined') {
      setTimeout(initOverlay, 250);
      return;
    }

    scoreboardInstance = new ScoreboardOverlay(svg);
    scoreboardInstance.setRoundFormat('label');
    scoreboardInstance.hideInjurySection();
    scoreboardInstance.resetInjuryTime();
    applyState(scoreboardCore.getState());
  }

  function applyState(state) {
    if (!scoreboardInstance || !state) return;
    const { match, scores, rounds, warnings, clock, injury } = state;

    if (match?.number != null) {
      scoreboardInstance.updateMatchNumber(match.number);
    }

    if (match?.weight) {
      const { genderLabel, weightLabel } = deriveCategoryLabels(match.weight);
      scoreboardInstance.updateCategoryLabels(genderLabel, weightLabel);
    }

    if (match?.athletes?.blue) updateAthlete('blue', match.athletes.blue);
    if (match?.athletes?.red) updateAthlete('red', match.athletes.red);

    if (scores?.current) {
      scoreboardInstance.updateScore('blue', scores.current.blue ?? 0);
      scoreboardInstance.updateScore('red', scores.current.red ?? 0);
    }

    if (warnings) {
      scoreboardInstance.updatePenalties('blue', null, warnings.blue ?? 0);
      scoreboardInstance.updatePenalties('red', null, warnings.red ?? 0);
    }

    if (rounds) {
      scoreboardInstance.updateRound(rounds.current ?? 1);
      const blueWins = (rounds.winners || []).filter((w) => w === 1).length;
      const redWins = (rounds.winners || []).filter((w) => w === 2).length;
      scoreboardInstance.updateRoundWins('blue', blueWins);
      scoreboardInstance.updateRoundWins('red', redWins);
    }

    if (clock?.time) {
      const { minutes, seconds } = splitTime(clock.time);
      scoreboardInstance.updateTimer(minutes, seconds);
    }

    if (injury) {
      if (injury.time) scoreboardInstance.updateInjuryTime(injury.time);
      if (injury.visible) scoreboardInstance.showInjurySection();
      else scoreboardInstance.hideInjurySection();
    }
  }

  function updateDebugPanel(state) {
    const blueName = state.match?.athletes?.blue?.short || state.match?.athletes?.blue?.long || '-';
    const redName = state.match?.athletes?.red?.short || state.match?.athletes?.red?.long || '-';
    const blueScore = state.scores?.current?.blue ?? 0;
    const redScore = state.scores?.current?.red ?? 0;

    const lastUpdateEl = document.getElementById('last-update');
    const blueEl = document.getElementById('debug-blue');
    const redEl = document.getElementById('debug-red');
    const blueScoreEl = document.getElementById('debug-blue-score');
    const redScoreEl = document.getElementById('debug-red-score');

    if (lastUpdateEl) {
      lastUpdateEl.textContent = state.metadata?.lastEventTs
        ? new Date(state.metadata.lastEventTs).toLocaleTimeString()
        : 'Never';
    }
    if (blueEl) blueEl.textContent = blueName;
    if (redEl) redEl.textContent = redName;
    if (blueScoreEl) blueScoreEl.textContent = String(blueScore);
    if (redScoreEl) redScoreEl.textContent = String(redScore);
  }

  function updateAthlete(side, info) {
    const name = info.short || info.long || '';
    if (name) scoreboardInstance.updatePlayerName(side, name);
    if (info.country) scoreboardInstance.updateCountry(side, info.country);
  }

  function splitTime(value) {
    if (!value) return { minutes: 0, seconds: 0 };
    if (value.includes(':')) {
      const [m = '0', s = '0'] = value.split(':');
      return {
        minutes: parseInt(m, 10) || 0,
        seconds: parseInt(s, 10) || 0,
      };
    }
    const total = parseInt(value, 10) || 0;
    return { minutes: Math.floor(total / 60), seconds: total % 60 };
  }

  function deriveCategoryLabels(weight) {
    const trimmed = String(weight).trim();
    const match = trimmed.match(/^([MW])\s*([+-])\s*(.+)$/i);
    if (match) {
      const base = match[1].toUpperCase() === 'M' ? "Men's" : "Women's";
      const qualifier = match[2] === '+' ? ' over' : ' under';
      return {
        genderLabel: `${base}${qualifier}`,
        weightLabel: match[3].trim(),
      };
    }
    return { genderLabel: '', weightLabel: trimmed };
  }

  scoreboardCore.on('state', (state) => {
    applyState(state);
    updateDebugPanel(state);
  });
  scoreboardCore.on('connection', ({ connected }) => setConnectionStatus(connected));

  document.addEventListener('DOMContentLoaded', () => {
    const obj = document.getElementById('scoreboard-svg');
    if (window.ScoreboardFontInjector) {
      window.ScoreboardFontInjector.initForObject('#scoreboard-svg');
    }
    obj?.addEventListener('load', initOverlay);
    if (obj?.contentDocument) {
      initOverlay();
    }
  });

  window.OlympicScoreboard = {
    core: scoreboardCore,
    overlay: () => scoreboardInstance,
  };
})();
