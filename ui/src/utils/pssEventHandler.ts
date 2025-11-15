import { usePssMatchStore } from '../stores';
import { logger } from './logger';
import { PssAthleteInfo, PssMatchConfig, PssScores, PssCurrentScores, PssWinnerRounds } from '../types';
import { getBestFlagCode } from './countryCodeMapping';
import eventBroadcaster from './eventBroadcaster';

/**
 * Handle PSS events and update the match store
 * This function processes PSS events from the backend and updates the store accordingly
 * Handles both match setup data and scoring data (for future use)
 */
export const handlePssEvent = (event: any) => {
  logger.debug('🎯 handlePssEvent', event);
  logger.debug('🎯 Event type', event.type);
  
  const store = usePssMatchStore.getState();
  logger.debug('🎯 Current store state', store.matchData);
  
  // Emit browser event for scoreboard overlays
  emitBrowserEvent(event);

  // Broadcast via localStorage so standalone overlays (served from /overlays) get updates instantly
  try {
    eventBroadcaster.broadcastPssEvent(event);
  } catch (error) {
    logger.warn('Failed to broadcast PSS event via localStorage bridge', error as Error);
  }
  
  // Avoid re-broadcasting to the WebSocket server here to prevent duplicate events in UI.
  // The backend UDP plugin already broadcasts events to the WebSocket server.
  
  // Handle different event types based on the event structure
  switch (event.type) {
    case 'athletes':
      logger.debug('🎯 Handling athletes event');
      handleAthletesEvent(event, store);
      break;
    case 'match_config':
      logger.debug('🎯 Handling match_config event');
      handleMatchConfigEvent(event, store);
      break;
    case 'scores':
      logger.debug('🎯 Handling scores event');
      handleScoresEvent(event, store);
      break;
    case 'current_scores':
      logger.debug('🎯 Handling current_scores event');
      handleCurrentScoresEvent(event, store);
      break;
    case 'winner_rounds':
      logger.debug('🎯 Handling winner_rounds event');
      handleWinnerRoundsEvent(event, store);
      break;
    case 'fight_loaded':
      logger.debug('🎯 Handling fight_loaded event');
      handleFightLoadedEvent(event, store);
      try { store.setReviewMode(false); } catch {}
      break;
    case 'fight_ready':
      logger.debug('🎯 Handling fight_ready event');
      handleFightReadyEvent(event, store);
      try { store.setReviewMode(false); } catch {}
      break;
    case 'points':
      // Handle points events (for future scoring features)
      logger.debug('🎯 Points event', event);
      break;
    case 'hit_level':
      // Handle hit level events (for future features)
      logger.debug('🎯 Hit level event', event);
      break;
    case 'warnings':
      // Handle warnings events (for future features)
      logger.debug('🎯 Warnings event', event);
      break;
    case 'clock':
      // Handle clock events (for future features)
      logger.debug('🎯 Clock event', event);
      handleClockEvent(event, store);
      break;
    case 'round':
      // Handle round events (for future features)
      logger.debug('🎯 Round event', event);
      handleRoundEvent(event, store);
      break;
    case 'injury':
      // Handle injury events (for future features)
      logger.debug('🎯 Injury event', event);
      break;
    case 'challenge':
      // Handle challenge events (for future features)
      logger.debug('🎯 Challenge event', event);
      // Toast for auto replay feedback
      try {
        if (typeof window !== 'undefined' && 'Notification' in window) {
          if (Notification.permission === 'granted') new Notification('Auto Replay triggered');
          else Notification.requestPermission();
        }
      } catch {}
      break;
    case 'break':
      // Handle break events (for future features)
      logger.debug('🎯 Break event', event);
      break;
    case 'winner':
      // Handle winner events (for future features)
      logger.debug('🎯 Winner event', event);
      try { store.setMatchLoaded(false); } catch {}
      // End of match: allow review selection window
      try { store.setReviewMode(false); } catch {}
      break;
    default:
      logger.debug('🎯 Unknown event type', event.type);
      // Handle raw events or unknown types
      if (event.event === 'FightLoaded') {
        handleFightLoadedEvent(event, store);
      } else if (event.event === 'FightReady') {
        handleFightReadyEvent(event, store);
      } else if (event.event === 'Athletes') {
        handleAthletesEvent(event, store);
      }

      // Parse raw match config (mch;) lines to update match config
      if (event.message && event.message.startsWith('mch;')) {
        logger.debug('🎯 Parsing raw match config message', event.message);
        parseRawMatchConfig(event.message, store);
      }
  }
  
  logger.debug('🎯 handlePssEvent completed');
};

/**
 * Broadcast PSS event via WebSocket to HTML overlays
 * This function sends PSS events to the Tauri WebSocket server
 */
const broadcastPssEventViaWebSocket = async (event: any) => {
  try {
    // Check if Tauri is available
    if (typeof window !== 'undefined' && window.__TAURI__) {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('websocket_broadcast_pss_event', { eventData: event });
      logger.debug('📡 Broadcasted PSS event via WebSocket', event.type);
    }
  } catch (error) {
    console.warn('Failed to broadcast PSS event via WebSocket:', error);
  }
};

