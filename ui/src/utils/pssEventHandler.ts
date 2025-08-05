import { usePssMatchStore } from '../stores';
import { PssAthleteInfo, PssMatchConfig, PssScores, PssCurrentScores, PssWinnerRounds } from '../types';
import { getBestFlagCode } from './countryCodeMapping';

/**
 * Handle PSS events and update the match store
 * This function processes PSS events from the backend and updates the store accordingly
 * Handles both match setup data and scoring data (for future use)
 */
export const handlePssEvent = (event: any) => {
  console.log('🎯 handlePssEvent called with event:', event);
  console.log('🎯 Event type:', event.type);
  console.log('🎯 Event structure:', JSON.stringify(event, null, 2));
  
  const store = usePssMatchStore.getState();
  console.log('🎯 Current store state:', store.matchData);
  
  // Emit browser event for scoreboard overlays
  emitBrowserEvent(event);
  
  // Broadcast event to HTML overlays via WebSocket
  broadcastPssEventViaWebSocket(event);
  
  // Handle different event types based on the event structure
  switch (event.type) {
    case 'athletes':
      console.log('🎯 Handling athletes event');
      handleAthletesEvent(event, store);
      break;
    case 'match_config':
      console.log('🎯 Handling match_config event');
      handleMatchConfigEvent(event, store);
      break;
    case 'scores':
      console.log('🎯 Handling scores event');
      handleScoresEvent(event, store);
      break;
    case 'current_scores':
      console.log('🎯 Handling current_scores event');
      handleCurrentScoresEvent(event, store);
      break;
    case 'winner_rounds':
      console.log('🎯 Handling winner_rounds event');
      handleWinnerRoundsEvent(event, store);
      break;
    case 'fight_loaded':
      console.log('🎯 Handling fight_loaded event');
      handleFightLoadedEvent(event, store);
      break;
    case 'fight_ready':
      console.log('🎯 Handling fight_ready event');
      handleFightReadyEvent(event, store);
      break;
    case 'points':
      // Handle points events (for future scoring features)
      console.log('🎯 Points event:', event);
      break;
    case 'hit_level':
      // Handle hit level events (for future features)
      console.log('🎯 Hit level event:', event);
      break;
    case 'warnings':
      // Handle warnings events (for future features)
      console.log('🎯 Warnings event:', event);
      break;
    case 'clock':
      // Handle clock events (for future features)
      console.log('🎯 Clock event:', event);
      handleClockEvent(event, store);
      break;
    case 'round':
      // Handle round events (for future features)
      console.log('🎯 Round event:', event);
      handleRoundEvent(event, store);
      break;
    case 'injury':
      // Handle injury events (for future features)
      console.log('🎯 Injury event:', event);
      break;
    case 'challenge':
      // Handle challenge events (for future features)
      console.log('🎯 Challenge event:', event);
      break;
    case 'break':
      // Handle break events (for future features)
      console.log('🎯 Break event:', event);
      break;
    case 'winner':
      // Handle winner events (for future features)
      console.log('🎯 Winner event:', event);
      break;
    default:
      console.log('🎯 Unknown event type:', event.type);
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
        console.log('🎯 Parsing raw match config message:', event.message);
        // TODO: Parse raw match config message
      }
  }
  
  console.log('🎯 handlePssEvent completed');
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
      console.log('📡 Broadcasted PSS event via WebSocket:', event.type);
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
      console.log('📡 Emitted browser PSS event:', event.type);
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
    console.log('🎯 handleAthletesEvent called with:', event);
    
    let athlete1: PssAthleteInfo;
    let athlete2: PssAthleteInfo;

    // Handle new JSON structure from UDP server
    if (event.athlete1 && event.athlete2) {
      console.log('🎯 Using new JSON structure');
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
      console.log('🎯 Using legacy structure');
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

    console.log('🎯 Processed athletes:', { athlete1, athlete2 });
    store.updateAthletes(athlete1, athlete2);
    console.log('✅ Updated athletes in store');
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
    console.log('🎯 handleMatchConfigEvent called with:', event);
    
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

    console.log('🎯 Processed match config:', matchConfig);
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

    console.log('📊 Updating current scores:', currentScores);
    store.updateCurrentScores(currentScores);
    
    // Also update current round and time if available in structured data
    if (event.structured_data) {
      if (event.structured_data.current_round !== undefined) {
        store.updateCurrentRound(event.structured_data.current_round);
        console.log('📊 Updated current round from current scores event:', event.structured_data.current_round);
      }
      if (event.structured_data.current_time !== undefined) {
        store.updateCurrentRoundTime(event.structured_data.current_time);
        console.log('📊 Updated current time from current scores event:', event.structured_data.current_time);
      }
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
    console.log('🎯 Fight ready event - clearing Event Table events');
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
    // Extract round and time from the event
    const currentRound = event.round || event.current_round || 1;
    const currentTime = event.time || '0:00';
    
    // Update store with current round and time
    store.updateCurrentRound(currentRound);
    store.updateCurrentRoundTime(currentTime);
    
    console.log('📊 Updated current round and time from clock event:', { currentRound, currentTime });
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
    
    console.log('📊 Updated current round from round event:', currentRound);
  } catch (error) {
    console.error('Error handling round event:', error);
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