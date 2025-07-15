// Frontend TypeScript types for reStrike VTA

// ============================================================================
// OBS Integration Types
// ============================================================================

export interface ObsConnection {
  name: string;
  host: string;
  port: number;
  password?: string;
  protocol_version: 'v4' | 'v5';
  enabled: boolean;
  status: ObsConnectionStatus;
  error?: string;
}

export type ObsConnectionStatus = 
  | 'Disconnected' 
  | 'Connecting' 
  | 'Connected' 
  | 'Authenticating' 
  | 'Authenticated' 
  | 'Error';

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