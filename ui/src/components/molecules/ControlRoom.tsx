import React, { useState, useEffect } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import { StatusDot } from '../atoms/StatusDot';
import { invoke } from '@tauri-apps/api/core';

interface StrConnection {
  name: string;
  host: string;
  port: number;
  status: 'Disconnected' | 'Connecting' | 'Connected' | 'Error';
  notes?: string;
}

interface ControlRoomState {
  isAuthenticated: boolean;
  isLoading: boolean;
  sessionId: string | null;
  password: string;
  connections: StrConnection[];
  showAddConnection: boolean;
  newConnection: {
    name: string;
    host: string;
    port: string;
    password: string;
    notes: string;
  };
}

const ControlRoom: React.FC = () => {
  const [state, setState] = useState<ControlRoomState>({
    isAuthenticated: false,
    isLoading: false,
    sessionId: null,
    password: '',
    connections: [],
    showAddConnection: false,
    newConnection: {
      name: '',
      host: '',
      port: '4455',
      password: '',
      notes: ''
    }
  });

  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Clear messages after 5 seconds
  useEffect(() => {
    if (error || success) {
      const timer = setTimeout(() => {
        setError(null);
        setSuccess(null);
      }, 5000);
      return () => clearTimeout(timer);
    }
  }, [error, success]);

  const handleAuthenticate = async () => {
    if (!state.password.trim()) {
      setError('Please enter a password');
      return;
    }

    setState(prev => ({ ...prev, isLoading: true }));
    setError(null);

    try {
      const result = await invoke('control_room_authenticate_async', {
        password: state.password
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; session_id?: string; error?: string };
        
        if (response.success) {
          setState(prev => ({
            ...prev,
            isAuthenticated: true,
            sessionId: response.session_id || 'authenticated',
            isLoading: false
          }));
          setSuccess('Successfully authenticated to Control Room');
          await loadConnections(response.session_id || 'authenticated');
        } else {
          setError(response.error || 'Authentication failed');
          setState(prev => ({ ...prev, isLoading: false }));
        }
      }
    } catch (err) {
      console.error('Authentication error:', err);
      setError('Authentication failed. Please check your password.');
      setState(prev => ({ ...prev, isLoading: false }));
    }
  };

  const loadConnections = async (sessionId: string) => {
    try {
      const result = await invoke('control_room_get_str_connections', {
        sessionId: sessionId
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; connections?: string[]; error?: string };
        if (response.success && response.connections) {
          // Convert connection names to connection objects
          const connections: StrConnection[] = response.connections.map(name => ({
            name,
            host: 'Unknown', // We'll need to get this from the backend later
            port: 4455, // Default port
            status: 'Disconnected' as const,
            notes: undefined
          }));
          setState(prev => ({ ...prev, connections }));
        } else {
          setState(prev => ({ ...prev, connections: [] }));
        }
      }
    } catch (err) {
      console.error('Failed to load connections:', err);
      // Start with empty list on error
      setState(prev => ({ ...prev, connections: [] }));
    }
  };

  const handleLogout = () => {
    setState(prev => ({
      ...prev,
      isAuthenticated: false,
      sessionId: null,
      password: '',
      connections: []
    }));
    setSuccess('Logged out of Control Room');
  };

  const handleAddConnection = async () => {
    if (!state.newConnection.name.trim() || !state.newConnection.host.trim()) {
      setError('Please enter connection name and host');
      return;
    }

    const port = parseInt(state.newConnection.port);
    if (isNaN(port) || port < 1 || port > 65535) {
      setError('Please enter a valid port number (1-65535)');
      return;
    }

    setState(prev => ({ ...prev, isLoading: true }));

    try {
      const result = await invoke('control_room_add_str_connection', {
        sessionId: state.sessionId,
        name: state.newConnection.name.trim(),
        host: state.newConnection.host.trim(),
        port: port,
        password: state.newConnection.password || null,
        notes: state.newConnection.notes.trim() || null
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; message?: string; error?: string };
        
        if (response.success) {
          // Add the connection to the list locally
          const newConnection: StrConnection = {
            name: state.newConnection.name.trim(),
            host: state.newConnection.host.trim(),
            port: port,
            status: 'Disconnected',
            notes: state.newConnection.notes.trim() || undefined
          };

          setState(prev => ({
            ...prev,
            connections: [...prev.connections, newConnection],
            showAddConnection: false,
            newConnection: {
              name: '',
              host: '',
              port: '4455',
              password: '',
              notes: ''
            },
            isLoading: false
          }));

          setSuccess(response.message || `STR connection "${newConnection.name}" added successfully`);
        } else {
          setError(response.error || 'Failed to add STR connection');
          setState(prev => ({ ...prev, isLoading: false }));
        }
      }
    } catch (err) {
      console.error('Failed to add connection:', err);
      setError('Failed to add STR connection');
      setState(prev => ({ ...prev, isLoading: false }));
    }
  };

  const getStatusColor = (status: StrConnection['status']) => {
    switch (status) {
      case 'Connected': return 'green';
      case 'Connecting': return 'yellow';
      case 'Error': return 'red';
      default: return 'gray';
    }
  };

  const getStatusText = (status: StrConnection['status']) => {
    switch (status) {
      case 'Connected': return 'Connected';
      case 'Connecting': return 'Connecting...';
      case 'Error': return 'Error';
      default: return 'Disconnected';
    }
  };

  const handleConnect = async (connectionName: string) => {
    setState(prev => ({
      ...prev,
      connections: prev.connections.map(conn =>
        conn.name === connectionName ? { ...conn, status: 'Connecting' } : conn
      )
    }));

    try {
      const result = await invoke('control_room_connect_str', {
        sessionId: state.sessionId,
        strName: connectionName
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; message?: string; error?: string };
        
        setState(prev => ({
          ...prev,
          connections: prev.connections.map(conn =>
            conn.name === connectionName 
              ? { ...conn, status: response.success ? 'Connected' : 'Error' }
              : conn
          )
        }));

        if (response.success) {
          setSuccess(response.message || `Connected to ${connectionName}`);
        } else {
          setError(response.error || `Failed to connect to ${connectionName}`);
        }
      }
    } catch (err) {
      console.error('Connection error:', err);
      setState(prev => ({
        ...prev,
        connections: prev.connections.map(conn =>
          conn.name === connectionName ? { ...conn, status: 'Error' } : conn
        )
      }));
      setError(`Failed to connect to ${connectionName}`);
    }
  };

  const handleDisconnect = async (connectionName: string) => {
    try {
      const result = await invoke('control_room_disconnect_str', {
        sessionId: state.sessionId,
        strName: connectionName
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; message?: string; error?: string };
        
        setState(prev => ({
          ...prev,
          connections: prev.connections.map(conn =>
            conn.name === connectionName ? { ...conn, status: 'Disconnected' } : conn
          )
        }));

        if (response.success) {
          setSuccess(response.message || `Disconnected from ${connectionName}`);
        } else {
          setError(response.error || `Failed to disconnect from ${connectionName}`);
        }
      }
    } catch (err) {
      console.error('Disconnection error:', err);
      setError(`Failed to disconnect from ${connectionName}`);
    }
  };

  const handleRemove = async (connectionName: string) => {
    if (!confirm(`Are you sure you want to remove the STR connection "${connectionName}"?`)) {
      return;
    }

    try {
      const result = await invoke('control_room_remove_str_connection', {
        sessionId: state.sessionId,
        strName: connectionName
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; message?: string; error?: string };
        
        if (response.success) {
          setState(prev => ({
            ...prev,
            connections: prev.connections.filter(conn => conn.name !== connectionName)
          }));
          setSuccess(response.message || `STR connection "${connectionName}" removed`);
        } else {
          setError(response.error || `Failed to remove STR connection "${connectionName}"`);
        }
      }
    } catch (err) {
      console.error('Remove error:', err);
      setError(`Failed to remove STR connection "${connectionName}"`);
    }
  };

  if (!state.isAuthenticated) {
    return (
      <div className="space-y-6">
        {/* Authentication Section */}
        <div className="p-6 bg-gradient-to-br from-indigo-900/80 to-purple-900/90 backdrop-blur-sm rounded-lg border border-indigo-600/30 shadow-lg">
          <h3 className="text-lg font-semibold mb-4 text-gray-100 flex items-center">
            🔐 Control Room Access
          </h3>
          <p className="text-sm text-gray-300 mb-4">
            Enter your master password to access the Control Room for managing STR connections.
          </p>
          
          <div className="space-y-4">
            <Input
              type="password"
              placeholder="Enter Control Room password"
              value={state.password}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setState(prev => ({ ...prev, password: e.target.value }))}
              onKeyPress={(e: React.KeyboardEvent<HTMLInputElement>) => e.key === 'Enter' && handleAuthenticate()}
              disabled={state.isLoading}
            />
            
            <Button
              onClick={handleAuthenticate}
              disabled={state.isLoading || !state.password.trim()}
              className="w-full"
            >
              {state.isLoading ? 'Authenticating...' : 'Enter Control Room'}
            </Button>
          </div>

          {error && (
            <div className="mt-4 p-3 bg-red-900/50 border border-red-600/30 rounded text-red-300 text-sm">
              {error}
            </div>
          )}
        </div>

        {/* Information Section */}
        <div className="p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
          <h3 className="text-lg font-semibold mb-3 text-gray-100">About Control Room</h3>
          <div className="space-y-2 text-sm text-gray-300">
            <p>• <strong>Centralized STR Management:</strong> Control multiple STR OBS instances from one interface</p>
            <p>• <strong>Bulk Operations:</strong> Mute all audio, change scenes, start/stop streaming across all STR connections</p>
            <p>• <strong>Separate Connection Management:</strong> Dedicated STR connections independent of regular OBS WebSocket connections</p>
            <p>• <strong>Secure Access:</strong> Password-protected access with session management</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header Section */}
      <div className="p-6 bg-gradient-to-br from-indigo-900/80 to-purple-900/90 backdrop-blur-sm rounded-lg border border-indigo-600/30 shadow-lg">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold text-gray-100 flex items-center">
              🎛️ Control Room - Authenticated
            </h3>
            <p className="text-sm text-gray-300 mt-1">
              Session ID: {state.sessionId} • {state.connections.length} STR connections configured
            </p>
          </div>
          <Button
            onClick={handleLogout}
            variant="outline"
            size="sm"
          >
            Logout
          </Button>
        </div>
      </div>

      {/* Connection Management */}
      <div className="p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-100">STR Connections</h3>
          <Button
            onClick={() => setState(prev => ({ ...prev, showAddConnection: !prev.showAddConnection }))}
            size="sm"
          >
            {state.showAddConnection ? 'Cancel' : 'Add STR Connection'}
          </Button>
        </div>

        {/* Add Connection Form */}
        {state.showAddConnection && (
          <div className="mb-6 p-4 bg-gray-700/50 rounded-lg border border-gray-600/30">
            <h4 className="font-medium text-gray-200 mb-3">Add New STR Connection</h4>
            <div className="grid grid-cols-2 gap-4">
              <Input
                placeholder="Connection Name"
                value={state.newConnection.name}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => setState(prev => ({
                  ...prev,
                  newConnection: { ...prev.newConnection, name: e.target.value }
                }))}
              />
              <Input
                placeholder="Host (e.g., 192.168.1.100)"
                value={state.newConnection.host}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => setState(prev => ({
                  ...prev,
                  newConnection: { ...prev.newConnection, host: e.target.value }
                }))}
              />
              <Input
                placeholder="Port"
                type="number"
                value={state.newConnection.port}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => setState(prev => ({
                  ...prev,
                  newConnection: { ...prev.newConnection, port: e.target.value }
                }))}
              />
              <Input
                placeholder="OBS Password (optional)"
                type="password"
                value={state.newConnection.password}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => setState(prev => ({
                  ...prev,
                  newConnection: { ...prev.newConnection, password: e.target.value }
                }))}
              />
            </div>
            <div className="mt-3">
              <Input
                placeholder="Notes (optional)"
                value={state.newConnection.notes}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => setState(prev => ({
                  ...prev,
                  newConnection: { ...prev.newConnection, notes: e.target.value }
                }))}
              />
            </div>
            <div className="flex justify-end space-x-2 mt-4">
              <Button
                onClick={() => setState(prev => ({ ...prev, showAddConnection: false }))}
                variant="outline"
                size="sm"
              >
                Cancel
              </Button>
              <Button
                onClick={handleAddConnection}
                disabled={state.isLoading}
                size="sm"
              >
                {state.isLoading ? 'Adding...' : 'Add Connection'}
              </Button>
            </div>
          </div>
        )}

        {/* Connection List */}
        {state.connections.length === 0 ? (
          <div className="text-center py-8">
            <p className="text-gray-400">No STR connections configured</p>
            <p className="text-sm text-gray-500 mt-1">Click "Add STR Connection" to get started</p>
          </div>
        ) : (
          <div className="space-y-3">
            {state.connections.map((connection, index) => (
              <div key={index} className="p-4 bg-gray-700/30 rounded-lg border border-gray-600/20">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3">
                    <StatusDot color={getStatusColor(connection.status)} />
                    <div>
                      <h4 className="font-medium text-gray-200">{connection.name}</h4>
                      <p className="text-sm text-gray-400">
                        {connection.host}:{connection.port} • {getStatusText(connection.status)}
                      </p>
                      {connection.notes && (
                        <p className="text-xs text-gray-500 mt-1">{connection.notes}</p>
                      )}
                    </div>
                  </div>
                  <div className="flex space-x-2">
                    <Button 
                      size="sm" 
                      variant="outline"
                      onClick={() => connection.status === 'Connected' ? handleDisconnect(connection.name) : handleConnect(connection.name)}
                      disabled={connection.status === 'Connecting' || state.isLoading}
                    >
                      {connection.status === 'Connected' ? 'Disconnect' : 'Connect'}
                    </Button>
                    <Button 
                      size="sm" 
                      variant="outline" 
                      className="text-red-400 hover:text-red-300"
                      onClick={() => handleRemove(connection.name)}
                      disabled={state.isLoading}
                    >
                      Remove
                    </Button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Bulk Operations */}
      <div className="p-6 bg-gradient-to-br from-orange-900/80 to-red-900/90 backdrop-blur-sm rounded-lg border border-orange-600/30 shadow-lg">
        <h3 className="text-lg font-semibold mb-4 text-gray-100">Bulk Operations</h3>
        <div className="grid grid-cols-2 gap-4">
          <Button disabled={state.connections.length === 0} variant="outline">
            🔇 Mute All STR Audio
          </Button>
          <Button disabled={state.connections.length === 0} variant="outline">
            🔊 Unmute All STR Audio
          </Button>
          <Button disabled={state.connections.length === 0} variant="outline">
            🎬 Change All Scenes
          </Button>
          <Button disabled={state.connections.length === 0} variant="outline">
            📺 Start All Streaming
          </Button>
          <Button disabled={state.connections.length === 0} variant="outline">
            ⏹️ Stop All Streaming
          </Button>
        </div>
        {state.connections.length === 0 && (
          <p className="text-sm text-gray-400 mt-3 text-center">
            Add and connect STR connections to enable bulk operations
          </p>
        )}
      </div>

      {/* Messages */}
      {error && (
        <div className="p-3 bg-red-900/50 border border-red-600/30 rounded text-red-300 text-sm">
          {error}
        </div>
      )}
      {success && (
        <div className="p-3 bg-green-900/50 border border-green-600/30 rounded text-green-300 text-sm">
          {success}
        </div>
      )}
    </div>
  );
};

export default ControlRoom;