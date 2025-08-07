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

interface ConnectionSelections {
  audioSource: string;
  mainScene: string;
  breakScene: string;
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
  // New state for dropdowns
  connectionSelections: Record<string, ConnectionSelections>;
  audioSources: Record<string, string[]>; // connection name -> audio sources
  scenes: Record<string, string[]>; // connection name -> scenes
  loadingData: Record<string, boolean>; // connection name -> loading state
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
    },
    // Initialize new state
    connectionSelections: {},
    audioSources: {},
    scenes: {},
    loadingData: {}
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

  // Initialize selections when connections change
  useEffect(() => {
    const newSelections: Record<string, ConnectionSelections> = {};
    state.connections.forEach(connection => {
      if (!state.connectionSelections[connection.name]) {
        newSelections[connection.name] = {
          audioSource: '',
          mainScene: '',
          breakScene: ''
        };
      } else {
        newSelections[connection.name] = state.connectionSelections[connection.name];
      }
    });
    
    if (Object.keys(newSelections).length > 0) {
      setState(prev => ({
        ...prev,
        connectionSelections: newSelections
      }));
    }
  }, [state.connections]);

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
            sessionId: response.session_id || null,
            isLoading: false
          }));
          
          if (response.session_id) {
            await loadConnections(response.session_id);
          }
        } else {
          setError(response.error || 'Authentication failed');
          setState(prev => ({ ...prev, isLoading: false }));
        }
      }
    } catch (err) {
      console.error('Authentication error:', err);
      setError('Authentication failed');
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
      setState(prev => ({ ...prev, connections: [] }));
    }
  };

  const handleLogout = () => {
    setState(prev => ({
      ...prev,
      isAuthenticated: false,
      sessionId: null,
      password: '',
      connections: [],
      connectionSelections: {},
      audioSources: {},
      scenes: {}
    }));
  };

  const handleAddConnection = async () => {
    if (!state.newConnection.name.trim() || !state.newConnection.host.trim()) {
      setError('Please fill in all required fields');
      return;
    }

    setState(prev => ({ ...prev, isLoading: true }));
    setError(null);

    try {
      const result = await invoke('control_room_add_obs_connection', {
        sessionId: state.sessionId,
        name: state.newConnection.name,
        host: state.newConnection.host,
        port: parseInt(state.newConnection.port),
        password: state.newConnection.password,
        notes: state.newConnection.notes
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; message?: string; error?: string };
        
        if (response.success) {
          setSuccess(response.message || 'OBS connection added successfully');
          setState(prev => ({
            ...prev,
            showAddConnection: false,
            newConnection: {
              name: '',
              host: '',
              port: '4455',
              password: '',
              notes: ''
            }
          }));
          
          if (state.sessionId) {
            await loadConnections(state.sessionId);
          }
        } else {
          setError(response.error || 'Failed to add OBS connection');
        }
      }
    } catch (err) {
      console.error('Failed to add connection:', err);
      setError('Failed to add OBS connection');
    } finally {
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
    if (!state.sessionId) return;

    setState(prev => ({ ...prev, isLoading: true }));
    setError(null);

    try {
      const result = await invoke('control_room_connect_obs', {
        sessionId: state.sessionId,
        obsName: connectionName
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; message?: string; error?: string };
        
        if (response.success) {
          setSuccess(response.message || 'Successfully connected to OBS');
          await loadConnections(state.sessionId);
        } else {
          setError(response.error || 'Failed to connect to OBS');
        }
      }
    } catch (err) {
      console.error('Failed to connect:', err);
      setError('Failed to connect to OBS');
    } finally {
      setState(prev => ({ ...prev, isLoading: false }));
    }
  };

  const handleDisconnect = async (connectionName: string) => {
    if (!state.sessionId) return;

    setState(prev => ({ ...prev, isLoading: true }));
    setError(null);

    try {
      const result = await invoke('control_room_disconnect_obs', {
        sessionId: state.sessionId,
        obsName: connectionName
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; message?: string; error?: string };
        
        if (response.success) {
          setSuccess(response.message || 'Successfully disconnected from OBS');
          await loadConnections(state.sessionId);
        } else {
          setError(response.error || 'Failed to disconnect from OBS');
        }
      }
    } catch (err) {
      console.error('Failed to disconnect:', err);
      setError('Failed to disconnect from OBS');
    } finally {
      setState(prev => ({ ...prev, isLoading: false }));
    }
  };

  const handleRemove = async (connectionName: string) => {
    if (!state.sessionId) return;

    setState(prev => ({ ...prev, isLoading: true }));
    setError(null);

    try {
      const result = await invoke('control_room_remove_obs_connection', {
        sessionId: state.sessionId,
        obsName: connectionName
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; message?: string; error?: string };
        
        if (response.success) {
          setSuccess(response.message || 'OBS connection removed successfully');
          await loadConnections(state.sessionId);
        } else {
          setError(response.error || 'Failed to remove OBS connection');
        }
      }
    } catch (err) {
      console.error('Failed to remove connection:', err);
      setError('Failed to remove OBS connection');
    } finally {
      setState(prev => ({ ...prev, isLoading: false }));
    }
  };

  const handleEdit = async (connectionName: string) => {
    if (!state.sessionId) return;

    setState(prev => ({ ...prev, isLoading: true }));
    setError(null);

    try {
      const result = await invoke('control_room_get_obs_connection', {
        sessionId: state.sessionId,
        obsName: connectionName
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; connection?: any; error?: string };
        
        if (response.success && response.connection) {
          setState(prev => ({
            ...prev,
            showEditConnection: true,
            editingConnection: connectionName,
            editConnection: {
              name: response.connection.name,
              host: response.connection.host,
              port: response.connection.port.toString(),
              password: response.connection.password || '',
              notes: response.connection.notes || ''
            }
          }));
        } else {
          setError(response.error || 'Failed to get connection details');
        }
      }
    } catch (err) {
      console.error('Failed to get connection details:', err);
      setError('Failed to get connection details');
    } finally {
      setState(prev => ({ ...prev, isLoading: false }));
    }
  };

  const handleUpdateConnection = async () => {
    if (!state.sessionId || !state.editingConnection) return;

    setState(prev => ({ ...prev, isLoading: true }));
    setError(null);

    try {
      const result = await invoke('control_room_update_obs_connection', {
        sessionId: state.sessionId,
        obsName: state.editingConnection,
        host: state.editConnection.host,
        port: parseInt(state.editConnection.port),
        password: state.editConnection.password,
        notes: state.editConnection.notes
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; message?: string; error?: string };
        
        if (response.success) {
          setSuccess(response.message || 'OBS connection updated successfully');
          setState(prev => ({
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
          }));
          
          await loadConnections(state.sessionId);
        } else {
          setError(response.error || 'Failed to update OBS connection');
        }
      }
    } catch (err) {
      console.error('Failed to update connection:', err);
      setError('Failed to update OBS connection');
    } finally {
      setState(prev => ({ ...prev, isLoading: false }));
    }
  };

  const handleConnectAll = async () => {
    if (!state.sessionId) return;

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
    if (!state.sessionId) return;

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

  // Handle dropdown selections
  const handleSelectionChange = (connectionName: string, field: keyof ConnectionSelections, value: string) => {
    setState(prev => ({
      ...prev,
      connectionSelections: {
        ...prev.connectionSelections,
        [connectionName]: {
          ...prev.connectionSelections[connectionName],
          [field]: value
        }
      }
    }));
  };

  // Fetch real audio sources from OBS connection
  const getAudioSources = async (connectionName: string): Promise<string[]> => {
    if (!state.sessionId) return [];
    
    try {
      const result = await invoke('control_room_get_audio_sources', {
        sessionId: state.sessionId,
        obsName: connectionName
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; sources?: string[]; error?: string };
        if (response.success && response.sources) {
          return response.sources;
        }
      }
    } catch (err) {
      console.error('Failed to get audio sources for', connectionName, ':', err);
    }
    
    return [];
  };

  // Fetch real scenes from OBS connection
  const getScenes = async (connectionName: string): Promise<string[]> => {
    if (!state.sessionId) return [];
    
    try {
      const result = await invoke('control_room_get_scenes', {
        sessionId: state.sessionId,
        obsName: connectionName
      });

      if (result && typeof result === 'object' && 'success' in result) {
        const response = result as { success: boolean; scenes?: string[]; error?: string };
        if (response.success && response.scenes) {
          return response.scenes;
        }
      }
    } catch (err) {
      console.error('Failed to get scenes for', connectionName, ':', err);
    }
    
    return [];
  };

  // Load audio sources and scenes for a connection
  const loadConnectionData = async (connectionName: string) => {
    if (!state.sessionId) return;
    
    // Set loading state
    setState(prev => ({
      ...prev,
      loadingData: {
        ...prev.loadingData,
        [connectionName]: true
      }
    }));
    
    try {
      const [audioSources, scenes] = await Promise.all([
        getAudioSources(connectionName),
        getScenes(connectionName)
      ]);
      
      setState(prev => ({
        ...prev,
        audioSources: {
          ...prev.audioSources,
          [connectionName]: audioSources
        },
        scenes: {
          ...prev.scenes,
          [connectionName]: scenes
        },
        loadingData: {
          ...prev.loadingData,
          [connectionName]: false
        }
      }));
    } catch (err) {
      console.error('Failed to load connection data for', connectionName, ':', err);
      setState(prev => ({
        ...prev,
        loadingData: {
          ...prev.loadingData,
          [connectionName]: false
        }
      }));
    }
  };

  // Load data for all connected OBS instances
  const loadAllConnectionData = async () => {
    if (!state.sessionId) return;
    
    const connectedConnections = state.connections.filter(conn => conn.status === 'Connected');
    
    for (const connection of connectedConnections) {
      await loadConnectionData(connection.name);
    }
  };

  // Load connection data when connections change
  useEffect(() => {
    if (state.sessionId && state.connections.length > 0) {
      loadAllConnectionData();
    }
  }, [state.sessionId, state.connections]);

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

        {/* Connection List - Two Column Layout */}
        {state.connections.length === 0 ? (
          <div className="text-center py-8">
            <p className="text-gray-400">No OBS connections configured</p>
            <p className="text-sm text-gray-500 mt-1">Click "Add OBS Connection" to get started</p>
          </div>
        ) : (
          <div className="grid grid-cols-2 gap-4">
            {state.connections.map((connection, index) => (
              <div key={index} className="p-4 bg-gray-700/30 rounded-lg border border-gray-600/20">
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center space-x-2">
                    <StatusDot color={getStatusColor(connection.status)} />
                    <div>
                      <h4 className="font-medium text-gray-200 text-sm">{connection.name}</h4>
                      <p className="text-xs text-gray-400">
                        {connection.host}:{connection.port} • {getStatusText(connection.status)}
                      </p>
                    </div>
                  </div>
                </div>
                
                {connection.notes && (
                  <p className="text-xs text-gray-500 mb-3">{connection.notes}</p>
                )}
                
                <div className="flex space-x-1">
                  <Button 
                    size="sm" 
                    variant="outline"
                    onClick={() => connection.status === 'Connected' ? handleDisconnect(connection.name) : handleConnect(connection.name)}
                    disabled={connection.status === 'Connecting' || state.isLoading}
                    className="text-xs px-2 py-1"
                  >
                    {connection.status === 'Connected' ? 'Disconnect' : 'Connect'}
                  </Button>
                  <Button 
                    size="sm" 
                    variant="outline"
                    onClick={() => handleEdit(connection.name)}
                    disabled={state.isLoading}
                    className="text-xs px-2 py-1"
                  >
                    Edit
                  </Button>
                  <Button 
                    size="sm" 
                    variant="outline" 
                    className="text-red-400 hover:text-red-300 text-xs px-2 py-1"
                    onClick={() => handleRemove(connection.name)}
                    disabled={state.isLoading}
                  >
                    Remove
                  </Button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Bulk Operations - Two Column Layout */}
      <div className="p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
        <h3 className="text-lg font-semibold mb-4 text-gray-100">Bulk Operations</h3>
        
        <div className="grid grid-cols-2 gap-6">
          {/* Column 1: Dropdowns */}
                     <div className="space-y-4">
             <div className="flex items-center justify-between">
               <h4 className="font-medium text-gray-200 text-sm">Connection Settings</h4>
               <Button
                 size="sm"
                 variant="outline"
                 onClick={loadAllConnectionData}
                 disabled={state.isLoading}
                 className="text-xs px-2 py-1"
               >
                 🔄 Refresh All
               </Button>
             </div>
            
            {state.connections.length > 0 ? (
              <div className="space-y-3">
                {state.connections.map((connection) => (
                  <div key={connection.name} className="p-3 bg-gray-700/30 rounded border border-gray-600/20">
                                         <div className="flex items-center justify-between mb-2">
                       <h5 className="font-medium text-gray-200 text-sm">{connection.name}</h5>
                       <Button
                         size="sm"
                         variant="outline"
                         onClick={() => loadConnectionData(connection.name)}
                         disabled={state.loadingData[connection.name] || state.isLoading}
                         className="text-xs px-2 py-1"
                       >
                         {state.loadingData[connection.name] ? 'Loading...' : '🔄'}
                       </Button>
                     </div>
                    
                                         <div className="grid grid-cols-3 gap-2">
                       {/* Audio Source Dropdown */}
                       <div>
                         <label htmlFor={`audio-source-${connection.name}`} className="block text-xs text-gray-400 mb-1">Audio</label>
                                                   <select 
                            id={`audio-source-${connection.name}`}
                            value={state.connectionSelections[connection.name]?.audioSource || ''}
                            onChange={(e) => handleSelectionChange(connection.name, 'audioSource', e.target.value)}
                            disabled={state.loadingData[connection.name]}
                            className="w-full px-2 py-1 text-xs bg-gray-700 border border-gray-600 rounded-md text-gray-300 focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:opacity-50"
                          >
                            <option value="">
                              {state.loadingData[connection.name] ? 'Loading...' : 'None'}
                            </option>
                            {(state.audioSources[connection.name] || []).map((source) => (
                              <option key={source} value={source}>
                                {source}
                              </option>
                            ))}
                          </select>
                       </div>
                       
                       {/* Main Scene Dropdown */}
                       <div>
                         <label htmlFor={`main-scene-${connection.name}`} className="block text-xs text-gray-400 mb-1">Main</label>
                                                   <select 
                            id={`main-scene-${connection.name}`}
                            value={state.connectionSelections[connection.name]?.mainScene || ''}
                            onChange={(e) => handleSelectionChange(connection.name, 'mainScene', e.target.value)}
                            disabled={state.loadingData[connection.name]}
                            className="w-full px-2 py-1 text-xs bg-gray-700 border border-gray-600 rounded-md text-gray-300 focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:opacity-50"
                          >
                            <option value="">
                              {state.loadingData[connection.name] ? 'Loading...' : 'None'}
                            </option>
                            {(state.scenes[connection.name] || []).map((scene) => (
                              <option key={scene} value={scene}>
                                {scene}
                              </option>
                            ))}
                          </select>
                       </div>
                       
                       {/* Break Scene Dropdown */}
                       <div>
                         <label htmlFor={`break-scene-${connection.name}`} className="block text-xs text-gray-400 mb-1">Break</label>
                                                   <select 
                            id={`break-scene-${connection.name}`}
                            value={state.connectionSelections[connection.name]?.breakScene || ''}
                            onChange={(e) => handleSelectionChange(connection.name, 'breakScene', e.target.value)}
                            disabled={state.loadingData[connection.name]}
                            className="w-full px-2 py-1 text-xs bg-gray-700 border border-gray-600 rounded-md text-gray-300 focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:opacity-50"
                          >
                            <option value="">
                              {state.loadingData[connection.name] ? 'Loading...' : 'None'}
                            </option>
                            {(state.scenes[connection.name] || []).map((scene) => (
                              <option key={scene} value={scene}>
                                {scene}
                              </option>
                            ))}
                          </select>
                       </div>
                     </div>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-gray-400 text-center py-4">
                Add and connect OBS connections to configure bulk operations
              </p>
            )}
          </div>
          
          {/* Column 2: Bulk Operation Buttons */}
          <div className="space-y-4">
            <h4 className="font-medium text-gray-200 text-sm">Bulk Actions</h4>
            
            <div className="space-y-3">
              <Button 
                disabled={state.connections.length === 0} 
                variant="outline"
                className="w-full justify-start"
              >
                🔇 Mute All OBS Audio
              </Button>
              <Button 
                disabled={state.connections.length === 0} 
                variant="outline"
                className="w-full justify-start"
              >
                🔊 Unmute All OBS Audio
              </Button>
              <Button 
                disabled={state.connections.length === 0} 
                variant="outline"
                className="w-full justify-start"
              >
                🎬 Change All Scenes
              </Button>
              <Button 
                disabled={state.connections.length === 0} 
                variant="outline"
                className="w-full justify-start"
              >
                📺 Start All OBS Streaming
              </Button>
              <Button 
                disabled={state.connections.length === 0} 
                variant="outline"
                className="w-full justify-start"
              >
                ⏹️ Stop All OBS Streaming
              </Button>
            </div>
            
            {state.connections.length === 0 && (
              <p className="text-sm text-gray-400 text-center py-4">
                Add and connect OBS connections to enable bulk operations
              </p>
            )}
          </div>
        </div>
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