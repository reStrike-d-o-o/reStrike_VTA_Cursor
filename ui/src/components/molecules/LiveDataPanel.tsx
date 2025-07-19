import React, { useState, useRef, useEffect } from 'react';
import { diagLogsCommands, configCommands } from '../../utils/tauriCommands';
import { useEnvironment } from '../../hooks/useEnvironment';
import { useLiveDataStore, LiveDataType } from '../../stores/liveDataStore';
import { listen } from '@tauri-apps/api/event';

// Use the proper Tauri v2 invoke function
const invoke = async (command: string, args?: any) => {
  try {
    // Use the global window.__TAURI__.core.invoke for Tauri v2
    if (typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.core) {
      return await window.__TAURI__.core.invoke(command, args);
    }
    throw new Error('Tauri v2 core module not available - ensure app is running in desktop mode');
  } catch (error) {
    console.error('Tauri invoke failed:', error);
    throw error;
  }
};

// Debug function to test Tauri API directly
const testTauriApiDirectly = async () => {
  console.log('🔍 Testing Tauri API directly...');
  console.log('window.__TAURI__:', window.__TAURI__);
  console.log('window.__TAURI__.core:', window.__TAURI__?.core);
  
  try {
    const result = await window.__TAURI__.core.invoke('get_app_status');
    console.log('✅ Direct invoke successful:', result);
    return true;
  } catch (error) {
    console.log('❌ Direct invoke failed:', error);
    return false;
  }
};

const logTypes = [
  { key: 'pss', label: 'PSS' },
  { key: 'obs', label: 'OBS' },
  { key: 'udp', label: 'UDP' },
];