/**
 * Emit browser event for scoreboard overlays
 * This function emits custom events that the HTML overlays can listen to
 */
const emitBrowserEvent = (event: any) => {
  try {
    // Check if we're in a browser environment
    if (typeof window !== 'undefined' && typeof CustomEvent !== 'undefined') {
      // Create a custom event for the browser overlays
      const browserEvent = new CustomEvent('pss-event', {
        detail: event,
        bubbles: true,
        cancelable: true
      });
      
      // Dispatch the event
      window.dispatchEvent(browserEvent);
      logger.debug('📡 Emitted browser PSS event', event.type);
    }
  } catch (error) {
    console.error('❌ Error emitting browser event:', error);
  }
};

/**
 * Handle Athletes event from PSS
 * Updates athlete information in the store
 */
const handleAthletesEvent = (event: any, store: any) => {
  try {
    logger.debug('🎯 handleAthletesEvent called with', event);
    
    let athlete1: PssAthleteInfo;
    let athlete2: PssAthleteInfo;

    // Handle new JSON structure from UDP server
    if (event.athlete1 && event.athlete2) {
      logger.debug('🎯 Using new JSON structure');
      athlete1 = {
        short: event.athlete1.short || '',
        long: event.athlete1.long || '',
        country: event.athlete1.country || '',
        iocCode: getBestFlagCode(event.athlete1.country || ''),
      };

      athlete2 = {
        short: event.athlete2.short || '',
        long: event.athlete2.long || '',
        country: event.athlete2.country || '',
        iocCode: getBestFlagCode(event.athlete2.country || ''),
      };
    } else {
      logger.debug('🎯 Using legacy structure');
      // Handle legacy structure
      athlete1 = {
        short: event.athlete1_short || '',
        long: event.athlete1_long || '',
        country: event.athlete1_country || '',
        iocCode: getBestFlagCode(event.athlete1_country || ''),
      };

      athlete2 = {
        short: event.athlete2_short || '',
        long: event.athlete2_long || '',
        country: event.athlete2_country || '',
        iocCode: getBestFlagCode(event.athlete2_country || ''),
      };
    }

    logger.debug('🎯 Processed athletes', { athlete1, athlete2 });
    store.updateAthletes(athlete1, athlete2);
    logger.debug('✅ Updated athletes in store');
  } catch (error) {
    console.error('Error handling athletes event:', error);
  }
};

/**
 * Handle MatchConfig event from PSS
 * Updates match configuration in the store
 */
const handleMatchConfigEvent = (event: any, store: any) => {
  try {
    logger.debug('🎯 handleMatchConfigEvent called with', event);
    
    const matchConfig: PssMatchConfig = {
      number: event.number || 0,
      category: event.category || '',
      weight: event.weight || '',
      division: event.division || '',
      totalRounds: event.total_rounds || 3,
      roundDuration: event.round_duration || 120,
      countdownType: event.countdown_type || '',
      format: event.format || 1,
    };

    logger.debug('🎯 Processed match config', matchConfig);
    store.updateMatchConfig(matchConfig);
    console.log('✅ Updated match config in store');
  } catch (error) {
    console.error('Error handling match config event:', error);
  }
};

/**
 * Handle Scores event from PSS
 * Updates round-by-round scores in the store
 */
const handleScoresEvent = (event: any, store: any) => {
  try {
    const scores: PssScores = {
      athlete1_r1: event.athlete1_r1 || 0,
      athlete2_r1: event.athlete2_r1 || 0,
      athlete1_r2: event.athlete1_r2 || 0,
      athlete2_r2: event.athlete2_r2 || 0,
      athlete1_r3: event.athlete1_r3 || 0,
      athlete2_r3: event.athlete2_r3 || 0,
    };

    store.updateScores(scores);
  } catch (error) {
    console.error('Error handling scores event:', error);
  }
};

/**
 * Handle CurrentScores event from PSS
 * Updates current total scores in the store
 */
const handleCurrentScoresEvent = (event: any, store: any) => {
  try {
    // Handle both flat and nested data structures
    let athlete1_score = 0;
    let athlete2_score = 0;
    
    if (event.athlete1_score !== undefined && event.athlete2_score !== undefined) {
      // Flat structure (direct access)
      athlete1_score = event.athlete1_score;
      athlete2_score = event.athlete2_score;
    } else if (event.structured_data) {
      // Nested structure (structured_data)
      athlete1_score = event.structured_data.athlete1_score || 0;
      athlete2_score = event.structured_data.athlete2_score || 0;
    } else if (event.raw_data) {
      // Raw data format (fallback parsing)
      try {
        const parts = event.raw_data.split(';');
        if (parts.length >= 4) {
          athlete1_score = parseInt(parts[1]) || 0;
          athlete2_score = parseInt(parts[3]) || 0;
        }
      } catch (error) {
        console.error('Error parsing raw_data for current scores:', error);
      }
    }
    
    const currentScores: PssCurrentScores = {
      athlete1_score,
      athlete2_score,
    };

    logger.debug('📊 Updating current scores', currentScores);
    store.updateCurrentScores(currentScores);
    
    // Do not update round from current_scores; it causes resets. Time may be updated if provided.
    if (event.structured_data && event.structured_data.current_time !== undefined) {
      store.updateCurrentRoundTime(event.structured_data.current_time);
      logger.debug('📊 Updated current time from current scores event', event.structured_data.current_time);
    }
  } catch (error) {
    console.error('Error handling current scores event:', error);
  }
};

