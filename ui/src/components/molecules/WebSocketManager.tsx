import React, { useState, useEffect } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import { StatusDot } from '../atoms/StatusDot';
import { Icon } from '../atoms/Icon';
import Toggle from '../atoms/Toggle';
import { useAppStore, ObsConnection } from '../../stores';
import { configCommands } from '../../utils/tauriCommands';
import { obsObwsCommands } from '../../utils/tauriCommandsObws';

// Reconnection settings interface
interface ReconnectionSettings {
  autoReconnect: boolean;
  reconnectDelay: number;
  maxAttempts: number;
  statusMonitoring: boolean;
  statusInterval: number;
}

interface WebSocketManagerProps {
  mode?: 'local' | 'remote';
}

const WebSocketManager: React.FC<WebSocketManagerProps> = ({ mode = 'local' }) => {
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

  // Remove the constant polling - we'll rely on event-driven updates instead
  // useEffect(() => {
  //   const interval = setInterval(() => {
  //     refreshConnectionStatuses();
  //   }, 3000); // Refresh every 3 seconds
  //
  //   return () => clearInterval(interval);
  // }, [obsConnections]);

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
        const configConnections = configResult.data.obs.connections
          .filter((conn: any) => {
            if (mode === 'local') {
              // Local mode: only OBS_REC and OBS_STR
              return conn.name === 'OBS_REC' || conn.name === 'OBS_STR';
            } else {
              // Remote mode: exclude OBS_REC and OBS_STR
              return conn.name !== 'OBS_REC' && conn.name !== 'OBS_STR';
            }
          })
          .map((conn: any) => ({
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
        for (const conn of configConnections) {
          try {
            await obsObwsCommands.addConnection({
              name: conn.name,
              host: conn.host,
              port: conn.port,
              password: conn.password,
              enabled: conn.enabled,
            });
          } catch (error) {
            // Connection already exists, which is expected
          }
        }
      } else {
        // Fallback to direct OBS plugin query
        const result = await obsObwsCommands.getConnections();
        if (result.success && result.data) {
          // Update store with connections from backend
          const backendConnections = result.data
            .filter((conn: any) => {
              if (mode === 'local') {
                // Local mode: only OBS_REC and OBS_STR
                return conn.name === 'OBS_REC' || conn.name === 'OBS_STR';
              } else {
                // Remote mode: exclude OBS_REC and OBS_STR
                return conn.name !== 'OBS_REC' && conn.name !== 'OBS_STR';
              }
            })
            .map((conn: any) => ({
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
          obsConnections.forEach(conn => removeObsConnection(conn.name));
        }
      }
    } catch (error) {
      console.error('Failed to load connections:', error);
    }
  };

  const refreshConnectionStatuses = async () => {
    try {
      // Get current status for each connection
      for (const connection of obsConnections) {
        try {
          const result = await obsObwsCommands.getConnectionStatus(connection.name);
          if (result.success) {
            // The backend returns { success: true, status: "Connected" }
            const status = (result as any).status || 'Disconnected';
            const error = (result as any).error;
            
            // Only update if status has changed
            if (connection.status !== status || connection.error !== error) {
              updateObsConnectionStatus(connection.name, status, error);
            }
          }
        } catch (error) {
          console.error(`Failed to get status for connection ${connection.name}:`, error);
        }
      }
    } catch (error) {
      console.error('Failed to refresh connection statuses:', error);
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
      const result = await obsObwsCommands.addConnection({
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

    // Ensure editingConnection is not null
    if (!editingConnection) {
      setError('No connection being edited');
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

      // Update the connection using the new update method
      const result = await obsObwsCommands.updateConnection(editingConnection, {
        name: formData.name,
        host: formData.host,
        port: formData.port,
        password: finalPassword || undefined,
        enabled: formData.enabled,
      });

      if (result.success) {
        // Remove old connection from frontend store
        if (editingConnection) {
          removeObsConnection(editingConnection);
        }
        
        // Add updated connection to frontend store (status will be set to Disconnected by default)
        addObsConnection({
          ...formData,
          password: finalPassword,
        });
        
        resetForm();
        setEditingConnection(null);
      } else {
        setError(result.error || 'Failed to save connection settings');
      }
    } catch (error) {
      setError(`Failed to save connection settings: ${error}`);
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
      const result = await obsObwsCommands.connect(connection.name);
      
      if (result.success) {
        updateObsConnectionStatus(connection.name, 'Connected');
        setActiveObsConnection(connection.name);
        
        // Start health monitoring for this connection
        const stopHealthMonitoring = startHealthMonitoring(connection.name);
        
        // Poll for status updates
        setTimeout(async () => {
          try {
            const statusResult = await obsObwsCommands.getConnectionStatus(connection.name);
            if (statusResult.success && statusResult.data) {
              const status = statusResult.data.status as ObsConnection['status'];
              updateObsConnectionStatus(connection.name, status);
              if (status === 'Authenticated') {
                setActiveObsConnection(connection.name);
                console.log(`Connection ${connection.name} authenticated successfully`);
              }
            }
          } catch (error) {
            console.error(`Failed to get connection status for ${connection.name}:`, error);
          }
        }, 2000);
      } else {
        // Enhanced error handling with more detailed messages
        const errorMessage = result.error || 'Unknown connection error';
        updateObsConnectionStatus(connection.name, 'Error', errorMessage);
        console.error(`Connection failed for ${connection.name}:`, errorMessage);
        
        // Log additional context for debugging
        console.log(`Connection attempt details:`, {
          name: connection.name,
          host: connection.host,
          port: connection.port,
          enabled: connection.enabled,
          timestamp: new Date().toISOString()
        });
      }
    } catch (error) {
      // Enhanced error handling for unexpected errors
      const errorMessage = `Connection failed: ${error}`;
      updateObsConnectionStatus(connection.name, 'Error', errorMessage);
      console.error(`Unexpected error for ${connection.name}:`, error);
      
      // Log additional context for debugging
      console.log(`Unexpected error details:`, {
        name: connection.name,
        error: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : undefined,
        timestamp: new Date().toISOString()
      });
    }
  };

  const handleDisconnect = async (connection: ObsConnection) => {
    try {
      const result = await obsObwsCommands.disconnect(connection.name);
      
      if (result.success) {
        updateObsConnectionStatus(connection.name, 'Disconnected');
        if (activeObsConnection === connection.name) {
          setActiveObsConnection(null);
        }
        console.log(`Successfully disconnected from ${connection.name}`);
      } else {
        // Enhanced error handling for disconnect failures
        const errorMessage = result.error || 'Unknown disconnect error';
        updateObsConnectionStatus(connection.name, 'Error', errorMessage);
        console.error(`Disconnect failed for ${connection.name}:`, errorMessage);
        
        // Log additional context for debugging
        console.log(`Disconnect attempt details:`, {
          name: connection.name,
          wasActive: activeObsConnection === connection.name,
          timestamp: new Date().toISOString()
        });
      }
    } catch (error) {
      // Enhanced error handling for unexpected disconnect errors
      const errorMessage = `Disconnect failed: ${error}`;
      updateObsConnectionStatus(connection.name, 'Error', errorMessage);
      console.error(`Unexpected disconnect error for ${connection.name}:`, error);
      
      // Log additional context for debugging
      console.log(`Unexpected disconnect error details:`, {
        name: connection.name,
        error: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : undefined,
        wasActive: activeObsConnection === connection.name,
        timestamp: new Date().toISOString()
      });
    }
  };

  // Connection health monitoring
  const startHealthMonitoring = (connectionName: string) => {
    const interval = setInterval(async () => {
      try {
        const status = await obsObwsCommands.getConnectionStatus(connectionName);
        if (status.success && status.data?.status === 'Error') {
          console.warn(`Connection ${connectionName} health check failed`);
          // Could trigger reconnection logic here in the future
        }
      } catch (error) {
        console.error(`Health check failed for ${connectionName}:`, error);
    }
    }, 30000); // Check every 30 seconds
    
    return () => clearInterval(interval);
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
        <h3 className="text-lg font-semibold">
          {mode === 'local' ? 'Local OBS WebSocket Connections' : 'Remote OBS WebSocket Connections'}
        </h3>
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
          <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor" className="mr-2">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
          </svg>
          Add Connection
        </Button>
      </div>

      {/* Add/Edit Form */}
      {(isAdding || editingConnection) && (
        <div className="p-4 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
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
                autoComplete="new-password"
                value={formData.password}
                onChange={(e) => setFormData({ ...formData, password: e.target.value })}
                placeholder={editingConnection && obsConnections.find(c => c.name === editingConnection)?.password 
                  ? "Password is set (click to change)" 
                  : "Leave empty if no password"}
              />
              {/* Hidden username field for accessibility */}
              <input 
                type="text" 
                autoComplete="username" 
                style={{ display: 'none' }} 
                aria-hidden="true"
                tabIndex={-1}
              />
            </div>

            <div className="flex items-center">
              <Toggle
                checked={formData.enabled}
                onChange={(e) => setFormData({ ...formData, enabled: e.target.checked })}
                label="Enabled"
                labelPosition="right"
              />
            </div>
          </div>

          <div className="flex space-x-2 mt-4">
            <Button
              onClick={editingConnection ? handleUpdateConnection : handleAddConnection}
              variant="primary"
              size="sm"
            >
              {editingConnection ? 'Save Connection Settings' : 'Add'} Connection
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
          <div className="p-4 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg text-center text-gray-400 border border-gray-600/30">
            No WebSocket connections configured
          </div>
        ) : (
          obsConnections.map((connection) => (
            <div
              key={connection.name}
              className={`p-4 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border ${
                activeObsConnection === connection.name ? 'border-blue-500 bg-blue-900/20' : 'border-gray-600/30 hover:border-gray-500'
              } cursor-pointer transition-colors`}
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
                      <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor" className="mr-2">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.141 0M1.394 9.393c5.857-5.857 15.355-5.857 21.213 0" />
                      </svg>
                      Disconnect
                    </Button>
                  ) : (
                    <Button
                      onClick={() => handleConnect(connection)}
                      variant="success"
                      size="sm"
                      disabled={!connection.enabled || connection.status === 'Connecting'}
                    >
                      <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor" className="mr-2">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                      </svg>
                      Connect
                    </Button>
                  )}
                  
                  <Button
                    onClick={() => handleEditConnection(connection)}
                    variant="secondary"
                    size="sm"
                    disabled={editingConnection !== null}
                  >
                    <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor" className="mr-2">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                    Edit
                  </Button>
                  
                  <Button
                    onClick={() => handleDeleteConnection(connection.name)}
                    variant="danger"
                    size="sm"
                    disabled={editingConnection !== null}
                  >
                    <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor" className="mr-2">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                    Delete
                  </Button>
                </div>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Reconnection Settings */}
      <div className="p-4 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
        <div className="flex items-center justify-between mb-3">
          <h4 className="text-md font-medium">Global Reconnection Settings</h4>
          <Button
            onClick={() => setShowReconnectionSettings(!showReconnectionSettings)}
            variant="secondary"
            size="sm"
          >
            <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor" className="mr-2">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={showReconnectionSettings ? "M5 15l7-7 7 7" : "M19 9l-7 7-7-7"} />
            </svg>
            {showReconnectionSettings ? 'Hide' : 'Show'} Settings
          </Button>
        </div>
        
        {showReconnectionSettings && (
          <div className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="flex items-center space-x-2">
                <Toggle
                  id="auto-reconnect"
                  checked={reconnectionSettings.autoReconnect}
                  onChange={(e) => setReconnectionSettings({
                    ...reconnectionSettings,
                    autoReconnect: e.target.checked
                  })}
                  label="Auto-reconnect on connection loss"
                  labelPosition="right"
                  title="Enable automatic reconnection when connection is lost"
                  aria-label="Auto-reconnect on connection loss"
                />
              </div>
              
              <div className="flex items-center space-x-2">
                <Toggle
                  id="status-monitoring"
                  checked={reconnectionSettings.statusMonitoring}
                  onChange={(e) => setReconnectionSettings({
                    ...reconnectionSettings,
                    statusMonitoring: e.target.checked
                  })}
                  label="Enable connection status monitoring"
                  labelPosition="right"
                  title="Enable continuous monitoring of connection status"
                  aria-label="Enable connection status monitoring"
                />
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
                <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor" className="mr-2">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
                </svg>
                Save Reconnection Settings
              </Button>
              <Button
                onClick={loadReconnectionSettings}
                variant="secondary"
                size="sm"
                disabled={isLoadingSettings}
              >
                <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor" className="mr-2">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
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
      <div className="p-4 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
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