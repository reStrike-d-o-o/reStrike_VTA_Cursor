// Tauri command utilities for reStrike VTA

import { TauriCommandResponse, ObsConnection, VideoClip, PssEvent } from '../types';

// Tauri v2 invoke function that uses the core module
const safeInvoke = async (command: string, args?: any) => {
  try {
    // Check if the global Tauri object is available
    if (typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.core) {
      console.log('✅ Using Tauri v2 core module for command:', command);
      // In Tauri v2, invoke is available through the core module
      return await window.__TAURI__.core.invoke(command, args);
    }
    
    throw new Error('Tauri v2 core module not available - ensure app is running in desktop mode');
  } catch (error) {
    console.error('Tauri invoke failed:', error);
    throw error;
  }
};

// ============================================================================
// OBS WebSocket Commands
// ============================================================================

export const obsCommands = {
  /**
   * Connect to OBS WebSocket
   */
  async connect(url: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_connect', { url });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },



  /**
   * Get OBS connection status
   */
  async getStatus(): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_get_status');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Start OBS recording
   */
  async startRecording(): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_start_recording');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Stop OBS recording
   */
  async stopRecording(): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_stop_recording');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Play video clip
   */
  async playClip(clipPath: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('video_play', { path: clipPath });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Stop video playback
   */
  async stopPlayback(): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('video_stop');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get video information
   */
  async getVideoInfo(path: string): Promise<TauriCommandResponse<VideoClip>> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('video_get_info', { path });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Add a new OBS connection
   */
  async addConnection(params: {
    name: string;
    host: string;
    port: number;
    password?: string;
    enabled: boolean;
  }) {
    const tauriParams = {
      name: params.name,
      host: params.host,
      port: params.port,
      password: params.password,
      enabled: params.enabled,
    };
    return executeTauriCommand('obs_add_connection', tauriParams);
  },

  /**
   * Connect to a specific OBS connection
   */
  async connectToConnection(connectionName: string) {
    return executeTauriCommand('obs_connect_to_connection', { connectionName });
  },

  /**
   * Get status of a specific OBS connection
   */
  async getConnectionStatus(connectionName: string) {
    return executeTauriCommand('obs_get_connection_status', { connectionName });
  },

  /**
   * Get all OBS connections
   */
  async getConnections() {
    return executeTauriCommand('obs_get_connections', {});
  },

  /**
   * Disconnect from OBS
   */
  async disconnect(connectionName: string) {
    return executeTauriCommand('obs_disconnect', { connectionName });
  },

  /**
   * Remove OBS connection configuration
   */
  async removeConnection(connectionName: string) {
    return executeTauriCommand('obs_remove_connection', { connectionName });
  },
};

// Configuration management commands
export const configCommands = {
  /**
   * Get all application settings
   */
  async getSettings() {
    return executeTauriCommand('get_settings', {});
  },

  /**
   * Update application settings
   */
  async updateSettings(settings: any) {
    return executeTauriCommand('update_settings', { settings });
  },

  /**
   * Get configuration statistics
   */
  async getConfigStats() {
    return executeTauriCommand('get_config_stats', {});
  },

  /**
   * Reset settings to defaults
   */
  async resetSettings() {
    return executeTauriCommand('reset_settings', {});
  },

  /**
   * Export settings to file
   */
  async exportSettings(exportPath: string) {
    return executeTauriCommand('export_settings', { exportPath });
  },

  /**
   * Import settings from file
   */
  async importSettings(importPath: string) {
    return executeTauriCommand('import_settings', { importPath });
  },

  /**
   * Restore settings from backup
   */
  async restoreSettingsBackup() {
    return executeTauriCommand('restore_settings_backup', {});
  },
};

// ============================================================================
// PSS Protocol Commands
// ============================================================================

export const pssCommands = {
  /**
   * Start PSS protocol listener
   */
  async startListener(port: number): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('pss_start_listener', { port });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Stop PSS protocol listener
   */
  async stopListener(): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('pss_stop_listener');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get PSS events
   */
  async getEvents(): Promise<TauriCommandResponse<PssEvent[]>> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('pss_get_events');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },
};

// ============================================================================
// System Commands
// ============================================================================

export const systemCommands = {
  /**
   * Get system information
   */
  async getSystemInfo(): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('system_get_info');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Open file dialog
   */
  async openFileDialog(): Promise<TauriCommandResponse<string[]>> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('system_open_file_dialog');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },
};

