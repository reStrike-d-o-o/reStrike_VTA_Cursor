import React, { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import Toggle from '../atoms/Toggle';
import { StatusDot } from '../atoms/StatusDot';
import { useObsStore, ObsConnection } from '../../stores/obsStore';
import { obsObwsCommands } from '../../utils/tauriCommandsObws';
import { configCommands } from '../../utils/tauriCommands';

// Use the proper Tauri v2 invoke function with fallback
const invoke = async (command: string, args?: any) => {
  try {
    // Try the proper Tauri v2 API first
    return await tauriInvoke(command, args);
  } catch (error) {
    // If that fails, try the global window.__TAURI__.core.invoke
    if (typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.core) {
      return await window.__TAURI__.core.invoke(command, args);
    }
    throw new Error('Tauri v2 core module not available - ensure app is running in desktop mode');
  }
};

/**
 * Check if Tauri is available (same logic as environment detection)
 */
const isTauriAvailable = (): boolean => {
  const tauriAvailable = typeof window !== 'undefined' && !!window.__TAURI__;
  const isTauriContext = tauriAvailable || 
    (typeof window !== 'undefined' && window.location.protocol === 'tauri:') ||
    (typeof window !== 'undefined' && window.location.hostname === 'localhost' && window.location.port === '3000');
  
  return isTauriContext;
};

interface ObsWebSocketManagerProps {
  mode: 'local' | 'remote';
}

const ObsWebSocketManager: React.FC<ObsWebSocketManagerProps> = ({ mode }) => {
  const {
    connections,
    events,
    loading,
    error,
    setConnections,
    setLoading,
    addEvent,
    updateConnectionStatus,
    getConnectionCount,
    updateObsStatus,
  } = useObsStore();

  // Add state for edit functionality
  const [isAdding, setIsAdding] = useState(false);
  const [editingConnection, setEditingConnection] = useState<string | null>(null);
  const [formData, setFormData] = useState({
    name: '',
    host: 'localhost',
    port: 4455,
    password: '',
    enabled: true,
  });
  const [formError, setFormError] = useState<string | null>(null);

  const resetForm = () => {
    setFormData({
      name: '',
      host: 'localhost',
      port: 4455,
      password: '',
      enabled: true,
    });
    setFormError(null);
  };

  // Load existing connections from configuration on component mount
  useEffect(() => {
    const loadConnections = async () => {
      if (isTauriAvailable()) {
        try {
          setLoading(true);
          
          // Load connections from configuration system
          const configResult = await configCommands.getSettings();
          if (configResult.success && configResult.data?.obs?.connections) {
            console.log('Configuration connections:', configResult.data.obs.connections);
            
            // Filter connections based on mode
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
                enabled: conn.enabled ?? true,
                status: 'disconnected' as const,
                error: undefined
              }));
            
            console.log(`Filtered ${mode} connections:`, configConnections);
            setConnections(configConnections);
            
            // Ensure all config connections are registered with the obws plugin
            for (const conn of configConnections) {
              try {
                await obsObwsCommands.addConnection({
                  name: conn.name,
                  host: conn.host,
                  port: conn.port,
                  password: undefined, // Will be loaded from config
                  enabled: conn.enabled,
                });
              } catch (error) {
                // Connection already exists, which is expected
                console.log(`Connection ${conn.name} already exists in obws plugin`);
              }
            }
          } else {
            // If no connections found in config, create default ones based on mode
            if (mode === 'local') {
              setConnections([
                {
                  name: 'OBS_REC',
                  host: 'localhost',
                  port: 4455,
                  enabled: true,
                  status: 'disconnected'
                },
                {
                  name: 'OBS_STR',
                  host: 'localhost',
                  port: 4466,
                  enabled: true,
                  status: 'disconnected'
                }
              ]);
            } else {
              setConnections([]);
            }
          }
        } catch (error) {
          console.error('Failed to load OBS connections:', error);
          // Set default connections on error based on mode
          if (mode === 'local') {
            setConnections([
              {
                name: 'OBS_REC',
                host: 'localhost',
                port: 4455,
                enabled: true,
                status: 'disconnected'
              },
              {
                name: 'OBS_STR',
                host: 'localhost',
                port: 4466,
                enabled: true,
                status: 'disconnected'
              }
            ]);
          } else {
            setConnections([]);
          }
        } finally {
          setLoading(false);
        }
      }
    };

    loadConnections();
  }, [mode]);

  // Setup OBS status listener (push-based)
  useEffect(() => {
    if (!isTauriAvailable()) return;

    console.log('🔧 ObsWebSocketManager: Setting up OBS status listener...');

    let unlistenPromise: Promise<() => void> = Promise.resolve(() => {});
    if (window.__TAURI__?.event) {
      console.log('🔧 ObsWebSocketManager: Listening for obs_status events...');
      unlistenPromise = window.__TAURI__.event.listen('obs_status', (event: any) => {
        console.log('🔧 ObsWebSocketManager: Received obs_status event:', event);
        if (event && event.payload) {
          console.log('🔧 ObsWebSocketManager: Updating OBS status with payload:', event.payload);
          updateObsStatus(event.payload);
        }
      });
    }

    return () => {
      unlistenPromise.then((unsub: () => void) => unsub()).catch(() => {});
    };
  }, []);

  // Listen for OBS events from backend
  useEffect(() => {
    if (!isTauriAvailable()) return;

    const handleObsEvent = (event: any) => {
      console.log('OBS Event received:', event);
      addEvent(event);
      
      // Update connection status if it's a connection status event
      if (event.eventType === 'ConnectionStatusChanged') {
        const { connection_name, status } = event;
        let frontendStatus: ObsConnection['status'] = 'disconnected';
        if (status === 'Connected' || status === 'Authenticated') {
          frontendStatus = 'connected';
        } else if (status === 'Connecting' || status === 'Authenticating') {
          frontendStatus = 'connecting';
        } else if (status === 'Error') {
          frontendStatus = 'error';
        } else {
          frontendStatus = 'disconnected';
        }
        updateConnectionStatus(connection_name, frontendStatus);
      }
    };

    // Listen for OBS events
    const unsubscribe: Promise<() => void> = window.__TAURI__?.event?.listen ? window.__TAURI__.event.listen('obs_event', handleObsEvent) : Promise.resolve(() => {});
    
    return () => {
      unsubscribe.then((unsub: () => void) => unsub());
    };
  }, []);

  // Refresh connections when component becomes visible
  useEffect(() => {
    const handleVisibilityChange = () => {
      if (!document.hidden) {
        console.log('Component became visible, refreshing connections...');
        refreshConnections();
        }
    };

    const handleFocus = () => {
      console.log('Window focused, refreshing connections...');
      refreshConnections();
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);
    window.addEventListener('focus', handleFocus);

    return () => {
      document.removeEventListener('visibilitychange', handleVisibilityChange);
      window.removeEventListener('focus', handleFocus);
    };
  }, []);

  const connectToObs = async (connectionName: string) => {
    console.log(`Connecting to OBS: ${connectionName}`);
    updateConnectionStatus(connectionName, 'connecting');
    
    try {
      // Use obws Tauri command for OBS connection
      if (isTauriAvailable()) {
        const result = await obsObwsCommands.connect(connectionName);
        
        if (result && typeof result === 'object' && 'success' in result && result.success) {
          console.log(`✅ Successfully connected to OBS: ${connectionName}`);
          updateConnectionStatus(connectionName, 'connected');
          
          // Refresh connection status from backend
          setTimeout(async () => {
            try {
              const statusResult = await obsObwsCommands.getConnectionStatus(connectionName);
              if (statusResult && statusResult.success && statusResult.data) {
                const status = statusResult.data.status;
                if (status === 'Connected' || status === 'Authenticated') {
                  updateConnectionStatus(connectionName, 'connected');
                } else if (status === 'Connecting' || status === 'Authenticating') {
                  updateConnectionStatus(connectionName, 'connecting');
                } else if (status === 'Error') {
                  updateConnectionStatus(connectionName, 'error', status);
                } else {
                  updateConnectionStatus(connectionName, 'disconnected');
                }
              }
            } catch (error) {
              console.error('Failed to refresh connection status:', error);
            }
          }, 1000);
        } else {
          console.error(`❌ Failed to connect to OBS: ${connectionName}`, result);
          updateConnectionStatus(connectionName, 'error', result.error || 'Connection failed');
        }
      } else {
        console.error('❌ Tauri not available for OBS connection');
        updateConnectionStatus(connectionName, 'error', 'Tauri not available');
      }
    } catch (error) {
      console.error(`❌ Error connecting to OBS: ${connectionName}`, error);
      updateConnectionStatus(connectionName, 'error', (error as Error)?.message || String(error));
    }
  };

  const disconnectFromObs = async (connectionName: string) => {
    console.log(`Disconnecting from OBS: ${connectionName}`);
    
    try {
      // Use obws Tauri command for OBS disconnection
      if (isTauriAvailable()) {
        const result = await obsObwsCommands.disconnect(connectionName);
        
        if (result && typeof result === 'object' && 'success' in result && result.success) {
          console.log(`✅ Successfully disconnected from OBS: ${connectionName}`);
          updateConnectionStatus(connectionName, 'disconnected');
          
          // Refresh connection status from backend
          setTimeout(async () => {
            try {
              const statusResult = await obsObwsCommands.getConnectionStatus(connectionName);
              if (statusResult && statusResult.success && statusResult.data) {
                const status = statusResult.data.status;
                if (status === 'Disconnected') {
                  updateConnectionStatus(connectionName, 'disconnected');
                } else {
                  updateConnectionStatus(connectionName, 'error', status);
                }
              }
            } catch (error) {
              console.error('Failed to refresh connection status:', error);
            }
          }, 1000);
        } else {
          console.error(`❌ Failed to disconnect from OBS: ${connectionName}`, result);
          updateConnectionStatus(connectionName, 'error', result.error || 'Disconnect failed');
        }
      } else {
        console.error('❌ Tauri not available for OBS disconnection');
        updateConnectionStatus(connectionName, 'disconnected');
      }
    } catch (error) {
      console.error('❌ Error disconnecting from OBS: ' + connectionName, error);
      updateConnectionStatus(connectionName, 'disconnected');
    }
  };

  const addConnection = () => {
    setIsAdding(true);
    setEditingConnection(null);
    resetForm();
  };

  const handleAddConnection = async () => {
    if (!formData.name.trim()) {
      setFormError('Connection name is required');
      return;
    }

    if (!formData.host.trim()) {
      setFormError('Host is required');
      return;
    }

    if (formData.port < 1 || formData.port > 65535) {
      setFormError('Port must be between 1 and 65535');
      return;
    }

    try {
      setFormError(null);
      
      const newConnection: ObsConnection = {
        name: formData.name,
        host: formData.host,
        port: formData.port,
        enabled: formData.enabled,
        status: 'disconnected'
      };
      
      await saveConnection(newConnection);
      setIsAdding(false);
      resetForm();
    } catch (error) {
      console.error('Failed to add connection:', error);
      setFormError('Failed to add connection: ' + (error as Error)?.message || String(error));
    }
  };

  const handleEditConnection = (connection: ObsConnection) => {
    setEditingConnection(connection.name);
    setFormData({
      name: connection.name,
      host: connection.host,
      port: connection.port,
      password: '', // Don't populate password for security
      enabled: connection.enabled,
    });
    setFormError(null);
  };

  const handleUpdateConnection = async () => {
    if (!formData.name.trim()) {
      setFormError('Connection name is required');
      return;
    }

    if (!formData.host.trim()) {
      setFormError('Host is required');
      return;
    }

    if (formData.port < 1 || formData.port > 65535) {
      setFormError('Port must be between 1 and 65535');
      return;
    }

    // Ensure editingConnection is not null
    if (!editingConnection) {
      setFormError('No connection being edited');
      return;
    }

    try {
      setFormError(null);
      
      // Update the connection using the new update method
      const result = await obsObwsCommands.updateConnection(editingConnection, {
        name: formData.name,
        host: formData.host,
        port: formData.port,
        password: undefined, // Will be loaded from config
        enabled: formData.enabled,
      });

      if (result.success) {
        // Update the connection in the local state
        const updatedConnection: ObsConnection = {
          name: formData.name,
          host: formData.host,
          port: formData.port,
          enabled: formData.enabled,
          status: 'disconnected'
        };
        
        // Remove old connection and add updated one
        const currentConnections = connections.filter(c => c.name !== editingConnection);
        setConnections([...currentConnections, updatedConnection]);
        
        setEditingConnection(null);
        resetForm();
      } else {
        setFormError(result.error || 'Failed to update connection');
      }
    } catch (error) {
      console.error('Failed to update connection:', error);
      setFormError('Failed to update connection: ' + (error as Error)?.message || String(error));
    }
  };

  const refreshConnections = async () => {
    if (isTauriAvailable()) {
      try {
        setLoading(true);
        
        // Load connections from configuration system
        const configResult = await configCommands.getSettings();
        if (configResult.success && configResult.data?.obs?.connections) {
          // Filter connections based on mode
          const configConnections = configResult.data.obs.connections
            .filter((conn: any) => {
              if (mode === 'local') {
                return conn.name === 'OBS_REC' || conn.name === 'OBS_STR';
              } else {
                return conn.name !== 'OBS_REC' && conn.name !== 'OBS_STR';
              }
            })
            .map((conn: any) => ({
              name: conn.name,
              host: conn.host,
              port: conn.port,
              enabled: conn.enabled ?? true,
              status: 'disconnected' as const,
              error: undefined
            }));
          
          setConnections(configConnections);
        }
      } catch (error) {
        console.error('Failed to refresh OBS connections:', error);
      } finally {
        setLoading(false);
      }
    }
  };

  const saveConnection = async (connection: ObsConnection) => {
    if (isTauriAvailable()) {
      try {
        await obsObwsCommands.addConnection({
          name: connection.name,
          host: connection.host,
          port: connection.port,
          password: undefined,
          enabled: connection.enabled
        });
        await refreshConnections();
      } catch (error) {
        console.error('Failed to save OBS connection:', error);
      }
    }
  };

  const deleteConnection = async (name: string) => {
    if (isTauriAvailable()) {
      try {
        await obsObwsCommands.removeConnection(name);
        await refreshConnections();
      } catch (error) {
        console.error('Failed to delete OBS connection:', error);
      }
    }
  };

  const syncFromConfig = async () => {
    if (isTauriAvailable()) {
      try {
        setLoading(true);
        await refreshConnections();
      } catch (error) {
        console.error('Failed to sync connections from config:', error);
      } finally {
        setLoading(false);
      }
    }
  };

  const removeConnection = (name: string) => {
    deleteConnection(name);
  };

  const getStatusColor = (status: ObsConnection['status']) => {
    switch (status) {
      case 'connected': return 'bg-green-500';
      case 'connecting': return 'bg-yellow-500';
      case 'error': return 'bg-red-500';
      default: return 'bg-gray-500';
    }
  };

  const getStatusText = (status: ObsConnection['status']) => {
    switch (status) {
      case 'connected': return 'Connected';
      case 'connecting': return 'Connecting...';
      case 'error': return 'Error';
      default: return 'Disconnected';
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-gray-800 rounded-lg p-6">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold text-white">
              {mode === 'local' ? 'Local OBS WebSocket Manager' : 'Remote OBS Control Room'}
            </h2>
            <p className="text-gray-400 mt-1">
              {mode === 'local' 
                ? 'Manage local OBS Studio connections (OBS_REC, OBS_STR) using obws implementation'
                : 'Control remote OBS instances on the network using obws implementation'
              }
            </p>
          </div>
          <div className="flex items-center space-x-4">
            <span className="px-3 py-1 bg-blue-600 text-white text-sm rounded-full">
              Windows Native
            </span>
            <span className="px-3 py-1 bg-purple-600 text-white text-sm rounded-full">
              obws Implementation
            </span>
            <span className={`px-3 py-1 text-white text-sm rounded-full ${
              mode === 'local' ? 'bg-green-600' : 'bg-orange-600'
            }`}>
              {mode === 'local' ? 'Local Mode' : 'Remote Mode'}
            </span>
            <span className="px-3 py-1 bg-green-600 text-white text-sm rounded-full">
              {connections.filter(c => c.status === 'connected').length}/{connections.length} Connected
            </span>
          </div>
        </div>
      </div>

      {/* Connection Management */}
      <div className="bg-gray-800 rounded-lg p-6">
        <div className="flex items-center justify-between mb-6">
          <h3 className="text-lg font-semibold text-white">
            {mode === 'local' ? 'Local OBS Connections' : 'Remote OBS Connections'}
          </h3>
          <div className="flex items-center space-x-2">
            <Button 
              onClick={refreshConnections} 
              variant="secondary" 
              size="sm"
              disabled={loading}
            >
              {loading ? 'Refreshing...' : 'Refresh'}
            </Button>
            <Button 
              onClick={syncFromConfig} 
              variant="secondary" 
              size="sm"
              disabled={loading}
            >
              {loading ? 'Syncing...' : 'Sync from Config'}
            </Button>
            <Button 
              onClick={async () => {
                console.log('Manual refresh triggered');
                await refreshConnections();
                // Also check OBS status
                try {
                  // Get the first connection name for status check
                  const connections = await obsObwsCommands.getConnections();
                  if (connections.success && connections.data && connections.data.length > 0) {
                    const firstConnection = connections.data[0];
                    const status = await obsObwsCommands.getStatus(firstConnection.name);
                    console.log('Manual OBS status check:', status);
                  }
                } catch (error) {
                  console.error('Failed to get OBS status:', error);
                }
              }} 
              variant="secondary" 
              size="sm"
            >
              Force Refresh
            </Button>
            <Button 
              onClick={async () => {
                try {
                  const result = await obsObwsCommands.testConnection();
                  console.log('OBS obws test result:', result);
                  alert(result.success ? 'Test passed!' : `Test failed: ${result.error}`);
                } catch (error) {
                  console.error('Failed to test obws connection:', error);
                  alert('Test failed: ' + error);
                }
              }} 
              variant="secondary" 
              size="sm"
            >
              Test obws
            </Button>
            <Button 
              onClick={addConnection} 
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
        </div>

        {/* Add/Edit Form */}
        {(isAdding || editingConnection) && (
          <div className="p-4 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg mb-4">
            <h4 className="text-md font-medium mb-3">
              {editingConnection ? 'Edit Connection' : 'Add New Connection'}
            </h4>
            
            {formError && (
              <div className="mb-3 p-2 bg-red-900/20 border border-red-700 rounded text-red-400 text-sm">
                {formError}
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
                  placeholder={editingConnection && connections.find(c => c.name === editingConnection)?.password 
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

        {/* Connection List */}
        <div className="space-y-4">
          {loading ? (
            <p className="text-gray-400">Loading OBS connections...</p>
          ) : connections.length === 0 ? (
            <p className="text-gray-400">
              {mode === 'local' 
                ? 'No local OBS connections found. Add OBS_REC and OBS_STR to get started!'
                : 'No remote OBS connections found. Add remote instances to control them.'
              }
            </p>
          ) : (
            connections.map((connection) => (
            <motion.div
              key={connection.name}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              className="bg-gray-700 rounded-lg p-4 border border-gray-600"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-4">
                  <div className="flex items-center space-x-2">
                    <StatusDot color={
                      connection.status === 'connected' ? 'bg-green-500' :
                      connection.status === 'error' ? 'bg-red-500' :
                      connection.status === 'connecting' ? 'bg-yellow-500' :
                      'bg-gray-500'
                    } />
                    <span className="text-white font-medium">{connection.name}</span>
                  </div>
                  <span className="text-gray-400">
                    {connection.host}:{connection.port}
                  </span>
                  <span className={`px-2 py-1 rounded text-xs font-medium ${
                    connection.status === 'connected' ? 'bg-green-600 text-white' :
                    connection.status === 'connecting' ? 'bg-yellow-600 text-white' :
                    connection.status === 'error' ? 'bg-red-600 text-white' :
                    'bg-gray-600 text-white'
                  }`}>
                    {getStatusText(connection.status)}
                  </span>
                </div>
                
                <div className="flex items-center space-x-2">
                  {connection.status === 'disconnected' && (
                    <Button
                      onClick={() => connectToObs(connection.name)}
                      variant="primary"
                      size="sm"
                    >
                      Connect
                    </Button>
                  )}
                  {connection.status === 'connected' && (
                    <Button
                      onClick={() => disconnectFromObs(connection.name)}
                      variant="secondary"
                      size="sm"
                    >
                      Disconnect
                    </Button>
                  )}
                  {connection.status === 'connecting' && (
                    <Button
                      disabled
                      variant="secondary"
                      size="sm"
                    >
                      Connecting...
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
                    onClick={() => removeConnection(connection.name)}
                    variant="danger"
                    size="sm"
                    disabled={editingConnection !== null}
                  >
                    Remove
                  </Button>
                </div>
              </div>
              
              {connection.error && (
                <div className="mt-2 p-2 bg-red-900/20 border border-red-700 rounded text-red-400 text-sm">
                  Error: {connection.error}
                </div>
              )}
            </motion.div>
            ))
          )}
        </div>
      </div>

      {/* OBS Events Section */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-white mb-4">OBS Events</h3>
        <div className="text-sm text-gray-300 space-y-2 mb-4">
          <div>Total Events: {events.length}</div>
          <div>Last Event: {events.length > 0 ? events[0].timestamp : 'None'}</div>
        </div>
        <div className="max-h-40 overflow-y-auto space-y-2">
          {events.length === 0 ? (
            <p className="text-gray-400 text-sm">No OBS events received yet. Events will appear here when OBS sends them.</p>
          ) : (
            events.slice(0, 10).map((event, index) => (
              <div key={index} className="bg-gray-700 p-2 rounded text-xs">
                <div className="font-medium text-blue-300">{event.eventType}</div>
                <div className="text-gray-400">{event.connection_name}</div>
                {event.scene_name && <div className="text-green-300">Scene: {event.scene_name}</div>}
                {event.is_recording !== undefined && (
                  <div className={event.is_recording ? 'text-red-300' : 'text-gray-400'}>
                    Recording: {event.is_recording ? 'ON' : 'OFF'}
                  </div>
                )}
                {event.is_streaming !== undefined && (
                  <div className={event.is_streaming ? 'text-orange-300' : 'text-gray-400'}>
                    Streaming: {event.is_streaming ? 'ON' : 'OFF'}
                  </div>
                )}
                <div className="text-gray-500 text-xs">{event.timestamp}</div>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Debug Section */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-white mb-4">Debug Information</h3>
        <div className="text-sm text-gray-300 space-y-2">
          {(() => {
            const counts = getConnectionCount();
            return (
              <>
                <div>Mode: {mode}</div>
                <div>Total Connections: {counts.total}</div>
                <div>Connected: {counts.connected}</div>
                <div>Connecting: {counts.connecting}</div>
                <div>Disconnected: {counts.disconnected}</div>
                <div>Errors: {counts.error}</div>
              </>
            );
          })()}
        </div>
        <div className="mt-4 space-x-2">
          <Button 
            onClick={async () => {
              try {
                const result = await obsObwsCommands.getConnections();
                console.log('Raw backend response:', result);
                alert('Check console for raw backend response');
              } catch (error) {
                console.error('Failed to get raw response:', error);
              }
            }} 
            variant="secondary" 
            size="sm"
          >
            Get Raw Backend Response
          </Button>
          <Button 
            onClick={async () => {
              try {
                // Get the first connection name for status check
                const connections = await obsObwsCommands.getConnections();
                if (connections.success && connections.data && connections.data.length > 0) {
                  const firstConnection = connections.data[0];
                  const result = await obsObwsCommands.getStatus(firstConnection.name);
                  console.log('OBS Status:', result);
                  alert('Check console for OBS status');
                } else {
                  alert('No connections available for status check');
                }
              } catch (error) {
                console.error('Failed to get OBS status:', error);
              }
            }} 
            variant="secondary" 
            size="sm"
          >
            Get OBS Status
          </Button>
        </div>
      </div>
    </div>
  );
};

export default ObsWebSocketManager;