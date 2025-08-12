// Tauri command utilities for OBS obws integration

import { TauriCommandResponse } from '../types';

// Tauri v2 invoke function that uses the core module
const safeInvoke = async (command: string, args?: any) => {
  try {
    console.log('🔍 obws safeInvoke called with:', { command, args });
    
    // Check if the global Tauri object is available
    if (typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.core) {
      console.log('🔍 Tauri v2 core module found, calling invoke');
      // In Tauri v2, invoke is available through the core module
      const result = await window.__TAURI__.core.invoke(command, args);
      console.log('🔍 Tauri invoke result:', result);
      return result;
    }
    
    console.log('🔍 Tauri v2 core module not available');
    throw new Error('Tauri v2 core module not available - ensure app is running in desktop mode');
  } catch (error) {
    console.error('🔍 Tauri invoke failed:', error);
    throw error;
  }
};

// Check if Tauri is available
const isTauriAvailable = (): boolean => {
  const tauriAvailable = typeof window !== 'undefined' && !!window.__TAURI__;
  const isTauriContext = tauriAvailable || 
    (typeof window !== 'undefined' && window.location.protocol === 'tauri:') ||
    (typeof window !== 'undefined' && window.location.hostname === 'localhost' && window.location.port === '3000');
  
  return isTauriContext;
};

// ============================================================================
// OBS obws Commands
// ============================================================================

