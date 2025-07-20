import React, { useState, useRef, useEffect } from 'react';
import { diagLogsCommands, configCommands } from '../../utils/tauriCommands';
import { useEnvironment } from '../../hooks/useEnvironment';
import { useLiveDataStore, LiveDataType } from '../../stores/liveDataStore';
import Toggle from '../atoms/Toggle';

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
  const [showFullEvents, setShowFullEvents] = useState(false);
  const [isLoadingSettings, setIsLoadingSettings] = useState(true);
  const liveDataRef = useRef<HTMLDivElement>(null);
  const pollingIntervalRef = useRef<NodeJS.Timeout | null>(null);

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

  // Load full events setting on mount
  useEffect(() => {
    const loadFullEventsSetting = async () => {
      try {
        const result = await invoke('obs_get_full_events_setting');
        if (result && result.success) {
          setShowFullEvents(result.enabled);
        }
      } catch (error) {
        console.error('Failed to load full events setting:', error);
      }
    };
    loadFullEventsSetting();
  }, []);

  // Poll for OBS events when full events are enabled
  useEffect(() => {
    if (!tauriAvailable || selectedType !== 'obs' || !showFullEvents) return;

    // Set up polling for OBS events every 3 seconds when full events are enabled
    const eventPollingInterval = setInterval(async () => {
      try {
        const result = await invoke('obs_get_recent_events');
        if (result && result.success && result.events && result.events.length > 0) {
          result.events.forEach((event: any) => {
            const formattedEvent = `[OBS-EVENT][${event.connection_name}] ${event.event_type}: ${JSON.stringify(event.data)}`;
            addData({
              subsystem: 'obs',
              data: formattedEvent,
              type: 'info'
            });
          });
        }
      } catch (error) {
        console.error('❌ Failed to fetch OBS events:', error);
      }
    }, 3000);

    return () => {
      clearInterval(eventPollingInterval);
    };
  }, [tauriAvailable, selectedType, showFullEvents, addData]);

  // Poll for UDP/PSS events when UDP is selected
  useEffect(() => {
    if (!tauriAvailable || selectedType !== 'udp' || !enabled) return;

    // Set up polling for UDP events every 2 seconds
    const udpEventPollingInterval = setInterval(async () => {
      try {
        const result = await invoke('pss_get_events');
        if (result && Array.isArray(result) && result.length > 0) {
          result.forEach((event: any) => {
            const formattedEvent = `${event.description}`;
            addData({
              subsystem: 'udp',
              data: formattedEvent,
              type: 'info'
            });
          });
        }
      } catch (error) {
        console.error('❌ Failed to fetch UDP events:', error);
      }
    }, 2000);

    return () => {
      clearInterval(udpEventPollingInterval);
    };
  }, [tauriAvailable, selectedType, enabled, addData]);

  // Handle full events toggle
  const handleFullEventsToggle = async () => {
    try {
      const newValue = !showFullEvents;
      const result = await invoke('obs_toggle_full_events', { enabled: newValue });
      if (result && result.success) {
        setShowFullEvents(newValue);
        console.log('Full events toggle:', result.message);
      }
    } catch (error) {
      console.error('Failed to toggle full events:', error);
    }
  };

  // Debug logging on mount only
  useEffect(() => {
    // Component mounted
  }, []);

  // Test Tauri API on component mount (only once)
  useEffect(() => {
    // Only test once on mount, not on every render
    const testOnce = async () => {
      try {
        await window.__TAURI__.core.invoke('get_app_status');
      } catch (error) {
        // Tauri API test failed, but we can continue
      }
    };
    testOnce();
  }, []);

  // Auto-scroll to top only if user is already at the top
  useEffect(() => {
    if (liveDataRef.current) {
      const element = liveDataRef.current;
      const isAtTop = element.scrollTop === 0;
      
      // Only auto-scroll if user is already at the top
      if (isAtTop) {
        element.scrollTop = 0;
      }
    }
  }, [data]);

  // Polling function to fetch live data - for OBS and UDP status updates
  const fetchLiveData = async () => {
    try {
      if (selectedType === 'obs' || selectedType === 'udp' || selectedType === 'pss') {
        const result = await invoke('get_live_data', { subsystem: selectedType });
        
        if (result && result.success && result.data) {
          // Only add data if there are actual changes
          const formattedData = formatLiveData(result.data);
          
          // Check if this is different from the last status
          const lastEntry = data[data.length - 1];
          if (!lastEntry || lastEntry.data !== formattedData) {
            addData({
              subsystem: selectedType,
              data: formattedData,
              type: 'info'
            });
          }
        }
      }
      // For other subsystems, don't poll - only show real events
    } catch (error) {
      console.error('Failed to fetch live data:', error);
      setError(`Failed to fetch live data: ${error}`);
    }
  };

  // Format live data for display
  const formatLiveData = (data: any): string => {
    switch (data.subsystem) {
      case 'obs':
        return `OBS Status: Recording=${data.is_recording ? 'ON' : 'OFF'}, Streaming=${data.is_streaming ? 'ON' : 'OFF'}, CPU=${data.cpu_usage.toFixed(1)}%, Recording Conn=${data.recording_connection || 'None'}, Streaming Conn=${data.streaming_connection || 'None'}`;
      
      case 'pss':
        return `PSS Stats: Packets=${data.packets_received}, Parsed=${data.packets_parsed}, Errors=${data.parse_errors}, Clients=${data.connected_clients}, Last Packet=${data.last_packet_time ? new Date(data.last_packet_time * 1000).toLocaleTimeString() : 'Never'}`;
      
      case 'udp':
        return `📡 UDP Server: ${data.status} | 🏃 Running: ${data.is_running ? 'Yes' : 'No'} | 📦 Packets: ${data.packets_received} | ✅ Parsed: ${data.packets_parsed} | ❌ Errors: ${data.parse_errors} | 👥 Clients: ${data.connected_clients} | 📊 Bytes: ${data.total_bytes_received} | ⏱️ Uptime: ${data.uptime}`;
      
      default:
        return JSON.stringify(data, null, 2);
    }
  };

  // Handle streaming toggle and type change
  useEffect(() => {
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
          // Skip the setLiveDataStreaming command for now and go straight to polling
          // This command might not be necessary for our polling approach
          
          clearData();
          
          if (enabled && (selectedType === 'obs' || selectedType === 'udp' || selectedType === 'pss')) {
            // Start polling for status every 5 seconds (less frequent)
            pollingIntervalRef.current = setInterval(fetchLiveData, 5000);
            
            // Fetch initial data immediately
            await fetchLiveData();
          } else if (enabled) {
            // For other subsystems, don't poll - only show real events
          } else {
            // Stop polling
            if (pollingIntervalRef.current) {
              clearInterval(pollingIntervalRef.current);
              pollingIntervalRef.current = null;
            }
          }
        } else {
          setError('Tauri not available - running in web mode');
        }
      } catch (err) {
        console.error('Streaming setup error:', err);
        setError(`Streaming error: ${err}`);
      } finally {
        setConnecting(false);
      }
    };

    setupStreaming();

    return () => {
      // Cleanup polling interval
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current);
        pollingIntervalRef.current = null;
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

  const scrollToTop = () => {
    if (liveDataRef.current) {
      liveDataRef.current.scrollTop = 0;
    }
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
        <Toggle
          checked={enabled} 
          onChange={handleToggle} 
          label="Enable"
          labelPosition="right"
          disabled={connecting}
        />
        {connecting && (
          <span className="text-blue-400 text-sm">Connecting...</span>
        )}
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
        {selectedType === 'obs' && (
          <Toggle
            checked={showFullEvents} 
            onChange={handleFullEventsToggle} 
            label="Full Events"
            labelPosition="right"
            disabled={connecting}
            className="ml-4"
          />
        )}
      </div>
      {error && (
        <div className="mb-3 p-2 bg-red-900/20 border border-red-700 rounded text-red-400 text-sm">
          {error}
        </div>
      )}
      <div className="relative">
        <div
          ref={liveDataRef}
          className="bg-black border border-gray-700 rounded p-3 h-48 overflow-y-auto font-mono text-sm text-green-400"
          style={{ whiteSpace: 'pre-wrap' }}
        >
          {data.length === 0 ? 'No live data available. Enable streaming to see data.' : 
            data.slice().reverse().map((entry, index) => (
              <div key={index} className="mb-1">
                <span className="text-green-600">[{entry.timestamp}]</span> {entry.data}
              </div>
            ))
          }
        </div>
        {data.length > 0 && (
          <button
            onClick={scrollToTop}
            className="absolute top-2 right-2 bg-gray-800 hover:bg-gray-700 text-green-400 text-xs px-2 py-1 rounded border border-gray-600 transition-colors"
            title="Scroll to top"
          >
            ↑ Top
          </button>
        )}
      </div>
    </div>
  );
};

export default LiveDataPanel; 