// ============================================================================
// Diagnostics & Logs Manager Commands
// ============================================================================

export const diagLogsCommands = {
  /**
   * List log files in the log/ directory, optionally filtered by subsystem
   */
  async listLogFiles(subsystem?: string) {
    return executeTauriCommand('list_log_files', { subsystem });
  },

  /**
   * Download a log file by filename
   */
  async downloadLogFile(filename: string) {
    return executeTauriCommand('download_log_file', { filename });
  },

  /**
   * List archive files in the log/archives/ directory
   */
  async listArchives() {
    return executeTauriCommand('list_archives');
  },

  /**
   * Extract a specific archive file
   */
  async extractArchive(archiveName: string) {
    return executeTauriCommand('extract_archive', { archiveName });
  },

  /**
   * Download a specific archive file
   */
  async downloadArchive(archiveName: string) {
    return executeTauriCommand('download_archive', { archiveName });
  },

  /**
   * Enable or disable live data streaming for a subsystem
   */
  async setLiveDataStreaming(subsystem: string, enabled: boolean) {
    return executeTauriCommand('set_live_data_streaming', { subsystem, enabled });
  },
};

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Check if Tauri is available
 */
export const isTauriAvailable = (): boolean => {
  // Check if we're in a browser environment and Tauri is available
  if (typeof window === 'undefined' || !window.__TAURI__) {
    console.log('🔍 isTauriAvailable: window.__TAURI__ not available');
    return false;
  }

  // Check if the core module and invoke function are actually available
  if (!window.__TAURI__.core || typeof window.__TAURI__.core.invoke !== 'function') {
    console.log('🔍 isTauriAvailable: window.__TAURI__.core.invoke not available');
    return false;
  }

  console.log('🔍 isTauriAvailable: Tauri v2 API appears to be available');
  return true;
};

/**
 * Test Tauri API availability with a simple command
 */
export const testTauriApi = async (): Promise<boolean> => {
  try {
    if (!isTauriAvailable()) {
      console.log('❌ Tauri not available for testing');
      return false;
    }

    console.log('🔍 Testing Tauri API with get_app_status command...');
    
    // Try multiple commands to test Tauri API
    try {
      const result = await safeInvoke('get_app_status');
      console.log('✅ get_app_status successful:', result);
      return true;
    } catch (error) {
      console.log('⚠️ get_app_status failed, trying obs_get_status...');
      
      try {
        const obsResult = await safeInvoke('obs_get_status');
        console.log('✅ obs_get_status successful:', obsResult);
        return true;
      } catch (obsError) {
        console.log('⚠️ obs_get_status failed, trying system_get_info...');
        
        try {
          const sysResult = await safeInvoke('system_get_info');
          console.log('✅ system_get_info successful:', sysResult);
          return true;
        } catch (sysError) {
          console.log('❌ All Tauri API tests failed');
          console.log('get_app_status error:', error);
          console.log('obs_get_status error:', obsError);
          console.log('system_get_info error:', sysError);
          return false;
        }
      }
    }
  } catch (error) {
    console.log('❌ Tauri API test failed:', error);
    return false;
  }
};

/**
 * Execute a Tauri command with error handling
 */
export const executeTauriCommand = async <T = any>(
  command: string,
  args: any = {},
  timeout: number = 10000
): Promise<TauriCommandResponse<T>> => {
  try {
    if (!isTauriAvailable()) {
      return { success: false, error: 'Tauri not available - running in web mode' };
    }

    // Use the proper Tauri v2 invoke function
    const result = await safeInvoke(command, args);
    
    // Check if the result is already in TauriCommandResponse format
    if (result && typeof result === 'object' && 'success' in result) {
      // Backend already returned TauriCommandResponse format
      return result as TauriCommandResponse<T>;
    } else {
      // Backend returned raw data, wrap it
      return { success: true, data: result as T };
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    
    // Provide more specific error messages
    if (errorMessage.includes('Cannot read properties of undefined')) {
      return { success: false, error: 'Tauri not available - ensure app is running in desktop mode' };
    }
    if (errorMessage.includes('timeout')) {
      return { success: false, error: `Command timed out: ${errorMessage}` };
    }
    if (errorMessage.includes('not found')) {
      return { success: false, error: `Command not found: ${command}` };
    }
    if (errorMessage.includes('permission')) {
      return { success: false, error: `Permission denied: ${errorMessage}` };
    }
    
    return { success: false, error: `Command failed: ${errorMessage}` };
  }
}; 