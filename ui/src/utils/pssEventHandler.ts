import { usePssMatchStore } from '../stores';
import { PssAthleteInfo, PssMatchConfig, PssScores, PssCurrentScores, PssWinnerRounds } from '../types';
import { getBestFlagCode } from './countryCodeMapping';

/**
 * Handle PSS events and update the match store
 * This function processes PSS events from the backend and updates the store accordingly
 * Handles both match setup data and scoring data (for future use)
 */
export const handlePssEvent = (event: any) => {
  const store = usePssMatchStore.getState();
  
  // Handle different event types based on the event structure
  switch (event.type) {
    case 'athletes':
      handleAthletesEvent(event, store);
      break;
    case 'match_config':
      handleMatchConfigEvent(event, store);
      break;
    case 'scores':
      handleScoresEvent(event, store);
      break;
    case 'current_scores':
      handleCurrentScoresEvent(event, store);
      break;
    case 'winner_rounds':
      handleWinnerRoundsEvent(event, store);
      break;
    case 'fight_loaded':
      handleFightLoadedEvent(event, store);
      break;
    case 'fight_ready':
      handleFightReadyEvent(event, store);
      break;
    default:
      // Handle raw events or unknown types
      if (event.event === 'FightLoaded') {
        handleFightLoadedEvent(event, store);
      } else if (event.event === 'FightReady') {
        handleFightReadyEvent(event, store);
      }
      break;
  }
};

/**
 * Handle Athletes event from PSS
 * Updates athlete information in the store
 */
const handleAthletesEvent = (event: any, store: any) => {
  try {
    // Extract athlete data from the event
    const athlete1: PssAthleteInfo = {
      short: event.athlete1_short || '',
      long: event.athlete1_long || '',
      country: event.athlete1_country || '',
      iocCode: getBestFlagCode(event.athlete1_country || ''), // Convert PSS code to IOC code
    };

    const athlete2: PssAthleteInfo = {
      short: event.athlete2_short || '',
      long: event.athlete2_long || '',
      country: event.athlete2_country || '',
      iocCode: getBestFlagCode(event.athlete2_country || ''), // Convert PSS code to IOC code
    };

    store.updateAthletes(athlete1, athlete2);
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

    store.updateMatchConfig(matchConfig);
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
    const currentScores: PssCurrentScores = {
      athlete1_score: event.athlete1_score || 0,
      athlete2_score: event.athlete2_score || 0,
    };

    store.updateCurrentScores(currentScores);
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

/**
 * Handle FightReady event from PSS
 * Indicates match is ready to start
 */
const handleFightReadyEvent = (event: any, store: any) => {
  try {
    // Could add additional logic here if needed
    console.log('Fight is ready to start');
  } catch (error) {
    console.error('Error handling fight ready event:', error);
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