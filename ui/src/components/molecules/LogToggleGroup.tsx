import React, { useState } from 'react';
import Label from '../atoms/Label';
import Checkbox from '../atoms/Checkbox';
import { diagLogsCommands } from '../../utils/tauriCommands';

type LogType = 'pss' | 'obs' | 'udp';

const logTypes: { key: LogType; label: string }[] = [
  { key: 'pss', label: 'PSS' },
  { key: 'obs', label: 'OBS' },
  { key: 'udp', label: 'UDP' },
];

const LogToggleGroup: React.FC = () => {
  const [logging, setLogging] = useState<Record<LogType, boolean>>({ pss: true, obs: false, udp: true });
  const [loading, setLoading] = useState<Record<LogType, boolean>>({ pss: false, obs: false, udp: false });
  const [errors, setErrors] = useState<Record<LogType, string>>({ pss: '', obs: '', udp: '' });

  const handleToggle = async (key: LogType) => {
    const newValue = !logging[key];
    setLoading(prev => ({ ...prev, [key]: true }));
    setErrors(prev => ({ ...prev, [key]: '' }));
    setLogging(prev => ({ ...prev, [key]: newValue })); // Optimistic update
    
    try {
      const result = await diagLogsCommands.setLoggingEnabled(key, newValue);
      if (!result.success) {
        // Revert on error
        setLogging(prev => ({ ...prev, [key]: !newValue }));
        setErrors(prev => ({ ...prev, [key]: result.error || 'Failed to update logging' }));
      }
    } catch (error) {
      // Revert on exception
      setLogging(prev => ({ ...prev, [key]: !newValue }));
      setErrors(prev => ({ ...prev, [key]: `Error: ${error}` }));
    } finally {
      setLoading(prev => ({ ...prev, [key]: false }));
    }
  };

  return (
    <div className="bg-[#181F26] rounded-lg p-4 mb-6 border border-gray-700 shadow">
      <h3 className="text-lg font-semibold mb-2 text-blue-300">Logging</h3>
      <div className="flex flex-col gap-3">
        {logTypes.map(type => (
          <div key={type.key} className="flex flex-col gap-1">
            <label className="flex items-center gap-3 cursor-pointer">
              <Checkbox 
                checked={logging[type.key]} 
                onChange={() => handleToggle(type.key)}
                disabled={loading[type.key]}
              />
              <span className="text-gray-200 font-medium">{type.label}</span>
              {loading[type.key] && (
                <span className="text-blue-400 text-sm">Updating...</span>
              )}
            </label>
            {errors[type.key] && (
              <span className="text-red-400 text-sm ml-6">{errors[type.key]}</span>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

export default LogToggleGroup; 