const LiveDataPanel: React.FC = () => {
  const { tauriAvailable, environment, isWindows, isWeb } = useEnvironment();
  const {
    enabled,
    selectedType,
    data,
    loading,
    error,
    connecting,
    setEnabled,
    setSelectedType,
    addData,
    clearData,
    setLoading,
    setError,
    clearError,
    setConnecting,
  } = useLiveDataStore();
  const [isLoadingSettings, setIsLoadingSettings] = useState(true);
  const liveDataRef = useRef<HTMLDivElement>(null);

  // Load live data settings from configuration
  const loadLiveDataSettings = async () => {
    try {
      setIsLoadingSettings(true);
      const result = await configCommands.getSettings();
      if (result.success && result.data?.logging?.live_data) {
        const liveDataSettings = result.data.logging.live_data;
        setEnabled(liveDataSettings.enabled ?? true);
        // Note: selectedType is not stored in config, so we keep the default
      }
    } catch (error) {
      console.error('Failed to load live data settings:', error);
    } finally {
      setIsLoadingSettings(false);
    }
  };

  // Save live data settings to configuration
  const saveLiveDataSettings = async (newEnabled: boolean) => {
    try {
      const result = await configCommands.getSettings();
      if (result.success) {
        const updatedSettings = {
          ...result.data,
          logging: {
            ...result.data.logging,
            live_data: {
              ...result.data.logging.live_data,
              enabled: newEnabled,
            },
          },
        };
        await configCommands.updateSettings(updatedSettings);
        console.log('Live data settings saved successfully');
      }
    } catch (error) {
      console.error('Failed to save live data settings:', error);
    }
  };

  // Load settings on component mount
  useEffect(() => {
    loadLiveDataSettings();
  }, []);

  // Debug logging on every render
  console.log('🔍 LiveDataPanel Render:', {
    tauriAvailable,
    environment,
    isWindows,
    isWeb,
    windowTauri: typeof window !== 'undefined' ? !!window.__TAURI__ : 'undefined',
    windowLocation: typeof window !== 'undefined' ? window.location.href : 'undefined'
  });

  // Test Tauri API on component mount
  useEffect(() => {
    testTauriApiDirectly();
  }, []);

  // Scroll to bottom on new data
  useEffect(() => {
    if (liveDataRef.current) {
      liveDataRef.current.scrollTop = liveDataRef.current.scrollHeight;
    }
  }, [data]);

  // Handle streaming toggle and type change
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    setError('');
    setConnecting(true);

    // Debug logging
    console.log('🔍 LiveDataPanel Environment Check:', {
      tauriAvailable,
      windowTauri: typeof window !== 'undefined' ? !!window.__TAURI__ : 'undefined',
      selectedType,
      enabled
    });

    const setupStreaming = async () => {
      try {
        if (tauriAvailable) {
          const result = await diagLogsCommands.setLiveDataStreaming(selectedType, enabled);
          if (!result.success) {
            setError(`Failed to ${enabled ? 'start' : 'stop'} streaming: ${result.error}`);
            return;
          }

          clearData();
          
          // Use the proper Tauri v2 event listening API
          try {
            console.log('Attempting to listen for live_data events...');
            const listener = await listen('live_data', (event: any) => {
              console.log('Live data event received:', event);
              if (event && event.payload && event.payload.subsystem === selectedType) {
                addData({
                  subsystem: selectedType,
                  data: event.payload.data,
                  type: 'info'
                });
              }
            });
            
            console.log('Event listener registered successfully');
            unlisten = listener;
          } catch (listenError) {
            console.error('Event listening error:', listenError);
            setError(`Event listening failed: ${listenError}`);
          }
        } else {
          setError('Tauri not available - running in web mode');
        }
      } catch (err) {
        setError(`Streaming error: ${err}`);
      } finally {
        setConnecting(false);
      }
    };

    setupStreaming();

    return () => {
      if (unlisten) {
        try {
          unlisten();
        } catch (err) {
          console.error('Error cleaning up listener:', err);
        }
      }
      if (tauriAvailable) {
        diagLogsCommands.setLiveDataStreaming(selectedType, false).catch((err: any) => {
          console.error('Error stopping streaming:', err);
        });
      }
    };
  }, [enabled, selectedType, tauriAvailable]);

  const handleToggle = async () => {
    clearError();
    const newEnabled = !enabled;
    setEnabled(newEnabled);
    await saveLiveDataSettings(newEnabled);
  };

  const handleTypeChange = (newType: LiveDataType) => {
    clearError();
    setSelectedType(newType);
  };

  if (isLoadingSettings) {
    return (
      <div className="bg-[#181F26] rounded-lg p-4 border border-gray-700 shadow">
        <h3 className="text-lg font-semibold mb-2 text-blue-300">LIVE DATA</h3>
        <div className="text-sm text-gray-400">Loading settings...</div>
      </div>
    );
  }

  return (
    <div className="bg-[#181F26] rounded-lg p-4 border border-gray-700 shadow">
      <h3 className="text-lg font-semibold mb-2 text-blue-300">LIVE DATA</h3>
      

      
      <div className="flex items-center gap-3 mb-3">
        <label className="flex items-center gap-2 cursor-pointer">
          <input 
            type="checkbox" 
            checked={enabled} 
            onChange={handleToggle} 
            className="accent-blue-500"
            disabled={connecting}
          />
          <span className="text-gray-200 font-medium">Enable</span>
          {connecting && (
            <span className="text-blue-400 text-sm">Connecting...</span>
          )}
        </label>
        <span className="text-gray-200 font-medium" id="live-type-label">Type:</span>
        <select
          className="bg-[#101820] border border-gray-700 rounded px-2 py-1 text-gray-100"
          value={selectedType}
          onChange={e => handleTypeChange(e.target.value as LiveDataType)}
          aria-labelledby="live-type-label"
          title="Select live data type"
          aria-label="Select live data type"
          disabled={connecting}
        >
          {logTypes.map(type => (
            <option key={type.key} value={type.key}>{type.label}</option>
          ))}
        </select>
      </div>
      {error && (
        <div className="mb-3 p-2 bg-red-900/20 border border-red-700 rounded text-red-400 text-sm">
          {error}
        </div>
      )}
      <div
        ref={liveDataRef}
        className="bg-[#101820] border border-gray-700 rounded p-3 h-48 overflow-y-auto font-mono text-sm text-gray-300"
        style={{ whiteSpace: 'pre-wrap' }}
      >
        {data.length === 0 ? 'No live data available. Enable streaming to see data.' : 
          data.map((entry, index) => (
            <div key={index} className="mb-1">
              <span className="text-gray-500">[{entry.timestamp}]</span> {entry.data}
            </div>
          ))
        }
      </div>
    </div>
  );
};

export default LiveDataPanel; 