export const obsObwsCommands = {
  /**
   * Update an existing OBS WebSocket connection
   */
  updateConnection: async (oldName: string, params: {
    name: string;
    host: string;
    port: number;
    password?: string;
    enabled: boolean;
  }): Promise<TauriCommandResponse<any>> => {
    try {
      const result = await safeInvoke('obs_obws_update_connection', {
        oldName: oldName,
        connection: params
      });
      
      return {
        success: result.success || false,
        data: result.data || null,
        error: result.error || null,
      };
    } catch (error) {
      console.error('Failed to update OBS connection:', error);
      return {
        success: false,
        data: null,
        error: String(error),
      };
    }
  },

  /**
   * Add a new OBS WebSocket connection
   */
  async addConnection(params: {
    name: string;
    host: string;
    port: number;
    password?: string;
    enabled: boolean;
  }): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_add_connection', { connection: params });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Connect to an OBS instance using obws
   */
  async connect(connectionName: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_connect', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Disconnect from an OBS instance using obws
   */
  async disconnect(connectionName: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_disconnect', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get connection status using obws
   */
  async getConnectionStatus(connectionName: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_connection_status', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get all connections using obws
   */
  async getConnections(): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_connections');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  

  /**
   * Remove a connection using obws
   */
  async removeConnection(connectionName: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_remove_connection', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get OBS status using obws
   */
  async getStatus(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_status', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Start recording using obws
   */
  async startRecording(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_start_recording', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Stop recording using obws
   */
  async stopRecording(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_stop_recording', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get recording status using obws
   */
  async getRecordingStatus(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_recording_status', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Start streaming using obws
   */
  async startStreaming(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_start_streaming', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Stop streaming using obws
   */
  async stopStreaming(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_stop_streaming', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get streaming status using obws
   */
  async getStreamingStatus(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_streaming_status', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get current scene using obws
   */
  async getCurrentScene(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_current_scene', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

    /**
   * Set current scene using obws
   */
  async setCurrentScene(sceneName: string, connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_set_current_scene', {
          sceneName: sceneName,
          connectionName: connectionName
        });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get scenes using obws
   */
  async getScenes(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_scenes', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get OBS version using obws
   */
  async getVersion(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_version', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get OBS stats using obws
   */
  async getStats(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_stats', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Test obws connection
   */
  async testConnection(): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_test_connection');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Set up status listener using obws
   */
  async setupStatusListener(): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_setup_status_listener');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  // ============================================================================
  // Replay Buffer Commands
  // ============================================================================

  /**
   * Start replay buffer using obws
   */
  async startReplayBuffer(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_start_replay_buffer', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Stop replay buffer using obws
   */
  async stopReplayBuffer(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_stop_replay_buffer', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Save replay buffer using obws
   */
  async saveReplayBuffer(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_save_replay_buffer', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get replay buffer status using obws
   */
  async getReplayBufferStatus(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_replay_buffer_status', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  // ============================================================================
  // Path Configuration Commands
  // ============================================================================

  /**
   * Get recording path settings using obws
   */
  async getRecordingPathSettings(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_recording_path_settings', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Set recording path using obws
   */
  async setRecordingPath(path: string, connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_set_recording_path', { 
          path: path,
          connectionName: connectionName 
        });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get replay buffer path settings using obws
   */
  async getReplayBufferPathSettings(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_replay_buffer_path_settings', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Set replay buffer path using obws
   */
  async setReplayBufferPath(path: string, connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_set_replay_buffer_path', { 
          path: path,
          connectionName: connectionName 
        });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  // ============================================================================
  // Recording Configuration Commands
  // ============================================================================

  /**
   * Get recording configuration from database
   */
  async getRecordingConfig(connectionName: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_recording_config', { connectionName: connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Save recording configuration to database
   */
  async saveRecordingConfig(config: any): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_save_recording_config', { config: config });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Create a new recording session
   */
  async createRecordingSession(session: any): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_create_recording_session', { session: session });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Update recording session status
   */
  async updateRecordingSessionStatus(sessionId: number, status: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_update_recording_session_status', { 
          sessionId: sessionId,
          status: status 
        });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  // ============================================================================
  // Path Generation Commands
  // ============================================================================

  /**
   * Generate recording path for a match
   */
  async generateRecordingPath(matchId: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_generate_recording_path', { matchId });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get Windows Videos folder path
   */
  async getWindowsVideosFolder(): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_windows_videos_folder');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Test path generation with sample data
   */
  async testPathGeneration(params: {
    matchId: string;
    tournamentName?: string;
    tournamentDay?: string;
    matchNumber?: string;
    player1Name?: string;
    player1Flag?: string;
    player2Name?: string;
    player2Flag?: string;
  }): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_test_path_generation', params);
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Create test folders in Windows (actually creates the directory structure)
   */
  async createTestFolders(params: {
    matchId: string;
    tournamentName?: string;
    tournamentDay?: string;
    matchNumber?: string;
    player1Name?: string;
    player1Flag?: string;
    player2Name?: string;
    player2Flag?: string;
  }): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_create_test_folders', params);
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Send recording configuration to OBS
   */
  async sendConfigToObs(connectionName: string, recordingPath: string, filenameTemplate: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_send_config_to_obs', { 
          connectionName, 
          recordingPath, 
          filenameTemplate 
        });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  // ============================================================================
  // Automatic Recording Commands
  // ============================================================================

  /**
   * Test recording functionality
   */
  async testRecording(connectionName: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_test_recording', { connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get automatic recording configuration
   */
  async getAutomaticRecordingConfig(): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_automatic_recording_config');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Update automatic recording configuration
   */
  async updateAutomaticRecordingConfig(params: {
    enabled: boolean;
    obs_connection_name?: string;
    autoStopOnMatchEnd: boolean;
    autoStopOnWinner: boolean;
    stopDelaySeconds: number;
    includeReplayBuffer: boolean;
  }): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        // Tauri expects snake_case parameter names; map from camelCase
        return await safeInvoke('obs_obws_update_automatic_recording_config', {
          enabled: params.enabled,
          // Send both snake_case and camelCase to be fully compatible with backend
          obs_connection_name: params.obs_connection_name,
          obsConnectionName: params.obs_connection_name,
          auto_stop_on_match_end: params.autoStopOnMatchEnd,
          auto_stop_on_winner: params.autoStopOnWinner,
          stop_delay_seconds: params.stopDelaySeconds,
          include_replay_buffer: params.includeReplayBuffer,
        });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get current recording session
   */
  async getCurrentRecordingSession(): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_current_recording_session');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Clear current recording session
   */
  async clearRecordingSession(): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_clear_recording_session');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Manually start recording for a match
   */
  async manualStartRecording(matchId: string, obsConnectionName: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_manual_start_recording', { matchId, obsConnectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Manually stop recording
   */
  async manualStopRecording(obsConnectionName: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_manual_stop_recording', { obsConnectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  // ============================================================================
  // IVR Replay Settings & Actions
  // ============================================================================

  async ivrGetReplaySettings(): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('ivr_get_replay_settings');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  async ivrSaveReplaySettings(params: {
    mpvPath?: string;
    secondsFromEnd: number;
    maxWaitMs: number;
    autoOnChallenge: boolean;
  }): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('ivr_save_replay_settings', {
          mpvPath: params.mpvPath,
          secondsFromEnd: params.secondsFromEnd,
          maxWaitMs: params.maxWaitMs,
          autoOnChallenge: params.autoOnChallenge,
        });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  async ivrRoundReplayNow(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('ivr_round_replay_now', { connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  async ivrValidateMpvPath(mpvPath: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('ivr_validate_mpv_path', { mpvPath });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  async getObsRecordDirectory(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_record_directory', { connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  async getObsFilenameFormatting(connectionName?: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_obws_get_filename_formatting', { connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },
};
