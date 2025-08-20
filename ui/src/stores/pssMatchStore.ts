/**
 * pssMatchStore
 * - Holds current match state and review mode flags
 */
import { create } from 'zustand';
import { PssMatchData, PssAthleteInfo, PssMatchConfig, PssScores, PssCurrentScores, PssWinnerRounds } from '../types';

interface PssMatchStore {
  // State
  matchData: PssMatchData;
  
  // Actions
  setReviewMode: (review: boolean) => void;
  updateAthletes: (athlete1: PssAthleteInfo, athlete2: PssAthleteInfo) => void;
  updateMatchConfig: (config: PssMatchConfig) => void;
  updateScores: (scores: PssScores) => void;
  updateCurrentScores: (scores: PssCurrentScores) => void;
  updateWinnerRounds: (winnerRounds: PssWinnerRounds) => void;
  updateCurrentRound: (round: number) => void;
  updateCurrentRoundTime: (time: string) => void;
  setMatchLoaded: (loaded: boolean) => void;
  resetMatchData: () => void;
  clearEvents: () => void;
  
  // Computed values
  getAthlete1: () => PssAthleteInfo | undefined;
  getAthlete2: () => PssAthleteInfo | undefined;
  getMatchNumber: () => number | undefined;
  getMatchCategory: () => string | undefined;
  getMatchWeight: () => string | undefined;
  getMatchDivision: () => string | undefined;
  getTotalScore: () => { athlete1: number; athlete2: number } | undefined;
}

const initialMatchData: PssMatchData = {
  isLoaded: false,
  isReviewMode: false,
  lastUpdated: new Date().toISOString(),
};

export const usePssMatchStore = create<PssMatchStore>((set, get) => ({
  // Initial state
  matchData: initialMatchData,
  
  // Actions
  setReviewMode: (review: boolean) => {
    set((state) => ({
      matchData: {
        ...state.matchData,
        isReviewMode: review,
        lastUpdated: new Date().toISOString(),
      },
    }));
  },
  updateAthletes: (athlete1: PssAthleteInfo, athlete2: PssAthleteInfo) => {
    console.log('🎯 updateAthletes called with:', { athlete1, athlete2 });
    set((state) => {
      const newState = {
        matchData: {
          ...state.matchData,
          athletes: { athlete1, athlete2 },
          lastUpdated: new Date().toISOString(),
        },
      };
      console.log('🎯 New store state after updateAthletes:', newState);
      return newState;
    });
  },
  
  updateMatchConfig: (config: PssMatchConfig) => {
    console.log('🎯 updateMatchConfig called with:', config);
    set((state) => {
      const newState = {
        matchData: {
          ...state.matchData,
          matchConfig: config,
          lastUpdated: new Date().toISOString(),
        },
      };
      console.log('🎯 New store state after updateMatchConfig:', newState);
      return newState;
    });
  },
  
  updateScores: (scores: PssScores) => {
    set((state) => ({
      matchData: {
        ...state.matchData,
        scores,
        lastUpdated: new Date().toISOString(),
      },
    }));
  },
  
  updateCurrentScores: (scores: PssCurrentScores) => {
    set((state) => ({
      matchData: {
        ...state.matchData,
        currentScores: scores,
        lastUpdated: new Date().toISOString(),
      },
    }));
  },
  
  updateWinnerRounds: (winnerRounds: PssWinnerRounds) => {
    set((state) => ({
      matchData: {
        ...state.matchData,
        winnerRounds,
        lastUpdated: new Date().toISOString(),
      },
    }));
  },
  
  updateCurrentRound: (round: number) => {
    console.log('🎯 updateCurrentRound called with:', round);
    set((state) => {
      const newState = {
        matchData: {
          ...state.matchData,
          currentRound: round,
          lastUpdated: new Date().toISOString(),
        },
      };
      console.log('🎯 New store state after updateCurrentRound:', newState);
      return newState;
    });
  },

  updateCurrentRoundTime: (time: string) => {
    console.log('🎯 updateCurrentRoundTime called with:', time);
    set((state) => {
      const newState = {
        matchData: {
          ...state.matchData,
          currentRoundTime: time,
          lastUpdated: new Date().toISOString(),
        },
      };
      console.log('🎯 New store state after updateCurrentRoundTime:', newState);
      return newState;
    });
  },
  
  setMatchLoaded: (loaded: boolean) => {
    set((state) => ({
      matchData: {
        ...state.matchData,
        isLoaded: loaded,
        // Turning match loaded on implies we are in live mode
        // unless explicitly set otherwise immediately after
        isReviewMode: loaded ? false : state.matchData.isReviewMode,
        lastUpdated: new Date().toISOString(),
      },
    }));
  },
  
  resetMatchData: () => {
    set({
      matchData: {
        isLoaded: false,
        isReviewMode: false,
        lastUpdated: new Date().toISOString(),
      },
    });
  },
  
  clearEvents: () => {
    // Event Table events are kept in matchData.events array if present
    set((state) => ({
      matchData: {
        ...state.matchData,
        events: [],
        lastUpdated: new Date().toISOString(),
      },
    }));
  },
  
  // Computed values
  getAthlete1: () => {
    return get().matchData.athletes?.athlete1;
  },
  
  getAthlete2: () => {
    return get().matchData.athletes?.athlete2;
  },
  
  getMatchNumber: () => {
    return get().matchData.matchConfig?.number;
  },
  
  getMatchCategory: () => {
    return get().matchData.matchConfig?.category;
  },
  
  getMatchWeight: () => {
    return get().matchData.matchConfig?.weight;
  },
  
  getMatchDivision: () => {
    return get().matchData.matchConfig?.division;
  },
  
  getTotalScore: () => {
    const currentScores = get().matchData.currentScores;
    if (currentScores) {
      return {
        athlete1: currentScores.athlete1_score,
        athlete2: currentScores.athlete2_score,
      };
    }
    return undefined;
  },
})); 