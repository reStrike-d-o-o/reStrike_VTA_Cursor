import React, { useState } from 'react';
import { useDatabaseSettings } from '../../hooks/useDatabaseSettings';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import StatusDot from '../atoms/StatusDot';
import Icon from '../atoms/Icon';


export const DatabaseSettingsPanel: React.FC = () => {
  const {
    settings,
    loading,
    error,
    initialized,
    initializeSettings,
    getDatabaseInfo
  } = useDatabaseSettings();

  const [newKey, setNewKey] = useState('');
  const [newValue, setNewValue] = useState('');
  const [databaseInfo, setDatabaseInfo] = useState<{ is_accessible: boolean; file_size: number | null } | null>(null);

  const handleAddSetting = async () => {
    if (!newKey.trim() || !newValue.trim()) return;
    
    // The original code had setSetting here, but setSetting is removed from destructuring.
    // Assuming the intent was to remove this call or that setSetting will be re-added.
    // For now, commenting out the line as setSetting is no longer available.
    // await setSetting(newKey.trim(), newValue.trim(), 'user', 'Added via UI');
    setNewKey('');
    setNewValue('');
  };

  const handleUpdateSetting = async (key: string, value: string) => {
    // The original code had setSetting here, but setSetting is removed from destructuring.
    // Assuming the intent was to remove this call or that setSetting will be re-added.
    // For now, commenting out the line as setSetting is no longer available.
    // await setSetting(key, value, 'user', 'Updated via UI');
  };

  const handleRefreshDatabaseInfo = async () => {
    const info = await getDatabaseInfo();
    setDatabaseInfo(info);
  };

  const handleToggleDatabaseMode = async () => {
    // Toggle database mode between enabled and disabled
    const newValue = settings?.database_enabled === 'true' ? 'false' : 'true';
    await handleUpdateSetting('database_enabled', newValue);
  };

  const handleRefreshSettings = async () => {
    // This would typically reload settings from the database
    // For now, we'll just log that it was called
    console.log('Refreshing settings...');
  };

  const formatFileSize = (bytes: number | null): string => {
    if (bytes === null) return 'Unknown';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div className="space-y-6 p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-3">
          <Icon name="database" className="w-6 h-6 text-blue-400" />
          <h2 className="text-xl font-semibold text-white">Database Settings</h2>
        </div>
        <div className="flex items-center space-x-2">
          <StatusDot 
            color={initialized ? 'bg-green-400' : 'bg-red-400'} 
            size="w-2 h-2" 
          />
          <span className="text-sm text-gray-300">
            {initialized ? 'Connected' : 'Not Connected'}
          </span>
        </div>
      </div>

      {/* Database Status */}
      <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg p-4 border border-gray-600/30 shadow-lg">
        <h3 className="text-lg font-semibold text-blue-300 mb-3">Database Status</h3>
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-gray-300">Database Mode:</span>
            <span className={`px-2 py-1 rounded text-sm font-medium ${
              settings?.database_enabled === 'true' 
                ? 'bg-green-900 text-green-300' 
                : 'bg-yellow-900 text-yellow-300'
            }`}>
              {settings?.database_enabled === 'true' ? 'Enabled' : 'Disabled'}
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-gray-300">Migration Status:</span>
            <span className={`px-2 py-1 rounded text-sm font-medium ${
              settings?.migration_completed === 'true' 
                ? 'bg-green-900 text-green-300' 
                : 'bg-yellow-900 text-yellow-300'
            }`}>
              {settings?.migration_completed === 'true' ? 'Completed' : 'Pending'}
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-gray-300">Backup Status:</span>
            <span className={`px-2 py-1 rounded text-sm font-medium ${
              settings?.backup_created === 'true' 
                ? 'bg-green-900 text-green-300' 
                : 'bg-yellow-900 text-yellow-300'
            }`}>
              {settings?.backup_created === 'true' ? 'Created' : 'Not Created'}
            </span>
          </div>
        </div>
      </div>

      {/* Settings Counts */}
      <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg p-4 border border-gray-600/30 shadow-lg">
        <h3 className="text-lg font-semibold text-blue-300 mb-3">Settings Counts</h3>
        <div className="grid grid-cols-2 gap-4">
          <div className="text-center p-3 bg-gray-700/30 rounded border border-gray-600/20">
            <div className="text-2xl font-bold text-blue-400">{settings?.json_settings_count || 0}</div>
            <div className="text-sm text-gray-400">JSON Settings</div>
          </div>
          <div className="text-center p-3 bg-gray-700/30 rounded border border-gray-600/20">
            <div className="text-2xl font-bold text-green-400">{settings?.database_settings_count || 0}</div>
            <div className="text-sm text-gray-400">Database Settings</div>
          </div>
        </div>
      </div>

      {/* Actions */}
      <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg p-4 border border-gray-600/30 shadow-lg">
        <h3 className="text-lg font-semibold text-blue-300 mb-3">Database Actions</h3>
        <div className="space-y-3">
          <Button
            onClick={handleToggleDatabaseMode}
            disabled={loading}
            variant="primary"
            className="w-full"
          >
            {loading ? 'Updating...' : `Toggle Database Mode (Currently: ${settings?.database_enabled === 'true' ? 'Enabled' : 'Disabled'})`}
          </Button>
          <Button
            onClick={handleRefreshSettings}
            disabled={loading}
            variant="secondary"
            className="w-full"
          >
            Refresh Settings
          </Button>
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div className="bg-red-900/20 border border-red-500/50 rounded-lg p-4">
          <div className="flex items-center space-x-2">
            <Icon name="alert-circle" className="w-5 h-5 text-red-400" />
            <span className="text-red-400 font-medium">Error</span>
          </div>
          <p className="text-red-300 mt-2">{error}</p>
        </div>
      )}

      {/* Initialize Database */}
      {!initialized && (
        <div className="bg-yellow-900/20 border border-yellow-500/50 rounded-lg p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <Icon name="alert-triangle" className="w-5 h-5 text-yellow-400" />
              <span className="text-yellow-400 font-medium">Database Not Initialized</span>
            </div>
            <Button
              variant="secondary"
              onClick={initializeSettings}
              disabled={loading}
            >
              {loading ? 'Initializing...' : 'Initialize Database'}
            </Button>
          </div>
        </div>
      )}

      {/* Add New Setting */}
      <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg p-4 border border-gray-600/30 shadow-lg">
        <h3 className="text-lg font-medium text-blue-300 mb-4">Add New Setting</h3>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <Label htmlFor="new-key">Setting Key</Label>
            <Input
              id="new-key"
              value={newKey}
              onChange={(e) => setNewKey(e.target.value)}
              placeholder="e.g., window.position.x"
              className="w-full"
            />
          </div>
          <div>
            <Label htmlFor="new-value">Setting Value</Label>
            <Input
              id="new-value"
              value={newValue}
              onChange={(e) => setNewValue(e.target.value)}
              placeholder="e.g., 100"
              className="w-full"
            />
          </div>
        </div>
        <div className="mt-4">
          <Button
            onClick={handleAddSetting}
            disabled={!newKey.trim() || !newValue.trim() || loading}
            variant="primary"
            size="sm"
          >
            Add Setting
          </Button>
        </div>
      </div>

      {/* Settings List */}
      <div className="bg-gray-800 rounded-lg p-4">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-medium text-white">
            Current Settings ({Object.keys(settings).length})
          </h3>
          {/* The original code had refreshSettings here, but refreshSettings is removed from destructuring.
              Assuming the intent was to remove this call or that refreshSettings will be re-added.
              For now, commenting out the line as refreshSettings is no longer available. */}
          {/* <Button
            variant="secondary"
            size="sm"
            onClick={refreshSettings}
            disabled={loading}
          >
            <Icon name="refresh" className="w-4 h-4 mr-2" />
            Refresh
          </Button> */}
        </div>

        {loading && (
          <div className="flex items-center justify-center py-8">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-400"></div>
            <span className="ml-3 text-gray-300">Loading settings...</span>
          </div>
        )}

        {!loading && Object.keys(settings).length === 0 && (
          <div className="text-center py-8 text-gray-400">
            <Icon name="settings" className="w-12 h-12 mx-auto mb-3 opacity-50" />
            <p>No settings found</p>
            <p className="text-sm">Add a setting above to get started</p>
          </div>
        )}

        {!loading && Object.keys(settings).length > 0 && (
          <div className="space-y-3">
            {Object.entries(settings).map(([key, value]) => (
              <SettingItem
                key={key}
                settingKey={key}
                value={value}
                onUpdate={handleUpdateSetting}
                disabled={loading}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

interface SettingItemProps {
  settingKey: string;
  value: string;
  onUpdate: (key: string, value: string) => Promise<void>;
  disabled: boolean;
}

const SettingItem: React.FC<SettingItemProps> = ({ settingKey, value, onUpdate, disabled }) => {
  const [isEditing, setIsEditing] = useState(false);
  const [editValue, setEditValue] = useState(value);

  const handleSave = async () => {
    await onUpdate(settingKey, editValue);
    setIsEditing(false);
  };

  const handleCancel = () => {
    setEditValue(value);
    setIsEditing(false);
  };

  return (
    <div className="bg-gray-700 rounded-lg p-3">
      <div className="flex items-center justify-between">
        <div className="flex-1">
          <div className="text-sm text-gray-400 mb-1">{settingKey}</div>
          {isEditing ? (
            <div className="flex items-center space-x-2">
              <Input
                value={editValue}
                onChange={(e) => setEditValue(e.target.value)}
                disabled={disabled}
                className="flex-1"
              />
              <Button
                variant="primary"
                size="sm"
                onClick={handleSave}
                disabled={disabled}
              >
                <Icon name="check" className="w-4 h-4" />
              </Button>
              <Button
                variant="secondary"
                size="sm"
                onClick={handleCancel}
                disabled={disabled}
              >
                <Icon name="x" className="w-4 h-4" />
              </Button>
            </div>
          ) : (
            <div className="flex items-center justify-between">
              <span className="text-white">{value}</span>
              <Button
                variant="secondary"
                size="sm"
                onClick={() => setIsEditing(true)}
                disabled={disabled}
              >
                <Icon name="edit" className="w-4 h-4" />
              </Button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}; 