import React, { useState, useEffect } from 'react';
import Button from '../atoms/Button';
import StatusDot from '../atoms/StatusDot';
import { useDatabaseSettings } from '../../hooks/useDatabaseSettings';

interface MigrationStatus {
  database_enabled: boolean;
  migration_completed: boolean;
  backup_created: boolean;
  json_settings_count: number;
  database_settings_count: number;
}

interface BackupFileInfo {
  name: string;
  path: string;
  size: number;
  modified: string;
}

const DatabaseMigrationPanel: React.FC = () => {
  const {
    createJsonBackup,
    migrateJsonToDatabase,
    enableDatabaseMode,
    listBackupFiles,
    restoreFromJsonBackup,
    getMigrationStatus
  } = useDatabaseSettings();

  const [backupFiles, setBackupFiles] = useState<BackupFileInfo[]>([]);
  const [selectedBackup, setSelectedBackup] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'backup' | 'migration' | 'status'>('backup');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [migrationStatus, setMigrationStatus] = useState<MigrationStatus | null>(null);

  useEffect(() => {
    loadBackupFiles();
    loadMigrationStatus();
  }, []);

  const loadBackupFiles = async () => {
    const files = await listBackupFiles();
    setBackupFiles(files);
  };

  const loadMigrationStatus = async () => {
    try {
      const status = await getMigrationStatus();
      setMigrationStatus(status);
    } catch (error) {
      console.error('❌ Failed to load migration status:', error);
    }
  };

  const handleRestoreBackup = async (backupPath: string) => {
    setLoading(true);
    setError(null);
    setSuccess(null); // Clear previous success message
    try {
      const result = await restoreFromJsonBackup(backupPath);
      if (result.success) {
        setSuccess(result.message || 'Settings restored successfully.');
        // Refresh the backup files list and migration status after restore
        await loadBackupFiles();
        await loadMigrationStatus();
      } else {
        setError(result.error || 'Failed to restore backup');
        console.error('❌ Failed to restore backup:', result.error);
      }
    } catch (error) {
      setError('Error restoring backup');
      console.error('❌ Error restoring backup:', error);
    } finally {
      setLoading(false);
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
    <div className="space-y-4">
      {/* Tab Navigation */}
      <div className="flex border-b border-gray-200">
        <button
          onClick={() => setActiveTab('backup')}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            activeTab === 'backup'
              ? 'border-blue-500 text-blue-600'
              : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
          }`}
        >
          Backup & Restore
        </button>
        <button
          onClick={() => setActiveTab('migration')}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            activeTab === 'migration'
              ? 'border-blue-500 text-blue-600'
              : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
          }`}
        >
          Migration
        </button>
        <button
          onClick={() => setActiveTab('status')}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            activeTab === 'status'
              ? 'border-blue-500 text-blue-600'
              : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
          }`}
        >
          Status
        </button>
      </div>

      {/* Backup & Restore Tab */}
      {activeTab === 'backup' && (
        <div className="bg-[#181F26] rounded-lg p-4 mb-6 border border-gray-700 shadow">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-lg font-semibold text-blue-300">Backup & Restore</h3>
            <Button
              onClick={async () => {
                setLoading(true);
                setError(null);
                setSuccess(null); // Clear previous success message
                try {
                  await createJsonBackup();
                  await loadBackupFiles(); // Refresh the backup files list
                  await loadMigrationStatus();
                } catch (error) {
                  setError('Failed to create backup');
                  console.error('❌ Error creating backup:', error);
                } finally {
                  setLoading(false);
                }
              }}
              disabled={loading}
              variant="primary"
              size="sm"
            >
              {loading ? 'Creating...' : 'Create Backup'}
            </Button>
          </div>

          {/* Backup Files List */}
          <div className="max-h-64 overflow-y-auto border border-gray-700 rounded">
            <table className="min-w-full text-left text-sm text-gray-200">
              <thead className="bg-[#101820]">
                <tr>
                  <th className="px-3 py-2 font-semibold">File Name</th>
                  <th className="px-3 py-2 font-semibold">Size</th>
                  <th className="px-3 py-2 font-semibold">Modified</th>
                  <th className="px-3 py-2 font-semibold">Action</th>
                </tr>
              </thead>
              <tbody>
                {backupFiles.length === 0 ? (
                  <tr>
                    <td colSpan={4} className="px-3 py-2 text-gray-400 text-center">
                      No backup files found
                      <br />
                      <span className="text-xs">Create a backup to see files here</span>
                    </td>
                  </tr>
                ) : (
                  backupFiles.map((file, index) => (
                    <tr
                      key={index}
                      className="hover:bg-blue-900 transition-colors"
                    >
                      <td className="px-3 py-2 whitespace-nowrap">
                        {file.name}
                      </td>
                      <td className="px-3 py-2 whitespace-nowrap">
                        {formatFileSize(file.size)}
                      </td>
                      <td className="px-3 py-2 whitespace-nowrap">
                        {file.modified}
                      </td>
                      <td className="px-3 py-2 whitespace-nowrap">
                        <Button
                          onClick={() => handleRestoreBackup(file.path)}
                          variant="ghost"
                          size="sm"
                          className="text-blue-400 hover:text-blue-300"
                        >
                          Restore
                        </Button>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Migration Tab */}
      {activeTab === 'migration' && (
        <div className="bg-[#181F26] rounded-lg p-4 mb-6 border border-gray-700 shadow">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-lg font-semibold text-blue-300">Database Migration</h3>
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between p-3 bg-[#101820] rounded border border-gray-700">
              <div>
                <h4 className="text-sm font-medium text-gray-200">Migrate JSON to Database</h4>
                <p className="text-xs text-gray-400 mt-1">
                  Convert JSON settings to SQLite database format
                </p>
              </div>
              <Button
                onClick={async () => {
                  await migrateJsonToDatabase();
                  await loadMigrationStatus();
                }}
                disabled={loading || migrationStatus?.migration_completed}
                variant="primary"
                size="sm"
              >
                {migrationStatus?.migration_completed ? 'Completed' : 'Migrate'}
              </Button>
            </div>

            <div className="flex items-center justify-between p-3 bg-[#101820] rounded border border-gray-700">
              <div>
                <h4 className="text-sm font-medium text-gray-200">Enable Database Mode</h4>
                <p className="text-xs text-gray-400 mt-1">
                  Switch from JSON to database storage
                </p>
              </div>
              <Button
                onClick={async () => {
                  await enableDatabaseMode();
                  await loadMigrationStatus();
                }}
                disabled={loading || migrationStatus?.database_enabled}
                variant="primary"
                size="sm"
              >
                {migrationStatus?.database_enabled ? 'Enabled' : 'Enable'}
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Status Tab */}
      {activeTab === 'status' && (
        <div className="bg-[#181F26] rounded-lg p-4 mb-6 border border-gray-700 shadow">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-lg font-semibold text-blue-300">Migration Status</h3>
          </div>

          <div className="max-h-64 overflow-y-auto border border-gray-700 rounded">
            <table className="min-w-full text-left text-sm text-gray-200">
              <thead className="bg-[#101820]">
                <tr>
                  <th className="px-3 py-2 font-semibold">Status</th>
                  <th className="px-3 py-2 font-semibold">Value</th>
                </tr>
              </thead>
              <tbody>
                <tr className="hover:bg-blue-900 transition-colors">
                  <td className="px-3 py-2 whitespace-nowrap">Database Enabled:</td>
                  <td className="px-3 py-2 whitespace-nowrap">
                    <div className="flex items-center">
                      <StatusDot
                        color={migrationStatus?.database_enabled ? 'green' : 'red'}
                        className="mr-2"
                      />
                      <span className="text-sm font-medium">
                        {migrationStatus?.database_enabled ? 'Yes' : 'No'}
                      </span>
                    </div>
                  </td>
                </tr>
                <tr className="hover:bg-blue-900 transition-colors">
                  <td className="px-3 py-2 whitespace-nowrap">Migration Completed:</td>
                  <td className="px-3 py-2 whitespace-nowrap">
                    <div className="flex items-center">
                      <StatusDot
                        color={migrationStatus?.migration_completed ? 'green' : 'red'}
                        className="mr-2"
                      />
                      <span className="text-sm font-medium">
                        {migrationStatus?.migration_completed ? 'Yes' : 'No'}
                      </span>
                    </div>
                  </td>
                </tr>
                <tr className="hover:bg-blue-900 transition-colors">
                  <td className="px-3 py-2 whitespace-nowrap">Backup Created:</td>
                  <td className="px-3 py-2 whitespace-nowrap">
                    <div className="flex items-center">
                      <StatusDot
                        color={migrationStatus?.backup_created ? 'green' : 'red'}
                        className="mr-2"
                      />
                      <span className="text-sm font-medium">
                        {migrationStatus?.backup_created ? 'Yes' : 'No'}
                      </span>
                    </div>
                  </td>
                </tr>
                <tr className="hover:bg-blue-900 transition-colors">
                  <td className="px-3 py-2 whitespace-nowrap">JSON Settings Count:</td>
                  <td className="px-3 py-2 whitespace-nowrap">
                    <span className="text-sm font-medium">
                      {migrationStatus?.json_settings_count || 0}
                    </span>
                  </td>
                </tr>
                <tr className="hover:bg-blue-900 transition-colors">
                  <td className="px-3 py-2 whitespace-nowrap">Database Settings Count:</td>
                  <td className="px-3 py-2 whitespace-nowrap">
                    <span className="text-sm font-medium">
                      {migrationStatus?.database_settings_count || 0}
                    </span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      )}

      {error && (
        <div className="mb-3 p-2 bg-red-900/20 border border-red-700 rounded text-red-400 text-sm">
          {error}
        </div>
      )}
      {success && (
        <div className="mb-3 p-2 bg-green-900/20 border border-green-700 rounded text-green-400 text-sm">
          {success}
        </div>
      )}
    </div>
  );
};

export default DatabaseMigrationPanel; 