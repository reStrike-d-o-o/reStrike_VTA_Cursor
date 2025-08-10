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
  getUiSetting: (key: string) => Promise<any>;
  setUiSetting: (key: string, value: any) => Promise<void>;
  getAllUiSettings: () => Promise<any>;
  getDatabaseInfo: () => Promise<any>;
  migrateJsonToDatabase: () => Promise<any>;
  createJsonBackup: () => Promise<any>;
  restoreFromJsonBackup: (backupPath: string) => Promise<any>;
  getMigrationStatus: () => Promise<any>;
  enableDatabaseMode: () => Promise<any>;
  listBackupFiles: () => Promise<BackupFileInfo[]>;
  getDatabasePreview: () => Promise<any>;
  getDatabaseTables: () => Promise<any>;
  getTableData: (tableName: string) => Promise<any>;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

export interface BackupFileInfo {
  name: string;
  path: string;
  size: number;
  modified: string;
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

  const getUiSetting = useCallback(async (key: string): Promise<string | null> => {
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

  const setUiSetting = useCallback(async (key: string, value: string, changedBy: string = 'user', changeReason?: string) => {
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

  const getAllUiSettings = useCallback(async (): Promise<Record<string, string>> => {
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

  const getDatabaseInfo = useCallback(async (): Promise<any> => {
    try {
      const result = await safeInvoke('db_get_database_info') as { 
        success: boolean; 
        path: string;
        size: number | null; 
        tables: number;
        settings_count: number;
        last_modified: string;
        status: string;
        is_accessible: boolean; 
        error?: string 
      };
      
      if (result.success) {
        return {
          path: result.path,
          size: result.size,
          tables: result.tables,
          settings_count: result.settings_count,
          last_modified: result.last_modified,
          status: result.status,
          is_accessible: result.is_accessible
        };
      } else {
        throw new Error(result.error || 'Failed to get database info');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      console.error('❌ Failed to get database info:', errorMessage);
      return { 
        path: 'Unknown',
        size: null, 
        tables: 0,
        settings_count: 0, 
        last_modified: 'Unknown', 
        status: 'Unknown', 
        is_accessible: false 
      };
    }
  }, []);

  const refreshSettings = useCallback(async () => {
    setLoading(true);
    setError(null);
    
    try {
      const allSettings = await getAllUiSettings();
      setSettings(allSettings);
      console.log('✅ Settings refreshed:', Object.keys(allSettings).length, 'settings loaded');
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      setError(errorMessage);
      console.error('❌ Failed to refresh settings:', errorMessage);
    } finally {
      setLoading(false);
    }
  }, [getAllUiSettings]);

  const listBackupFiles = async (): Promise<BackupFileInfo[]> => {
    try {
      const result = await safeInvoke('list_backup_files');
      return result || [];
    } catch (error) {
      console.error('❌ Failed to list backup files:', error);
      return [];
    }
  };

  const migrateJsonToDatabase = useCallback(async (): Promise<any> => {
    setLoading(true);
    setError(null);
    
    try {
      const result = await safeInvoke('migrate_json_to_database') as { success: boolean; message?: string; error?: string };
      
      if (result.success) {
        console.log('✅ JSON to database migration completed:', result.message);
        return result;
      } else {
        throw new Error(result.error || 'Failed to migrate JSON to database');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      setError(errorMessage);
      console.error('❌ Failed to migrate JSON to database:', errorMessage);
      return { success: false, error: errorMessage };
    } finally {
      setLoading(false);
    }
  }, []);

  const createJsonBackup = useCallback(async (): Promise<any> => {
    setLoading(true);
    setError(null);
    
    try {
      const result = await safeInvoke('create_json_backup') as { success: boolean; message?: string; error?: string };
      
      if (result.success) {
        console.log('✅ JSON backup created:', result.message);
        return result;
      } else {
        throw new Error(result.error || 'Failed to create JSON backup');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      setError(errorMessage);
      console.error('❌ Failed to create JSON backup:', errorMessage);
      return { success: false, error: errorMessage };
    } finally {
      setLoading(false);
    }
  }, []);

  const restoreFromJsonBackup = useCallback(async (backupPath: string): Promise<any> => {
    setLoading(true);
    setError(null);
    
    try {
      const result = await safeInvoke('restore_from_json_backup', { backupPath }) as { success: boolean; message?: string; error?: string };
      
      if (result.success) {
        console.log('✅ Backup restored successfully:', result.message);
        return result;
      } else {
        throw new Error(result.error || 'Failed to restore from backup');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      setError(errorMessage);
      console.error('❌ Failed to restore from backup:', errorMessage);
      return { success: false, error: errorMessage };
    } finally {
      setLoading(false);
    }
  }, []);

  const getMigrationStatus = useCallback(async (): Promise<any> => {
    try {
      const result = await safeInvoke('get_migration_status') as { success: boolean; status?: any; error?: string };
      
      if (result.success) {
        return result.status;
      } else {
        throw new Error(result.error || 'Failed to get migration status');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      console.error('❌ Failed to get migration status:', errorMessage);
      return null;
    }
  }, []);

  const enableDatabaseMode = useCallback(async (): Promise<any> => {
    setLoading(true);
    setError(null);
    
    try {
      const result = await safeInvoke('enable_database_mode') as { success: boolean; message?: string; error?: string };
      
      if (result.success) {
        console.log('✅ Database mode enabled:', result.message);
        return result;
      } else {
        throw new Error(result.error || 'Failed to enable database mode');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      setError(errorMessage);
      console.error('❌ Failed to enable database mode:', errorMessage);
      return { success: false, error: errorMessage };
    } finally {
      setLoading(false);
    }
  }, []);

  const getDatabasePreview = useCallback(async (): Promise<any> => {
    try {
      const result = await safeInvoke('get_database_preview') as { success: boolean; database_settings?: any[]; json_settings?: any[]; database_count?: number; json_count?: number; error?: string };
      
      if (result.success) {
        return result;
      } else {
        throw new Error(result.error || 'Failed to get database preview');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      console.error('❌ Failed to get database preview:', errorMessage);
      return null;
    }
  }, []);

  const getDatabaseTables = useCallback(async (): Promise<any> => {
    try {
      const result = await safeInvoke('get_database_tables') as { success: boolean; tables?: string[]; error?: string };
      
      if (result.success) {
        return result;
      } else {
        throw new Error(result.error || 'Failed to get database tables');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      console.error('❌ Failed to get database tables:', errorMessage);
      return null;
    }
  }, []);

  const getTableData = useCallback(async (tableName: string): Promise<any> => {
    try {
      const result = await safeInvoke('get_table_data', { tableName }) as { success: boolean; table_name?: string; columns?: any[]; rows?: any[]; row_count?: number; error?: string };
      
      if (result.success) {
        return result;
      } else {
        throw new Error(result.error || 'Failed to get table data');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      console.error('❌ Failed to get table data:', errorMessage);
      return null;
    }
  }, []);

  // Initialize settings on mount
  useEffect(() => {
    if (!initialized) {
      initializeSettings();
    }
  }, [initialized, initializeSettings]);

  return {
    settings,
    loading,
    error,
    initialized,
    initializeSettings,
    initializeDatabase,
    getUiSetting,
    setUiSetting,
    getAllUiSettings,
    getDatabaseInfo,
    listBackupFiles,
    migrateJsonToDatabase,
    createJsonBackup,
    restoreFromJsonBackup,
    getMigrationStatus,
    enableDatabaseMode,
    getDatabasePreview,
    getDatabaseTables,
    getTableData,
    setLoading,
    setError,
  };
} 