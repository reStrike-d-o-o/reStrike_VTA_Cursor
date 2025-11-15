export type ScoreboardEventType =
  | 'points'
  | 'hit_level'
  | 'warnings'
  | 'injury'
  | 'challenge'
  | 'break'
  | 'winner_rounds'
  | 'winner'
  | 'athletes'
  | 'match_config'
  | 'scores'
  | 'current_scores'
  | 'clock'
  | 'round'
  | 'fight_loaded'
  | 'fight_ready'
  | 'supremacy'
  | 'video_time'
  | 'raw';

export interface PssEventPayload {
  type: ScoreboardEventType;
  timestamp: number;
  raw?: string;
  data: Record<string, unknown>;
}

export interface ScoreboardState {
  match: MatchInfo;
  clock: ClockState;
  injury: InjuryState;
  scores: ScoreState;
  rounds: RoundState;
  warnings: WarningState;
  metadata: MetadataState;
}

export interface MatchInfo {
  number?: string;
  category?: string;
  weight?: string;
  division?: string;
  format?: number;
  colors?: {
    blueBg: string;
    blueFg: string;
    redBg: string;
    redFg: string;
  };
  athletes?: {
    blue: AthleteInfo;
    red: AthleteInfo;
  };
}

export interface AthleteInfo {
  short?: string;
  long?: string;
  country?: string;
}

export interface ClockState {
  time: string;
  lastAction?: 'start' | 'stop';
}

export interface InjuryState {
  visible: boolean;
  athlete: 0 | 1 | 2;
  time: string;
}

export interface ScoreState {
  current: {
    blue: number;
    red: number;
  };
  byRound: {
    blue: [number, number, number];
    red: [number, number, number];
  };
}

export interface RoundState {
  current: number;
  winners: [0 | 1 | 2, 0 | 1 | 2, 0 | 1 | 2];
}

export interface WarningState {
  blue: number;
  red: number;
}

export interface MetadataState {
  fightLoaded: boolean;
  fightReady: boolean;
  supremacy?: number;
  videoTime?: number;
  lastEventTs?: number;
}

export const defaultScoreboardState: ScoreboardState = {
  match: {},
  clock: { time: '0:00' },
  injury: { visible: false, athlete: 0, time: '0:00' },
  scores: {
    current: { blue: 0, red: 0 },
    byRound: {
      blue: [0, 0, 0],
      red: [0, 0, 0],
    },
  },
  rounds: { current: 1, winners: [0, 0, 0] },
  warnings: { blue: 0, red: 0 },
  metadata: { fightLoaded: false, fightReady: false },
};
