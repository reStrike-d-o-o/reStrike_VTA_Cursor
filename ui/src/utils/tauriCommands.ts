// Tauri command utilities for reStrike VTA

import { TauriCommandResponse, ObsConnection, VideoClip, PssEvent } from '../types';

// Tauri v2 invoke function that uses the core module
const safeInvoke = async (command: string, args?: any) => {
  try {
    // Check if the global Tauri object is available
    if (typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.core) {
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

  /**
   * Setup OBS status listener
   */
  async setupStatusListener(): Promise<TauriCommandResponse> {
    return executeTauriCommand('obs_setup_status_listener');
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
   * Get PSS events from the backend
   */
  async getEvents(): Promise<TauriCommandResponse<PssEvent[]>> {
    try {
      if (isTauriAvailable()) {
        const result = await safeInvoke('pss_get_events');
        
        // The backend returns either Vec<serde_json::Value> or String (error)
        if (Array.isArray(result)) {
          return {
            success: true,
            data: result || [],
          };
        } else {
          // If result is a string, it's an error message
          return { 
            success: false, 
            error: typeof result === 'string' ? result : 'Unknown error', 
            data: [] 
          };
        }
      }
      return { success: false, error: 'Tauri not available', data: [] };
    } catch (error) {
      return { success: false, error: String(error), data: [] };
    }
  },

  /**
   * Emit a PSS event to the frontend
   */
  async emitEvent(eventData: any): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('pss_emit_event', { eventData });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Emit pending PSS events to the frontend
   */
  async emitPendingEvents(): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('pss_emit_pending_events');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Set up PSS event listener for real-time events
   */
  async setupEventListener(): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('pss_setup_event_listener');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },
};

// ============================================================================
// CPU Monitor Commands
// ============================================================================

export const cpuCommands = {
  async setupStatsListener(): Promise<TauriCommandResponse> {
    return executeTauriCommand('cpu_setup_stats_listener');
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

  // ========================================================================
  // New Log Archive & Google Drive Commands
  // ========================================================================

  /**
   * Create a complete archive of all current logs
   */
  async createCompleteLogArchive() {
    return executeTauriCommand('create_complete_log_archive');
  },

  /**
   * Create archive and upload to Google Drive
   */
  async createAndUploadLogArchive() {
    return executeTauriCommand('create_and_upload_log_archive');
  },

  /**
   * Create archive, upload to Google Drive, and delete local file
   */
  async createUploadAndCleanupLogArchive() {
    return executeTauriCommand('create_upload_and_cleanup_log_archive');
  },

  /**
   * Get auto-archive configuration
   */
  async getAutoArchiveConfig() {
    return executeTauriCommand('get_auto_archive_config');
  },

  /**
   * Set auto-archive configuration
   */
  async setAutoArchiveConfig(config: {
    enabled: boolean;
    schedule: 'Weekly' | 'Monthly' | 'Quarterly' | 'Biannual' | 'Annual';
    upload_to_drive: boolean;
    delete_after_upload: boolean;
    last_archive_time?: string;
  }) {
    return executeTauriCommand('set_auto_archive_config', { config });
  },

  /**
   * Check auto-archive status
   */
  async checkAutoArchiveStatus(config: {
    enabled: boolean;
    schedule: 'Weekly' | 'Monthly' | 'Quarterly' | 'Biannual' | 'Annual';
    upload_to_drive: boolean;
    delete_after_upload: boolean;
    last_archive_time?: string;
  }) {
    return executeTauriCommand('check_auto_archive_status', { config });
  },

  /**
   * Perform auto-archive operation
   */
  async performAutoArchive(config: {
    enabled: boolean;
    schedule: 'Weekly' | 'Monthly' | 'Quarterly' | 'Biannual' | 'Annual';
    upload_to_drive: boolean;
    delete_after_upload: boolean;
    last_archive_time?: string;
  }) {
    return executeTauriCommand('perform_auto_archive', { config });
  },

  /**
   * Delete a specific log archive
   */
  async deleteLogArchive(archiveName: string) {
    return executeTauriCommand('delete_log_archive', { archiveName });
  },
};

// ============================================================================
// Window Management Commands
// ============================================================================

export const windowCommands = {
  /**
   * Set window to fullscreen mode
   */
  async setFullscreen(): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('set_window_fullscreen');
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Set window to compact mode (custom dimensions)
   */
  async setCompact(width?: number, height?: number): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('set_window_compact', { width, height });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Set window to custom size
   */
  async setCustomSize(width: number, height: number): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('set_window_custom_size', { width, height });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Get screen size information
   */
  async getScreenSize(): Promise<TauriCommandResponse<{ width: number; height: number }>> {
    try {
      if (isTauriAvailable()) {
        const result = await safeInvoke('get_screen_size');
        return { success: true, data: result };
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Save window settings
   */
  async saveWindowSettings(settings: any): Promise<TauriCommandResponse> {
    try {
      if (isTauriAvailable()) {
        return await safeInvoke('save_window_settings', { settings });
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  /**
   * Load window settings
   */
  async loadWindowSettings(): Promise<TauriCommandResponse<any>> {
    try {
      if (isTauriAvailable()) {
        const result = await safeInvoke('load_window_settings');
        return { success: true, data: result };
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
  // Check if we're in a browser environment and Tauri is available
  if (typeof window === 'undefined' || !window.__TAURI__) {
    return false;
  }

  // Check if the core module and invoke function are actually available
  if (!window.__TAURI__.core || typeof window.__TAURI__.core.invoke !== 'function') {
    return false;
  }

  return true;
};

/**
 * Test Tauri API availability with a simple command
 */
export const testTauriApi = async (): Promise<boolean> => {
  try {
    if (!isTauriAvailable()) {
      return false;
    }
    
    // Try multiple commands to test Tauri API
    try {
      const result = await safeInvoke('get_app_status');
      return true;
    } catch (error) {
      try {
        const obsResult = await safeInvoke('obs_get_status');
        return true;
      } catch (obsError) {
        try {
          const sysResult = await safeInvoke('system_get_info');
          return true;
        } catch (sysError) {
          return false;
        }
      }
    }
  } catch (error) {
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
      // Backend already returned TauriCommandResponse format - preserve all properties
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