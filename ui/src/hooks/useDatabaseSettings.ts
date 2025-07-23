import { useState, useEffect, useCallback } from 'react';

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

export interface DatabaseSetting {
  key: string;
  value: string;
  changed_by?: string;
  change_reason?: string;
}

export interface DatabaseSettingsState {
  settings: Record<string, string>;
  loading: boolean;
  error: string | null;
  initialized: boolean;
}

export interface DatabaseSettingsActions {
  initializeSettings: () => Promise<void>;
  initializeDatabase: () => Promise<void>; // Alias for initializeSettings
  getSetting: (key: string) => Promise<string | null>;
  setSetting: (key: string, value: string, changedBy?: string, changeReason?: string) => Promise<void>;
  getAllSettings: () => Promise<Record<string, string>>;
  getDatabaseInfo: () => Promise<{ is_accessible: boolean; file_size: number | null }>;
  refreshSettings: () => Promise<void>;
  restoreBackupWithDialog: () => Promise<{ success: boolean; message?: string; error?: string }>;
}

export function useDatabaseSettings(): DatabaseSettingsState & DatabaseSettingsActions {
  const [settings, setSettings] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [initialized, setInitialized] = useState(false);

  const initializeSettings = useCallback(async () => {
    setLoading(true);
    setError(null);
    
    try {
      const result = await safeInvoke('db_initialize_ui_settings') as { success: boolean; message?: string; error?: string };
      
      if (result.success) {
        console.log('✅ Database settings initialized:', result.message);
        await refreshSettings();
        setInitialized(true);
      } else {
        throw new Error(result.error || 'Failed to initialize database settings');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      setError(errorMessage);
      console.error('❌ Failed to initialize database settings:', errorMessage);
    } finally {
      setLoading(false);
    }
  }, []);

  const initializeDatabase = useCallback(async () => {
    // Alias for initializeSettings
    return initializeSettings();
  }, [initializeSettings]);

  const getSetting = useCallback(async (key: string): Promise<string | null> => {
    try {
      const result = await safeInvoke('db_get_ui_setting', { key }) as { success: boolean; key: string; value: string | null; error?: string };
      
      if (result.success) {
        return result.value;
      } else {
        throw new Error(result.error || 'Failed to get setting');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      console.error(`❌ Failed to get setting '${key}':`, errorMessage);
      return null;
    }
  }, []);

  const setSetting = useCallback(async (key: string, value: string, changedBy: string = 'user', changeReason?: string) => {
    setLoading(true);
    setError(null);
    
    try {
      const result = await safeInvoke('db_set_ui_setting', {
        key,
        value,
        changedBy,
        changeReason: changeReason || undefined
      }) as { success: boolean; message?: string; error?: string };
      
      if (result.success) {
        console.log(`✅ Setting '${key}' updated:`, result.message);
        // Update local state
        setSettings(prev => ({ ...prev, [key]: value }));
      } else {
        throw new Error(result.error || 'Failed to set setting');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      setError(errorMessage);
      console.error(`❌ Failed to set setting '${key}':`, errorMessage);
    } finally {
      setLoading(false);
    }
  }, []);

  const getAllSettings = useCallback(async (): Promise<Record<string, string>> => {
    try {
      const result = await safeInvoke('db_get_all_ui_settings') as { success: boolean; settings: Record<string, string>; error?: string };
      
      if (result.success) {
        return result.settings;
      } else {
        throw new Error(result.error || 'Failed to get all settings');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      console.error('❌ Failed to get all settings:', errorMessage);
      return {};
    }
  }, []);

  const getDatabaseInfo = useCallback(async (): Promise<{ is_accessible: boolean; file_size: number | null }> => {
    try {
      const result = await safeInvoke('db_get_database_info') as { success: boolean; is_accessible: boolean; file_size: number | null; error?: string };
      
      if (result.success) {
        return {
          is_accessible: result.is_accessible,
          file_size: result.file_size
        };
      } else {
        throw new Error(result.error || 'Failed to get database info');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      console.error('❌ Failed to get database info:', errorMessage);
      return { is_accessible: false, file_size: null };
    }
  }, []);

  const refreshSettings = useCallback(async () => {
    setLoading(true);
    setError(null);
    
    try {
      const allSettings = await getAllSettings();
      setSettings(allSettings);
      console.log('✅ Settings refreshed:', Object.keys(allSettings).length, 'settings loaded');
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      setError(errorMessage);
      console.error('❌ Failed to refresh settings:', errorMessage);
    } finally {
      setLoading(false);
    }
  }, [getAllSettings]);

  const restoreBackupWithDialog = useCallback(async (): Promise<{ success: boolean; message?: string; error?: string }> => {
    setLoading(true);
    setError(null);
    
    try {
      const result = await safeInvoke('restore_backup_with_dialog') as { success: boolean; message?: string; error?: string };
      
      if (result.success) {
        console.log('✅ Backup restored successfully:', result.message);
        await refreshSettings(); // Refresh settings after restore
        return result;
      } else {
        const errorMessage = result.error || 'Failed to restore backup';
        setError(errorMessage);
        console.error('❌ Failed to restore backup:', errorMessage);
        return result;
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      setError(errorMessage);
      console.error('❌ Failed to restore backup:', errorMessage);
      return { success: false, error: errorMessage };
    } finally {
      setLoading(false);
    }
  }, [refreshSettings]);

  // Initialize settings on mount
  useEffect(() => {
    if (!initialized) {
      initializeSettings();
    }
  }, [initialized, initializeSettings]);

  return {
    // State
    settings,
    loading,
    error,
    initialized,
    
    // Actions
    initializeSettings,
    initializeDatabase,
    getSetting,
    setSetting,
    getAllSettings,
    getDatabaseInfo,
    refreshSettings,
    restoreBackupWithDialog,
  };
} 