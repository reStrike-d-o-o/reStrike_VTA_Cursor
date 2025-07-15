import React, { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { useAppStore } from '../stores';

interface ObsConnection {
  name: string;
  host: string;
  port: number;
  status: 'disconnected' | 'connecting' | 'connected' | 'error';
  error?: string;
}

const ObsWebSocketManager: React.FC = () => {
  const { obsConnections } = useAppStore();
  const [connections, setConnections] = useState<ObsConnection[]>([
    { name: 'OBS Studio', host: 'localhost', port: 4455, status: 'disconnected' }
  ]);

  // Windows-specific initialization
  useEffect(() => {
    console.log('Initializing Windows-only OBS WebSocket Manager');
    initializeWindowsFeatures();
  }, []);

  const initializeWindowsFeatures = async () => {
    try {
      // Initialize Tauri commands for OBS integration
      if (window.__TAURI__) {
        console.log('✅ Tauri environment detected for OBS integration');
        
        // Initialize OBS WebSocket connection
        // Initialize video playback system
        // Initialize PSS protocol listener
      }
    } catch (error) {
      console.error('❌ Failed to initialize Windows features:', error);
    }
  };

  const connectToObs = async (connectionName: string) => {
    console.log(`Connecting to OBS: ${connectionName}`);
    
    try {
      // Use Tauri command for OBS connection
      if (window.__TAURI__) {
        const result = await window.__TAURI__.invoke('obs_connect', {
          url: `ws://localhost:4455`
        });
        
        if (result.success) {
          console.log(`✅ Successfully connected to OBS: ${connectionName}`);
          updateConnectionStatus(connectionName, 'connected');
        } else {
          console.error(`❌ Failed to connect to OBS: ${connectionName}`, result.error);
          updateConnectionStatus(connectionName, 'error', result.error);
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
      if (window.__TAURI__) {
        const result = await window.__TAURI__.invoke('obs_disconnect', {
          connectionName
        });
        
        if (result.success) {
          console.log(`✅ Successfully disconnected from OBS: ${connectionName}`);
          updateConnectionStatus(connectionName, 'disconnected');
        } else {
          console.error(`❌ Failed to disconnect from OBS: ${connectionName}`, result.error);
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

  const testObsStatus = async () => {
    console.log('Testing OBS status');
    
    try {
      // Use Tauri command for OBS status
      if (window.__TAURI__) {
        const result = await window.__TAURI__.invoke('obs_get_status');
        
        if (result.success) {
          console.log('✅ OBS status retrieved successfully', result.data);
          return result.data;
        } else {
          console.error('❌ Failed to get OBS status', result.error);
          return null;
        }
      } else {
        console.error('❌ Tauri not available for OBS status');
        return null;
      }
    } catch (error) {
      console.error('❌ Error getting OBS status', error);
      return null;
    }
  };

  const updateConnectionStatus = (name: string, status: ObsConnection['status'], error?: string) => {
    setConnections(prev => prev.map(conn => 
      conn.name === name 
        ? { ...conn, status, error }
        : conn
    ));
  };

  const addConnection = () => {
    const newConnection: ObsConnection = {
      name: `OBS Studio ${connections.length + 1}`,
      host: 'localhost',
      port: 4455,
      status: 'disconnected'
    };
    setConnections(prev => [...prev, newConnection]);
  };

  const removeConnection = (name: string) => {
    setConnections(prev => prev.filter(conn => conn.name !== name));
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
          <button
            onClick={addConnection}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
          >
            Add Connection
          </button>
        </div>

        <div className="space-y-4">
          {connections.map((connection) => (
            <motion.div
              key={connection.name}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              className="bg-gray-700 rounded-lg p-4"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-4">
                  <div className={`w-3 h-3 rounded-full ${getStatusColor(connection.status)}`} />
                  <div>
                    <h4 className="text-white font-medium">{connection.name}</h4>
                    <p className="text-gray-400 text-sm">
                      {connection.host}:{connection.port}
                    </p>
                  </div>
                </div>
                
                <div className="flex items-center space-x-3">
                  <span className={`px-3 py-1 rounded-full text-sm font-medium ${
                    connection.status === 'connected' 
                      ? 'bg-green-600 text-white'
                      : connection.status === 'connecting'
                      ? 'bg-yellow-600 text-white'
                      : connection.status === 'error'
                      ? 'bg-red-600 text-white'
                      : 'bg-gray-600 text-gray-300'
                  }`}>
                    {getStatusText(connection.status)}
                  </span>
                  
                  <div className="flex space-x-2">
                    {connection.status === 'disconnected' && (
                      <button
                        onClick={() => connectToObs(connection.name)}
                        className="px-3 py-1 bg-green-600 hover:bg-green-700 text-white text-sm rounded transition-colors"
                      >
                        Connect
                      </button>
                    )}
                    
                    {connection.status === 'connected' && (
                      <button
                        onClick={() => disconnectFromObs(connection.name)}
                        className="px-3 py-1 bg-red-600 hover:bg-red-700 text-white text-sm rounded transition-colors"
                      >
                        Disconnect
                      </button>
                    )}
                    
                    <button
                      onClick={() => testObsStatus()}
                      className="px-3 py-1 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded transition-colors"
                    >
                      Test
                    </button>
                    
                    <button
                      onClick={() => removeConnection(connection.name)}
                      className="px-3 py-1 bg-gray-600 hover:bg-gray-500 text-white text-sm rounded transition-colors"
                    >
                      Remove
                    </button>
                  </div>
                </div>
              </div>
              
              {connection.error && (
                <div className="mt-3 p-3 bg-red-900 border border-red-700 rounded-lg">
                  <p className="text-red-300 text-sm">{connection.error}</p>
                </div>
              )}
            </motion.div>
          ))}
        </div>
      </div>

      {/* OBS Status Information */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-white mb-4">OBS Status</h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <span className="text-gray-300">Recording</span>
              <span className="w-3 h-3 bg-red-500 rounded-full"></span>
            </div>
            <p className="text-white font-medium mt-1">Not Recording</p>
          </div>
          
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <span className="text-gray-300">Streaming</span>
              <span className="w-3 h-3 bg-red-500 rounded-full"></span>
            </div>
            <p className="text-white font-medium mt-1">Not Streaming</p>
          </div>
          
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <span className="text-gray-300">CPU Usage</span>
              <span className="w-3 h-3 bg-green-500 rounded-full"></span>
            </div>
            <p className="text-white font-medium mt-1">0.0%</p>
          </div>
        </div>
      </div>

      {/* Connection Instructions */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-white mb-4">Setup Instructions</h3>
        <div className="space-y-3 text-gray-300">
          <p>1. Open OBS Studio on your Windows machine</p>
          <p>2. Go to Tools → WebSocket Server Settings</p>
          <p>3. Enable WebSocket server on port 4455</p>
          <p>4. <strong>Important:</strong> Disable authentication (no password)</p>
          <p>5. Click "Connect" above to establish connection</p>
        </div>
      </div>
    </div>
  );
};

export default ObsWebSocketManager;