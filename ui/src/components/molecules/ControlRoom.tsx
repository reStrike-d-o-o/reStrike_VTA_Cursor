import React, { useState, useEffect } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import { StatusDot } from '../atoms/StatusDot';
import { invoke } from '@tauri-apps/api/core';

interface ObsConnection {
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
  connections: ObsConnection[];
  showAddConnection: boolean;
  showEditConnection: boolean;
  editingConnection: string | null;
  newConnection: {
    name: string;
    host: string;
    port: string;
    password: string;
    notes: string;
  };
  editConnection: {
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
    showEditConnection: false,
    editingConnection: null,
    newConnection: {
      name: '',
      host: '',
      port: '4455',
      password: '',
      notes: ''
    },
    editConnection: {
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
      const result = await invoke('control_room_get_obs_connections_with_details', {
        sessionId: sessionId
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; connections?: Array<{name: string; host: string; port: number; status: string; notes?: string}>; error?: string };
        if (response.success && response.connections) {
          // Convert connection data to connection objects with real status and details
          const connections: ObsConnection[] = response.connections.map(conn => ({
            name: conn.name,
            host: conn.host,
            port: conn.port,
            status: conn.status as ObsConnection['status'],
            notes: conn.notes
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
              const result = await invoke('control_room_add_obs_connection', {
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
                     const newConnection: ObsConnection = {
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

          setSuccess(response.message || `OBS connection "${newConnection.name}" added successfully`);
        } else {
          setError(response.error || 'Failed to add OBS connection');
          setState(prev => ({ ...prev, isLoading: false }));
        }
      }
    } catch (err) {
      console.error('Failed to add connection:', err);
      setError('Failed to add OBS connection');
      setState(prev => ({ ...prev, isLoading: false }));
    }
  };

  const getStatusColor = (status: ObsConnection['status']) => {
    switch (status) {
      case 'Connected': return 'green';
      case 'Connecting': return 'yellow';
      case 'Error': return 'red';
      default: return 'gray';
    }
  };

  const getStatusText = (status: ObsConnection['status']) => {
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
                      const result = await invoke('control_room_connect_obs', {
          sessionId: state.sessionId,
          obsName: connectionName
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
                      const result = await invoke('control_room_disconnect_obs', {
          sessionId: state.sessionId,
          obsName: connectionName
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
    if (!confirm(`Are you sure you want to remove the OBS connection "${connectionName}"?`)) {
      return;
    }

    try {
      const result = await invoke('control_room_remove_obs_connection', {
        sessionId: state.sessionId,
        obsName: connectionName
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; message?: string; error?: string };
        
        if (response.success) {
          setState(prev => ({
            ...prev,
            connections: prev.connections.filter(conn => conn.name !== connectionName)
          }));
          setSuccess(response.message || `OBS connection "${connectionName}" removed`);
        } else {
          setError(response.error || `Failed to remove OBS connection "${connectionName}"`);
        }
      }
    } catch (err) {
      console.error('Remove error:', err);
      setError(`Failed to remove OBS connection "${connectionName}"`);
    }
  };

  const handleEdit = async (connectionName: string) => {
    try {
      const result = await invoke('control_room_get_obs_connection', {
        sessionId: state.sessionId,
        obsName: connectionName
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; connection?: any; error?: string };
        
        if (response.success && response.connection) {
          const connection = response.connection;
          setState(prev => ({
            ...prev,
            showEditConnection: true,
            editingConnection: connectionName,
            editConnection: {
              name: connection.name,
              host: connection.host,
              port: connection.port.toString(),
              password: connection.password || '',
              notes: connection.notes || ''
            }
          }));
        } else {
          setError(response.error || `Failed to get OBS connection "${connectionName}"`);
        }
      }
    } catch (err) {
      console.error('Edit error:', err);
      setError(`Failed to get OBS connection "${connectionName}"`);
    }
  };

  const handleUpdateConnection = async () => {
    if (!state.editConnection.name.trim() || !state.editConnection.host.trim()) {
      setError('Please enter connection name and host');
      return;
    }

    const port = parseInt(state.editConnection.port);
    if (isNaN(port) || port < 1 || port > 65535) {
      setError('Please enter a valid port number (1-65535)');
      return;
    }

    setState(prev => ({ ...prev, isLoading: true }));

    try {
      const result = await invoke('control_room_update_obs_connection', {
        sessionId: state.sessionId,
        obsName: state.editingConnection,
        host: state.editConnection.host.trim(),
        port: port,
        password: state.editConnection.password || null,
        notes: state.editConnection.notes.trim() || null
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; message?: string; error?: string };
        
        if (response.success) {
          // Update the connection in the list locally
          setState(prev => ({
            ...prev,
            connections: prev.connections.map(conn =>
              conn.name === state.editingConnection
                ? {
                    ...conn,
                    host: state.editConnection.host.trim(),
                    port: port,
                    notes: state.editConnection.notes.trim() || undefined
                  }
                : conn
            ),
            showEditConnection: false,
            editingConnection: null,
            editConnection: {
              name: '',
              host: '',
              port: '4455',
              password: '',
              notes: ''
            },
            isLoading: false
          }));

          setSuccess(response.message || `OBS connection "${state.editConnection.name}" updated successfully`);
        } else {
          setError(response.error || 'Failed to update OBS connection');
          setState(prev => ({ ...prev, isLoading: false }));
        }
      }
    } catch (err) {
      console.error('Failed to update connection:', err);
      setError('Failed to update OBS connection');
      setState(prev => ({ ...prev, isLoading: false }));
    }
  };

  const handleConnectAll = async () => {
    if (!state.sessionId) {
      setError('No active session');
      return;
    }

    setState(prev => ({ ...prev, isLoading: true }));
    setError(null);

    try {
      const result = await invoke('control_room_connect_all_obs', {
        sessionId: state.sessionId
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { 
          success: boolean; 
          message?: string; 
          error?: string;
          connected_count?: number;
          total_count?: number;
          failed_connections?: string[];
        };
        
        if (response.success) {
          // Reload connections to get updated status
          await loadConnections(state.sessionId);
          setSuccess(response.message || 'Successfully connected all OBS connections');
        } else {
          setError(response.error || 'Failed to connect all OBS connections');
        }
      }
    } catch (err) {
      console.error('Failed to connect all connections:', err);
      setError('Failed to connect all OBS connections');
    } finally {
      setState(prev => ({ ...prev, isLoading: false }));
    }
  };

  const handleDisconnectAll = async () => {
    if (!state.sessionId) {
      setError('No active session');
      return;
    }

    setState(prev => ({ ...prev, isLoading: true }));
    setError(null);

    try {
      const result = await invoke('control_room_disconnect_all_obs', {
        sessionId: state.sessionId
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { 
          success: boolean; 
          message?: string; 
          error?: string;
          disconnected_count?: number;
          total_count?: number;
          failed_connections?: string[];
        };
        
        if (response.success) {
          // Reload connections to get updated status
          await loadConnections(state.sessionId);
          setSuccess(response.message || 'Successfully disconnected all OBS connections');
        } else {
          setError(response.error || 'Failed to disconnect all OBS connections');
        }
      }
    } catch (err) {
      console.error('Failed to disconnect all connections:', err);
      setError('Failed to disconnect all OBS connections');
    } finally {
      setState(prev => ({ ...prev, isLoading: false }));
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
            Enter your master password to access the Control Room for managing OBS connections.
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
                    <p>• <strong>Centralized OBS Management:</strong> Control multiple OBS instances from one interface</p>
        <p>• <strong>Bulk Operations:</strong> Mute all audio, change scenes, start/stop streaming across all OBS connections</p>
        <p>• <strong>Separate Connection Management:</strong> Dedicated OBS connections independent of regular OBS WebSocket connections</p>
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
              Session ID: {state.sessionId} • {state.connections.length} OBS connections configured
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
          <h3 className="text-lg font-semibold text-gray-100">OBS Connections</h3>
          <div className="flex space-x-2">
            <Button
              onClick={handleConnectAll}
              disabled={state.isLoading || state.connections.length === 0}
              variant="outline"
              size="sm"
            >
              {state.isLoading ? 'Connecting...' : 'Connect All'}
            </Button>
            <Button
              onClick={handleDisconnectAll}
              disabled={state.isLoading || state.connections.length === 0}
              variant="outline"
              size="sm"
            >
              {state.isLoading ? 'Disconnecting...' : 'Disconnect All'}
            </Button>
            <Button
              onClick={() => setState(prev => ({ ...prev, showAddConnection: !prev.showAddConnection }))}
              size="sm"
            >
              {state.showAddConnection ? 'Cancel' : 'Add OBS Connection'}
            </Button>
          </div>
        </div>

        {/* Add Connection Form */}
        {state.showAddConnection && (
          <div className="mb-6 p-4 bg-gray-700/50 rounded-lg border border-gray-600/30">
                         <h4 className="font-medium text-gray-200 mb-3">Add New OBS Connection</h4>
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

         {/* Edit Connection Form */}
         {state.showEditConnection && (
           <div className="mb-6 p-4 bg-gray-700/50 rounded-lg border border-gray-600/30">
             <h4 className="font-medium text-gray-200 mb-3">Edit OBS Connection: {state.editingConnection}</h4>
             <div className="grid grid-cols-2 gap-4">
               <Input
                 placeholder="Connection Name"
                 value={state.editConnection.name}
                 disabled={true} // Name cannot be changed
               />
               <Input
                 placeholder="Host (e.g., 192.168.1.100)"
                 value={state.editConnection.host}
                 onChange={(e: React.ChangeEvent<HTMLInputElement>) => setState(prev => ({
                   ...prev,
                   editConnection: { ...prev.editConnection, host: e.target.value }
                 }))}
               />
               <Input
                 placeholder="Port"
                 type="number"
                 value={state.editConnection.port}
                 onChange={(e: React.ChangeEvent<HTMLInputElement>) => setState(prev => ({
                   ...prev,
                   editConnection: { ...prev.editConnection, port: e.target.value }
                 }))}
               />
               <Input
                 placeholder="OBS Password (optional)"
                 type="password"
                 value={state.editConnection.password}
                 onChange={(e: React.ChangeEvent<HTMLInputElement>) => setState(prev => ({
                   ...prev,
                   editConnection: { ...prev.editConnection, password: e.target.value }
                 }))}
               />
             </div>
             <div className="mt-3">
               <Input
                 placeholder="Notes (optional)"
                 value={state.editConnection.notes}
                 onChange={(e: React.ChangeEvent<HTMLInputElement>) => setState(prev => ({
                   ...prev,
                   editConnection: { ...prev.editConnection, notes: e.target.value }
                 }))}
               />
             </div>
             <div className="flex justify-end space-x-2 mt-4">
               <Button
                 onClick={() => setState(prev => ({ 
                   ...prev, 
                   showEditConnection: false,
                   editingConnection: null,
                   editConnection: {
                     name: '',
                     host: '',
                     port: '4455',
                     password: '',
                     notes: ''
                   }
                 }))}
                 variant="outline"
                 size="sm"
               >
                 Cancel
               </Button>
               <Button
                 onClick={handleUpdateConnection}
                 disabled={state.isLoading}
                 size="sm"
               >
                 {state.isLoading ? 'Updating...' : 'Update Connection'}
               </Button>
             </div>
           </div>
         )}

         {/* Connection List */}
        {state.connections.length === 0 ? (
          <div className="text-center py-8">
                         <p className="text-gray-400">No OBS connections configured</p>
             <p className="text-sm text-gray-500 mt-1">Click "Add OBS Connection" to get started</p>
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
                       onClick={() => handleEdit(connection.name)}
                       disabled={state.isLoading}
                     >
                       Edit
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
             🔇 Mute All OBS Audio
           </Button>
           <Button disabled={state.connections.length === 0} variant="outline">
             🔊 Unmute All OBS Audio
          </Button>
          <Button disabled={state.connections.length === 0} variant="outline">
            🎬 Change All Scenes
          </Button>
                     <Button disabled={state.connections.length === 0} variant="outline">
             📺 Start All OBS Streaming
           </Button>
           <Button disabled={state.connections.length === 0} variant="outline">
             ⏹️ Stop All OBS Streaming
          </Button>
        </div>
        {state.connections.length === 0 && (
          <p className="text-sm text-gray-400 mt-3 text-center">
                         Add and connect OBS connections to enable bulk operations
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