import React, { useState, useEffect, useRef } from 'react';
import Button from '../atoms/Button';
import Toggle from '../atoms/Toggle';
import Input from '../atoms/Input';
import Checkbox from '../atoms/Checkbox';
import Label from '../atoms/Label';
import StatusDot from '../atoms/StatusDot';
import Icon from '../atoms/Icon';
import TabGroup from '../molecules/TabGroup';
import TabIcons from '../atoms/TabIcons';
import FlagManagementPanel from './FlagManagementPanel';
import ScoreboardManager from './ScoreboardManager';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';

// Use the proper Tauri v2 invoke function with fallback
const invoke = async (command: string, args?: any) => {
  try {
    // Try the proper Tauri v2 API first
    return await tauriInvoke(command, args);
  } catch (error) {
    // If that fails, try the global window.__TAURI__.core.invoke
    if (typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.core) {
      return await window.__TAURI__.core.invoke(command, args);
    }
    throw new Error('Tauri v2 core module not available - ensure app is running in desktop mode');
  }
};

interface ProtocolVersion {
  version: string;
  filename: string;
  file_path: string;
  description: string;
  created_date: string;
  last_modified: string;
  is_active: boolean;
  file_size: number;
  checksum?: string;
}

interface ProtocolFile {
  version: string;
  description: string;
  year: number;
  streams: Record<string, any>;
  examples: any[];
  metadata: any;
}

interface NetworkInterface {
  name: string;
  type: 'ethernet' | 'wifi' | 'loopback' | 'bluetooth' | 'virtual' | 'unknown';
  ip_addresses: string[];
  subnet_masks: string[];
  default_gateway?: string;
  dns_suffix?: string;
  media_state: 'connected' | 'disconnected' | 'unknown';
  is_up: boolean;
  is_loopback: boolean;
  description?: string;
}

interface UdpSettings {
  port: number;
  bind_address: string;
  enabled: boolean;
  network_interface: {
    auto_detect: boolean;
    preferred_type: string;
    fallback_to_localhost: boolean;
    selected_interface: string | null;
  };
}

interface PssDrawerProps {
  className?: string;
}

