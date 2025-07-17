import React, { useState } from 'react';
import Label from '../atoms/Label';
import Checkbox from '../atoms/Checkbox';

type LogType = 'pss' | 'obs' | 'udp';

const logTypes: { key: LogType; label: string }[] = [
  { key: 'pss', label: 'PSS' },
  { key: 'obs', label: 'OBS' },
  { key: 'udp', label: 'UDP' },
];

const LogToggleGroup: React.FC = () => {
  const [logging, setLogging] = useState<Record<LogType, boolean>>({ pss: true, obs: false, udp: true });

  const handleToggle = (key: LogType) => {
    setLogging(prev => ({ ...prev, [key]: !prev[key] }));
  };

  return (
    <div className="bg-[#181F26] rounded-lg p-4 mb-6 border border-gray-700 shadow">
      <h3 className="text-lg font-semibold mb-2 text-blue-300">Logging</h3>
      <div className="flex flex-col gap-3">
        {logTypes.map(type => (
          <label key={type.key} className="flex items-center gap-3 cursor-pointer">
            <Checkbox checked={logging[type.key]} onChange={() => handleToggle(type.key)} />
            <span className="text-gray-200 font-medium">{type.label}</span>
          </label>
        ))}
      </div>
    </div>
  );
};

export default LogToggleGroup; 