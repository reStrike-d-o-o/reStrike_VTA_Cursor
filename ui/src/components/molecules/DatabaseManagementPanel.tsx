import React, { useState } from 'react';
import { useDatabaseSettings } from '../../hooks/useDatabaseSettings';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import StatusDot from '../atoms/StatusDot';
import Toggle from '../atoms/Toggle';

export const DatabaseManagementPanel: React.FC = () => {
  const {
    settings,
    loading,
    error,
    initialized,
    initializeSettings,
    setSetting,
    refreshSettings,
    getDatabaseInfo
  } = useDatabaseSettings();

  const [newKey, setNewKey] = useState('');
  const [newValue, setNewValue] = useState('');
  const [databaseInfo, setDatabaseInfo] = useState<{ is_accessible: boolean; file_size: number | null } | null>(null);

  const handleAddSetting = async () => {
    if (!newKey.trim() || !newValue.trim()) return;
    
    await setSetting(newKey.trim(), newValue.trim(), 'user', 'Added via UI');
    setNewKey('');
    setNewValue('');
  };

  const handleUpdateSetting = async (key: string, value: string) => {
    await setSetting(key, value, 'user', 'Updated via UI');
  };

  const handleToggleSetting = async (key: string, currentValue: string) => {
    const newValue = currentValue === 'true' ? 'false' : 'true';
    await setSetting(key, newValue, 'user', 'Toggled via UI');
  };

  const handleRefreshDatabaseInfo = async () => {
    const info = await getDatabaseInfo();
    setDatabaseInfo(info);
  };

  const formatFileSize = (bytes: number | null): string => {
    if (bytes === null) return 'Unknown';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const isBooleanValue = (value: string): boolean => {
    return value === 'true' || value === 'false';
  };

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-white">Database Management</h2>
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

      {/* Database Information */}
      <div className="bg-gray-800 rounded-lg p-3">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-base font-medium text-white">Database Information</h3>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleRefreshDatabaseInfo}
            disabled={loading}
          >
            Refresh
          </Button>
        </div>
        
        {databaseInfo && (
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span className="text-gray-400">Status:</span>
              <div className="flex items-center mt-1">
                <StatusDot 
                  color={databaseInfo.is_accessible ? 'bg-green-400' : 'bg-red-400'} 
                  size="w-2 h-2" 
                />
                <span className="ml-2 text-white">
                  {databaseInfo.is_accessible ? 'Accessible' : 'Not Accessible'}
                </span>
              </div>
            </div>
            <div>
              <span className="text-gray-400">File Size:</span>
              <div className="text-white mt-1">
                {formatFileSize(databaseInfo.file_size)}
              </div>
            </div>
          </div>
        )}

        {!databaseInfo && (
          <div className="text-center py-4 text-gray-400">
            <p className="text-sm">No database information available</p>
            <p className="text-xs mt-1">Click refresh to load database info</p>
          </div>
        )}
      </div>

      {/* Add New Setting */}
      <div className="bg-gray-800 rounded-lg p-3">
        <h3 className="text-base font-medium text-white mb-3">Add New Setting</h3>
        <div className="grid grid-cols-2 gap-3">
          <div>
            <Label htmlFor="new-key" className="text-sm">Setting Key</Label>
            <Input
              id="new-key"
              value={newKey}
              onChange={(e) => setNewKey(e.target.value)}
              placeholder="e.g., window.position.x"
              disabled={loading}
              className="text-sm"
            />
          </div>
          <div>
            <Label htmlFor="new-value" className="text-sm">Setting Value</Label>
            <Input
              id="new-value"
              value={newValue}
              onChange={(e) => setNewValue(e.target.value)}
              placeholder="e.g., 150"
              disabled={loading}
              className="text-sm"
            />
          </div>
        </div>
        <Button
          variant="primary"
          size="sm"
          onClick={handleAddSetting}
          disabled={loading || !newKey.trim() || !newValue.trim()}
          className="mt-3"
        >
          Add Setting
        </Button>
      </div>

      {/* Settings List */}
      <div className="bg-gray-800 rounded-lg p-3">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-base font-medium text-white">
            Current Settings ({Object.keys(settings).length})
          </h3>
          <Button
            variant="ghost"
            size="sm"
            onClick={refreshSettings}
            disabled={loading}
          >
            Refresh
          </Button>
        </div>

        {loading && (
          <div className="flex items-center justify-center py-6">
            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-400"></div>
            <span className="ml-2 text-gray-300 text-sm">Loading settings...</span>
          </div>
        )}

        {!loading && Object.keys(settings).length === 0 && (
          <div className="text-center py-6 text-gray-400">
            <p className="text-sm">No settings found</p>
            <p className="text-xs mt-1">Add a setting above to get started</p>
          </div>
        )}

        {!loading && Object.keys(settings).length > 0 && (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {Object.entries(settings).map(([key, value]) => (
              <SettingItem
                key={key}
                settingKey={key}
                value={value}
                onUpdate={handleUpdateSetting}
                onToggle={handleToggleSetting}
                disabled={loading}
              />
            ))}
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

interface SettingItemProps {
  settingKey: string;
  value: string;
  onUpdate: (key: string, value: string) => Promise<void>;
  onToggle: (key: string, value: string) => Promise<void>;
  disabled: boolean;
}

const SettingItem: React.FC<SettingItemProps> = ({ settingKey, value, onUpdate, onToggle, disabled }) => {
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

  const isBoolean = value === 'true' || value === 'false';

  return (
    <div className="bg-gray-700 rounded p-2">
      <div className="flex items-center justify-between">
        <div className="flex-1 min-w-0">
          <div className="text-xs text-gray-400 truncate">{settingKey}</div>
          {isEditing ? (
            <div className="flex items-center space-x-1 mt-1">
              <Input
                value={editValue}
                onChange={(e) => setEditValue(e.target.value)}
                disabled={disabled}
                className="flex-1 text-sm"
              />
              <Button
                variant="success"
                size="sm"
                onClick={handleSave}
                disabled={disabled}
                className="px-2"
              >
                ✓
              </Button>
              <Button
                variant="danger"
                size="sm"
                onClick={handleCancel}
                disabled={disabled}
                className="px-2"
              >
                ✕
              </Button>
            </div>
          ) : (
            <div className="flex items-center justify-between mt-1">
              {isBoolean ? (
                <Toggle
                  checked={value === 'true'}
                  onChange={() => onToggle(settingKey, value)}
                  disabled={disabled}
                />
              ) : (
                <span className="text-white text-sm truncate">{value}</span>
              )}
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setIsEditing(true)}
                disabled={disabled}
                className="px-2 text-xs"
              >
                Edit
              </Button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}; 