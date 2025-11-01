// Tauri command utilities for reStrike VTA

import { TauriCommandResponse, ObsConnection, VideoClip, PssEvent, OpenApiStateResponse, OpenApiSaveResponse, OpenApiUploadResponse, OpenApiValidateResponse, SchemaFormat, MedalCeremonySummary, MedalCeremonyDetail, FlagAnimationAsset, AnthemAsset, MedalCeremonyDivision, MedalCeremonyDivisionOption, MedalCeremonyAthleteOption } from '../types';

// Tauri v2 invoke function that uses the core module
const safeInvoke = async (command: string, args?: any) => {
  if (typeof window !== 'undefined' && window.__TAURI__?.core?.invoke) {
    return window.__TAURI__.core.invoke(command, args);
  }
  throw new Error('Tauri v2 core module not available - ensure app is running in desktop mode');
};

// OBS WebSocket commands have been moved to tauriCommandsObws.ts

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
// OpenAPI Schema management commands
export const openApiCommands = {
  async getState(format?: SchemaFormat): Promise<OpenApiStateResponse> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    const args = format ? { request: { format } } : {};
    return await safeInvoke('openapi_get_state', args) as OpenApiStateResponse;
  },

  async saveSchema(schema: string, format: SchemaFormat): Promise<OpenApiSaveResponse> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    return await safeInvoke('openapi_save_schema', { request: { schema, format } }) as OpenApiSaveResponse;
  },

  async validateSchema(schema: string, format: SchemaFormat): Promise<OpenApiValidateResponse> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    return await safeInvoke('openapi_validate_schema', { request: { schema, format } }) as OpenApiValidateResponse;
  },

  async uploadSchema(path: string): Promise<OpenApiUploadResponse> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    return await safeInvoke('openapi_upload_schema', { request: { path } }) as OpenApiUploadResponse;
  },

  async exportSchema(path: string, format: SchemaFormat): Promise<void> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    await safeInvoke('openapi_export_schema', { request: { path, format } });
  },
};

// ============================================================================
// Medal Ceremony Commands
// ============================================================================

export const medalCeremonyCommands = {
  async listDivisionOptions(): Promise<MedalCeremonyDivisionOption[]> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    const result = await safeInvoke('medal_ceremony_list_divisions');
    return (Array.isArray(result) ? result : []) as MedalCeremonyDivisionOption[];
  },

  async listAthletes(division: string): Promise<MedalCeremonyAthleteOption[]> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    const result = await safeInvoke('medal_ceremony_list_athletes', { division });
    return (Array.isArray(result) ? result : []) as MedalCeremonyAthleteOption[];
  },

  async list(): Promise<MedalCeremonySummary[]> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    const result = await safeInvoke('medal_ceremony_list');
    return (Array.isArray(result) ? result : []) as MedalCeremonySummary[];
  },

  async get(ceremonyId: string): Promise<MedalCeremonyDetail | null> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    const detail = await safeInvoke('medal_ceremony_get', { ceremonyId });
    return (detail ?? null) as MedalCeremonyDetail | null;
  },

  async save(detail: MedalCeremonyDetail): Promise<string> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    return await safeInvoke('medal_ceremony_save', { payload: detail }) as string;
  },

  async remove(ceremonyId: string): Promise<void> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    await safeInvoke('medal_ceremony_delete', { ceremonyId });
  },

  async prepare(ceremonyId: string): Promise<MedalCeremonyDivision[]> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    const result = await safeInvoke('medal_ceremony_prepare', { ceremonyId });
    return (Array.isArray(result) ? result : []) as MedalCeremonyDivision[];
  },

  async markDivisionPlayed(divisionId: string): Promise<void> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    await safeInvoke('medal_ceremony_mark_division_played', { divisionId });
  },

  async resetPlayback(ceremonyId: string): Promise<void> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    await safeInvoke('medal_ceremony_reset_playback', { ceremonyId });
  },

  async setShowExternal(ceremonyId: string, enabled: boolean): Promise<void> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    await safeInvoke('medal_ceremony_set_show_external', { ceremonyId, enabled });
  },

  async listFlagAssets(): Promise<FlagAnimationAsset[]> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    const result = await safeInvoke('medal_ceremony_list_flag_assets');
    return (Array.isArray(result) ? result : []) as FlagAnimationAsset[];
  },

  async saveFlagAsset(asset: FlagAnimationAsset): Promise<string> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    return await safeInvoke('medal_ceremony_save_flag_asset', { asset }) as string;
  },

  async deleteFlagAsset(assetId: string): Promise<void> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    await safeInvoke('medal_ceremony_delete_flag_asset', { assetId });
  },

  async listAnthemAssets(): Promise<AnthemAsset[]> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    const result = await safeInvoke('medal_ceremony_list_anthems');
    return (Array.isArray(result) ? result : []) as AnthemAsset[];
  },

  async saveAnthemAsset(asset: AnthemAsset): Promise<string> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    return await safeInvoke('medal_ceremony_save_anthem', { asset }) as string;
  },

  async deleteAnthemAsset(assetId: string): Promise<void> {
    if (!isTauriAvailable()) {
      throw new Error('Tauri not available');
    }
    await safeInvoke('medal_ceremony_delete_anthem', { assetId });
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
// License Commands
// ============================================================================

export const licenseCommands = {
  async getStatus(): Promise<TauriCommandResponse<any>> {
    try {
      if (isTauriAvailable()) {
        const result = await safeInvoke('get_license_status');
        return { success: true, data: result };
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  async validate(): Promise<TauriCommandResponse<any>> {
    try {
      if (isTauriAvailable()) {
        const result = await safeInvoke('validate_license');
        return { success: true, data: result };
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  async activate(key: string): Promise<TauriCommandResponse<any>> {
    try {
      if (isTauriAvailable()) {
        const result = await safeInvoke('activate_license', { key });
        return { success: true, data: result };
      }
      return { success: false, error: 'Tauri not available' };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  async getMachineIdentity(): Promise<TauriCommandResponse<{ uid: string; machine_hash: string }>> {
    try {
      if (isTauriAvailable()) {
        const result = await safeInvoke('get_machine_identity');
        return { success: true, data: result };
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
