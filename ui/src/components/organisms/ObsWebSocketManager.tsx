import React, { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import Button from '../atoms/Button';
import { StatusDot } from '../atoms/StatusDot';

// Use the proper Tauri v2 invoke function with fallback
const invoke = async (command: string, args?: any) => {
  try {
    // Try the proper Tauri v2 API first
    return await tauriInvoke(command, args);
  } catch (error) {
    // If that fails, try the global window.__TAURI__.invoke
    if (typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.invoke) {
      return await window.__TAURI__.invoke(command, args);
    }
    throw new Error('Tauri invoke method not available - ensure app is running in desktop mode');
  }
};

interface ObsConnection {
  name: string;
  host: string;
  port: number;
  status: 'disconnected' | 'connecting' | 'connected' | 'error';
  error?: string;
}

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
  const [connections, setConnections] = useState<ObsConnection[]>([
    {
      name: 'OBS Studio 1',
      host: 'localhost',
      port: 4455,
      status: 'disconnected'
    }
  ]);

  // Initialize Windows-specific features
  useEffect(() => {
    const initializeWindowsFeatures = async () => {
      try {
        // Initialize Tauri commands for OBS integration
        if (isTauriAvailable()) {
          console.log('✅ Tauri environment detected for OBS integration');
          
          // Initialize OBS status monitoring
          // Initialize WebSocket connection management
          // Initialize recording controls
        }
      } catch (error) {
        console.error('❌ Failed to initialize Windows features:', error);
      }
    };

    initializeWindowsFeatures();
  }, []);

  const connectToObs = async (connectionName: string) => {
    console.log(`Connecting to OBS: ${connectionName}`);
    updateConnectionStatus(connectionName, 'connecting');
    
    try {
      // Use Tauri command for OBS connection
      if (isTauriAvailable()) {
        const url = `ws://localhost:4455`;
        const result = await invoke('obs_connect', { url });
        
        if (result && typeof result === 'object' && 'success' in result && result.success) {
          console.log(`✅ Successfully connected to OBS: ${connectionName}`);
          updateConnectionStatus(connectionName, 'connected');
        } else {
          console.error(`❌ Failed to connect to OBS: ${connectionName}`, result);
          updateConnectionStatus(connectionName, 'error', 'Connection failed');
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
        } else {
          console.error(`❌ Failed to disconnect from OBS: ${connectionName}`, result);
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
      if (isTauriAvailable()) {
        const result = await invoke('obs_get_status');
        
        if (result && typeof result === 'object' && 'success' in result && result.success) {
          console.log('✅ OBS status retrieved successfully', result);
          return result;
        } else {
          console.error('❌ Failed to get OBS status', result);
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
          <Button onClick={addConnection} variant="primary" size="sm">
            Add Connection
          </Button>
        </div>

        {/* Connection List */}
        <div className="space-y-4">
          {connections.map((connection) => (
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
          ))}
        </div>
      </div>

      {/* Test Controls */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-white mb-4">Test Controls</h3>
        <div className="flex space-x-4">
          <Button
            onClick={testObsStatus}
            variant="secondary"
            size="sm"
          >
            Test OBS Status
          </Button>
          <Button
            onClick={() => connections.forEach(conn => connectToObs(conn.name))}
            variant="primary"
            size="sm"
          >
            Connect All
          </Button>
          <Button
            onClick={() => connections.forEach(conn => disconnectFromObs(conn.name))}
            variant="secondary"
            size="sm"
          >
            Disconnect All
          </Button>
        </div>
      </div>
    </div>
  );
};

export default ObsWebSocketManager;