import React, { useEffect } from 'react';
import { motion } from 'framer-motion';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import Button from '../atoms/Button';
import { StatusDot } from '../atoms/StatusDot';
import { useObsStore, ObsConnection } from '../../stores/obsStore';
import { obsCommands } from '../../utils/tauriCommands';

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

const ObsWebSocketManager: React.FC = () => {
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

  // Load existing connections from backend on component mount
  useEffect(() => {
    const loadConnections = async () => {
      if (isTauriAvailable()) {
        try {
          setLoading(true);
          const result = await invoke('obs_connections_get_all');
          
          if (result && Array.isArray(result)) {
            console.log('Database connections:', result);
            const formattedConnections: ObsConnection[] = result.map((conn: any) => {
              // Map database status to frontend status
              let frontendStatus: ObsConnection['status'] = 'disconnected';
              if (conn.status === 'connected' || conn.status === 'authenticated') {
                frontendStatus = 'connected';
              } else if (conn.status === 'connecting' || conn.status === 'authenticating') {
                frontendStatus = 'connecting';
              } else if (conn.status === 'error') {
                frontendStatus = 'error';
              } else {
                frontendStatus = 'disconnected';
              }
              
              return {
                name: conn.name,
                host: conn.host,
                port: conn.port,
                enabled: conn.is_active ?? true,
                status: frontendStatus,
                error: conn.error
              };
            });
            console.log('Formatted connections:', formattedConnections);
            setConnections(formattedConnections);
          } else {
            // If no connections found, create a default one
            setConnections([{
              name: 'OBS Studio 1',
              host: 'localhost',
              port: 4455,
              enabled: true,
              status: 'disconnected'
            }]);
          }
        } catch (error) {
          console.error('Failed to load OBS connections:', error);
          // Set default connection on error
          setConnections([{
            name: 'OBS Studio 1',
            host: 'localhost',
            port: 4455,
            enabled: true,
            status: 'disconnected'
          }]);
        } finally {
          setLoading(false);
        }
      }
    };

    loadConnections();
  }, []);

  // Setup OBS status listener (push-based)
  useEffect(() => {
    if (!isTauriAvailable()) return;

    // Start backend listener once
    obsCommands.setupStatusListener().catch((e) => console.error('obs status listener setup failed', e));

    let unlistenPromise: Promise<() => void> = Promise.resolve(() => {});
    if (window.__TAURI__?.event) {
      unlistenPromise = window.__TAURI__.event.listen('obs_status', (event: any) => {
      if (event && event.payload) {
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
      // Use Tauri command for OBS connection
      if (isTauriAvailable()) {
        const result = await invoke('obs_connect_to_connection', { connectionName });
        
        if (result && typeof result === 'object' && 'success' in result && result.success) {
          console.log(`✅ Successfully connected to OBS: ${connectionName}`);
          updateConnectionStatus(connectionName, 'connected');
          
          // Refresh connection status from backend
          setTimeout(async () => {
            try {
              const statusResult = await invoke('obs_get_connection_status', { connectionName });
              if (statusResult && typeof statusResult === 'object' && 'success' in statusResult && statusResult.success) {
                const status = statusResult.status;
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
      // Use Tauri command for OBS disconnection
      if (isTauriAvailable()) {
        const result = await invoke('obs_disconnect', {
          connectionName
        });
        
        if (result && typeof result === 'object' && 'success' in result && result.success) {
          console.log(`✅ Successfully disconnected from OBS: ${connectionName}`);
          updateConnectionStatus(connectionName, 'disconnected');
          
          // Refresh connection status from backend
          setTimeout(async () => {
            try {
              const statusResult = await invoke('obs_get_connection_status', { connectionName });
              if (statusResult && typeof statusResult === 'object' && 'success' in statusResult && statusResult.success) {
                const status = statusResult.status;
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
    const newConnection: ObsConnection = {
      name: `OBS Studio ${connections.length + 1}`,
      host: 'localhost',
      port: 4455,
      enabled: true,
      status: 'disconnected'
    };
    
    saveConnection(newConnection);
  };

  const refreshConnections = async () => {
    if (isTauriAvailable()) {
      try {
        setLoading(true);
        const result = await invoke('obs_connections_get_all');
        
        if (result && Array.isArray(result)) {
          console.log('Refreshed database connections:', result);
          const formattedConnections: ObsConnection[] = result.map((conn: any) => {
            // Map database status to frontend status
            let frontendStatus: ObsConnection['status'] = 'disconnected';
            if (conn.status === 'connected' || conn.status === 'authenticated') {
              frontendStatus = 'connected';
            } else if (conn.status === 'connecting' || conn.status === 'authenticating') {
              frontendStatus = 'connecting';
            } else if (conn.status === 'error') {
              frontendStatus = 'error';
            } else {
              frontendStatus = 'disconnected';
            }
            
            return {
              name: conn.name,
              host: conn.host,
              port: conn.port,
              enabled: conn.is_active ?? true,
              status: frontendStatus,
              error: conn.error
            };
          });
          setConnections(formattedConnections);
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
        const payload = {
          id: undefined,
          name: connection.name,
          host: connection.host,
          port: connection.port,
          password: undefined,
          is_active: connection.enabled,
          status: connection.status,
          error: connection.error
        };
        
        await invoke('obs_connections_save', { connection: payload });
        await refreshConnections();
      } catch (error) {
        console.error('Failed to save OBS connection:', error);
      }
    }
  };

  const deleteConnection = async (name: string) => {
    if (isTauriAvailable()) {
      try {
        await invoke('obs_connections_delete', { name });
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
        await invoke('obs_connections_sync_from_config');
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
            <h2 className="text-2xl font-bold text-white">OBS WebSocket Manager</h2>
            <p className="text-gray-400 mt-1">Manage OBS Studio connections for Windows desktop application</p>
          </div>
          <div className="flex items-center space-x-4">
            <span className="px-3 py-1 bg-blue-600 text-white text-sm rounded-full">
              Windows Native
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
          <h3 className="text-lg font-semibold text-white">OBS Connections</h3>
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
                  const status = await invoke('obs_get_status');
                  console.log('Manual OBS status check:', status);
                } catch (error) {
                  console.error('Failed to get OBS status:', error);
                }
              }} 
              variant="secondary" 
              size="sm"
            >
              Force Refresh
            </Button>
            <Button onClick={addConnection} variant="primary" size="sm">
              Add Connection
            </Button>
          </div>
        </div>

        {/* Connection List */}
        <div className="space-y-4">
          {loading ? (
            <p className="text-gray-400">Loading OBS connections...</p>
          ) : connections.length === 0 ? (
            <p className="text-gray-400">No OBS connections found. Add one to get started!</p>
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
                    onClick={() => removeConnection(connection.name)}
                    variant="danger"
                    size="sm"
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
                const result = await invoke('obs_get_connections');
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
                const result = await invoke('obs_get_status');
                console.log('OBS Status:', result);
                alert('Check console for OBS status');
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