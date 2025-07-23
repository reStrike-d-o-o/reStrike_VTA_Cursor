import React, { useState, useEffect } from 'react';
import Button from '../atoms/Button';
import StatusDot from '../atoms/StatusDot';
import { useDatabaseSettings } from '../../hooks/useDatabaseSettings';

interface MigrationStatus {
  database_enabled: boolean;
  json_fallback_enabled: boolean;
  migration_completed: boolean;
  last_migration?: string;
  settings_count: number;
}

interface MigrationResult {
  total_settings: number;
  migrated_settings: number;
  failed_settings: number;
  success_rate: number;
  errors: string[];
}

type TabType = 'backup' | 'migration' | 'status';

const DatabaseMigrationPanel: React.FC = () => {
  const [activeTab, setActiveTab] = useState<TabType>('backup');
  const [migrationStatus, setMigrationStatus] = useState<MigrationStatus | null>(null);
  const [migrationResult, setMigrationResult] = useState<MigrationResult | null>(null);
  const [isMigrating, setIsMigrating] = useState(false);
  const [isCreatingBackup, setIsCreatingBackup] = useState(false);
  const [backupPath, setBackupPath] = useState<string>('');
  const [restorePath, setRestorePath] = useState<string>('');
  const [isRestoring, setIsRestoring] = useState(false);
  const [databaseMode, setDatabaseMode] = useState<boolean>(true);

  const { 
    initializeDatabase, 
    getDatabaseInfo, 
    getAllSettings, 
    setSetting,
    error: dbError,
    loading: dbLoading,
    restoreBackupWithDialog
  } = useDatabaseSettings();

  // Load migration status on component mount
  useEffect(() => {
    loadMigrationStatus();
  }, []);

  const loadMigrationStatus = async () => {
    try {
      const response = await window.__TAURI__.core.invoke('get_migration_status');
      if (response.success) {
        setMigrationStatus(response.status);
        setDatabaseMode(response.status.database_enabled);
      }
    } catch (error) {
      console.error('Failed to load migration status:', error);
    }
  };

  const handleMigration = async () => {
    setIsMigrating(true);
    setMigrationResult(null);
    
    try {
      // Step 1: Create backup
      console.log('Creating JSON backup...');
      const backupResponse = await window.__TAURI__.core.invoke('create_json_backup');
      if (backupResponse.success) {
        setBackupPath(backupResponse.backup_path);
        console.log('Backup created:', backupResponse.backup_path);
      }

      // Step 2: Perform migration
      console.log('Starting migration...');
      const migrationResponse = await window.__TAURI__.core.invoke('migrate_json_to_database');
      
      if (migrationResponse.success) {
        setMigrationResult(migrationResponse.result);
        console.log('Migration completed:', migrationResponse.result);
        
        // Step 3: Refresh status
        await loadMigrationStatus();
        
        // Step 4: Refresh database settings
        await getAllSettings();
      } else {
        console.error('Migration failed:', migrationResponse.error);
      }
    } catch (error) {
      console.error('Migration error:', error);
    } finally {
      setIsMigrating(false);
    }
  };

  const handleCreateBackup = async () => {
    setIsCreatingBackup(true);
    
    try {
      const response = await window.__TAURI__.core.invoke('create_json_backup');
      if (response.success) {
        setBackupPath(response.backup_path);
        console.log('Backup created:', response.backup_path);
      } else {
        console.error('Backup failed:', response.error);
      }
    } catch (error) {
      console.error('Backup error:', error);
    } finally {
      setIsCreatingBackup(false);
    }
  };

  const handleRestore = async () => {
    if (!restorePath.trim()) {
      alert('Please enter a backup file path');
      return;
    }

    setIsRestoring(true);
    
    try {
      const response = await window.__TAURI__.core.invoke('restore_from_json_backup', {
        backupPath: restorePath
      });
      
      if (response.success) {
        console.log('Restore completed:', response.message);
        await loadMigrationStatus();
      } else {
        console.error('Restore failed:', response.error);
      }
    } catch (error) {
      console.error('Restore error:', error);
    } finally {
      setIsRestoring(false);
    }
  };

  const handleRestoreWithDialog = async () => {
    setIsRestoring(true);
    
    try {
      const result = await restoreBackupWithDialog();
      if (result.success) {
        console.log('Backup restored successfully:', result.message);
        await loadMigrationStatus();
      } else {
        console.error('Restore failed:', result.error);
      }
    } catch (error) {
      console.error('Restore error:', error);
    } finally {
      setIsRestoring(false);
    }
  };

  const handleToggleDatabaseMode = async (enabled: boolean) => {
    try {
      const response = await window.__TAURI__.core.invoke('enable_database_mode', {
        enabled
      });
      
      if (response.success) {
        setDatabaseMode(enabled);
        console.log(response.message);
        await loadMigrationStatus();
      } else {
        console.error('Failed to toggle database mode:', response.error);
      }
    } catch (error) {
      console.error('Toggle database mode error:', error);
    }
  };

  const getStatusColor = (): string => {
    if (!migrationStatus) return 'gray';
    if (migrationStatus.migration_completed && migrationStatus.database_enabled) return 'green';
    if (migrationStatus.migration_completed) return 'yellow';
    return 'red';
  };

  const getStatusText = (): string => {
    if (!migrationStatus) return 'Unknown';
    if (migrationStatus.migration_completed && migrationStatus.database_enabled) return 'Database Active';
    if (migrationStatus.migration_completed) return 'Migrated (JSON Fallback)';
    return 'JSON Only';
  };

  const renderTabButton = (tab: TabType, label: string) => (
    <button
      key={tab}
      onClick={() => setActiveTab(tab)}
      className={`px-4 py-2 rounded-lg font-medium transition-all duration-200 ${
        activeTab === tab
          ? 'bg-blue-600 text-white shadow-lg'
          : 'bg-gray-700 text-gray-300 hover:bg-gray-600 hover:text-white'
      }`}
    >
      {label}
    </button>
  );

  const renderBackupTab = () => (
    <div className="space-y-4">
      <div className="bg-yellow-900/20 border border-yellow-500/50 rounded-lg p-4">
        <p className="text-yellow-300 text-sm">
          Create backups of your JSON settings or restore from a previous backup.
        </p>
      </div>

      <div className="space-y-4">
        <div>
          <h4 className="font-medium text-white mb-2">Create Backup</h4>
          <div className="flex gap-4">
            <Button
              onClick={handleCreateBackup}
              disabled={isCreatingBackup}
              className="bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded-lg"
            >
              {isCreatingBackup ? 'Creating...' : 'Create Backup'}
            </Button>
            {backupPath && (
              <div className="text-sm text-gray-400">
                Last backup: {backupPath}
              </div>
            )}
          </div>
        </div>

        <div>
          <h4 className="font-medium text-white mb-2">Restore from Backup</h4>
          <div className="flex gap-4">
            <Button
              onClick={handleRestoreWithDialog}
              disabled={isRestoring}
              className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg"
            >
              {isRestoring ? 'Restoring...' : 'Browse & Restore'}
            </Button>
            <div className="text-sm text-gray-400">
              Click to open file dialog and select backup file
            </div>
          </div>
        </div>
      </div>
    </div>
  );

  const renderMigrationTab = () => (
    <div className="space-y-4">
      <div className="bg-blue-900/20 border border-blue-500/50 rounded-lg p-4">
        <p className="text-blue-300 text-sm">
          This will migrate all JSON-based settings to the SQLite database. 
          A backup will be created automatically before migration.
        </p>
      </div>

      <div className="flex gap-4">
        <Button
          onClick={handleMigration}
          disabled={isMigrating}
          className="bg-blue-600 hover:bg-blue-700 text-white px-6 py-2 rounded-lg"
        >
          {isMigrating ? 'Migrating...' : 'Start Migration'}
        </Button>
        
        <Button
          onClick={() => handleToggleDatabaseMode(!databaseMode)}
          className={`px-6 py-2 rounded-lg ${
            databaseMode 
              ? 'bg-yellow-600 hover:bg-yellow-700 text-white' 
              : 'bg-green-600 hover:bg-green-700 text-white'
          }`}
        >
          {databaseMode ? 'Disable Database' : 'Enable Database'}
        </Button>
      </div>

      {migrationResult && (
        <div className="bg-gray-700 border border-gray-600 rounded-lg p-4">
          <h4 className="font-medium text-white mb-3">Migration Results</h4>
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span className="text-gray-400">Total Settings:</span>
              <span className="ml-2 font-medium text-white">{migrationResult.total_settings}</span>
            </div>
            <div>
              <span className="text-gray-400">Migrated:</span>
              <span className="ml-2 font-medium text-green-400">{migrationResult.migrated_settings}</span>
            </div>
            <div>
              <span className="text-gray-400">Failed:</span>
              <span className="ml-2 font-medium text-red-400">{migrationResult.failed_settings}</span>
            </div>
            <div>
              <span className="text-gray-400">Success Rate:</span>
              <span className="ml-2 font-medium text-white">{(migrationResult.success_rate * 100).toFixed(1)}%</span>
            </div>
          </div>
          
          {migrationResult.errors.length > 0 && (
            <div className="mt-4">
              <h5 className="font-medium text-red-400 mb-2">Errors:</h5>
              <div className="bg-red-900/20 border border-red-500/50 rounded p-3 max-h-32 overflow-y-auto">
                {migrationResult.errors.map((error, index) => (
                  <div key={index} className="text-red-300 text-sm mb-1">
                    • {error}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );

  const renderStatusTab = () => (
    <div className="space-y-4">
      <div className="grid grid-cols-2 gap-6">
        <div className="bg-gray-700 border border-gray-600 rounded-lg p-4">
          <h4 className="font-medium text-white mb-3">Migration Status</h4>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-400">Database Enabled:</span>
              <StatusDot 
                color={migrationStatus?.database_enabled ? 'green' : 'red'} 
                size="sm" 
              />
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">JSON Fallback:</span>
              <StatusDot 
                color={migrationStatus?.json_fallback_enabled ? 'green' : 'red'} 
                size="sm" 
              />
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Migration Completed:</span>
              <StatusDot 
                color={migrationStatus?.migration_completed ? 'green' : 'red'} 
                size="sm" 
              />
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Settings Count:</span>
              <span className="font-medium text-white">{migrationStatus?.settings_count || 0}</span>
            </div>
          </div>
        </div>

        <div className="bg-gray-700 border border-gray-600 rounded-lg p-4">
          <h4 className="font-medium text-white mb-3">Database Info</h4>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-400">Accessible:</span>
              <StatusDot 
                color={dbError ? 'red' : 'green'} 
                size="sm" 
              />
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Loading:</span>
              <StatusDot 
                color={dbLoading ? 'yellow' : 'green'} 
                size="sm" 
              />
            </div>
          </div>
        </div>
      </div>

      <Button
        onClick={loadMigrationStatus}
        className="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded-lg"
      >
        Refresh Status
      </Button>
    </div>
  );

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-white">Database Migration</h2>
        <div className="flex items-center gap-2">
          <StatusDot color={getStatusColor()} size="md" />
          <span className="text-sm text-gray-300">{getStatusText()}</span>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="flex gap-2 bg-gray-800 rounded-lg p-2">
        {renderTabButton('backup', 'Backup & Restore')}
        {renderTabButton('migration', 'Migration')}
        {renderTabButton('status', 'Status')}
      </div>

      {/* Tab Content */}
      <div className="bg-gray-800 rounded-lg p-4">
        {activeTab === 'backup' && renderBackupTab()}
        {activeTab === 'migration' && renderMigrationTab()}
        {activeTab === 'status' && renderStatusTab()}
      </div>

      {dbError && (
        <div className="bg-red-900/20 border border-red-500/50 rounded-lg p-4">
          <h4 className="font-medium text-red-400 mb-2">Database Error</h4>
          <p className="text-red-300 text-sm">{dbError}</p>
        </div>
      )}
    </div>
  );
};

export default DatabaseMigrationPanel; 