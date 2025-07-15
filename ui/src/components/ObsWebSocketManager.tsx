import React, { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { useAppStore, ObsConnection } from '../stores';

interface ObsConnectionConfig {
  name: string;
  host: string;
  port: number;
  password?: string;
  protocol_version: 'v4' | 'v5';
  enabled: boolean;
}

const ObsWebSocketManager: React.FC = () => {
  const { 
    obsConnections, 
    addObsConnection, 
    removeObsConnection, 
    updateObsConnectionStatus 
  } = useAppStore();
  
  const [newConnection, setNewConnection] = useState<ObsConnectionConfig>({
    name: '',
    host: 'localhost',
    port: 4455,
    password: '',
    protocol_version: 'v5',
    enabled: true,
  });
  const [isAddingConnection, setIsAddingConnection] = useState(false);

  const addConnection = async () => {
    if (!newConnection.name.trim()) {
      alert('Connection name is required');
      return;
    }

    if (obsConnections.some(c => c.name === newConnection.name)) {
      alert('Connection name must be unique');
      return;
    }

    const connectionToAdd = {
      ...newConnection,
      password: newConnection.password || undefined,
    };

    addObsConnection(connectionToAdd);

    // Reset form
    setNewConnection({
      name: '',
      host: 'localhost',
      port: 4455,
      password: '',
      protocol_version: 'v5',
      enabled: true,
    });
    setIsAddingConnection(false);

    // Connect if enabled
    if (connectionToAdd.enabled) {
      await connectToObs(connectionToAdd.name);
    }
  };

  const removeConnection = async (name: string) => {
    // Disconnect first if connected
    const connection = obsConnections.find(c => c.name === name);
    if (connection && (connection.status === 'Connected' || connection.status === 'Authenticated')) {
      await disconnectFromObs(name);
    }
    
    removeObsConnection(name);
  };

  const connectToObs = async (connectionName: string) => {
    try {
      // Update status to connecting
      updateObsConnectionStatus(connectionName, 'Connecting');

      // Find the connection configuration
      const connection = obsConnections.find(c => c.name === connectionName);
      if (!connection) {
        throw new Error('Connection not found');
      }

      // Call the backend to connect
      if (window.__TAURI__) {
        await window.__TAURI__.invoke('obs_connect', { connectionName });
      } else {
        // Fallback for development - test WebSocket connection directly
        console.log(`Testing WebSocket connection to ${connection.host}:${connection.port}...`);
        
        // Create a simple WebSocket test
        const wsUrl = `ws://${connection.host}:${connection.port}`;
        const ws = new WebSocket(wsUrl);
        
        ws.onopen = () => {
          console.log(`WebSocket connected to ${connectionName}`);
          updateObsConnectionStatus(connectionName, 'Connected');
          
          // Send OBS WebSocket v5 identify request
          const identifyRequest = {
            op: 1,
            d: {
              rpcVersion: 1,
              authentication: connection.password ? {
                challenge: "test_challenge",
                salt: "test_salt"
              } : null,
              eventSubscriptions: 0
            }
          };
          
          ws.send(JSON.stringify(identifyRequest));
        };
        
        ws.onmessage = (event) => {
          try {
            const response = JSON.parse(event.data);
            console.log(`OBS Response for ${connectionName}:`, response);
            
            if (response.op === 2) { // Identify response
              updateObsConnectionStatus(connectionName, 'Authenticated');
            }
          } catch (error) {
            console.error(`Failed to parse OBS response for ${connectionName}:`, error);
          }
        };
        
        ws.onerror = (error) => {
          console.error(`WebSocket error for ${connectionName}:`, error);
          updateObsConnectionStatus(connectionName, 'Error', 'WebSocket connection failed');
        };
        
        ws.onclose = () => {
          console.log(`WebSocket closed for ${connectionName}`);
          updateObsConnectionStatus(connectionName, 'Disconnected');
        };
        
        // Store WebSocket reference for later disconnection
        (window as any)[`obs_ws_${connectionName}`] = ws;
      }

    } catch (error) {
      updateObsConnectionStatus(
        connectionName, 
        'Error', 
        error instanceof Error ? error.message : 'Unknown error'
      );
    }
  };

  const disconnectFromObs = async (connectionName: string) => {
    try {
      // Close WebSocket if it exists
      const ws = (window as any)[`obs_ws_${connectionName}`];
      if (ws) {
        ws.close();
        delete (window as any)[`obs_ws_${connectionName}`];
      }

      // Call the backend to disconnect
      if (window.__TAURI__) {
        await window.__TAURI__.invoke('obs_disconnect', { connectionName });
      } else {
        // Fallback for development
        console.log('Tauri not available, WebSocket closed directly');
      }

      updateObsConnectionStatus(connectionName, 'Disconnected');
    } catch (error) {
      console.error('Failed to disconnect:', error);
      updateObsConnectionStatus(connectionName, 'Error', 'Failed to disconnect');
    }
  };

  // Test OBS status polling
  const testObsStatus = async () => {
    const { updateObsStatus } = useAppStore.getState();
    
    // Check if we have any connected OBS instances
    const connectedConnections = obsConnections.filter(c => 
      c.status === 'Connected' || c.status === 'Authenticated'
    );
    
    if (connectedConnections.length === 0) {
      console.log('No connected OBS instances to test status');
      return;
    }
    
    console.log(`Testing OBS status for ${connectedConnections.length} connections...`);
    
    // For each connected connection, try to get status
    for (const connection of connectedConnections) {
      const ws = (window as any)[`obs_ws_${connection.name}`];
      if (ws && ws.readyState === WebSocket.OPEN) {
        try {
          // Send GetRecordStatus request
          const recordStatusRequest = {
            op: 6,
            d: {
              requestType: "GetRecordStatus",
              requestId: `record_${Date.now()}`
            }
          };
          ws.send(JSON.stringify(recordStatusRequest));
          
          // Send GetStreamStatus request
          const streamStatusRequest = {
            op: 6,
            d: {
              requestType: "GetStreamStatus",
              requestId: `stream_${Date.now()}`
            }
          };
          ws.send(JSON.stringify(streamStatusRequest));
          
          // Send GetStats request for CPU usage
          const statsRequest = {
            op: 6,
            d: {
              requestType: "GetStats",
              requestId: `stats_${Date.now()}`
            }
          };
          ws.send(JSON.stringify(statsRequest));
          
        } catch (error) {
          console.error(`Failed to send status requests to ${connection.name}:`, error);
        }
      }
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Connected':
      case 'Authenticated':
        return 'text-green-500';
      case 'Connecting':
      case 'Authenticating':
        return 'text-yellow-500';
      case 'Error':
        return 'text-red-500';
      default:
        return 'text-gray-500';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'Connected':
      case 'Authenticated':
        return '🟢';
      case 'Connecting':
      case 'Authenticating':
        return '🟡';
      case 'Error':
        return '🔴';
      default:
        return '⚪';
    }
  };

  return (
    <div className="p-6 bg-gray-900 text-white rounded-lg">
      <h2 className="text-2xl font-bold mb-6">OBS WebSocket Manager</h2>
      
      {/* Test Controls */}
      <div className="mb-6 flex gap-2">
        <button
          onClick={testObsStatus}
          className="bg-purple-600 hover:bg-purple-700 px-4 py-2 rounded-lg transition-colors"
        >
          🧪 Test OBS Status
        </button>
        <button
          onClick={() => {
            const { updateObsStatus } = useAppStore.getState();
            updateObsStatus({
              is_recording: true,
              is_streaming: false,
              cpu_usage: 45,
              recording_connection: 'OBS_REC',
            });
          }}
          className="bg-orange-600 hover:bg-orange-700 px-4 py-2 rounded-lg transition-colors"
        >
          📹 Test Recording Status
        </button>
      </div>
      
      {/* Add New Connection */}
      <div className="mb-8">
        <button
          onClick={() => setIsAddingConnection(!isAddingConnection)}
          className="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-lg transition-colors"
        >
          {isAddingConnection ? 'Cancel' : '+ Add OBS Connection'}
        </button>

        {isAddingConnection && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="mt-4 p-4 bg-gray-800 rounded-lg"
          >
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium mb-1">Connection Name *</label>
                <input
                  type="text"
                  value={newConnection.name}
                  onChange={(e) => setNewConnection(prev => ({ ...prev, name: e.target.value }))}
                  className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500"
                  placeholder="e.g., Main OBS, Backup OBS"
                />
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">Protocol Version</label>
                <select
                  value={newConnection.protocol_version}
                  onChange={(e) => setNewConnection(prev => ({ 
                    ...prev, 
                    protocol_version: e.target.value as 'v4' | 'v5',
                    port: e.target.value === 'v4' ? 4444 : 4455
                  }))}
                  className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500"
                >
                  <option value="v5">OBS WebSocket v5 (Latest)</option>
                  <option value="v4">OBS WebSocket v4 (Legacy)</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">Host</label>
                <input
                  type="text"
                  value={newConnection.host}
                  onChange={(e) => setNewConnection(prev => ({ ...prev, host: e.target.value }))}
                  className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500"
                  placeholder="localhost"
                />
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">Port</label>
                <input
                  type="number"
                  value={newConnection.port}
                  onChange={(e) => setNewConnection(prev => ({ ...prev, port: parseInt(e.target.value) }))}
                  className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500"
                  placeholder={newConnection.protocol_version === 'v4' ? '4444' : '4455'}
                />
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">Password (Optional)</label>
                <input
                  type="password"
                  value={newConnection.password}
                  onChange={(e) => setNewConnection(prev => ({ ...prev, password: e.target.value }))}
                  className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500"
                  placeholder="Leave empty if disabled"
                />
              </div>

              <div className="flex items-center">
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={newConnection.enabled}
                    onChange={(e) => setNewConnection(prev => ({ ...prev, enabled: e.target.checked }))}
                    className="mr-2"
                  />
                  <span className="text-sm">Connect automatically</span>
                </label>
              </div>
            </div>

            <div className="mt-4 flex gap-2">
              <button
                onClick={addConnection}
                className="bg-green-600 hover:bg-green-700 px-4 py-2 rounded-lg transition-colors"
              >
                Add Connection
              </button>
              <button
                onClick={() => setIsAddingConnection(false)}
                className="bg-gray-600 hover:bg-gray-700 px-4 py-2 rounded-lg transition-colors"
              >
                Cancel
              </button>
            </div>
          </motion.div>
        )}
      </div>

      {/* Connection List */}
      <div className="space-y-4">
        <h3 className="text-xl font-semibold">Active Connections</h3>
        
        {obsConnections.length === 0 ? (
          <div className="text-gray-400 text-center py-8">
            No OBS connections configured. Add your first connection above.
          </div>
        ) : (
          obsConnections.map((connection) => {
            const currentStatus = connection.status || 'Disconnected';
            const isConnected = currentStatus === 'Connected' || currentStatus === 'Authenticated';

            return (
              <motion.div
                key={connection.name}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                className="p-4 bg-gray-800 rounded-lg border border-gray-700"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3">
                    <span className="text-lg">{getStatusIcon(currentStatus)}</span>
                    <div>
                      <h4 className="font-semibold">{connection.name}</h4>
                      <p className="text-sm text-gray-400">
                        {connection.host}:{connection.port} ({connection.protocol_version.toUpperCase()})
                      </p>
                      <p className={`text-sm ${getStatusColor(currentStatus)}`}>
                        {currentStatus}
                        {connection.error && ` - ${connection.error}`}
                      </p>
                    </div>
                  </div>

                  <div className="flex items-center space-x-2">
                    {isConnected ? (
                      <button
                        onClick={() => disconnectFromObs(connection.name)}
                        className="bg-red-600 hover:bg-red-700 px-3 py-1 rounded text-sm transition-colors"
                      >
                        Disconnect
                      </button>
                    ) : (
                      <button
                        onClick={() => connectToObs(connection.name)}
                        className="bg-green-600 hover:bg-green-700 px-3 py-1 rounded text-sm transition-colors"
                      >
                        Connect
                      </button>
                    )}
                    
                    <button
                      onClick={() => removeConnection(connection.name)}
                      className="bg-gray-600 hover:bg-gray-700 px-3 py-1 rounded text-sm transition-colors"
                    >
                      Remove
                    </button>
                  </div>
                </div>
              </motion.div>
            );
          })
        )}
      </div>

      {/* Protocol Information */}
      <div className="mt-8 p-4 bg-gray-800 rounded-lg">
        <h3 className="text-lg font-semibold mb-3">Protocol Information</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
          <div>
            <h4 className="font-medium text-blue-400 mb-2">OBS WebSocket v5 (Recommended)</h4>
            <ul className="text-gray-300 space-y-1">
              <li>• Latest protocol with enhanced features</li>
              <li>• SHA256 challenge-response authentication</li>
              <li>• Batch requests and event subscriptions</li>
              <li>• Better error handling and status codes</li>
              <li>• Default port: 4455</li>
            </ul>
          </div>
          <div>
            <h4 className="font-medium text-yellow-400 mb-2">OBS WebSocket v4 (Legacy)</h4>
            <ul className="text-gray-300 space-y-1">
              <li>• Legacy protocol for older OBS versions</li>
              <li>• Simple password-based authentication</li>
              <li>• Basic request/response functionality</li>
              <li>• Compatible with older plugins</li>
              <li>• Default port: 4444</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ObsWebSocketManager; 