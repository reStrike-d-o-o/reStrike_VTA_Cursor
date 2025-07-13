import React, { useState, useEffect } from 'react';
import { motion } from 'framer-motion';

interface ObsConnectionConfig {
  name: string;
  host: string;
  port: number;
  password?: string;
  protocol_version: 'v4' | 'v5';
  enabled: boolean;
}

interface ObsConnectionStatus {
  connection_name: string;
  status: 'Disconnected' | 'Connecting' | 'Connected' | 'Authenticating' | 'Authenticated' | 'Error';
  error?: string;
}

const ObsWebSocketManager: React.FC = () => {
  const [connections, setConnections] = useState<ObsConnectionConfig[]>([]);
  const [statuses, setStatuses] = useState<ObsConnectionStatus[]>([]);
  const [newConnection, setNewConnection] = useState<ObsConnectionConfig>({
    name: '',
    host: 'localhost',
    port: 4455,
    password: '',
    protocol_version: 'v5',
    enabled: true,
  });
  const [isAddingConnection, setIsAddingConnection] = useState(false);

  // Load connections from storage on component mount
  useEffect(() => {
    loadConnections();
  }, []);

  const loadConnections = async () => {
    try {
      // In a real implementation, this would load from the backend
      const savedConnections = localStorage.getItem('obs_connections');
      if (savedConnections) {
        setConnections(JSON.parse(savedConnections));
      }
    } catch (error) {
      console.error('Failed to load OBS connections:', error);
    }
  };

  const saveConnections = async (newConnections: ObsConnectionConfig[]) => {
    try {
      localStorage.setItem('obs_connections', JSON.stringify(newConnections));
      setConnections(newConnections);
    } catch (error) {
      console.error('Failed to save OBS connections:', error);
    }
  };

  const addConnection = async () => {
    if (!newConnection.name.trim()) {
      alert('Connection name is required');
      return;
    }

    if (connections.some(c => c.name === newConnection.name)) {
      alert('Connection name must be unique');
      return;
    }

    const connectionToAdd = {
      ...newConnection,
      password: newConnection.password || undefined,
    };

    const updatedConnections = [...connections, connectionToAdd];
    await saveConnections(updatedConnections);

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
    const updatedConnections = connections.filter(c => c.name !== name);
    await saveConnections(updatedConnections);
    
    // Remove from statuses
    setStatuses(prev => prev.filter(s => s.connection_name !== name));
  };

  const connectToObs = async (connectionName: string) => {
    try {
      // Update status to connecting
      setStatuses(prev => [
        ...prev.filter(s => s.connection_name !== connectionName),
        { connection_name: connectionName, status: 'Connecting' }
      ]);

      // In a real implementation, this would call the backend
      // await window.__TAURI__.invoke('obs_connect', { connectionName });

      // Simulate connection process
      setTimeout(() => {
        setStatuses(prev => [
          ...prev.filter(s => s.connection_name !== connectionName),
          { connection_name: connectionName, status: 'Connected' }
        ]);
      }, 1000);

    } catch (error) {
      setStatuses(prev => [
        ...prev.filter(s => s.connection_name !== connectionName),
        { 
          connection_name: connectionName, 
          status: 'Error',
          error: error instanceof Error ? error.message : 'Unknown error'
        }
      ]);
    }
  };

  const disconnectFromObs = async (connectionName: string) => {
    try {
      // In a real implementation, this would call the backend
      // await window.__TAURI__.invoke('obs_disconnect', { connectionName });

      setStatuses(prev => [
        ...prev.filter(s => s.connection_name !== connectionName),
        { connection_name: connectionName, status: 'Disconnected' }
      ]);
    } catch (error) {
      console.error('Failed to disconnect:', error);
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
        
        {connections.length === 0 ? (
          <div className="text-gray-400 text-center py-8">
            No OBS connections configured. Add your first connection above.
          </div>
        ) : (
          connections.map((connection) => {
            const status = statuses.find(s => s.connection_name === connection.name);
            const currentStatus = status?.status || 'Disconnected';
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
                        {status?.error && ` - ${status.error}`}
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