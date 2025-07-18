import React, { useState, useRef, useEffect } from 'react';
import { diagLogsCommands } from '../../utils/tauriCommands';

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

/**
 * Check if Tauri is available (same logic as environment detection)
 */
const isTauriAvailable = (): boolean => {
  const tauriAvailable = typeof window !== 'undefined' && !!window.__TAURI__;
  const isTauriContext = tauriAvailable || 
    (typeof window !== 'undefined' && window.location.protocol === 'tauri:') ||
    (typeof window !== 'undefined' && window.location.hostname === 'localhost' && window.location.port === '3000');
  
  return isTauriContext;
};

const LiveDataPanel: React.FC = () => {
  const [enabled, setEnabled] = useState(true);
  const [selectedType, setSelectedType] = useState<LogType>('pss');
  const [liveData, setLiveData] = useState<string>('');
  const [error, setError] = useState<string>('');
  const [connecting, setConnecting] = useState(false);
  const liveDataRef = useRef<HTMLDivElement>(null);

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

    const setupStreaming = async () => {
      try {
        if (isTauriAvailable() && window.__TAURI__ && window.__TAURI__.event) {
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
      if (isTauriAvailable() && window.__TAURI__ && window.__TAURI__.event) {
        diagLogsCommands.setLiveDataStreaming(selectedType, false).catch(err => {
          console.error('Error stopping streaming:', err);
        });
      }
    };
  }, [enabled, selectedType]);

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