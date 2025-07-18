import React, { useState, useRef, useEffect } from 'react';
import { diagLogsCommands } from '../../utils/tauriCommands';
import { useEnvironment } from '../../hooks/useEnvironment';

const logTypes = [
  { key: 'pss', label: 'PSS' },
  { key: 'obs', label: 'OBS' },
  { key: 'udp', label: 'UDP' },
];

type LogType = 'pss' | 'obs' | 'udp';

declare global {
  interface Window {
    __TAURI__?: any;
  }
}

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
    windowTauriEvent: typeof window !== 'undefined' && window.__TAURI__ ? !!window.__TAURI__.event : 'undefined',
    windowLocation: typeof window !== 'undefined' ? window.location.href : 'undefined'
  });

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
      windowTauriEvent: typeof window !== 'undefined' && window.__TAURI__ ? !!window.__TAURI__.event : 'undefined',
      selectedType,
      enabled
    });

    const setupStreaming = async () => {
      try {
        if (tauriAvailable && window.__TAURI__ && window.__TAURI__.event) {
          const result = await diagLogsCommands.setLiveDataStreaming(selectedType, enabled);
          if (!result.success) {
            setError(`Failed to ${enabled ? 'start' : 'stop'} streaming: ${result.error}`);
            return;
          }

          setLiveData('');
          
          const listener = await window.__TAURI__.event.listen('live_data', (event: any) => {
            if (event && event.payload && event.payload.subsystem === selectedType) {
              setLiveData(prev => prev + (prev ? '\n' : '') + event.payload.data);
            }
          });
          
          unlisten = listener;
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
      if (tauriAvailable && window.__TAURI__ && window.__TAURI__.event) {
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
        <div className="text-pink-400">window.__TAURI__.invoke: {typeof window !== 'undefined' && window.__TAURI__ && !!window.__TAURI__.invoke ? 'true' : 'false'}</div>
        <button 
          onClick={() => {
            console.log('🔍 Manual Tauri Test:', {
              window: typeof window !== 'undefined',
              tauri: typeof window !== 'undefined' ? !!window.__TAURI__ : false,
              invoke: typeof window !== 'undefined' && window.__TAURI__ ? !!window.__TAURI__.invoke : false,
              event: typeof window !== 'undefined' && window.__TAURI__ ? !!window.__TAURI__.event : false,
              location: typeof window !== 'undefined' ? window.location.href : 'undefined'
            });
            if (typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.invoke) {
              window.__TAURI__.invoke('system_get_info').then((result: any) => {
                console.log('✅ Tauri command test successful:', result);
                alert('Tauri is working! Check console for details.');
              }).catch((error: any) => {
                console.error('❌ Tauri command test failed:', error);
                alert('Tauri command failed! Check console for details.');
              });
            } else {
              alert('Tauri not available! Check console for details.');
            }
          }}
          className="mt-2 px-2 py-1 bg-blue-600 text-white text-xs rounded hover:bg-blue-700"
        >
          Test Tauri
        </button>
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