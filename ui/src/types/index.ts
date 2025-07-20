// Frontend TypeScript types for reStrike VTA

// ============================================================================
// OBS Integration Types
// ============================================================================

export interface ObsConnection {
  name: string;
  host: string;
  port: number;
  password?: string;
  enabled: boolean;
  status: 'Disconnected' | 'Connecting' | 'Connected' | 'Authenticating' | 'Authenticated' | 'Error';
  error?: string;
}

export interface ObsStatusInfo {
  is_recording: boolean;
  is_streaming: boolean;
  cpu_usage: number;
  recording_connection?: string;
  streaming_connection?: string;
}

// ============================================================================
// Video System Types
// ============================================================================

export interface VideoClip {
  id: string;
  name: string;
  path: string;
  duration: number;
  timestamp: Date;
  tags: string[];
  metadata?: Record<string, string>;
}

export interface VideoSettings {
  volume: number;
  playback_rate: number;
  loop_enabled: boolean;
  hardware_acceleration: boolean;
}

export interface OverlaySettings {
  opacity: number;
  position: OverlayPosition;
  scale: number;
  visible: boolean;
  theme: OverlayTheme;
}

export type OverlayPosition = 
  | 'top-left' 
  | 'top-right' 
  | 'bottom-left' 
  | 'bottom-right' 
  | 'center';

export type OverlayTheme = 'dark' | 'light' | 'transparent';

// ============================================================================
// PSS Protocol Types
// ============================================================================

export interface PssEvent {
  id: string;
  timestamp: string;
  type: PssEventType;
  player: PssPlayer;
  description: string;
  value?: string;
  raw_data?: Uint8Array;
}

export type PssEventType = 
  | 'point' 
  | 'warning' 
  | 'clock' 
  | 'round' 
  | 'score' 
  | 'athlete' 
  | 'unknown';

export type PssPlayer = 'RED' | 'BLUE' | 'YELLOW' | 'NONE';

// ============================================================================
// PSS Match Data Types
// ============================================================================

export interface PssAthleteInfo {
  short: string;
  long: string;
  country: string;
  iocCode: string;
}

export interface PssMatchConfig {
  number: number;
  category: string;
  weight: string;
  division: string;
  totalRounds: number;
  roundDuration: number;
  countdownType: string;
  format: number;
}

export interface PssScores {
  athlete1_r1: number;
  athlete2_r1: number;
  athlete1_r2: number;
  athlete2_r2: number;
  athlete1_r3: number;
  athlete2_r3: number;
}

export interface PssCurrentScores {
  athlete1_score: number;
  athlete2_score: number;
}

export interface PssWinnerRounds {
  round1_winner: number; // 0=none, 1=athlete1, 2=athlete2
  round2_winner: number;
  round3_winner: number;
}

export interface PssMatchData {
  athletes?: {
    athlete1: PssAthleteInfo;
    athlete2: PssAthleteInfo;
  };
  matchConfig?: PssMatchConfig;
  scores?: PssScores;
  currentScores?: PssCurrentScores;
  winnerRounds?: PssWinnerRounds;
  isLoaded: boolean;
  lastUpdated: string;
}

// ============================================================================
// Application State Types
// ============================================================================

export interface AppState {
  // OBS Connections
  obsConnections: ObsConnection[];
  activeObsConnection: string | null;
  obsStatus: ObsStatusInfo | null;
  
  // Overlay Settings
  overlaySettings: OverlaySettings;
  
  // Video Clips
  videoClips: VideoClip[];
  currentClip: VideoClip | null;
  isPlaying: boolean;
  
  // UI State
  currentView: AppView;
  isLoading: boolean;
  error: string | null;
}

export type AppView = 
  | 'sidebar-test' 
  | 'overlay' 
  | 'settings' 
  | 'clips' 
  | 'obs-manager';

// ============================================================================
// API Response Types
// ============================================================================

export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface TauriCommandResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
}

// ============================================================================
// Flag System Types
// ============================================================================

export interface FlagInfo {
  code: string;
  name: string;
  emoji: string;
  imagePath?: string;
}

// ============================================================================
// Constants
// ============================================================================

export const DEFAULT_OBS_PORT = 4455;
export const DEFAULT_OBS_PASSWORD = 'cekPIbj@245';
export const DEFAULT_VIDEO_VOLUME = 1.0;
export const DEFAULT_PLAYBACK_RATE = 1.0;
export const DEFAULT_OVERLAY_OPACITY = 0.9;
export const DEFAULT_OVERLAY_SCALE = 1.0; 