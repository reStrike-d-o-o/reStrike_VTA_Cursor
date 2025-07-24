import React, { useState } from 'react';
import { useDatabaseSettings } from '../../hooks/useDatabaseSettings';
import Button from '../atoms/Button';
import ConnectDriveButton from '../atoms/ConnectDriveButton';
import Icon from '../atoms/Icon';

export const DatabaseManagementPanel: React.FC = () => {
  const {
    settings,
    loading,
    error,
    initialized,
    initializeSettings,
    getUiSetting,
    setUiSetting,
    getAllUiSettings,
    getDatabaseInfo,
  } = useDatabaseSettings();

  const [databaseInfo, setDatabaseInfo] = useState<any>(null);

  const handleUpdateSetting = async (key: string, value: string) => {
    try {
      await setUiSetting(key, value);
      console.log('✅ Setting updated:', key, value);
    } catch (error) {
      console.error('❌ Failed to update setting:', error);
    }
  };

  const handleToggleSetting = async (key: string, currentValue: string) => {
    const newValue = currentValue === 'true' ? 'false' : 'true';
    await handleUpdateSetting(key, newValue);
  };

  const handleRefreshDatabaseInfo = async () => {
    try {
      const info = await getDatabaseInfo();
      setDatabaseInfo(info);
    } catch (error) {
      console.error('❌ Failed to get database info:', error);
    }
  };

  const formatFileSize = (bytes: number | null): string => {
    if (bytes === null || bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const isBooleanValue = (value: string): boolean => {
    return value === 'true' || value === 'false';
  };

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-white">Database Management</h2>
          <p className="text-sm text-gray-400">Manage application settings and database</p>
        </div>
        <ConnectDriveButton />
      </div>

      {/* Error Display */}
      {error && (
        <div className="bg-red-900/20 border border-red-500/50 rounded-lg p-3">
          <span className="text-red-400 font-medium">Error</span>
          <p className="text-red-300 mt-1 text-sm">{error}</p>
        </div>
      )}

      {/* Initialize Database */}
      {!initialized && (
        <div className="bg-yellow-900/20 border border-yellow-500/50 rounded-lg p-3">
          <div className="flex items-center justify-between">
            <span className="text-yellow-400 font-medium">Database Not Initialized</span>
            <Button
              variant="secondary"
              size="sm"
              onClick={initializeSettings}
              disabled={loading}
            >
              {loading ? 'Initializing...' : 'Initialize'}
            </Button>
          </div>
        </div>
      )}

      {/* Database Info */}
      <div className="bg-gray-800 rounded-lg p-3">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-base font-medium text-white">Database Information</h3>
          <Button
            variant="secondary"
            size="sm"
            onClick={handleRefreshDatabaseInfo}
            disabled={loading}
          >
            <Icon name="refresh" className="w-4 h-4 mr-2" />
            Refresh
          </Button>
        </div>

        {databaseInfo ? (
          <div className="grid grid-cols-2 gap-3 text-sm">
            <div className="flex items-center justify-between">
              <span className="text-gray-400">Database Size:</span>
              <span className="text-white">{formatFileSize(databaseInfo.database_size)}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-400">Settings Count:</span>
              <span className="text-white">{databaseInfo.settings_count || 0}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-400">Last Modified:</span>
              <span className="text-white">{databaseInfo.last_modified || 'Unknown'}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-400">Status:</span>
              <span className="text-white">{databaseInfo.status || 'Unknown'}</span>
            </div>
          </div>
        ) : (
          <div className="text-center py-4">
            <p className="text-gray-400 text-sm">No database information available</p>
            <p className="text-xs mt-1">Click refresh to load database info</p>
          </div>
        )}
      </div>

      {/* Database Features */}
      <div className="bg-gray-800 rounded-lg p-3">
        <h3 className="text-base font-medium text-white mb-3">Database Features</h3>
        <div className="grid grid-cols-2 gap-3">
          <div className="flex items-center space-x-2 p-2 bg-gray-700 rounded">
            <div className="w-4 h-4 bg-blue-400 rounded"></div>
            <div>
              <div className="text-white text-sm font-medium">UI Settings</div>
              <div className="text-gray-400 text-xs">Persistent settings</div>
            </div>
          </div>
          <div className="flex items-center space-x-2 p-2 bg-gray-700 rounded">
            <div className="w-4 h-4 bg-green-400 rounded"></div>
            <div>
              <div className="text-white text-sm font-medium">Data Integrity</div>
              <div className="text-gray-400 text-xs">ACID compliance</div>
            </div>
          </div>
          <div className="flex items-center space-x-2 p-2 bg-gray-700 rounded">
            <div className="w-4 h-4 bg-yellow-400 rounded"></div>
            <div>
              <div className="text-white text-sm font-medium">Change Tracking</div>
              <div className="text-gray-400 text-xs">Audit trail</div>
            </div>
          </div>
          <div className="flex items-center space-x-2 p-2 bg-gray-700 rounded">
            <div className="w-4 h-4 bg-purple-400 rounded"></div>
            <div>
              <div className="text-white text-sm font-medium">Real-time Sync</div>
              <div className="text-gray-400 text-xs">Instant updates</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}; 