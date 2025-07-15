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
  args: any = {}
): Promise<TauriCommandResponse<T>> => {
  try {
    if (!isTauriAvailable()) {
      return { success: false, error: 'Tauri not available' };
    }

    const result = await window.__TAURI__.invoke(command, args);
    return { success: true, data: result };
  } catch (error) {
    return { success: false, error: String(error) };
  }
}; 