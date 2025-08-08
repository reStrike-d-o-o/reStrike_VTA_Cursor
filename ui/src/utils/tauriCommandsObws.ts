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
  }): Promise<TauriCommandResponse<any>> => {
    try {
      const result = await safeInvoke('obs_obws_update_connection', {
        old_name: oldName,
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
        return await safeInvoke('obs_obws_connect', { connection_name: connectionName });
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
        return await safeInvoke('obs_obws_disconnect', { connection_name: connectionName });
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
        return await safeInvoke('obs_obws_get_connection_status', { connection_name: connectionName });
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
        return await safeInvoke('obs_obws_remove_connection', { connection_name: connectionName });
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
        return await safeInvoke('obs_obws_get_status', { connection_name: connectionName });
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
        return await safeInvoke('obs_obws_start_recording', { connection_name: connectionName });
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
        return await safeInvoke('obs_obws_stop_recording', { connection_name: connectionName });
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
        return await safeInvoke('obs_obws_get_recording_status', { connection_name: connectionName });
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
        return await safeInvoke('obs_obws_start_streaming', { connection_name: connectionName });
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
        return await safeInvoke('obs_obws_stop_streaming', { connection_name: connectionName });
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
        return await safeInvoke('obs_obws_get_streaming_status', { connection_name: connectionName });
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
        return await safeInvoke('obs_obws_get_current_scene', { connection_name: connectionName });
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
          scene_name: sceneName, 
          connection_name: connectionName 
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
        return await safeInvoke('obs_obws_get_scenes', { connection_name: connectionName });
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
        return await safeInvoke('obs_obws_get_version', { connection_name: connectionName });
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
        return await safeInvoke('obs_obws_get_stats', { connection_name: connectionName });
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
};
