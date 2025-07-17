// Tauri command utilities for reStrike VTA

import { TauriCommandResponse, ObsConnection, VideoClip, PssEvent } from '../types';

// ============================================================================
// OBS Commands
// ============================================================================

export const obsCommands = {
  /**
   * Connect to OBS WebSocket
   */
  async connect(url: string): Promise<TauriCommandResponse> {
    try {
      if (window.__TAURI__) {
        return await window.__TAURI__.invoke('obs_connect', { url });
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
      if (window.__TAURI__) {
        return await window.__TAURI__.invoke('obs_disconnect', { connectionName });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get OBS status
   */
  async getStatus(): Promise<TauriCommandResponse> {
    try {
      if (window.__TAURI__) {
        return await window.__TAURI__.invoke('obs_get_status');
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
      if (window.__TAURI__) {
        return await window.__TAURI__.invoke('obs_start_recording');
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
      if (window.__TAURI__) {
        return await window.__TAURI__.invoke('obs_stop_recording');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },
};

// ============================================================================
// Video Commands
// ============================================================================

export const videoCommands = {
  /**
   * Play video clip
   */
  async playClip(clipPath: string): Promise<TauriCommandResponse> {
    try {
      if (window.__TAURI__) {
        return await window.__TAURI__.invoke('video_play', { path: clipPath });
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
      if (window.__TAURI__) {
        return await window.__TAURI__.invoke('video_stop');
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
      if (window.__TAURI__) {
        return await window.__TAURI__.invoke('video_get_info', { path });
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
      if (window.__TAURI__) {
        return await window.__TAURI__.invoke('pss_start_listener', { port });
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
      if (window.__TAURI__) {
        return await window.__TAURI__.invoke('pss_stop_listener');
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
      if (window.__TAURI__) {
        return await window.__TAURI__.invoke('pss_get_events');
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
      if (window.__TAURI__) {
        return await window.__TAURI__.invoke('system_get_info');
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
      if (window.__TAURI__) {
        return await window.__TAURI__.invoke('system_open_file_dialog');
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
  return typeof window !== 'undefined' && !!window.__TAURI__;
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

    // Add timeout handling
    const timeoutPromise = new Promise<never>((_, reject) => {
      setTimeout(() => reject(new Error(`Command ${command} timed out after ${timeout}ms`)), timeout);
    });

    const commandPromise = window.__TAURI__!.invoke(command, args);
    const result = await Promise.race([commandPromise, timeoutPromise]);
    
    return { success: true, data: result };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    
    // Provide more specific error messages
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