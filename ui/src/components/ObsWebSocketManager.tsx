import React, { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { useAppStore, ObsConnection } from '../stores';
import { env, isWindows, invokeTauri, log, logError } from '../config/environment';

// TypeScript declarations for Tauri
declare global {
  interface Window {
    __TAURI__?: {
      invoke: (command: string, args?: any) => Promise<any>;
    };
  }
}

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
      log(`Connecting to OBS: ${connectionName} (Environment: ${env.environment})`);
      
      // Update status to connecting
      updateObsConnectionStatus(connectionName, 'Connecting');

      // Find the connection configuration
      const connection = obsConnections.find(c => c.name === connectionName);
      if (!connection) {
        throw new Error('Connection not found');
      }

      if (isWindows()) {
        // Use Tauri commands for Windows environment
        log(`Using Tauri commands for ${connectionName}`);
        try {
          await invokeTauri('obs_connect', { connectionName });
          log(`Tauri connection successful for ${connectionName}`);
        } catch (error) {
          logError(`Tauri connection failed for ${connectionName}`, error);
          updateObsConnectionStatus(connectionName, 'Error', `Tauri connection failed: ${error}`);
        }
      } else {
        // Use direct WebSocket for web environment
        log(`Using direct WebSocket for ${connectionName}`);
        await connectWebSocketDirect(connectionName, connection);
      }

    } catch (error) {
      logError(`Connection failed for ${connectionName}`, error);
      updateObsConnectionStatus(
        connectionName, 
        'Error', 
        error instanceof Error ? error.message : 'Unknown error'
      );
    }
  };

  // Direct WebSocket connection for web environment
  const connectWebSocketDirect = async (connectionName: string, connection: ObsConnection) => {
    console.log(`Testing WebSocket connection to ${connection.host}:${connection.port}...`);
    
    // Create a simple WebSocket test
    const wsUrl = `ws://${connection.host}:${connection.port}`;
    const ws = new WebSocket(wsUrl);
    
    ws.onopen = () => {
      console.log(`WebSocket connected to ${connectionName}`);
      updateObsConnectionStatus(connectionName, 'Connected');
    };
    
    ws.onerror = (error) => {
      console.error(`WebSocket error for ${connectionName}:`, error);
      updateObsConnectionStatus(connectionName, 'Error', 'WebSocket connection failed');
    };
    
    ws.onclose = () => {
      console.log(`WebSocket closed for ${connectionName}`);
      updateObsConnectionStatus(connectionName, 'Disconnected');
    };
    
    // Handle authentication based on protocol version
    if (connection.protocol_version === 'v5') {
      try {
        await handleV5Authentication(ws, connection, connectionName);
      } catch (error) {
        console.error(`V5 authentication failed for ${connectionName}:`, error);
        updateObsConnectionStatus(connectionName, 'Error', `Authentication failed: ${error}`);
        ws.close();
        return;
      }
    } else if (connection.protocol_version === 'v4') {
      try {
        await handleV4Authentication(ws, connection, connectionName);
      } catch (error) {
        console.error(`V4 authentication failed for ${connectionName}:`, error);
        updateObsConnectionStatus(connectionName, 'Error', `Authentication failed: ${error}`);
        ws.close();
        return;
      }
    } else {
      // Unknown protocol version
      console.error(`Unknown protocol version for ${connectionName}: ${connection.protocol_version}`);
      updateObsConnectionStatus(connectionName, 'Error', `Unknown protocol version: ${connection.protocol_version}`);
      ws.close();
      return;
    }
    
    // Store WebSocket reference for later disconnection
    (window as any)[`obs_ws_${connectionName}`] = ws;
  };

  const disconnectFromObs = async (connectionName: string) => {
    try {
      log(`Disconnecting from OBS: ${connectionName} (Environment: ${env.environment})`);

      if (isWindows()) {
        // Use Tauri commands for Windows environment
        log(`Using Tauri disconnect for ${connectionName}`);
        try {
          await invokeTauri('obs_disconnect', { connectionName });
          log(`Tauri disconnect successful for ${connectionName}`);
        } catch (error) {
          logError(`Tauri disconnect failed for ${connectionName}`, error);
        }
      } else {
        // Use direct WebSocket disconnect for web environment
        log(`Using direct WebSocket disconnect for ${connectionName}`);
        const ws = (window as any)[`obs_ws_${connectionName}`];
        if (ws) {
          ws.close();
          delete (window as any)[`obs_ws_${connectionName}`];
          log(`WebSocket closed for ${connectionName}`);
        }
      }

      updateObsConnectionStatus(connectionName, 'Disconnected');
    } catch (error) {
      logError(`Disconnect failed for ${connectionName}`, error);
      updateObsConnectionStatus(connectionName, 'Error', 'Failed to disconnect');
    }
  };

  // Generate authentication response for OBS WebSocket v5
  const generateAuthResponse = async (password: string, challenge: string, salt: string): Promise<string> => {
    try {
      // OBS WebSocket v5 uses SHA256(challenge + salt + password)
      const combined = challenge + salt + password;
      
      // Convert string to ArrayBuffer
      const encoder = new TextEncoder();
      const data = encoder.encode(combined);
      
      // Generate SHA256 hash
      const hashBuffer = await crypto.subtle.digest('SHA-256', data);
      
      // Convert to base64 using a more reliable method
      const hashArray = new Uint8Array(hashBuffer);
      let binary = '';
      for (let i = 0; i < hashArray.length; i++) {
        binary += String.fromCharCode(hashArray[i]);
      }
      const hashBase64 = btoa(binary);
      
      console.log('Auth debug:', {
        challenge: challenge.substring(0, 20) + '...',
        salt: salt.substring(0, 20) + '...',
        password: password.substring(0, 3) + '***', // Don't log full password
        combinedLength: combined.length,
        hashBase64: hashBase64.substring(0, 20) + '...'
      });
      
      return hashBase64;
    } catch (error) {
      console.error('Authentication generation failed:', error);
      throw error;
    }
  };

  // Handle OBS WebSocket v5 authentication properly
  const handleV5Authentication = async (ws: WebSocket, connection: ObsConnection, connectionName: string) => {
    return new Promise<void>((resolve, reject) => {
      const messageHandler = async (event: MessageEvent) => {
        try {
          const response = JSON.parse(event.data);
          console.log(`OBS Response for ${connectionName}:`, response);
          
          if (response.op === 0) { // Hello message
            // Handle authentication challenge
            if (response.d.authentication && connection.password) {
              const { challenge, salt } = response.d.authentication;
              
              try {
                // Generate proper authentication response
                const authResponse = await generateAuthResponse(connection.password, challenge, salt);
                
                const identifyRequest = {
                  op: 1,
                  d: {
                    rpcVersion: 1,
                    authentication: authResponse,
                    eventSubscriptions: 0
                  }
                };
                
                console.log(`Sending authentication for ${connectionName}:`, identifyRequest);
                ws.send(JSON.stringify(identifyRequest));
              } catch (error) {
                console.error(`Authentication failed for ${connectionName}:`, error);
                reject(new Error(`Authentication failed: ${error}`));
              }
            } else {
              // No authentication required
              const identifyRequest = {
                op: 1,
                d: {
                  rpcVersion: 1,
                  authentication: null,
                  eventSubscriptions: 0
                }
              };
              
              console.log(`No authentication required for ${connectionName}`);
              ws.send(JSON.stringify(identifyRequest));
            }
          } else if (response.op === 2) { // Identified message
            console.log(`Successfully authenticated with ${connectionName}`);
            updateObsConnectionStatus(connectionName, 'Authenticated');
            ws.removeEventListener('message', messageHandler);
            resolve();
          } else if (response.op === 7) { // RequestResponse
            console.log(`Request response from ${connectionName}:`, response);
          }
        } catch (error) {
          console.error(`Failed to parse OBS response for ${connectionName}:`, error);
          reject(error);
        }
      };
      
      ws.addEventListener('message', messageHandler);
      
      // Set a timeout for authentication
      setTimeout(() => {
        ws.removeEventListener('message', messageHandler);
        reject(new Error('Authentication timeout'));
      }, 10000); // 10 second timeout
    });
  };

  // Handle OBS WebSocket v4 authentication
  const handleV4Authentication = async (ws: WebSocket, connection: ObsConnection, connectionName: string) => {
    return new Promise<void>((resolve, reject) => {
      const messageHandler = async (event: MessageEvent) => {
        try {
          const response = JSON.parse(event.data);
          console.log(`OBS v4 Response for ${connectionName}:`, response);
          
          // V4 authentication is simpler - just check if we get a valid response
          if (response['error'] || response['error-id']) {
            console.error(`V4 authentication failed for ${connectionName}:`, response);
            reject(new Error(`V4 authentication failed: ${response['error'] || response['error-id']}`));
          } else {
            console.log(`V4 authentication successful for ${connectionName}`);
            updateObsConnectionStatus(connectionName, 'Authenticated');
            ws.removeEventListener('message', messageHandler);
            resolve();
          }
        } catch (error) {
          console.error(`Failed to parse OBS v4 response for ${connectionName}:`, error);
          reject(error);
        }
      };
      
      ws.addEventListener('message', messageHandler);
      
      // For v4, we can send a simple request to test authentication
      if (connection.password) {
        const authRequest = {
          "request-type": "GetAuthRequired",
          "message-id": `auth_${Date.now()}`
        };
        ws.send(JSON.stringify(authRequest));
      } else {
        // No password required, just resolve
        updateObsConnectionStatus(connectionName, 'Authenticated');
        ws.removeEventListener('message', messageHandler);
        resolve();
      }
      
      // Set a timeout for authentication
      setTimeout(() => {
        ws.removeEventListener('message', messageHandler);
        reject(new Error('V4 authentication timeout'));
      }, 10000); // 10 second timeout
    });
  };

  // Test OBS status polling
  const testObsStatus = async () => {
    log(`Testing OBS status (Environment: ${env.environment})`);
    
    if (isWindows()) {
      // Use Tauri commands for Windows environment
      log('Using Tauri commands for OBS status');
      try {
        const { updateObsStatus } = useAppStore.getState();
        const status = await invokeTauri('obs_get_status');
        if (status.success && status.data) {
          updateObsStatus(status.data);
          log('OBS status updated via Tauri');
        }
      } catch (error) {
        logError('Tauri OBS status failed', error);
      }
    } else {
      // Use direct WebSocket for web environment
      log('Using direct WebSocket for OBS status');
      const { updateObsStatus } = useAppStore.getState();
      
      // Check if we have any connected OBS instances
      const connectedConnections = obsConnections.filter(c => 
        c.status === 'Connected' || c.status === 'Authenticated'
      );
      
      if (connectedConnections.length === 0) {
        log('No connected OBS instances to test status');
        return;
      }
      
      log(`Testing OBS status for ${connectedConnections.length} connections...`);
      
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
            logError(`Failed to send status requests to ${connection.name}`, error);
          }
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