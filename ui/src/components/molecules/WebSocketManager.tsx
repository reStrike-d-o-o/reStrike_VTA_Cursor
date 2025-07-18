import React, { useState, useEffect } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import { StatusDot } from '../atoms/StatusDot';
import { Icon } from '../atoms/Icon';
import { useAppStore } from '../../stores';

interface WebSocketConnection {
  name: string;
  host: string;
  port: number;
  password?: string;
  protocol_version: 'v4' | 'v5';
  enabled: boolean;
  status: 'Disconnected' | 'Connecting' | 'Connected' | 'Authenticating' | 'Authenticated' | 'Error';
  error?: string;
}

const WebSocketManager: React.FC = () => {
  const { obsConnections, addObsConnection, removeObsConnection, updateObsConnectionStatus, setActiveObsConnection, activeObsConnection } = useAppStore();
  
  const [isAdding, setIsAdding] = useState(false);
  const [editingConnection, setEditingConnection] = useState<string | null>(null);
  const [formData, setFormData] = useState({
    name: '',
    host: 'localhost',
    port: 4455,
    password: '',
    protocol_version: 'v5' as 'v4' | 'v5',
    enabled: true,
  });
  const [error, setError] = useState<string | null>(null);

  const resetForm = () => {
    setFormData({
      name: '',
      host: 'localhost',
      port: 4455,
      password: '',
      protocol_version: 'v5',
      enabled: true,
    });
    setError(null);
  };

  const handleAddConnection = () => {
    if (!formData.name.trim()) {
      setError('Connection name is required');
      return;
    }

    if (obsConnections.some(c => c.name === formData.name)) {
      setError('Connection name already exists');
      return;
    }

    if (formData.port < 1 || formData.port > 65535) {
      setError('Port must be between 1 and 65535');
      return;
    }

    addObsConnection(formData);
    resetForm();
    setIsAdding(false);
  };

  const handleEditConnection = (connection: WebSocketConnection) => {
    setEditingConnection(connection.name);
    setFormData({
      name: connection.name,
      host: connection.host,
      port: connection.port,
      password: connection.password || '',
      protocol_version: connection.protocol_version,
      enabled: connection.enabled,
    });
  };

  const handleUpdateConnection = () => {
    if (!editingConnection) return;

    if (!formData.name.trim()) {
      setError('Connection name is required');
      return;
    }

    if (formData.name !== editingConnection && obsConnections.some(c => c.name === formData.name)) {
      setError('Connection name already exists');
      return;
    }

    if (formData.port < 1 || formData.port > 65535) {
      setError('Port must be between 1 and 65535');
      return;
    }

    // Remove old connection and add updated one
    removeObsConnection(editingConnection);
    addObsConnection(formData);
    
    resetForm();
    setEditingConnection(null);
  };

  const handleDeleteConnection = (name: string) => {
    if (window.confirm(`Are you sure you want to delete the connection "${name}"?`)) {
      removeObsConnection(name);
    }
  };

  const handleConnect = (connection: WebSocketConnection) => {
    // Simulate connection process
    updateObsConnectionStatus(connection.name, 'Connecting');
    
    setTimeout(() => {
      // Simulate connection result (in real app, this would be actual WebSocket connection)
      const success = Math.random() > 0.3; // 70% success rate for demo
      if (success) {
        updateObsConnectionStatus(connection.name, 'Connected');
        setActiveObsConnection(connection.name);
      } else {
        updateObsConnectionStatus(connection.name, 'Error', 'Connection failed');
      }
    }, 1000);
  };

  const handleDisconnect = (connection: WebSocketConnection) => {
    updateObsConnectionStatus(connection.name, 'Disconnected');
    if (activeObsConnection === connection.name) {
      setActiveObsConnection(null);
    }
  };

  const getStatusColor = (status: WebSocketConnection['status']) => {
    switch (status) {
      case 'Connected':
      case 'Authenticated':
        return 'bg-green-500';
      case 'Connecting':
      case 'Authenticating':
        return 'bg-yellow-500';
      case 'Error':
        return 'bg-red-500';
      default:
        return 'bg-gray-500';
    }
  };

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold">WebSocket Connections</h3>
        <Button
          onClick={() => {
            setIsAdding(true);
            setEditingConnection(null);
            resetForm();
          }}
          variant="primary"
          size="sm"
          disabled={isAdding || editingConnection !== null}
        >
          <Icon name="➕" />
          Add Connection
        </Button>
      </div>

      {/* Add/Edit Form */}
      {(isAdding || editingConnection) && (
        <div className="p-4 bg-gray-800 rounded-lg border border-gray-700">
          <h4 className="text-md font-medium mb-3">
            {editingConnection ? 'Edit Connection' : 'Add New Connection'}
          </h4>
          
          {error && (
            <div className="mb-3 p-2 bg-red-900/20 border border-red-700 rounded text-red-400 text-sm">
              {error}
            </div>
          )}

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <Label htmlFor="connection-name">Connection Name *</Label>
              <Input
                id="connection-name"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                placeholder="e.g., OBS_REC, OBS_STR"
                disabled={editingConnection !== null}
              />
            </div>

            <div>
              <Label htmlFor="connection-host">Host</Label>
              <Input
                id="connection-host"
                value={formData.host}
                onChange={(e) => setFormData({ ...formData, host: e.target.value })}
                placeholder="localhost"
              />
            </div>

            <div>
              <Label htmlFor="connection-port">Port</Label>
              <Input
                id="connection-port"
                type="number"
                value={formData.port}
                onChange={(e) => setFormData({ ...formData, port: parseInt(e.target.value) || 4455 })}
                placeholder="4455"
                min="1"
                max="65535"
              />
            </div>

            <div>
              <Label htmlFor="connection-password">Password (optional)</Label>
              <Input
                id="connection-password"
                type="password"
                value={formData.password}
                onChange={(e) => setFormData({ ...formData, password: e.target.value })}
                placeholder="Leave empty if no password"
              />
            </div>

            <div>
              <Label htmlFor="connection-protocol">Protocol Version</Label>
              <select
                id="connection-protocol"
                value={formData.protocol_version}
                onChange={(e) => setFormData({ ...formData, protocol_version: e.target.value as 'v4' | 'v5' })}
                className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500"
              >
                <option value="v5">OBS WebSocket v5 (Recommended)</option>
                <option value="v4">OBS WebSocket v4 (Legacy)</option>
              </select>
            </div>

            <div className="flex items-center">
              <label className="flex items-center space-x-2">
                <input
                  type="checkbox"
                  checked={formData.enabled}
                  onChange={(e) => setFormData({ ...formData, enabled: e.target.checked })}
                  className="rounded border-gray-600 bg-gray-700 text-blue-500 focus:ring-blue-500"
                />
                <span className="text-sm">Enabled</span>
              </label>
            </div>
          </div>

          <div className="flex space-x-2 mt-4">
            <Button
              onClick={editingConnection ? handleUpdateConnection : handleAddConnection}
              variant="primary"
              size="sm"
            >
              {editingConnection ? 'Update' : 'Add'} Connection
            </Button>
            <Button
              onClick={() => {
                setIsAdding(false);
                setEditingConnection(null);
                resetForm();
              }}
              variant="secondary"
              size="sm"
            >
              Cancel
            </Button>
          </div>
        </div>
      )}

      {/* Connections List */}
      <div className="space-y-2">
        {obsConnections.length === 0 ? (
          <div className="p-4 bg-gray-800 rounded-lg text-center text-gray-400">
            No WebSocket connections configured
          </div>
        ) : (
          obsConnections.map((connection) => (
            <div
              key={connection.name}
              className={`p-4 bg-gray-800 rounded-lg border ${
                activeObsConnection === connection.name ? 'border-blue-500' : 'border-gray-700'
              }`}
            >
              <div className="flex items-center justify-between">
                <div className="flex-1">
                  <div className="flex items-center space-x-2 mb-2">
                    <h4 className="font-medium">{connection.name}</h4>
                    <StatusDot color={getStatusColor(connection.status)} />
                    <span className="text-sm text-gray-400">{connection.status}</span>
                    {activeObsConnection === connection.name && (
                      <span className="text-xs bg-blue-500 text-white px-2 py-1 rounded">Active</span>
                    )}
                  </div>
                  
                  <div className="text-sm text-gray-400 space-y-1">
                    <div>{connection.host}:{connection.port} ({connection.protocol_version})</div>
                    {connection.password && <div>Password: {'•'.repeat(8)}</div>}
                    {connection.error && (
                      <div className="text-red-400">Error: {connection.error}</div>
                    )}
                  </div>
                </div>

                <div className="flex items-center space-x-2">
                  {connection.status === 'Connected' || connection.status === 'Authenticated' ? (
                    <Button
                      onClick={() => handleDisconnect(connection)}
                      variant="danger"
                      size="sm"
                    >
                      <Icon name="🔌" />
                      Disconnect
                    </Button>
                  ) : (
                    <Button
                      onClick={() => handleConnect(connection)}
                      variant="success"
                      size="sm"
                      disabled={!connection.enabled || connection.status === 'Connecting'}
                    >
                      <Icon name="🔗" />
                      Connect
                    </Button>
                  )}
                  
                  <Button
                    onClick={() => handleEditConnection(connection)}
                    variant="secondary"
                    size="sm"
                    disabled={editingConnection !== null}
                  >
                    <Icon name="✏️" />
                    Edit
                  </Button>
                  
                  <Button
                    onClick={() => handleDeleteConnection(connection.name)}
                    variant="danger"
                    size="sm"
                    disabled={editingConnection !== null}
                  >
                    <Icon name="🗑️" />
                    Delete
                  </Button>
                </div>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Protocol Information */}
      <div className="p-4 bg-gray-800 rounded-lg">
        <h4 className="text-md font-medium mb-3">Protocol Information</h4>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
          <div>
            <h5 className="font-medium text-blue-400 mb-2">OBS WebSocket v5</h5>
            <ul className="text-gray-300 space-y-1">
              <li>• Default port: 4455</li>
              <li>• SHA256 authentication</li>
              <li>• Enhanced features</li>
              <li>• Better error handling</li>
            </ul>
          </div>
          <div>
            <h5 className="font-medium text-yellow-400 mb-2">OBS WebSocket v4</h5>
            <ul className="text-gray-300 space-y-1">
              <li>• Default port: 4444</li>
              <li>• Password authentication</li>
              <li>• Legacy support</li>
              <li>• Basic functionality</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
};

export default WebSocketManager; 