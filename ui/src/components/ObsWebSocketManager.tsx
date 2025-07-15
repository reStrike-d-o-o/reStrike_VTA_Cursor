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

    addObsConnection(newConnection);

    // Reset form
    setNewConnection({
      name: '',
      host: 'localhost',
      port: 4455,
      protocol_version: 'v5',
      enabled: true,
    });
    setIsAddingConnection(false);

    // Connect if enabled
    if (newConnection.enabled) {
      await connectToObs(newConnection.name);
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
    
    // Handle protocol-specific connection
    if (connection.protocol_version === 'v5') {
      try {
        await handleV5Connection(ws, connection, connectionName);
      } catch (error) {
        console.error(`V5 connection failed for ${connectionName}:`, error);
        updateObsConnectionStatus(connectionName, 'Error', `Connection failed: ${error}`);
        ws.close();
        return;
      }
    } else if (connection.protocol_version === 'v4') {
      try {
        await handleV4Connection(ws, connection, connectionName);
      } catch (error) {
        console.error(`V4 connection failed for ${connectionName}:`, error);
        updateObsConnectionStatus(connectionName, 'Error', `Connection failed: ${error}`);
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
    }
  };

  // Handle OBS WebSocket v5 connection (no authentication)
  const handleV5Connection = async (ws: WebSocket, connection: ObsConnection, connectionName: string) => {
    return new Promise<void>((resolve, reject) => {
      const messageHandler = async (event: MessageEvent) => {
        try {
          const response = JSON.parse(event.data);
          console.log(`OBS Response for ${connectionName}:`, response);
          
          if (response.op === 0) { // Hello message
            // No authentication required - send Identify without auth
            const identifyRequest = {
              op: 1,
              d: {
                rpcVersion: 1,
                authentication: null,
                eventSubscriptions: 0
              }
            };
            
            console.log(`Sending Identify for ${connectionName} (no auth):`, identifyRequest);
            ws.send(JSON.stringify(identifyRequest));
          } else if (response.op === 2) { // Identified message
            console.log(`Successfully connected to ${connectionName}`);
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
      
      // Set a timeout for connection
      setTimeout(() => {
        ws.removeEventListener('message', messageHandler);
        reject(new Error('Connection timeout'));
      }, 10000); // 10 second timeout
    });
  };

  // Handle OBS WebSocket v4 connection (no authentication)
  const handleV4Connection = async (ws: WebSocket, connection: ObsConnection, connectionName: string) => {
    return new Promise<void>((resolve, reject) => {
      const messageHandler = async (event: MessageEvent) => {
        try {
          const response = JSON.parse(event.data);
          console.log(`OBS v4 Response for ${connectionName}:`, response);
          
          // V4 connection is simpler - just check if we get a valid response
          if (response['error'] || response['error-id']) {
            console.error(`V4 connection failed for ${connectionName}:`, response);
            reject(new Error(`V4 connection failed: ${response['error'] || response['error-id']}`));
          } else {
            console.log(`V4 connection successful for ${connectionName}`);
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
      
      // For v4, send a simple request to test connection
      const testRequest = {
        "request-type": "GetVersion",
        "message-id": `test_${Date.now()}`
      };
      ws.send(JSON.stringify(testRequest));
      
      // Set a timeout for connection
      setTimeout(() => {
        ws.removeEventListener('message', messageHandler);
        reject(new Error('V4 connection timeout'));
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
            log(`Sent record status request to ${connection.name}`);
          } catch (error) {
            logError(`Failed to send status request to ${connection.name}`, error);
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
    <div className="p-6 bg-gray-900 text-white min-h-screen">
      <div className="max-w-6xl mx-auto">
        {/* Password warning */}
        <div className="mb-6 p-4 bg-yellow-700 bg-opacity-80 rounded-lg flex items-center gap-3 border-l-4 border-yellow-400">
          <span className="text-2xl">⚠️</span>
          <div>
            <span className="font-semibold">Password authentication is currently disabled.</span>
            <br />
            <span className="text-yellow-200 text-sm">OBS connections do <b>not</b> accept passwords. All authentication is disabled for this version. Please ensure your OBS WebSocket server does not require a password.</span>
          </div>
        </div>
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold text-blue-400 mb-2">
              OBS WebSocket Manager
            </h1>
            <p className="text-gray-400">
              Manage OBS Studio connections and monitor status
            </p>
          </div>
          <div className="flex gap-4">
            <button
              onClick={testObsStatus}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
            >
              Test Status
            </button>
            <button
              onClick={() => setIsAddingConnection(true)}
              className="px-4 py-2 bg-green-600 hover:bg-green-700 rounded-lg transition-colors"
            >
              Add Connection
            </button>
          </div>
        </div>

        {/* Add Connection Modal */}
        {isAddingConnection && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
          >
            <motion.div
              initial={{ scale: 0.9, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              className="bg-gray-800 p-6 rounded-lg w-96"
            >
              <h2 className="text-xl font-bold mb-4">Add OBS Connection</h2>
              
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium mb-1">Connection Name</label>
                  <input
                    type="text"
                    value={newConnection.name}
                    onChange={(e) => setNewConnection(prev => ({ ...prev, name: e.target.value }))}
                    className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500"
                    placeholder="e.g., Main OBS"
                  />
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
                    onChange={(e) => setNewConnection(prev => ({ ...prev, port: parseInt(e.target.value) || 4455 }))}
                    className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500"
                    placeholder="4455"
                  />
                </div>
                
                <div>
                  <label className="block text-sm font-medium mb-1">Protocol Version</label>
                  <select
                    value={newConnection.protocol_version}
                    onChange={(e) => setNewConnection(prev => ({ ...prev, protocol_version: e.target.value as 'v4' | 'v5' }))}
                    className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500"
                  >
                    <option value="v5">v5 (Recommended)</option>
                    <option value="v4">v4 (Legacy)</option>
                  </select>
                </div>
                
                <div className="flex items-center">
                  <input
                    type="checkbox"
                    id="enabled"
                    checked={newConnection.enabled}
                    onChange={(e) => setNewConnection(prev => ({ ...prev, enabled: e.target.checked }))}
                    className="mr-2"
                  />
                  <label htmlFor="enabled" className="text-sm">Connect automatically</label>
                </div>
              </div>
              
              <div className="flex gap-3 mt-6">
                <button
                  onClick={addConnection}
                  className="flex-1 px-4 py-2 bg-green-600 hover:bg-green-700 rounded-lg transition-colors"
                >
                  Add Connection
                </button>
                <button
                  onClick={() => setIsAddingConnection(false)}
                  className="flex-1 px-4 py-2 bg-gray-600 hover:bg-gray-700 rounded-lg transition-colors"
                >
                  Cancel
                </button>
              </div>
            </motion.div>
          </motion.div>
        )}

        {/* Connections List */}
        <div className="grid gap-6">
          {obsConnections.map((connection) => (
            <motion.div
              key={connection.name}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              className="bg-gray-800 p-6 rounded-lg border border-gray-700"
            >
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-3">
                  <span className="text-2xl">{getStatusIcon(connection.status)}</span>
                  <div>
                    <h3 className="text-xl font-semibold">{connection.name}</h3>
                    <p className="text-gray-400">
                      {connection.host}:{connection.port} ({connection.protocol_version})
                    </p>
                  </div>
                </div>
                
                <div className="flex items-center gap-3">
                  <span className={`font-medium ${getStatusColor(connection.status)}`}>
                    {connection.status}
                  </span>
                  
                  {connection.status === 'Disconnected' && (
                    <button
                      onClick={() => connectToObs(connection.name)}
                      className="px-3 py-1 bg-blue-600 hover:bg-blue-700 rounded text-sm transition-colors"
                    >
                      Connect
                    </button>
                  )}
                  
                  {(connection.status === 'Connected' || connection.status === 'Authenticated') && (
                    <button
                      onClick={() => disconnectFromObs(connection.name)}
                      className="px-3 py-1 bg-red-600 hover:bg-red-700 rounded text-sm transition-colors"
                    >
                      Disconnect
                    </button>
                  )}
                  
                  <button
                    onClick={() => removeConnection(connection.name)}
                    className="px-3 py-1 bg-gray-600 hover:bg-gray-700 rounded text-sm transition-colors"
                  >
                    Remove
                  </button>
                </div>
              </div>
              
              {connection.error && (
                <div className="mt-3 p-3 bg-red-900 border border-red-700 rounded text-red-200 text-sm">
                  Error: {connection.error}
                </div>
              )}
            </motion.div>
          ))}
        </div>

        {/* Features List */}
        <div className="mt-12 bg-gray-800 p-6 rounded-lg border border-gray-700">
          <h2 className="text-xl font-bold mb-4 text-blue-400">OBS WebSocket Features</h2>
          <div className="grid md:grid-cols-2 gap-6">
            <div>
              <h3 className="font-semibold mb-2 text-green-400">Protocol v5 Support</h3>
              <ul className="text-sm text-gray-300 space-y-1">
                <li>• Modern WebSocket protocol</li>
                <li>• Enhanced error handling</li>
                <li>• Better performance</li>
                <li>• Future-proof compatibility</li>
              </ul>
            </div>
            <div>
              <h3 className="font-semibold mb-2 text-yellow-400">Protocol v4 Support</h3>
              <ul className="text-sm text-gray-300 space-y-1">
                <li>• Legacy protocol support</li>
                <li>• Backward compatibility</li>
                <li>• Simple connection handling</li>
                <li>• Basic OBS control</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ObsWebSocketManager; 