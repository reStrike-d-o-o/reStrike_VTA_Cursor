// Tauri command utilities for reStrike VTA

import { invoke } from '@tauri-apps/api/core';
import { TauriCommandResponse, ObsConnection, VideoClip, PssEvent } from '../types';

// Fallback invoke function for compatibility
const safeInvoke = async (command: string, args?: any) => {
  try {
    // Try the proper Tauri v2 API first
    return await invoke(command, args);
  } catch (error) {
    // If that fails, try the global window.__TAURI__.invoke
    if (typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.invoke) {
      return await window.__TAURI__.invoke(command, args);
    }
    throw new Error('Tauri invoke method not available - ensure app is running in desktop mode');
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
   * Disconnect from OBS WebSocket
   */
  async disconnect(connectionName: string): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('obs_disconnect', { connectionName });
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
   * Enable or disable logging for a subsystem (pss, obs, udp)
   */
  async setLoggingEnabled(subsystem: string, enabled: boolean) {
    return executeTauriCommand('set_logging_enabled', { subsystem, enabled });
  },

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

  // Check if the invoke function is actually available
  if (typeof window.__TAURI__.invoke !== 'function') {
    console.log('🔍 isTauriAvailable: window.__TAURI__.invoke not available');
    return false;
  }

  console.log('🔍 isTauriAvailable: Tauri API appears to be available');
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