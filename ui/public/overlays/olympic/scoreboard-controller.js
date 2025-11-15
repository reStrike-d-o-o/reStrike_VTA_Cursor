(function () {
  const RETRY_DELAY_MS = 2000;
  const MAX_RETRIES = 5;
  const eventQueue = [];
  let scoreboardInstance = null;
  let roundDuration = 120;
  let ws = null;
  let retryCount = 0;
  let cacheAthletes = null;
  let cacheScores = null;
  let cacheWarnings = null;

  function updateConnectionStatus(connected) {
    const statusElement = document.getElementById('connection-status');
    const pssStatusElement = document.getElementById('pss-status');
    if (statusElement) {
      statusElement.classList.toggle('connected', Boolean(connected));
    }
    if (pssStatusElement) {
      pssStatusElement.textContent = connected ? 'Connected' : 'Disconnected';
      pssStatusElement.style.color = connected ? '#10b981' : '#ef4444';
    }
  }

  function setDebugValue(id, value) {
    const el = document.getElementById(id);
    if (el) {
      el.textContent = value ?? '-';
    }
  }

  function markLastUpdate() {
    setDebugValue('last-update', new Date().toLocaleTimeString());
  }

  function parseTime(value) {
    if (!value) {
      return { minutes: 0, seconds: 0 };
    }
    if (value.includes(':')) {
      const [minPart = '0', secPart = '0'] = value.split(':');
      const minutes = parseInt(minPart, 10) || 0;
      const seconds = parseInt(secPart, 10) || 0;
      return { minutes, seconds };
    }
    const totalSeconds = parseInt(value, 10) || 0;
    return {
      minutes: Math.floor(totalSeconds / 60),
      seconds: totalSeconds % 60,
    };
  }

  function determineEventType(payload) {
    if (!payload) return '';
    const candidate = payload.type || payload.event_type || payload.eventType || payload.kind;
    return typeof candidate === 'string' ? candidate.toLowerCase() : '';
  }

  function normalizeAthlete(input) {
    if (!input) return { name: '', country: '' };
    if (typeof input === 'string') return { name: input, country: '' };
    const first = input.first_name || input.firstName || '';
    const last = input.last_name || input.lastName || '';
    const joined = [first, last].filter(Boolean).join(' ').trim();
    const nameCandidates = [
      input.long,
      input.display_name,
      input.displayName,
      input.full_name,
      input.fullName,
      input.name,
      input.nickname,
      input.short,
      joined,
      input.short_name,
      input.shortName,
    ];
    const countryCandidates = [
      input.country_code,
      input.countryCode,
      input.country,
      input.ioc,
      input.iocCode,
      input.ioc_code,
      input.flag,
      input.nation,
      input.nationality,
    ];
    const name = nameCandidates.find((val) => typeof val === 'string' && val.trim().length) || '';
    const countryRaw = countryCandidates.find((val) => typeof val === 'string' && val.trim().length) || '';
    return { name, country: countryRaw.toUpperCase() };
  }

  function resolveAthleteSource(payload, primaryKey) {
    if (!payload) return null;
    if (payload[primaryKey] && typeof payload[primaryKey] === 'object') {
      return payload[primaryKey];
    }
    return {
      short: payload[`${primaryKey}_short`],
      long: payload[`${primaryKey}_long`],
      country: payload[`${primaryKey}_country`],
    };
  }

  function deriveCategoryLabels(weight) {
    if (!weight) return { genderLabel: '', weightLabel: '' };
    const trimmed = String(weight).trim();
    const match = trimmed.match(/^([MW])\s*([+-])\s*(.+)$/i);
    if (match) {
      const base = match[1].toUpperCase() === 'M' ? "Men's" : "Women's";
      const qualifier = match[2] === '+' ? ' over' : ' under';
      return {
        genderLabel: `${base}${qualifier}`,
        weightLabel: match[3].trim()
      };
    }
    return { genderLabel: '', weightLabel: trimmed };
  }

  function applyCategoryLabels(weight) {
    if (!scoreboardInstance) return;
    const { genderLabel, weightLabel } = deriveCategoryLabels(weight);
    scoreboardInstance.updateCategoryLabels(genderLabel, weightLabel);
  }

  function computeRoundWins(payload) {
    const rounds = [payload.round1_winner, payload.round2_winner, payload.round3_winner].map((val) => Number(val));
    return {
      blue: rounds.filter((val) => val === 1).length,
      red: rounds.filter((val) => val === 2).length,
    };
  }

  function handleInjuryPayload(payload) {
    if (!scoreboardInstance) return;
    const timeValue = payload.time || '';
    if (timeValue) {
      scoreboardInstance.updateInjuryTime(timeValue);
    }
    const actionValue = (payload.action || '').toLowerCase();
    if (['show', 'start', 'resume'].includes(actionValue)) {
      scoreboardInstance.showInjurySection();
    } else if (['hide', 'stop', 'reset', 'clear'].includes(actionValue)) {
      scoreboardInstance.hideInjurySection();
      scoreboardInstance.resetInjuryTime();
    }
  }

  function updateAthlete(side, snapshot) {
    if (!scoreboardInstance || !snapshot) return;
    if (snapshot.name) {
      scoreboardInstance.updatePlayerName(side, snapshot.name);
    }
    if (snapshot.country) {
      scoreboardInstance.updateCountry(side, snapshot.country);
    }
    setDebugValue(side === 'blue' ? 'debug-blue' : 'debug-red', snapshot.name || snapshot.country || '-');
  }

  function applyAthleteSnapshot(snapshot) {
    updateAthlete('blue', snapshot.blue);
    updateAthlete('red', snapshot.red);
  }

  function reapplyCachedState() {
    if (!scoreboardInstance) return;
    if (cacheAthletes) {
      applyAthleteSnapshot(cacheAthletes);
    }
    if (cacheScores) {
      scoreboardInstance.updateScore('blue', cacheScores.blue);
      scoreboardInstance.updateScore('red', cacheScores.red);
      setDebugValue('debug-blue-score', cacheScores.blue);
      setDebugValue('debug-red-score', cacheScores.red);
    }
    if (cacheWarnings) {
      scoreboardInstance.updatePenalties('blue', null, cacheWarnings.blue);
      scoreboardInstance.updatePenalties('red', null, cacheWarnings.red);
    }
  }

  function initialize() {
    if (scoreboardInstance) {
      flushQueuedEvents();
      return;
    }
    const obj = document.getElementById('scoreboard-svg');
    const doc = obj?.contentDocument || obj?.contentWindow?.document;
    const svg = doc?.querySelector('svg');
    if (!svg || typeof ScoreboardOverlay === 'undefined') {
      setTimeout(initialize, 500);
      return;
    }

    scoreboardInstance = new ScoreboardOverlay(svg);
    scoreboardInstance.setRoundFormat('label');
    scoreboardInstance.hideInjurySection();
    scoreboardInstance.resetInjuryTime();
    reapplyCachedState();
    flushQueuedEvents();
  }

  function flushQueuedEvents() {
    if (!scoreboardInstance || !eventQueue.length) return;
    while (eventQueue.length) {
      const next = eventQueue.shift();
      processEvent(next);
    }
  }

  function processEvent(payload) {
    if (!scoreboardInstance || !payload) return;
    const eventType = determineEventType(payload);

    switch (eventType) {
      case 'athletes': {
        const blue = normalizeAthlete(resolveAthleteSource(payload, 'athlete1'));
        const red = normalizeAthlete(resolveAthleteSource(payload, 'athlete2'));
        cacheAthletes = { blue, red };
        applyAthleteSnapshot(cacheAthletes);
        break;
      }
      case 'match_config': {
        if (payload.round_duration != null) {
          const parsed = parseInt(payload.round_duration, 10);
          if (!Number.isNaN(parsed)) {
            roundDuration = parsed;
          }
        }
        if (payload.number != null) {
          scoreboardInstance.updateMatchNumber(payload.number);
        }
        applyCategoryLabels(payload.weight);
        const currentRound = payload.current_round ?? 1;
        scoreboardInstance.updateRound(currentRound);
        const mins = Math.floor(roundDuration / 60);
        const secs = roundDuration % 60;
        scoreboardInstance.updateTimer(mins, secs);
        scoreboardInstance.updateScore('blue', 0);
        scoreboardInstance.updateScore('red', 0);
        scoreboardInstance.updatePenalties('blue', null, 0);
        scoreboardInstance.updatePenalties('red', null, 0);
        scoreboardInstance.updateRoundWins('blue', 0);
        scoreboardInstance.updateRoundWins('red', 0);
        scoreboardInstance.hideInjurySection();
        scoreboardInstance.resetInjuryTime();
        cacheScores = { blue: 0, red: 0 };
        cacheWarnings = { blue: 0, red: 0 };
        setDebugValue('debug-blue-score', '0');
        setDebugValue('debug-red-score', '0');
        break;
      }
      case 'current_scores':
      case 'score_update':
      case 'score': {
        const blueScore = Number(payload.athlete1_score ?? 0);
        const redScore = Number(payload.athlete2_score ?? 0);
        scoreboardInstance.updateScore('blue', blueScore);
        scoreboardInstance.updateScore('red', redScore);
        cacheScores = { blue: blueScore, red: redScore };
        setDebugValue('debug-blue-score', blueScore);
        setDebugValue('debug-red-score', redScore);
        break;
      }
      case 'warnings':
      case 'gamjeom':
      case 'gam_jeom':
      case 'wg':
      case 'penalty':
      case 'penalties':
      case 'foul':
      case 'fouls': {
        const blueWarnings = Number(payload.athlete1_warnings ?? cacheWarnings?.blue ?? 0);
        const redWarnings = Number(payload.athlete2_warnings ?? cacheWarnings?.red ?? 0);
        scoreboardInstance.updatePenalties('blue', null, blueWarnings);
        scoreboardInstance.updatePenalties('red', null, redWarnings);
        cacheWarnings = { blue: blueWarnings, red: redWarnings };
        break;
      }
      case 'injury':
      case 'injury_time':
      case 'ij': {
        handleInjuryPayload(payload);
        break;
      }
      case 'winner_rounds':
      case 'winnerrounds': {
        const wins = computeRoundWins(payload);
        scoreboardInstance.updateRoundWins('blue', wins.blue);
        scoreboardInstance.updateRoundWins('red', wins.red);
        break;
      }
      case 'clock':
      case 'timer':
      case 'match_timer':
      case 'break':
      case 'rest':
      case 'timeout': {
        const target = payload.time ?? payload.timer ?? payload.remaining ?? payload.clock;
        if (target != null) {
          const { minutes, seconds } = parseTime(String(target));
          scoreboardInstance.updateTimer(minutes, seconds);
        }
        break;
      }
      case 'round': {
        const roundNumber = payload.current_round ?? payload.round ?? payload.roundNumber;
        if (roundNumber != null) {
          scoreboardInstance.updateRound(roundNumber);
          const mins = Math.floor(roundDuration / 60);
          const secs = roundDuration % 60;
          scoreboardInstance.updateTimer(mins, secs);
        }
        break;
      }
      default:
        break;
    }
    markLastUpdate();
  }

  function handleEvent(event) {
    if (!event) return;
    const payload = event.data && typeof event.data === 'object' ? event.data : event;
    if (!payload || typeof payload !== 'object') return;
    if (!scoreboardInstance) {
      eventQueue.push(payload);
      return;
    }
    processEvent(payload);
  }

  function teardownWebSocket() {
    if (!ws) return;
    ws.onopen = null;
    ws.onclose = null;
    ws.onerror = null;
    ws.onmessage = null;
    try {
      ws.close();
    } catch (error) {
      console.warn('Failed to close WebSocket cleanly', error);
    }
    ws = null;
  }

  function resolveWebSocketUrl() {
    try {
      const params = new URLSearchParams(window.location.search);
      const explicitUrl = params.get('wsUrl');
      if (explicitUrl) {
        return explicitUrl;
      }
      const protocol = window.location.protocol;
      let host = params.get('wsHost') || window.location.hostname || '127.0.0.1';
      if (host === 'localhost' || (protocol !== 'http:' && protocol !== 'https:')) {
        host = '127.0.0.1';
      }
      const port = params.get('wsPort') || '3001';
      const wsProtocol = protocol === 'https:' ? 'wss' : 'ws';
      return `${wsProtocol}://${host}:${port}`;
    } catch (error) {
      console.warn('Failed to resolve WebSocket URL, using default', error);
      return 'ws://127.0.0.1:3001';
    }
  }

  function scheduleReconnect() {
    if (retryCount >= MAX_RETRIES) {
      return;
    }
    retryCount += 1;
    window.setTimeout(connect, RETRY_DELAY_MS);
  }

  function connect() {
    teardownWebSocket();
    const url = resolveWebSocketUrl();

    try {
      ws = new WebSocket(url);
    } catch (error) {
      console.error('Unable to create WebSocket connection', error);
      updateConnectionStatus(false);
      scheduleReconnect();
      return;
    }

    ws.onopen = () => {
      retryCount = 0;
      updateConnectionStatus(true);
    };

    ws.onerror = () => {
      updateConnectionStatus(false);
    };

    ws.onclose = () => {
      updateConnectionStatus(false);
      scheduleReconnect();
    };

    ws.onmessage = (event) => {
      try {
        const parsed = JSON.parse(event.data);
        if (!parsed) return;
        if (parsed?.type === 'connection') {
          updateConnectionStatus(Boolean(parsed.connected));
          return;
        }
        const payload = parsed?.type === 'pss_event' && parsed?.data ? parsed.data : parsed;
        handleEvent(payload);
      } catch (error) {
        console.warn('Failed to parse incoming message', error);
      }
    };
  }

  function attachFallbackListeners() {
    window.addEventListener('pss-event', (event) => {
      if (event?.detail) {
        handleEvent(event.detail);
      }
    });

    window.addEventListener('storage', (event) => {
      if (event.key !== 'pss_event' || !event.newValue) {
        return;
      }
      try {
        const payload = JSON.parse(event.newValue);
        if (!payload) return;
        if (payload?.type === 'pss_event' && payload?.data) {
          handleEvent(payload.data);
        } else if (payload?.type !== 'connection') {
          handleEvent(payload);
        }
      } catch (error) {
        console.warn('Failed to process storage event payload', error);
      }
    });
  }

  document.addEventListener('DOMContentLoaded', () => {
    const obj = document.getElementById('scoreboard-svg');
    if (window.ScoreboardFontInjector) {
      window.ScoreboardFontInjector.initForObject('#scoreboard-svg');
    }
    obj?.addEventListener('load', initialize);
    if (obj?.contentDocument) {
      initialize();
    }
    attachFallbackListeners();
    connect();
  });

  window.OlympicScoreboard = {
    handle: handleEvent,
    reconnect: connect,
    resolveWebSocketUrl,
  };
  window.ScoreboardOverlayManager = {
    handlePssEvent: handleEvent,
    updateConnectionStatus,
    scoreboardInstance: () => scoreboardInstance,
    nameManager: () => null,
  };
})();
