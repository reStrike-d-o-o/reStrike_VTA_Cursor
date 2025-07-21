import React, { useState, useRef, useEffect } from 'react';
import { diagLogsCommands, configCommands } from '../../utils/tauriCommands';
import { useEnvironment } from '../../hooks/useEnvironment';
import { useLiveDataStore, LiveDataType } from '../../stores/liveDataStore';
import Toggle from '../atoms/Toggle';
import { useLiveDataEvents } from '../../hooks/useLiveDataEvents';

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
  // Polling removed – backend now pushes events via Tauri
  const pollingIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const [autoScroll, setAutoScroll] = useState(false);

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
    // Disabled polling for PSS/UDP events – we now rely on push events via Tauri
    return () => {};
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

  // Auto-scroll to bottom when new data arrives if autoScroll is true
  useEffect(() => {
    if (autoScroll && liveDataRef.current) {
      liveDataRef.current.scrollTop = liveDataRef.current.scrollHeight;
    }
  }, [data, autoScroll]);

  // Polling removed – no longer required

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
          // Tell backend to start / stop streaming for selected subsystem
          try {
            await diagLogsCommands.setLiveDataStreaming(selectedType, enabled);
          } catch (err) {
            console.error('setLiveDataStreaming error:', err);
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
      // Ensure backend stops pushing events when component unmounts
      if (tauriAvailable) {
        diagLogsCommands.setLiveDataStreaming(selectedType, false).catch(() => {});
      }
    };
  }, [enabled, selectedType, tauriAvailable]);

  // Subscribe to pushed live_data events (filtered by current type)
  useLiveDataEvents(enabled, selectedType);

  const handleToggle = async () => {
    clearError();
    const newEnabled = !enabled;
    setEnabled(newEnabled);
    await saveLiveDataSettings(newEnabled);
  };

  const handleTypeChange = (newType: LiveDataType) => {
    clearError();
    clearData(); // wipe previous log when type changes
    setSelectedType(newType);
  };

  const scrollToTop = () => {
    if (liveDataRef.current) {
      liveDataRef.current.scrollTop = 0;
    }
  };

  const scrollToBottom = () => {
    if (liveDataRef.current) {
      liveDataRef.current.scrollTop = liveDataRef.current.scrollHeight;
      setAutoScroll(true);
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
          onScroll={() => {
            if (!liveDataRef.current) return;
            const el = liveDataRef.current;
            const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 5;
            setAutoScroll(atBottom);
          }}
        >
          {data.filter(e => e.subsystem === selectedType).length === 0 ? 'No live data available. Enable streaming to see data.' : 
            data.filter(e => e.subsystem === selectedType).map((entry, index) => {
              const color = entry.type === 'error' ? 'text-red-400' : entry.type === 'warning' ? 'text-yellow-300' : entry.type === 'debug' ? 'text-blue-300' : 'text-green-400';
              const emoji = entry.type === 'error' ? '❌' : entry.type === 'warning' ? '⚠️' : entry.type === 'debug' ? '🐞' : '📡';
              return (
                <div key={index} className={`mb-1 ${color}`}>
                  <span className="text-green-600">[{entry.timestamp}]</span> {emoji} {entry.data}
                </div>
              );
            })
          }
        </div>
        {data.length > 0 && (
          <div className="absolute top-2 right-2 flex gap-1">
            <button
              onClick={scrollToTop}
              className="bg-gray-800 hover:bg-gray-700 text-green-400 text-xs px-2 py-1 rounded border border-gray-600 transition-colors"
              title="Scroll to top"
            >
              ↑ Top
            </button>
            <button
              onClick={scrollToBottom}
              className="bg-gray-800 hover:bg-gray-700 text-green-400 text-xs px-2 py-1 rounded border border-gray-600 transition-colors"
              title="Scroll to bottom"
            >
              ↓ End
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

export default LiveDataPanel; 