/**
 * Handle WinnerRounds event from PSS
 * Updates round winners in the store
 */
const handleWinnerRoundsEvent = (event: any, store: any) => {
  try {
    const winnerRounds: PssWinnerRounds = {
      round1_winner: event.round1_winner || 0,
      round2_winner: event.round2_winner || 0,
      round3_winner: event.round3_winner || 0,
    };

    store.updateWinnerRounds(winnerRounds);
  } catch (error) {
    console.error('Error handling winner rounds event:', error);
  }
};

/**
 * Handle FightLoaded event from PSS
 * Sets match as loaded in the store and triggers polling for match data
 */
const handleFightLoadedEvent = (event: any, store: any) => {
  try {
    store.setMatchLoaded(true);
    // This will trigger the polling to start in usePssEvents hook
  } catch (error) {
    console.error('Error handling fight loaded event:', error);
  }
};

// Internal state to avoid duplicate logs
let lastFightReadyEmitted = false;

/**
 * Handle FightReady event from PSS
 * Indicates match is ready to start
 */
const handleFightReadyEvent = (event: any, store: any) => {
  try {
    if (lastFightReadyEmitted) return;
    lastFightReadyEmitted = true;
    
    // Clear Event Table events when fight is ready
    // This prevents events from before the match started from appearing
    logger.debug('🎯 Fight ready event - clearing Event Table events');
    store.clearEvents();
    
  } catch (error) {
    console.error('Error handling fight ready event:', error);
  }
};

/**
 * Handle Clock event from PSS
 * Updates the current round and time in the store
 */
const handleClockEvent = (event: any, store: any) => {
  try {
    // Extract time and optional round from the event
    const maybeRound = event.round ?? event.current_round;
    const currentTime = event.time || '0:00';
    
    // Only update round when the payload explicitly includes it
    if (typeof maybeRound === 'number' && Number.isFinite(maybeRound)) {
      store.updateCurrentRound(maybeRound);
    }
    // Always update time
    store.updateCurrentRoundTime(currentTime);
    
    logger.debug('📊 Updated current time from clock event', { currentTime, maybeRound });
  } catch (error) {
    console.error('Error handling clock event:', error);
  }
};

/**
 * Handle Round event from PSS
 * Updates the current round in the store
 */
const handleRoundEvent = (event: any, store: any) => {
  try {
    // Extract round from the event
    const currentRound = event.round || event.current_round || 1;
    
    // Update store with current round
    store.updateCurrentRound(currentRound);
    
    logger.debug('📊 Updated current round from round event', currentRound);
  } catch (error) {
    console.error('Error handling round event:', error);
  }
};

/**
 * Parse raw match config message from PSS protocol
 * Handles messages in format: 'mch;number;category;gender;weight;rounds;duration;countdown_type;format'
 */
const parseRawMatchConfig = (message: string, store: any) => {
  try {
    logger.debug('🎯 parseRawMatchConfig called with', message);

    // Remove the 'mch;' prefix and split by semicolon
    const parts = message.substring(4).split(';');

    if (parts.length < 8) {
      logger.warn('⚠️ Invalid match config message format - insufficient parts', { parts, message });
      return;
    }

    const [
      numberStr,
      category,
      gender,
      weight,
      roundsStr,
      durationStr,
      countdownType,
      formatStr
    ] = parts;

    // Parse numeric values
    const number = parseInt(numberStr) || 0;
    const totalRounds = parseInt(roundsStr) || 3;
    const roundDuration = parseInt(durationStr) || 120;
    const format = parseInt(formatStr) || 1;

    // Create match config object
    const matchConfig: PssMatchConfig = {
      number,
      category: `${category} ${gender}`.trim(), // Combine category and gender
      weight: weight,
      division: category, // Use category as division
      totalRounds,
      roundDuration,
      countdownType: countdownType,
      format
    };

    logger.debug('🎯 Parsed raw match config', matchConfig);
    store.updateMatchConfig(matchConfig);
    logger.info('✅ Updated match config from raw message');

  } catch (error) {
    logger.error('❌ Error parsing raw match config message', { message, error });
  }
};

/**
 * Process a batch of PSS events
 * Useful when receiving multiple events at once
 */
export const processPssEvents = (events: any[]) => {
  events.forEach(event => {
    handlePssEvent(event);
  });
}; 
