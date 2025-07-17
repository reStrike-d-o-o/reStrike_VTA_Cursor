import React, { useState, useRef, useEffect } from 'react';

const logTypes = [
  { key: 'pss', label: 'PSS' },
  { key: 'obs', label: 'OBS' },
  { key: 'udp', label: 'UDP' },
];

type LogType = 'pss' | 'obs' | 'udp';

const dummyLiveData: Record<LogType, string> = {
  pss: 'PSS: [12:01:01] Event: HIT | Player: RED | Power: 32\nPSS: [12:01:03] Event: PENALTY | Player: BLUE | Reason: Out of bounds',
  obs: 'OBS: [12:01:05] Scene changed to Replay\nOBS: [12:01:07] Recording started',
  udp: 'UDP: [12:01:10] Datagram received | Length: 128 bytes',
};

const LiveDataPanel: React.FC = () => {
  const [enabled, setEnabled] = useState(true);
  const [selectedType, setSelectedType] = useState<LogType>('pss');
  const liveDataRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (liveDataRef.current) {
      liveDataRef.current.scrollTop = liveDataRef.current.scrollHeight;
    }
  }, [enabled, selectedType]);

  return (
    <div className="bg-[#181F26] rounded-lg p-4 border border-gray-700 shadow">
      <h3 className="text-lg font-semibold mb-2 text-blue-300">LIVE DATA</h3>
      <div className="flex items-center gap-3 mb-3">
        <label className="flex items-center gap-2 cursor-pointer">
          <input type="checkbox" checked={enabled} onChange={() => setEnabled(e => !e)} className="accent-blue-500" />
          <span className="text-gray-200 font-medium">Enable</span>
        </label>
        <span className="text-gray-200 font-medium" id="live-type-label">Type:</span>
        <select
          className="bg-[#101820] border border-gray-700 rounded px-2 py-1 text-gray-100"
          value={selectedType}
          onChange={e => setSelectedType(e.target.value as LogType)}
          aria-labelledby="live-type-label"
          title="Select live data type"
          aria-label="Select live data type"
        >
          {logTypes.map(type => (
            <option key={type.key} value={type.key}>{type.label}</option>
          ))}
        </select>
      </div>
      <div
        ref={liveDataRef}
        className="bg-[#101820] rounded p-3 min-h-[100px] max-h-48 overflow-y-auto text-sm text-green-200 font-mono whitespace-pre-line border border-gray-800"
      >
        {enabled ? dummyLiveData[selectedType] : <span className="text-gray-500">Live data is disabled.</span>}
      </div>
    </div>
  );
};

export default LiveDataPanel; 