import { create } from 'zustand';
import { PssMatchData, PssAthleteInfo, PssMatchConfig, PssScores, PssCurrentScores, PssWinnerRounds } from '../types';

interface PssMatchStore {
  // State
  matchData: PssMatchData;
  
  // Actions
  updateAthletes: (athlete1: PssAthleteInfo, athlete2: PssAthleteInfo) => void;
  updateMatchConfig: (config: PssMatchConfig) => void;
  updateScores: (scores: PssScores) => void;
  updateCurrentScores: (scores: PssCurrentScores) => void;
  updateWinnerRounds: (winnerRounds: PssWinnerRounds) => void;
  setMatchLoaded: (loaded: boolean) => void;
  resetMatchData: () => void;
  
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
  lastUpdated: new Date().toISOString(),
};

export const usePssMatchStore = create<PssMatchStore>((set, get) => ({
  // Initial state
  matchData: initialMatchData,
  
  // Actions
  updateAthletes: (athlete1: PssAthleteInfo, athlete2: PssAthleteInfo) => {
    set((state) => ({
      matchData: {
        ...state.matchData,
        athletes: { athlete1, athlete2 },
        lastUpdated: new Date().toISOString(),
      },
    }));
  },
  
  updateMatchConfig: (config: PssMatchConfig) => {
    set((state) => ({
      matchData: {
        ...state.matchData,
        matchConfig: config,
        lastUpdated: new Date().toISOString(),
      },
    }));
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
  
  setMatchLoaded: (loaded: boolean) => {
    set((state) => ({
      matchData: {
        ...state.matchData,
        isLoaded: loaded,
        lastUpdated: new Date().toISOString(),
      },
    }));
  },
  
  resetMatchData: () => {
    set({ matchData: initialMatchData });
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