import React, { useState, useEffect } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import { StatusDot } from '../atoms/StatusDot';
import { Icon } from '../atoms/Icon';
import { useAppStore, ObsConnection } from '../../stores';
import { obsCommands, configCommands } from '../../utils/tauriCommands';

// Reconnection settings interface
interface ReconnectionSettings {
  autoReconnect: boolean;
  reconnectDelay: number;
  maxAttempts: number;
  statusMonitoring: boolean;
  statusInterval: number;
}

const WebSocketManager: React.FC = () => {
  const { obsConnections, addObsConnection, removeObsConnection, updateObsConnectionStatus, setActiveObsConnection, activeObsConnection } = useAppStore();
  
  const [isAdding, setIsAdding] = useState(false);
  const [editingConnection, setEditingConnection] = useState<string | null>(null);
  const [showReconnectionSettings, setShowReconnectionSettings] = useState(false);
  const [formData, setFormData] = useState({
    name: '',
    host: 'localhost',
    port: 4455,
    password: '',
    enabled: true,
  });
  const [reconnectionSettings, setReconnectionSettings] = useState<ReconnectionSettings>({
    autoReconnect: true,
    reconnectDelay: 5,
    maxAttempts: 5,
    statusMonitoring: true,
    statusInterval: 30,
  });
  const [error, setError] = useState<string | null>(null);
  const [isLoadingSettings, setIsLoadingSettings] = useState(false);

  const resetForm = () => {
    setFormData({
      name: '',
      host: 'localhost',
      port: 4455,
      password: '',
      enabled: true,
    });
    setError(null);
  };

  // Load connections from backend on component mount
  useEffect(() => {
    loadConnections();
    loadReconnectionSettings();
  }, []);

  // Load reconnection settings from configuration
  const loadReconnectionSettings = async () => {
    try {
      setIsLoadingSettings(true);
      const result = await configCommands.getSettings();
      if (result.success && result.data?.obs?.behavior) {
        const behavior = result.data.obs.behavior;
        setReconnectionSettings({
          autoReconnect: behavior.auto_reconnect ?? true,
          reconnectDelay: behavior.reconnect_delay ?? 5,
          maxAttempts: behavior.max_attempts ?? 5,
          statusMonitoring: behavior.status_monitoring ?? true,
          statusInterval: behavior.status_interval ?? 30,
        });
      }
    } catch (error) {
      console.error('Failed to load reconnection settings:', error);
    } finally {
      setIsLoadingSettings(false);
    }
  };

  // Save reconnection settings to configuration
  const saveReconnectionSettings = async () => {
    try {
      setIsLoadingSettings(true);
      const result = await configCommands.getSettings();
      if (result.success && result.data) {
        const updatedSettings = {
          ...result.data,
          obs: {
            ...result.data.obs,
            behavior: {
              ...result.data.obs.behavior,
              auto_reconnect: reconnectionSettings.autoReconnect,
              reconnect_delay: reconnectionSettings.reconnectDelay,
              max_attempts: reconnectionSettings.maxAttempts,
              status_monitoring: reconnectionSettings.statusMonitoring,
              status_interval: reconnectionSettings.statusInterval,
            },
          },
        };
        
        await configCommands.updateSettings(updatedSettings);
        console.log('Reconnection settings saved successfully');
      }
    } catch (error) {
      console.error('Failed to save reconnection settings:', error);
      setError(`Failed to save reconnection settings: ${error}`);
    } finally {
      setIsLoadingSettings(false);
    }
  };

  const loadConnections = async () => {
    try {
      // First try to get connections from configuration system
      const configResult = await configCommands.getSettings();
      if (configResult.success && configResult.data?.obs?.connections) {
        const configConnections = configResult.data.obs.connections.map((conn: any) => ({
          name: conn.name,
          host: conn.host,
          port: conn.port,
          password: conn.password,
          enabled: conn.enabled,
          status: 'Disconnected' as const, // Will be updated by status check
          error: undefined,
        }));
        
        // Update frontend store with configuration connections
        obsConnections.forEach(conn => removeObsConnection(conn.name));
        configConnections.forEach((conn: ObsConnection) => addObsConnection(conn));
        
        // Ensure all config connections are registered with the OBS plugin
        console.log('Syncing config connections to OBS plugin...');
        for (const conn of configConnections) {
          try {
            await obsCommands.addConnection({
              name: conn.name,
              host: conn.host,
              port: conn.port,
              password: conn.password,
              enabled: conn.enabled,
            });
            console.log(`Synced connection ${conn.name} to OBS plugin`);
          } catch (error) {
            console.log(`Connection ${conn.name} already exists in OBS plugin (expected)`);
          }
        }
      } else {
        // Fallback to direct OBS plugin query
        const result = await obsCommands.getConnections();
        if (result.success && result.data) {
          // Update store with connections from backend
          console.log('Loaded connections from backend:', result.data);
          const backendConnections = result.data.map((conn: any) => ({
            name: conn.name,
            host: conn.host,
            port: conn.port,
            password: conn.password,
            enabled: conn.enabled,
            status: conn.status || 'Disconnected',
            error: undefined,
          }));
          
          // Update frontend store
          obsConnections.forEach(conn => removeObsConnection(conn.name));
          backendConnections.forEach((conn: ObsConnection) => addObsConnection(conn));
        } else {
          // If no connections anywhere, initialize with empty state
          console.log('No connections found, initializing empty state');
          obsConnections.forEach(conn => removeObsConnection(conn.name));
        }
      }
    } catch (error) {
      console.error('Failed to load connections:', error);
    }
  };

  const handleAddConnection = async () => {
    if (!formData.name.trim()) {
      setError('Connection name is required');
      return;
    }

    if (obsConnections.some(c => c.name === formData.name)) {
      setError('Connection name already exists');
      return;
    }

    if (formData.port < 1 || formData.port > 65535) {
      setError('Port must be between 1 and 65535');
      return;
    }

    try {
      const result = await obsCommands.addConnection({
        name: formData.name,
        host: formData.host,
        port: formData.port,
        password: formData.password || undefined,
        enabled: formData.enabled,
      });

      if (result.success) {
        addObsConnection(formData);
        resetForm();
        setIsAdding(false);
      } else {
        setError(result.error || 'Failed to add connection');
      }
    } catch (error) {
      setError(`Failed to add connection: ${error}`);
    }
  };

  const handleEditConnection = (connection: ObsConnection) => {
    setEditingConnection(connection.name);
    setFormData({
      name: connection.name,
      host: connection.host,
      port: connection.port,
      password: connection.password || '',
      enabled: connection.enabled,
    });
  };

  const handleUpdateConnection = async () => {
    if (!editingConnection) return;

    if (!formData.name.trim()) {
      setError('Connection name is required');
      return;
    }

    if (formData.name !== editingConnection && obsConnections.some(c => c.name === formData.name)) {
      setError('Connection name already exists');
      return;
    }

    if (formData.port < 1 || formData.port > 65535) {
      setError('Port must be between 1 and 65535');
      return;
    }

    try {
      // Ensure password is preserved - if formData.password is empty but we're editing,
      // try to get the password from the original connection
      let finalPassword = formData.password;
      if (!finalPassword && editingConnection) {
        const originalConnection = obsConnections.find(c => c.name === editingConnection);
        if (originalConnection && originalConnection.password) {
          finalPassword = originalConnection.password;
        }
      }

      // First, remove the old connection from backend
      try {
        await obsCommands.removeConnection(editingConnection);
      } catch (error) {
        // Ignore errors if connection wasn't found
        console.log('Remove connection error (expected if not found):', error);
      }

      // Add the updated connection to backend
      const result = await obsCommands.addConnection({
        name: formData.name,
        host: formData.host,
        port: formData.port,
        password: finalPassword || undefined,
        enabled: formData.enabled,
      });

      if (result.success) {
        // Remove old connection from frontend store
        removeObsConnection(editingConnection);
        
        // Add updated connection to frontend store
        addObsConnection({
          ...formData,
          password: finalPassword,
        });
        
        resetForm();
        setEditingConnection(null);
      } else {
        setError(result.error || 'Failed to update connection');
      }
    } catch (error) {
      setError(`Failed to update connection: ${error}`);
    }
  };

  const handleDeleteConnection = (name: string) => {
    if (window.confirm(`Are you sure you want to delete the connection "${name}"?`)) {
      removeObsConnection(name);
    }
  };

  const handleConnect = async (connection: ObsConnection) => {
    updateObsConnectionStatus(connection.name, 'Connecting');
    
    try {
      const result = await obsCommands.connectToConnection(connection.name);
      
      if (result.success) {
        updateObsConnectionStatus(connection.name, 'Connected');
        setActiveObsConnection(connection.name);
        
        // Poll for status updates
        setTimeout(async () => {
          try {
            const statusResult = await obsCommands.getConnectionStatus(connection.name);
            if (statusResult.success && statusResult.data) {
              const status = statusResult.data.status as ObsConnection['status'];
              updateObsConnectionStatus(connection.name, status);
              if (status === 'Authenticated') {
                setActiveObsConnection(connection.name);
              }
            }
          } catch (error) {
            console.error('Failed to get connection status:', error);
          }
        }, 2000);
      } else {
        updateObsConnectionStatus(connection.name, 'Error', result.error || 'Connection failed');
      }
    } catch (error) {
      updateObsConnectionStatus(connection.name, 'Error', `Connection failed: ${error}`);
    }
  };

  const handleDisconnect = async (connection: ObsConnection) => {
    try {
      const result = await obsCommands.disconnect(connection.name);
      
      if (result.success) {
        updateObsConnectionStatus(connection.name, 'Disconnected');
        if (activeObsConnection === connection.name) {
          setActiveObsConnection(null);
        }
      } else {
        updateObsConnectionStatus(connection.name, 'Error', result.error || 'Disconnect failed');
      }
    } catch (error) {
      updateObsConnectionStatus(connection.name, 'Error', `Disconnect failed: ${error}`);
    }
  };

  const getStatusColor = (status: ObsConnection['status']) => {
    switch (status) {
      case 'Connected':
      case 'Authenticated':
        return 'bg-green-500';
      case 'Connecting':
      case 'Authenticating':
        return 'bg-yellow-500';
      case 'Error':
        return 'bg-red-500';
      default:
        return 'bg-gray-500';
    }
  };

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold">WebSocket Connections</h3>
        <Button
          onClick={() => {
            setIsAdding(true);
            setEditingConnection(null);
            resetForm();
          }}
          variant="primary"
          size="sm"
          disabled={isAdding || editingConnection !== null}
        >
          <Icon name="➕" />
          Add Connection
        </Button>
      </div>

      {/* Add/Edit Form */}
      {(isAdding || editingConnection) && (
        <div className="p-4 bg-gray-800 rounded-lg border border-gray-700">
          <h4 className="text-md font-medium mb-3">
            {editingConnection ? 'Edit Connection' : 'Add New Connection'}
          </h4>
          
          {error && (
            <div className="mb-3 p-2 bg-red-900/20 border border-red-700 rounded text-red-400 text-sm">
              {error}
            </div>
          )}

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <Label htmlFor="connection-name">Connection Name *</Label>
              <Input
                id="connection-name"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                placeholder="e.g., OBS_REC, OBS_STR"
                disabled={editingConnection !== null}
              />
            </div>

            <div>
              <Label htmlFor="connection-host">Host</Label>
              <Input
                id="connection-host"
                value={formData.host}
                onChange={(e) => setFormData({ ...formData, host: e.target.value })}
                placeholder="localhost"
              />
            </div>

            <div>
              <Label htmlFor="connection-port">Port</Label>
              <Input
                id="connection-port"
                type="number"
                value={formData.port}
                onChange={(e) => setFormData({ ...formData, port: parseInt(e.target.value) || 4455 })}
                placeholder="4455"
                min="1"
                max="65535"
              />
            </div>

            <div>
              <Label htmlFor="connection-password">Password (optional)</Label>
              <Input
                id="connection-password"
                type="password"
                value={formData.password}
                onChange={(e) => setFormData({ ...formData, password: e.target.value })}
                placeholder={editingConnection && obsConnections.find(c => c.name === editingConnection)?.password 
                  ? "Password is set (click to change)" 
                  : "Leave empty if no password"}
              />
            </div>

            <div className="flex items-center">
              <label className="flex items-center space-x-2">
                <input
                  type="checkbox"
                  checked={formData.enabled}
                  onChange={(e) => setFormData({ ...formData, enabled: e.target.checked })}
                  className="rounded border-gray-600 bg-gray-700 text-blue-500 focus:ring-blue-500"
                />
                <span className="text-sm">Enabled</span>
              </label>
            </div>
          </div>

          <div className="flex space-x-2 mt-4">
            <Button
              onClick={editingConnection ? handleUpdateConnection : handleAddConnection}
              variant="primary"
              size="sm"
            >
              {editingConnection ? 'Update' : 'Add'} Connection
            </Button>
            <Button
              onClick={() => {
                setIsAdding(false);
                setEditingConnection(null);
                resetForm();
              }}
              variant="secondary"
              size="sm"
            >
              Cancel
            </Button>
          </div>
        </div>
      )}

      {/* Connections List */}
      <div className="space-y-2">
        {obsConnections.length === 0 ? (
          <div className="p-4 bg-gray-800 rounded-lg text-center text-gray-400">
            No WebSocket connections configured
          </div>
        ) : (
          obsConnections.map((connection) => (
            <div
              key={connection.name}
              className={`p-4 bg-gray-800 rounded-lg border ${
                activeObsConnection === connection.name ? 'border-blue-500' : 'border-gray-700'
              }`}
            >
              <div className="flex items-center justify-between">
                <div className="flex-1">
                  <div className="flex items-center space-x-2 mb-2">
                    <h4 className="font-medium">{connection.name}</h4>
                    <StatusDot color={getStatusColor(connection.status)} />
                    <span className="text-sm text-gray-400">{connection.status}</span>
                    {activeObsConnection === connection.name && (
                      <span className="text-xs bg-blue-500 text-white px-2 py-1 rounded">Active</span>
                    )}
                  </div>
                  
                  <div className="text-sm text-gray-400 space-y-1">
                    <div>{connection.host}:{connection.port} (v5)</div>
                    {connection.password && <div>Password: {'•'.repeat(8)}</div>}
                    {connection.error && (
                      <div className="text-red-400">Error: {connection.error}</div>
                    )}
                  </div>
                </div>

                <div className="flex items-center space-x-2">
                  {connection.status === 'Connected' || connection.status === 'Authenticated' ? (
                    <Button
                      onClick={() => handleDisconnect(connection)}
                      variant="danger"
                      size="sm"
                    >
                      <Icon name="🔌" />
                      Disconnect
                    </Button>
                  ) : (
                    <Button
                      onClick={() => handleConnect(connection)}
                      variant="success"
                      size="sm"
                      disabled={!connection.enabled || connection.status === 'Connecting'}
                    >
                      <Icon name="🔗" />
                      Connect
                    </Button>
                  )}
                  
                  <Button
                    onClick={() => handleEditConnection(connection)}
                    variant="secondary"
                    size="sm"
                    disabled={editingConnection !== null}
                  >
                    <Icon name="✏️" />
                    Edit
                  </Button>
                  
                  <Button
                    onClick={() => handleDeleteConnection(connection.name)}
                    variant="danger"
                    size="sm"
                    disabled={editingConnection !== null}
                  >
                    <Icon name="🗑️" />
                    Delete
                  </Button>
                </div>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Reconnection Settings */}
      <div className="p-4 bg-gray-800 rounded-lg border border-gray-700">
        <div className="flex items-center justify-between mb-3">
          <h4 className="text-md font-medium">Global Reconnection Settings</h4>
          <Button
            onClick={() => setShowReconnectionSettings(!showReconnectionSettings)}
            variant="secondary"
            size="sm"
          >
            <Icon name={showReconnectionSettings ? "🔽" : "🔼"} />
            {showReconnectionSettings ? 'Hide' : 'Show'} Settings
          </Button>
        </div>
        
        {showReconnectionSettings && (
          <div className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="flex items-center space-x-2">
                <input
                  type="checkbox"
                  id="auto-reconnect"
                  checked={reconnectionSettings.autoReconnect}
                  onChange={(e) => setReconnectionSettings({
                    ...reconnectionSettings,
                    autoReconnect: e.target.checked
                  })}
                  className="rounded border-gray-600 bg-gray-700 text-blue-500 focus:ring-blue-500"
                  title="Enable automatic reconnection when connection is lost"
                  aria-label="Auto-reconnect on connection loss"
                />
                <Label htmlFor="auto-reconnect">Auto-reconnect on connection loss</Label>
              </div>
              
              <div className="flex items-center space-x-2">
                <input
                  type="checkbox"
                  id="status-monitoring"
                  checked={reconnectionSettings.statusMonitoring}
                  onChange={(e) => setReconnectionSettings({
                    ...reconnectionSettings,
                    statusMonitoring: e.target.checked
                  })}
                  className="rounded border-gray-600 bg-gray-700 text-blue-500 focus:ring-blue-500"
                  title="Enable continuous monitoring of connection status"
                  aria-label="Enable connection status monitoring"
                />
                <Label htmlFor="status-monitoring">Enable connection status monitoring</Label>
              </div>
              
              <div>
                <Label htmlFor="reconnect-delay">Reconnection Delay (seconds)</Label>
                <Input
                  id="reconnect-delay"
                  type="number"
                  value={reconnectionSettings.reconnectDelay}
                  onChange={(e) => setReconnectionSettings({
                    ...reconnectionSettings,
                    reconnectDelay: parseInt(e.target.value) || 5
                  })}
                  min="1"
                  max="60"
                  placeholder="5"
                />
              </div>
              
              <div>
                <Label htmlFor="max-attempts">Maximum Reconnection Attempts</Label>
                <Input
                  id="max-attempts"
                  type="number"
                  value={reconnectionSettings.maxAttempts}
                  onChange={(e) => setReconnectionSettings({
                    ...reconnectionSettings,
                    maxAttempts: parseInt(e.target.value) || 5
                  })}
                  min="1"
                  max="20"
                  placeholder="5"
                />
              </div>
              
              <div>
                <Label htmlFor="status-interval">Status Check Interval (seconds)</Label>
                <Input
                  id="status-interval"
                  type="number"
                  value={reconnectionSettings.statusInterval}
                  onChange={(e) => setReconnectionSettings({
                    ...reconnectionSettings,
                    statusInterval: parseInt(e.target.value) || 30
                  })}
                  min="5"
                  max="300"
                  placeholder="30"
                />
              </div>
            </div>
            
            <div className="flex space-x-2">
              <Button
                onClick={saveReconnectionSettings}
                variant="primary"
                size="sm"
                disabled={isLoadingSettings}
              >
                <Icon name="💾" />
                Save Reconnection Settings
              </Button>
              <Button
                onClick={loadReconnectionSettings}
                variant="secondary"
                size="sm"
                disabled={isLoadingSettings}
              >
                <Icon name="🔄" />
                Reload Settings
              </Button>
            </div>
            
            {error && (
              <div className="p-2 bg-red-900/20 border border-red-700 rounded text-red-400 text-sm">
                {error}
              </div>
            )}
          </div>
        )}
      </div>

      {/* Protocol Information */}
      <div className="p-4 bg-gray-800 rounded-lg">
        <h4 className="text-md font-medium mb-3">Protocol Information</h4>
        <div className="text-sm">
          <div>
            <h5 className="font-medium text-blue-400 mb-2">OBS WebSocket v5</h5>
            <ul className="text-gray-300 space-y-1">
              <li>• Default port: 4455</li>
              <li>• SHA256 authentication</li>
              <li>• Enhanced features and API</li>
              <li>• Better error handling</li>
              <li>• Modern WebSocket implementation</li>
              <li>• Recommended for all new installations</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
};

export default WebSocketManager; 