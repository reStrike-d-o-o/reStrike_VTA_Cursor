import React, { useState, useRef, useEffect } from 'react';
import { diagLogsCommands, configCommands } from '../../utils/tauriCommands';
import { useEnvironment } from '../../hooks/useEnvironment';
import { useLiveDataStore } from '../../stores/liveDataStore';
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

const LiveDataPanel: React.FC = () => {
  const { tauriAvailable, environment, isWindows, isWeb } = useEnvironment();
  const { 
    events, 
    currentRound, 
    currentRoundTime, 
    isConnected
  } = useLiveDataStore();
  
  // Use the new live data events hook
  const { isConnected: wsConnected, eventCount } = useLiveDataEvents();
  
  const [showFullEvents, setShowFullEvents] = useState(false);
  const [isLoadingSettings, setIsLoadingSettings] = useState(true);
  const liveDataRef = useRef<HTMLDivElement>(null);
  const [autoScroll, setAutoScroll] = useState(false);

  // Load live data settings from configuration
  const loadLiveDataSettings = async () => {
    try {
      setIsLoadingSettings(true);
      const result = await configCommands.getSettings();
      if (result.success && result.data?.logging?.live_data) {
        const liveDataSettings = result.data.logging.live_data;
        // Note: Live data is now automatically managed by the PSS event system
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
        const result = await configCommands.getSettings();
        if (result.success && result.data?.logging?.full_events !== undefined) {
          setShowFullEvents(result.data.logging.full_events);
        }
      } catch (error) {
        console.error('Failed to load full events setting:', error);
      }
    };

    loadFullEventsSetting();
  }, []);

  // Save full events setting
  const handleFullEventsToggle = async () => {
    const newValue = !showFullEvents;
    setShowFullEvents(newValue);
    
    try {
      const result = await configCommands.getSettings();
      if (result.success) {
        const updatedSettings = {
          ...result.data,
          logging: {
            ...result.data.logging,
            full_events: newValue,
          },
        };
        await configCommands.updateSettings(updatedSettings);
        console.log('Full events setting saved successfully');
      }
    } catch (error) {
      console.error('Failed to save full events setting:', error);
    }
  };

  // Format live data for display
  const formatLiveData = (data: any): string => {
    if (typeof data === 'string') {
      return data;
    }
    
    if (typeof data === 'object') {
      try {
        return JSON.stringify(data, null, 2);
      } catch {
        return String(data);
      }
    }
    
    return String(data);
  };

  // Auto-scroll to bottom when new events arrive
  useEffect(() => {
    if (autoScroll && liveDataRef.current) {
      liveDataRef.current.scrollTop = liveDataRef.current.scrollHeight;
    }
  }, [events, autoScroll]);

  // Polling removed – no longer required

  const handleToggle = async () => {
    // Live data is now automatically managed by the PSS event system
    console.log('Live data toggle - now managed automatically');
  };

  const handleTypeChange = (newType: string) => {
    // Type selection is now handled by the PSS event system
    console.log('Live data type change - now managed automatically');
  };

  const scrollToTop = () => {
    if (liveDataRef.current) {
      liveDataRef.current.scrollTop = 0;
    }
  };

  const scrollToBottom = () => {
    if (liveDataRef.current) {
      liveDataRef.current.scrollTop = liveDataRef.current.scrollHeight;
    }
  };

  if (isLoadingSettings) {
    return (
      <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg p-4 border border-gray-600/30 shadow-lg">
        <h3 className="text-lg font-semibold mb-2 text-blue-300">LIVE DATA</h3>
        <div className="text-sm text-gray-400">Loading settings...</div>
      </div>
    );
  }

  return (
    <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg p-4 border border-gray-600/30 shadow-lg">
      <h3 className="text-lg font-semibold mb-2 text-blue-300">LIVE DATA</h3>
      

      
      <div className="flex items-center gap-3 mb-3">
        <Toggle
          checked={true} 
          onChange={handleToggle} 
          label="Enable"
          labelPosition="right"
          disabled={false}
        />
        <span className="text-gray-200 font-medium" id="live-type-label">Type:</span>
        <select
          className="bg-[#101820] border border-gray-700 rounded px-2 py-1 text-gray-100"
          value="pss"
          onChange={e => handleTypeChange(e.target.value)}
          aria-labelledby="live-type-label"
          title="Select live data type"
          aria-label="Select live data type"
          disabled={false}
        >
          <option value="pss">PSS</option>
          <option value="udp">UDP</option>
          <option value="obs">OBS</option>
        </select>
        <Toggle
          checked={showFullEvents}
          onChange={handleFullEventsToggle}
          label="Full Events"
          labelPosition="right"
          disabled={false}
          className="ml-4"
        />
      </div>

      {/* Connection Status */}
      <div className="flex items-center gap-2 mb-3 text-sm">
        <span className={`w-2 h-2 rounded-full ${wsConnected ? 'bg-green-500' : 'bg-red-500'}`}></span>
        <span className="text-gray-300">
          {wsConnected ? 'Connected' : 'Disconnected'} | 
          Events: {eventCount} | 
          Round: {currentRound} | 
                      Time: {currentRoundTime}
        </span>
      </div>

      {/* Live Data Display */}
      <div 
        ref={liveDataRef}
        className="flex-1 bg-[#0a0f14] border border-gray-700 rounded p-3 overflow-y-auto text-sm font-mono"
        style={{ 
          maxHeight: '400px',
          minHeight: '200px'
        }}
      >
        {events.length === 0 ? (
          <div className="text-gray-500 text-center py-8">
            No PSS events available. 
            {!wsConnected && ' WebSocket not connected.'}
          </div>
        ) : (
          events.map((event, index) => {
            const color = 'text-green-400';
            const emoji = '📡';
            return (
              <div key={event.id} className={`${color} mb-1`}>
                <span className="text-gray-500">[{index + 1}]</span> {emoji} {event.description}
                <div className="text-gray-400 ml-4 text-xs">
                  Round: {event.round} | Time: {event.time} | Type: {event.eventCode}
                </div>
              </div>
            );
          })
        )}
      </div>
      
      {events.length > 0 && (
        <div className="absolute top-2 right-2 flex gap-1">
          <button
            onClick={scrollToTop}
            className="bg-gray-700 hover:bg-gray-600 text-white px-2 py-1 rounded text-xs"
            title="Scroll to top"
          >
            ↑
          </button>
          <button
            onClick={scrollToBottom}
            className="bg-gray-700 hover:bg-gray-600 text-white px-2 py-1 rounded text-xs"
            title="Scroll to bottom"
          >
            ↓
          </button>
          <button
            onClick={() => setAutoScroll(!autoScroll)}
            className={`px-2 py-1 rounded text-xs ${autoScroll ? 'bg-blue-600 text-white' : 'bg-gray-700 text-white hover:bg-gray-600'}`}
            title="Toggle auto-scroll"
          >
            🔄
          </button>
        </div>
      )}
    </div>
  );
};

export default LiveDataPanel; 