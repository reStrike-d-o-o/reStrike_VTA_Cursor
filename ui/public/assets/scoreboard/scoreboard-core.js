(() => {
  const cloneState = (value) =>
    typeof structuredClone === 'function'
      ? structuredClone(value)
      : JSON.parse(JSON.stringify(value));

  const DEFAULT_STATE = {
    match: {},
    clock: { time: '0:00' },
    injury: { visible: false, athlete: 0, time: '0:00' },
    scores: {
      current: { blue: 0, red: 0 },
      byRound: { blue: [0, 0, 0], red: [0, 0, 0] },
    },
    rounds: { current: 1, winners: [0, 0, 0] },
    warnings: { blue: 0, red: 0 },
    metadata: { fightLoaded: false, fightReady: false },
  };

  class ScoreboardCore {
    constructor(options = {}) {
      this.ws = null;
      this.state = cloneState(DEFAULT_STATE);
      this.listeners = new Map();
      this.retryCount = 0;
      this.maxRetries = options.maxRetries ?? 5;
      this.retryDelayMs = options.retryDelayMs ?? 2000;
      this.resolveUrl = options.resolveUrl ?? this.defaultUrlResolver;
      // Track previous warnings so we can derive Gam-jeom points when needed
      this.prevWarnings = { blue: 0, red: 0 };
      this.connect();
      this.attachFallbackListeners();
    }

    defaultUrlResolver() {
      try {
        const params = new URLSearchParams(window.location.search);
        const explicit = params.get('wsUrl');
        if (explicit) return explicit;
        const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
        const host = params.get('wsHost') || window.location.hostname || '127.0.0.1';
        const port = params.get('wsPort') || '3001';
        return `${protocol}://${host}:${port}`;
      } catch (err) {
        console.warn('ScoreboardCore: failed to resolve WS URL, using default', err);
        return 'ws://127.0.0.1:3001';
      }
    }

    connect() {
      const url = this.resolveUrl();
      try {
        this.ws = new WebSocket(url);
      } catch (err) {
        console.error('ScoreboardCore: WebSocket creation failed', err);
        this.scheduleReconnect();
        return;
      }

      this.ws.onopen = () => {
        this.retryCount = 0;
        this.emit('connection', { connected: true });
      };

      this.ws.onerror = () => {
        this.emit('connection', { connected: false });
      };

      this.ws.onclose = () => {
        this.emit('connection', { connected: false });
        this.scheduleReconnect();
      };

      this.ws.onmessage = (event) => {
        try {
          const payload = JSON.parse(event.data);
          this.handlePayload(payload);
        } catch (err) {
          console.warn('ScoreboardCore: failed to parse message', err);
        }
      };
    }

    scheduleReconnect() {
      if (this.retryCount >= this.maxRetries) return;
      this.retryCount += 1;
      setTimeout(() => this.connect(), this.retryDelayMs);
    }

    attachFallbackListeners() {
      window.addEventListener('pss-event', (evt) => {
        if (evt?.detail) this.handlePayload(evt.detail);
      });

      window.addEventListener('storage', (evt) => {
        if (evt.key !== 'pss_event' || !evt.newValue) return;
        try {
          const payload = JSON.parse(evt.newValue);
          if (payload?.type === 'pss_event' && payload?.data) {
            this.handlePayload(payload.data);
          } else if (payload) {
            this.handlePayload(payload);
          }
        } catch (err) {
          console.warn('ScoreboardCore: storage event parse failed', err);
        }
      });
    }

    on(event, handler) {
      if (!this.listeners.has(event)) {
        this.listeners.set(event, new Set());
      }
      this.listeners.get(event).add(handler);
      return () => this.off(event, handler);
    }

    off(event, handler) {
      const set = this.listeners.get(event);
      if (!set) return;
      set.delete(handler);
    }

    emit(event, payload) {
      const set = this.listeners.get(event);
      if (!set) return;
      for (const handler of set) {
        try {
          handler(payload);
        } catch (err) {
          console.error(`ScoreboardCore listener for ${event} failed`, err);
        }
      }
    }

    getState() {
      return cloneState(this.state);
    }

    resetMatchState() {
      this.state = cloneState(DEFAULT_STATE);
      this.emit('state', this.getState());
    }

    handlePayload(payload) {
      if (!payload) return;
      const eventPayload = payload.type === 'pss_event' ? payload.data : payload;
      if (!eventPayload || typeof eventPayload !== 'object') return;
      this.processEvent(this.mergeStructuredData(eventPayload));
    }

    mergeStructuredData(event) {
      if (!event || typeof event !== 'object') return event;

      let merged = event;

      // First, merge explicit structured payloads if present
      const structured =
        event.structured_data ||
        event.structuredData ||
        event.data ||
        null;
      if (structured && typeof structured === 'object') {
        merged = { ...structured, ...merged };
      }

      // Some sources embed the useful fields inside raw_data as a JSON string
      const raw = event.raw_data || event.rawData;
      if (typeof raw === 'string') {
        const trimmed = raw.trim();
        if (trimmed.startsWith('{') && trimmed.endsWith('}')) {
          try {
            const parsed = JSON.parse(trimmed);
            if (parsed && typeof parsed === 'object') {
              merged = { ...parsed, ...merged };
            }
          } catch (_) {
            // Ignore parse failures and keep merged as-is
          }
        }
      }

      return merged;
    }

    processEvent(event) {
      const type = (event.type || event.event_type || event.eventType || '').toLowerCase();
      const timestamp = event.timestamp || Date.now();
      switch (type) {
        case 'match_config':
          this.handleMatchConfig(event);
          break;
        case 'athletes':
          this.handleAthletes(event);
          break;
        case 'clock':
          this.updateClock(event.time, event.action);
          break;
        case 'round': {
          const nextRound = Number(event.current_round || event.round || 1);
          const prevRound = Number(this.state.rounds.current || 0);
          if (nextRound !== prevRound) {
            this.state.rounds.current = nextRound;
            // Reset per-round scores and warnings; they do not carry over.
            this.state.scores.current.blue = 0;
            this.state.scores.current.red = 0;
            this.state.warnings.blue = 0;
            this.state.warnings.red = 0;
            this.prevWarnings = { blue: 0, red: 0 };
          } else {
            this.state.rounds.current = nextRound;
          }
          break;
        }
        case 'winner_rounds':
          this.updateWinnerRounds(event);
          break;
        case 'current_scores':
        case 'score':
          this.state.scores.current.blue = Number(event.athlete1_score ?? this.state.scores.current.blue);
          this.state.scores.current.red = Number(event.athlete2_score ?? this.state.scores.current.red);
          break;
        case 'scores':
          this.updateRoundScores(event);
          break;
        case 'warnings': {
          const prevBlue = this.prevWarnings?.blue ?? 0;
          const prevRed = this.prevWarnings?.red ?? 0;
          this.updateWarnings(event);
          const nextBlue = this.state.warnings.blue || 0;
          const nextRed = this.state.warnings.red || 0;
          const deltaBlue = nextBlue - prevBlue;
          const deltaRed = nextRed - prevRed;
          // Each additional warning for one athlete gives +1 to the opponent.
          if (deltaBlue > 0) {
            this.state.scores.current.red =
              Number(this.state.scores.current.red || 0) + deltaBlue;
          }
          if (deltaRed > 0) {
            this.state.scores.current.blue =
              Number(this.state.scores.current.blue || 0) + deltaRed;
          }
          this.prevWarnings = { blue: nextBlue, red: nextRed };
          break;
        }
        case 'injury':
          this.updateInjury(event);
          break;
        case 'fight_loaded':
          this.state.metadata.fightLoaded = true;
          this.resetForNewFight();
          break;
        case 'fight_ready':
          this.state.metadata.fightReady = true;
          break;
        case 'supremacy':
          this.state.metadata.supremacy = Number(event.value ?? this.state.metadata.supremacy ?? 0);
          break;
        case 'video_time':
          this.state.metadata.videoTime = Number(event.value ?? this.state.metadata.videoTime ?? 0);
          break;
        default:
          break;
      }
      this.state.metadata.lastEventTs = timestamp;
      this.emit('state', this.getState());
    }

    resetForNewFight() {
      this.state.clock = { time: '0:00' };
      this.state.injury = { visible: false, athlete: 0, time: '0:00' };
      this.state.scores = cloneState(DEFAULT_STATE.scores);
      this.state.rounds = cloneState(DEFAULT_STATE.rounds);
      this.state.warnings = cloneState(DEFAULT_STATE.warnings);
    }

    handleMatchConfig(event) {
      this.resetForNewFight();
      this.state.match.number = String(event.number ?? '');
      this.state.match.category = event.category ?? '';
      this.state.match.weight = event.weight ?? '';
      this.state.match.division = event.division ?? '';
      this.state.match.format = Number(event.format ?? this.state.match.format ?? 0);
      this.state.match.colors = {
        blueBg: event.colors?.[0] ?? event.color1_bg ?? '#0000ff',
        blueFg: event.colors?.[1] ?? event.color1_fg ?? '#ffffff',
        redBg: event.colors?.[2] ?? event.color2_bg ?? '#ff0000',
        redFg: event.colors?.[3] ?? event.color2_fg ?? '#ffffff',
      };
      if (event.round_duration) {
        this.state.metadata.roundDuration = Number(event.round_duration);
      }
    }

    handleAthletes(event) {
      this.state.match.athletes = this.state.match.athletes || { blue: {}, red: {} };
      this.state.match.athletes.blue = {
        short: event.athlete1_short ?? event?.athlete1?.short,
        long: event.athlete1_long ?? event?.athlete1?.long,
        country: (event.athlete1_country ?? event?.athlete1?.country ?? '').toUpperCase(),
      };
      this.state.match.athletes.red = {
        short: event.athlete2_short ?? event?.athlete2?.short,
        long: event.athlete2_long ?? event?.athlete2?.long,
        country: (event.athlete2_country ?? event?.athlete2?.country ?? '').toUpperCase(),
      };
    }

    updateClock(time, action) {
      if (typeof time === 'string') {
        this.state.clock.time = time;
      }
      if (action) {
        this.state.clock.lastAction = action;
      }
    }

    updateWinnerRounds(event) {
      const winners = [
        Number(event.round1_winner ?? 0),
        Number(event.round2_winner ?? 0),
        Number(event.round3_winner ?? 0),
      ];
      this.state.rounds.winners = winners;
    }

    updateRoundScores(event) {
      const rounds = this.state.scores.byRound;
      if (event.athlete1_r1 !== undefined) rounds.blue[0] = Number(event.athlete1_r1);
      if (event.athlete2_r1 !== undefined) rounds.red[0] = Number(event.athlete2_r1);
      if (event.athlete1_r2 !== undefined) rounds.blue[1] = Number(event.athlete1_r2);
      if (event.athlete2_r2 !== undefined) rounds.red[1] = Number(event.athlete2_r2);
      if (event.athlete1_r3 !== undefined) rounds.blue[2] = Number(event.athlete1_r3);
      if (event.athlete2_r3 !== undefined) rounds.red[2] = Number(event.athlete2_r3);
    }

    updateWarnings(event) {
      const blueValue =
        event.athlete1_warnings ??
        event.athlete1Warnings ??
        event.blue ??
        event.blueWarnings ??
        event.warnings_blue ??
        event.warningsBlue;
      const redValue =
        event.athlete2_warnings ??
        event.athlete2Warnings ??
        event.red ??
        event.redWarnings ??
        event.warnings_red ??
        event.warningsRed;

      if (blueValue !== undefined) {
        this.state.warnings.blue = Number(blueValue);
      }
      if (redValue !== undefined) {
        this.state.warnings.red = Number(redValue);
      }
    }

    updateInjury(event) {
      if (event.athlete !== undefined) {
        this.state.injury.athlete = Number(event.athlete);
      }
      if (event.time) {
        this.state.injury.time = event.time;
      }
      const action = (event.action || '').toLowerCase();
      if (['show', 'start', 'resume'].includes(action)) {
        this.state.injury.visible = true;
      } else if (['hide', 'stop', 'reset'].includes(action)) {
        this.state.injury.visible = false;
      }
    }
  }

  window.ScoreboardCore = ScoreboardCore;
})();
