/**
 * DatabaseManagementPanel
 * - Inspect tables, preview data, manage backups
 */
import React, { useEffect, useState } from 'react';
import { useDatabaseSettings } from '../../hooks/useDatabaseSettings';
import Button from '../atoms/Button';

export const DatabaseManagementPanel: React.FC = () => {
  const {
    settings,
    loading,
    error,
    initialized,
    initializeSettings,
    getDatabaseInfo,
  } = useDatabaseSettings();

  const [databaseInfo, setDatabaseInfo] = useState<any>(null);

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

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-white">Database Management</h2>
          <p className="text-sm text-gray-400">Manage application settings and database</p>
        </div>
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
      <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg p-3 border border-gray-600/30 shadow-lg">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-base font-medium text-white">Database Information</h3>
          <Button
            variant="secondary"
            size="sm"
            onClick={handleRefreshDatabaseInfo}
            disabled={loading}
          >
            Refresh
          </Button>
        </div>
        {databaseInfo ? (
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-300">Database Path:</span>
              <span className="text-gray-400 font-mono">{databaseInfo.path}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-300">File Size:</span>
              <span className="text-gray-400">{formatFileSize(databaseInfo.size)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-300">Tables:</span>
              <span className="text-gray-400">{databaseInfo.tables}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-300">Settings Count:</span>
              <span className="text-gray-400">{databaseInfo.settings_count}</span>
            </div>
          </div>
        ) : (
          <p className="text-gray-400 text-sm">Click Refresh to load database information</p>
        )}
      </div>
    </div>
  );
}; 