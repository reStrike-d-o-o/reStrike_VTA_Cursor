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
  currentRound?: number;
  currentRoundTime?: string; // Format: "mm:ss" or "m:ss"
  isLoaded: boolean;
  // Review mode indicates the UI is showing a historical match review
  // and should not apply live gating logic (e.g., disabling dropdown)
  isReviewMode?: boolean;
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
  message?: string;
  updated_config?: any;
}

// ============================================================================
// OpenAPI Integration Types
// ============================================================================

export type SchemaFormat = 'json' | 'yaml';

export interface SchemaValidationError {
  message: string;
  pointer?: string | null;
  line?: number | null;
  column?: number | null;
}

export interface SchemaValidationOutcome {
  format: SchemaFormat;
  valid: boolean;
  errors: SchemaValidationError[];
}

export interface OpenApiEndpoints {
  base_url: string;
  json: string;
  yaml: string;
  health: string;
}

export interface OpenApiStateResponse {
  schema: string;
  format: SchemaFormat;
  updated_at: string;
  validation: SchemaValidationOutcome;
  endpoints: OpenApiEndpoints;
}

export interface OpenApiSaveResponse {
  format: SchemaFormat;
  updated_at: string;
  validation: SchemaValidationOutcome;
}

export interface OpenApiUploadResponse {
  schema: string;
  format: SchemaFormat;
  validation: SchemaValidationOutcome;
}

export interface OpenApiValidateResponse {
  validation: SchemaValidationOutcome;
}

// ============================================================================
// Medal Ceremony Types
// ============================================================================

export type MedalType = 'gold' | 'silver' | 'bronze';

export interface MedalCeremonySummary {
  id: string;
  tournament_id?: number | null;
  name: string;
  prepared_at?: string | null;
  prepared_version: number;
  show_external: boolean;
  created_at: string;
  updated_at: string;
}

export interface MedalCeremonyRecord {
  id?: string | null;
  tournament_id?: number | null;
  name: string;
  background_path?: string | null;
  break_path?: string | null;
  animation_duration: number;
  animation_speed: number;
  photo_time: number;
  prepared_at?: string | null;
  prepared_version: number;
  show_external: boolean;
  created_at?: string | null;
  updated_at?: string | null;
}

export interface MedalCeremonyMedalist {
  id?: string | null;
  medal_type: MedalType | string;
  medal_rank: number;
  athlete_id?: number | null;
  athlete_name: string;
  athlete_short_name?: string | null;
  ioc_code?: string | null;
  flag_asset?: string | null;
  anthem_asset?: string | null;
}

export interface MedalCeremonyDivision {
  id?: string | null;
  division_id?: number | null;
  division_name: string;
  order_index: number;
  played_at?: string | null;
  medalists: MedalCeremonyMedalist[];
}

export interface MedalCeremonyDetail {
  ceremony: MedalCeremonyRecord;
  divisions: MedalCeremonyDivision[];
}

export interface MedalCeremonyPlaylistEntry {
  division_id: string;
  division_name: string;
  order_index: number;
  gold_flag_asset?: string | null;
  gold_anthem_asset?: string | null;
  played_at?: string | null;
}

export interface FlagAnimationAsset {
  id?: string | null;
  ioc_code: string;
  file_name: string;
  file_path: string;
  display_name?: string | null;
  duration_ms?: number | null;
  is_default: boolean;
  created_at?: string | null;
  updated_at?: string | null;
}

export interface AnthemAsset {
  id?: string | null;
  ioc_code: string;
  file_name: string;
  file_path: string;
  display_name?: string | null;
  duration_ms?: number | null;
  is_default: boolean;
  created_at?: string | null;
  updated_at?: string | null;
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

export interface MedalCeremonyDivisionOption {
  name: string;
  category?: string | null;
  gender?: string | null;
  weight_class?: string | null;
}

export interface MedalCeremonyAthleteOption {
  id: number;
  full_name: string;
  short_name?: string | null;
  country_code?: string | null;
  ioc_code?: string | null;
  athlete_code?: string | null;
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
