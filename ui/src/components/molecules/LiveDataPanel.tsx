import React, { useState, useRef, useEffect } from 'react';
import { diagLogsCommands } from '../../utils/tauriCommands';
import { useEnvironment } from '../../hooks/useEnvironment';

// Tauri v2 event listening
const { listen } = window.__TAURI__?.event || {};

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

type LogType = 'pss' | 'obs' | 'udp';

const LiveDataPanel: React.FC = () => {
  const { tauriAvailable, environment, isWindows, isWeb } = useEnvironment();
  const [enabled, setEnabled] = useState(true);
  const [selectedType, setSelectedType] = useState<LogType>('pss');
  const [liveData, setLiveData] = useState<string>('');
  const [error, setError] = useState<string>('');
  const [connecting, setConnecting] = useState(false);
  const liveDataRef = useRef<HTMLDivElement>(null);

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
  }, [liveData]);

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

          setLiveData('');
          
          // Use the proper Tauri v2 event listening API
          if (listen && typeof listen === 'function') {
            const listener = await listen('live_data', (event: any) => {
              if (event && event.payload && event.payload.subsystem === selectedType) {
                setLiveData(prev => prev + (prev ? '\n' : '') + event.payload.data);
              }
            });
            
            unlisten = listener;
          } else {
            setError('Event listening not available - Tauri event API not found');
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
        diagLogsCommands.setLiveDataStreaming(selectedType, false).catch(err => {
          console.error('Error stopping streaming:', err);
        });
      }
    };
  }, [enabled, selectedType, tauriAvailable]);

  const handleToggle = async () => {
    setError('');
    setEnabled(prev => !prev);
  };

  const handleTypeChange = (newType: LogType) => {
    setError('');
    setSelectedType(newType);
  };

  return (
    <div className="bg-[#181F26] rounded-lg p-4 border border-gray-700 shadow">
      <h3 className="text-lg font-semibold mb-2 text-blue-300">LIVE DATA</h3>
      
      {/* Environment Status Debug */}
      <div className="mb-3 p-2 bg-gray-800 rounded text-xs">
        <div className="text-gray-300">Environment Debug:</div>
        <div className="text-green-400">tauriAvailable: {tauriAvailable ? 'true' : 'false'}</div>
        <div className="text-blue-400">environment: {environment}</div>
        <div className="text-yellow-400">isWindows: {isWindows ? 'true' : 'false'}</div>
        <div className="text-orange-400">isWeb: {isWeb ? 'true' : 'false'}</div>
        <div className="text-purple-400">window.__TAURI__: {typeof window !== 'undefined' && !!window.__TAURI__ ? 'true' : 'false'}</div>
      </div>
      
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
          onChange={e => handleTypeChange(e.target.value as LogType)}
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
        className="bg-[#101820] rounded p-3 min-h-[100px] max-h-48 overflow-y-auto text-sm text-green-200 font-mono whitespace-pre-line border border-gray-800"
      >
        {enabled ? (
          connecting ? (
            <span className="text-blue-400">Connecting to live data stream...</span>
          ) : liveData ? (
            liveData
          ) : (
            <span className="text-gray-500">Waiting for live data...</span>
          )
        ) : (
          <span className="text-gray-500">Live data is disabled.</span>
        )}
      </div>
    </div>
  );
};

export default LiveDataPanel; 