const PssDrawer: React.FC<PssDrawerProps> = ({ className = '' }) => {
  // Tab state
  const [activeTab, setActiveTab] = useState('udp');
  
  const [protocolVersions, setProtocolVersions] = useState<ProtocolVersion[]>([]);
  const [currentProtocol, setCurrentProtocol] = useState<ProtocolFile | null>(null);
  const [activeVersion, setActiveVersion] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0);
  const [error, setError] = useState<string>('');
  const [success, setSuccess] = useState<string>('');
  const fileInputRef = useRef<HTMLInputElement>(null);

  // UDP Server state
  const [udpEnabled, setUdpEnabled] = useState(false);
  const [udpPort, setUdpPort] = useState(8888);
  const [udpStatus, setUdpStatus] = useState<string>('Stopped');
  const [showAdvancedSettings, setShowAdvancedSettings] = useState(false);
  
  // Network settings state
  const [networkInterfaces, setNetworkInterfaces] = useState<NetworkInterface[]>([]);
  const [bestInterface, setBestInterface] = useState<NetworkInterface | null>(null);
  const [udpSettings, setUdpSettings] = useState<UdpSettings>({
    port: 8888,
    bind_address: '127.0.0.1',
    enabled: false,
    network_interface: {
      auto_detect: true,
      preferred_type: 'ethernet',
      fallback_to_localhost: true,
      selected_interface: null,
    },
  });

  // Load protocol versions on component mount
  useEffect(() => {
    loadProtocolVersions();
    loadUdpStatus();
    loadNetworkInterfaces();
    loadUdpSettings();
  }, []);

  // Periodically refresh UDP status to keep UI in sync
  useEffect(() => {
    const interval = setInterval(() => {
      loadUdpStatus();
    }, 2000); // Refresh every 2 seconds

    return () => clearInterval(interval);
  }, []);

  // Reload network interfaces when auto-detect changes
  useEffect(() => {
    if (!udpSettings.network_interface.auto_detect) {
      loadNetworkInterfaces();
    }
  }, [udpSettings.network_interface.auto_detect]);

  const loadProtocolVersions = async () => {
    try {
      setIsLoading(true);
      setError('');
      
      const result = await invoke('protocol_get_versions');
      const data = result as any;
      
      if (data.success) {
        setProtocolVersions(data.versions || []);
        setCurrentProtocol(data.current_protocol);
        
        // Find active version
        const active = data.versions?.find((v: ProtocolVersion) => v.is_active);
        if (active) {
          setActiveVersion(active.version);
        }
      } else {
        setError('Failed to load protocol versions');
      }
    } catch (err) {
      console.error('Error loading protocol versions:', err);
      setError('Failed to load protocol versions');
    } finally {
      setIsLoading(false);
    }
  };

  const loadUdpStatus = async () => {
    try {
      const status = await invoke('get_udp_status');
      const statusStr = status as string;
      
      setUdpStatus(statusStr);
      
      // Sync the toggle state with actual server status
      const isRunning = statusStr.includes("Running");
      setUdpEnabled(isRunning);
    } catch (err) {
      console.error('Error loading UDP status:', err);
    }
  };

  const loadNetworkInterfaces = async () => {
    try {
      const result = await invoke('get_network_interfaces');
      if (result && typeof result === 'object' && 'success' in result) {
        const data = result as any;
        if (data.success) {
          setNetworkInterfaces(data.interfaces || []);
        }
      }
    } catch (err) {
      console.error('Failed to load network interfaces:', err);
    }
  };

  const loadUdpSettings = async () => {
    try {
      const result = await invoke('get_settings');
      if (result && typeof result === 'object' && 'udp' in result) {
        const udpSettings = (result as any).udp;
        setUdpSettings({
          port: udpSettings.listener.port || 8888,
          bind_address: udpSettings.listener.bind_address || '127.0.0.1',
          enabled: udpSettings.listener.enabled || false,
          network_interface: {
            auto_detect: udpSettings.listener.network_interface?.auto_detect ?? true,
            preferred_type: udpSettings.listener.network_interface?.preferred_type ?? 'ethernet',
            fallback_to_localhost: udpSettings.listener.network_interface?.fallback_to_localhost ?? true,
            selected_interface: udpSettings.listener.network_interface?.selected_interface ?? null,
          },
        });
        setUdpPort(udpSettings.listener.port || 8888);
      }
    } catch (err) {
      console.error('Failed to load UDP settings:', err);
    }
  };

  const loadBestInterface = async () => {
    try {
      const result = await invoke('get_best_network_interface');
      if (result && typeof result === 'object' && 'success' in result) {
        const data = result as any;
        if (data.success) {
          setBestInterface(data.interface);
        }
      }
    } catch (err) {
      console.error('Failed to load best interface:', err);
    }
  };

  const saveUdpSettings = async () => {
    try {
      setError('');
      setSuccess('');
      
      // Get current settings first
      const currentSettings = await invoke('get_settings');
      if (!currentSettings || typeof currentSettings !== 'object') {
        throw new Error('Failed to get current settings');
      }

      // Merge UDP settings with current settings
      const updatedSettings = {
        ...currentSettings,
        udp: {
          ...(currentSettings as any).udp,
          listener: {
            port: udpSettings.port,
            bind_address: udpSettings.bind_address,
            enabled: udpSettings.enabled,
            buffer_size: 8192,
            timeout_seconds: 30,
            network_interface: udpSettings.network_interface,
          },
        },
      };

      await invoke('update_settings', { settings: updatedSettings });
      
      setUdpPort(udpSettings.port);
      await loadBestInterface();
      setSuccess('UDP settings saved successfully');
    } catch (err) {
      console.error('Failed to save UDP settings:', err);
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      setError(`Failed to save UDP settings: ${errorMessage}`);
    }
  };

  const handleVersionChange = async (version: string) => {
    try {
      setError('');
      setSuccess('');
      
      const result = await invoke('protocol_set_active_version', { version });
      const data = result as any;
      
      if (data.success) {
        setActiveVersion(version);
        setSuccess(data.message);
        
        // Reload protocol versions to update active status
        await loadProtocolVersions();
      } else {
        setError(data.error || 'Failed to set active version');
      }
    } catch (err) {
      console.error('Error setting active version:', err);
      setError('Failed to set active version');
    }
  };

  const handleFileUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    try {
      setError('');
      setSuccess('');
      setUploadProgress(0);
      setIsLoading(true);

      // Read file content
      const arrayBuffer = await file.arrayBuffer();
      const fileContent = new Uint8Array(arrayBuffer);
      
      setUploadProgress(50);

      // Upload to backend
      const result = await invoke('protocol_upload_file', {
        fileContent: Array.from(fileContent),
        filename: file.name
      });
      
      const data = result as any;
      
      if (data.success) {
        setSuccess(data.message);
        setUploadProgress(100);
        
        // Reload protocol versions
        await loadProtocolVersions();
        
        // Clear file input
        if (fileInputRef.current) {
          fileInputRef.current.value = '';
        }
      } else {
        setError(data.error || 'Failed to upload protocol file');
      }
    } catch (err) {
      console.error('Error uploading protocol file:', err);
      setError('Failed to upload protocol file');
    } finally {
      setIsLoading(false);
      setUploadProgress(0);
    }
  };

  const handleDeleteVersion = async (version: string) => {
    if (!confirm(`Are you sure you want to delete protocol version ${version}?`)) {
      return;
    }

    try {
      setError('');
      setSuccess('');
      
      const result = await invoke('protocol_delete_version', { version });
      const data = result as any;
      
      if (data.success) {
        setSuccess(data.message);
        
        // Reload protocol versions
        await loadProtocolVersions();
      } else {
        setError(data.error || 'Failed to delete protocol version');
      }
    } catch (err) {
      console.error('Error deleting protocol version:', err);
      setError('Failed to delete protocol version');
    }
  };

  const handleExportVersion = async (version: string) => {
    try {
      const fileContent = await invoke('protocol_export_file', { version }) as number[];
      
      // Create blob and download
      const blob = new Blob([new Uint8Array(fileContent)], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `protocol_v${version}.txt`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      
      setSuccess(`Protocol version ${version} exported successfully`);
    } catch (err) {
      console.error('Error exporting protocol version:', err);
      setError('Failed to export protocol version');
    }
  };

  const handleUdpToggle = async (enabled: boolean) => {
    try {
      if (enabled) {
        await invoke('start_udp_server');
      } else {
        await invoke('stop_udp_server');
      }
      
      await loadUdpStatus();
    } catch (err) {
      console.error('Error toggling UDP server:', err);
      setError('Failed to toggle UDP server');
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatDate = (dateString: string): string => {
    try {
      return new Date(dateString).toLocaleDateString();
    } catch {
      return dateString;
    }
  };

  // UDP Server & Protocol Content
  const UdpServerContent = () => (
    <div className="space-y-6">
      {/* Error and Success Messages */}
      {error && (
        <div className="p-4 bg-red-900/20 border border-red-600/30 rounded-lg text-red-300">
          {error}
        </div>
      )}
      {success && (
        <div className="p-4 bg-green-900/20 border border-green-600/30 rounded-lg text-green-300">
          {success}
        </div>
      )}

      {/* UDP Server Section */}
      <div className="p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-100">UDP Server</h3>
          <Button
            size="sm"
            variant="secondary"
            onClick={() => setShowAdvancedSettings(!showAdvancedSettings)}
          >
            <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor" className="mr-2">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={showAdvancedSettings ? "M5 15l7-7 7 7" : "M19 9l-7 7-7-7"} />
            </svg>
            {showAdvancedSettings ? 'Hide' : 'Show'} Settings
          </Button>
        </div>
        
        {/* Basic UDP Status */}
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-300">UDP Server Status</p>
              <p className="text-xs text-gray-400">Port: {udpPort}</p>
            </div>
            <div className="flex items-center space-x-3">
              <span className={`px-2 py-1 rounded text-xs font-medium ${
                udpStatus === 'Running' 
                  ? 'bg-green-900/30 text-green-300 border border-green-600/30' 
                  : 'bg-red-900/30 text-red-300 border border-red-600/30'
              }`}>
                {udpStatus}
              </span>
              <Toggle
                id="udp-enabled"
                checked={udpEnabled}
                onChange={(e) => handleUdpToggle(e.target.checked)}
                label="Enable UDP Server"
                labelPosition="left"
              />
            </div>
          </div>
        </div>

        {/* Advanced Settings (Expandable) */}
        {showAdvancedSettings && (
          <div className="mt-6 pt-4 border-t border-gray-600/30">
            <h4 className="text-sm font-medium text-gray-300 mb-4">Advanced Settings</h4>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {/* Port Configuration */}
              <div className="space-y-4">
                <h5 className="text-xs font-medium text-gray-400 uppercase tracking-wide">Port Configuration</h5>
                
                <div>
                  <Label htmlFor="udp-port" className="text-xs text-gray-400">UDP Port</Label>
                  <Input
                    id="udp-port"
                    type="number"
                    value={udpSettings.port}
                    onChange={(e) => setUdpSettings(prev => ({ ...prev, port: parseInt(e.target.value) || 8888 }))}
                    min={1024}
                    max={65535}
                    className="mt-1 text-sm"
                  />
                </div>

                <div>
                  <Label htmlFor="bind-address" className="text-xs text-gray-400">Bind Address</Label>
                  <Input
                    id="bind-address"
                    type="text"
                    value={udpSettings.bind_address}
                    onChange={(e) => setUdpSettings(prev => ({ ...prev, bind_address: e.target.value }))}
                    placeholder="127.0.0.1"
                    className="mt-1 text-sm"
                  />
                </div>
              </div>

              {/* Network Interface Settings */}
              <div className="space-y-4">
                <h5 className="text-xs font-medium text-gray-400 uppercase tracking-wide">Network Interface</h5>
                
                <div>
                  <Checkbox
                    id="auto-detect"
                    checked={udpSettings.network_interface.auto_detect}
                    onChange={(e) => setUdpSettings(prev => ({
                      ...prev,
                      network_interface: {
                        ...prev.network_interface,
                        auto_detect: e.target.checked,
                      }
                    }))}
                  />
                  <Label htmlFor="auto-detect" className="ml-2 text-xs text-gray-400">
                    Auto-detect network interface
                  </Label>
                </div>

                {udpSettings.network_interface.auto_detect && (
                  <div>
                    <Label htmlFor="preferred-type" className="text-xs text-gray-400">Preferred Type</Label>
                    <select
                      id="preferred-type"
                      value={udpSettings.network_interface.preferred_type}
                      onChange={(e) => setUdpSettings(prev => ({
                        ...prev,
                        network_interface: {
                          ...prev.network_interface,
                          preferred_type: e.target.value,
                        }
                      }))}
                      className="mt-1 block w-full rounded-md border-gray-600 bg-gray-700 text-gray-200 text-sm"
                      aria-label="Preferred interface type"
                    >
                      <option value="ethernet">Ethernet</option>
                      <option value="wifi">WiFi</option>
                      <option value="any">Any</option>
                    </select>
                  </div>
                )}

                {!udpSettings.network_interface.auto_detect && (
                  <div>
                    <div className="flex items-center justify-between">
                      <Label htmlFor="selected-interface" className="text-xs text-gray-400">Select Interface</Label>
                      <button
                        onClick={loadNetworkInterfaces}
                        className="text-xs text-blue-400 hover:text-blue-300 underline"
                      >
                        Refresh
                      </button>
                    </div>
                    <select
                      id="selected-interface"
                      value={udpSettings.network_interface.selected_interface || ''}
                      onChange={(e) => setUdpSettings(prev => ({
                        ...prev,
                        network_interface: {
                          ...prev.network_interface,
                          selected_interface: e.target.value || null,
                        }
                      }))}
                      className="mt-1 block w-full rounded-md border-gray-600 bg-gray-700 text-gray-200 text-sm"
                      aria-label="Select network interface"
                    >
                      <option value="">Select an interface...</option>
                      {networkInterfaces.map((iface) => (
                        <option key={iface.name} value={iface.name}>
                          {iface.name} ({iface.type}) - {iface.media_state === 'connected' ? 'Connected' : 'Disconnected'} - {iface.ip_addresses.join(', ')}
                        </option>
                      ))}
                    </select>
                  </div>
                )}

                <div>
                  <Checkbox
                    id="fallback-localhost"
                    checked={udpSettings.network_interface.fallback_to_localhost}
                    onChange={(e) => setUdpSettings(prev => ({
                      ...prev,
                      network_interface: {
                        ...prev.network_interface,
                        fallback_to_localhost: e.target.checked,
                      }
                    }))}
                  />
                  <Label htmlFor="fallback-localhost" className="ml-2 text-xs text-gray-400">
                    Fallback to localhost (127.0.0.1)
                  </Label>
                </div>
              </div>
            </div>

            {/* Available Network Interfaces */}
            <div className="mt-6">
              <h5 className="text-xs font-medium text-gray-400 uppercase tracking-wide mb-3">Available Interfaces</h5>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                {networkInterfaces.map((iface) => (
                  <div
                    key={iface.name}
                    className={`p-3 rounded-lg border text-xs ${
                      bestInterface?.name === iface.name
                        ? 'border-green-500 bg-green-900/20'
                        : 'border-gray-600 bg-gray-700/30'
                    }`}
                  >
                    <div className="flex items-center justify-between mb-1">
                      <span className="font-medium text-gray-200 truncate">
                        {iface.name}
                      </span>
                      <StatusDot color={iface.media_state === 'connected' ? 'green' : 'red'} />
                    </div>
                    <div className="text-gray-400 space-y-1">
                      <div className="flex items-center space-x-2">
                        <span className="capitalize">{iface.type}</span>
                        {iface.media_state === 'disconnected' && (
                          <span className="text-red-400 text-xs">(Disconnected)</span>
                        )}
                      </div>
                      {iface.description && (
                        <div className="text-xs text-gray-500 truncate">
                          {iface.description}
                        </div>
                      )}
                      {iface.ip_addresses.length > 0 && (
                        <div>
                          <div className="font-medium text-gray-300">IP Addresses:</div>
                          {iface.ip_addresses.map((ip, index) => (
                            <div key={index} className="ml-2 text-xs">
                              {ip}
                              {iface.subnet_masks[index] && (
                                <span className="text-gray-500 ml-1">/ {iface.subnet_masks[index]}</span>
                              )}
                            </div>
                          ))}
                        </div>
                      )}
                      {iface.default_gateway && (
                        <div className="text-xs">
                          <span className="text-gray-500">Gateway:</span> {iface.default_gateway}
                        </div>
                      )}
                      {iface.dns_suffix && (
                        <div className="text-xs">
                          <span className="text-gray-500">DNS:</span> {iface.dns_suffix}
                        </div>
                      )}
                      {bestInterface?.name === iface.name && (
                        <div className="text-green-400 font-medium mt-1">
                          ⭐ Recommended
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Save Button */}
            <div className="mt-6 pt-4 border-t border-gray-600/30">
              <Button
                size="sm"
                variant="primary"
                onClick={saveUdpSettings}
              >
                Save Settings
              </Button>
            </div>
          </div>
        )}
      </div>

      {/* Protocol Management Section */}
      <div className="p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
        <h3 className="text-lg font-semibold mb-4 text-gray-100">Protocol Management</h3>
        
        {/* Current Protocol */}
        {currentProtocol && (
          <div className="mb-6 p-4 bg-blue-900/20 border border-blue-600/30 rounded-lg">
            <h4 className="text-sm font-medium text-blue-300 mb-2">Current Protocol</h4>
            <div className="text-sm text-gray-300">
              <p><span className="text-gray-400">Version:</span> {currentProtocol.version}</p>
              <p><span className="text-gray-400">Description:</span> {currentProtocol.description}</p>
              <p><span className="text-gray-400">Year:</span> {currentProtocol.year}</p>
            </div>
          </div>
        )}

        {/* Protocol Version Selection */}
        <div className="mb-6">
          <h4 className="text-sm font-medium text-gray-300 mb-3">Available Versions</h4>
          {isLoading ? (
            <div className="text-sm text-gray-400">Loading protocol versions...</div>
          ) : protocolVersions.length === 0 ? (
            <div className="text-sm text-gray-400">No protocol versions available</div>
          ) : (
            <div className="space-y-2">
              {protocolVersions.map((version) => (
                <div
                  key={version.version}
                  className={`p-3 rounded-lg border transition-all duration-200 ${
                    version.is_active
                      ? 'bg-blue-900/30 border-blue-600/50'
                      : 'bg-gray-700/30 border-gray-600/50 hover:bg-gray-700/50'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex-1">
                      <div className="flex items-center space-x-2">
                        <span className="text-sm font-medium text-gray-200">
                          Version {version.version}
                        </span>
                        {version.is_active && (
                          <span className="px-2 py-1 bg-green-900/30 text-green-300 text-xs rounded border border-green-600/30">
                            Active
                          </span>
                        )}
                      </div>
                      <p className="text-xs text-gray-400 mt-1">{version.description}</p>
                      <div className="flex items-center space-x-4 mt-2 text-xs text-gray-500">
                        <span>Size: {formatFileSize(version.file_size)}</span>
                        <span>Created: {formatDate(version.created_date)}</span>
                        <span>Modified: {formatDate(version.last_modified)}</span>
                      </div>
                    </div>
                    <div className="flex items-center space-x-2">
                      {!version.is_active && (
                        <Button
                          size="sm"
                          variant="secondary"
                          onClick={() => handleVersionChange(version.version)}
                        >
                          Activate
                        </Button>
                      )}
                      <Button
                        size="sm"
                        variant="secondary"
                        onClick={() => handleExportVersion(version.version)}
                      >
                        Export
                      </Button>
                      {!version.is_active && (
                        <Button
                          size="sm"
                          variant="danger"
                          onClick={() => handleDeleteVersion(version.version)}
                        >
                          Delete
                        </Button>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Upload New Protocol */}
        <div className="border-t border-gray-600/30 pt-4">
          <h4 className="text-sm font-medium text-gray-300 mb-3">Upload New Protocol</h4>
          <div className="space-y-3">
            <div className="flex items-center space-x-3">
              <input
                ref={fileInputRef}
                type="file"
                accept=".txt"
                onChange={handleFileUpload}
                className="hidden"
                aria-label="Choose protocol file"
                title="Choose protocol file"
              />
              <Button
                size="sm"
                variant="primary"
                onClick={() => fileInputRef.current?.click()}
                disabled={isLoading}
              >
                Choose Protocol File
              </Button>
              <span className="text-xs text-gray-400">TXT format only</span>
            </div>
            
            {uploadProgress > 0 && (
              <div className="w-full bg-gray-700 rounded-full h-2">
                <div
                  className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${uploadProgress}%` }}
                ></div>
              </div>
            )}
            
            <p className="text-xs text-gray-400">
              Upload a new protocol file to add it to the available versions. 
              The file should be in TXT format and contain valid protocol definitions.
            </p>
          </div>
        </div>
      </div>
    </div>
  );

  // Flag Management Content
  const FlagManagementContent = () => (
    <FlagManagementPanel />
  );

  // Scoreboard Content
  const ScoreboardContent = () => (
    <ScoreboardManager />
  );

  return (
    <div className={className}>
      <TabGroup
        tabs={[
          {
            id: 'udp',
            label: 'UDP Server & Protocol',
            icon: TabIcons.udp,
            content: <UdpServerContent />
          },
          {
            id: 'flags',
            label: 'Flag Management',
            icon: TabIcons.flags,
            content: <FlagManagementContent />
          },
          {
            id: 'scoreboard',
            label: 'Scoreboard',
            icon: TabIcons.scoreboard,
            content: <ScoreboardContent />
          }
        ]}
        activeTab={activeTab}
        onTabChange={setActiveTab}
      />
    </div>
  );
};

export default PssDrawer; 