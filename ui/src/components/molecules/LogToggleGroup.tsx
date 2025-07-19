import React, { useState, useEffect, useCallback } from 'react';
import Label from '../atoms/Label';
import Checkbox from '../atoms/Checkbox';
import { diagLogsCommands, configCommands } from '../../utils/tauriCommands';

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
  const [isLoadingSettings, setIsLoadingSettings] = useState(true);
  const [isUpdating, setIsUpdating] = useState(false);

  // Load logging settings from configuration
  const loadLoggingSettings = async () => {
    try {
      setIsLoadingSettings(true);
      const result = await configCommands.getSettings();
      if (result.success && result.data?.logging?.subsystems) {
        const subsystems = result.data.logging.subsystems;
        setLogging({
          pss: subsystems.pss?.enabled ?? true,
          obs: subsystems.obs?.enabled ?? false,
          udp: subsystems.udp?.enabled ?? true,
        });
      }
    } catch (error) {
      console.error('Failed to load logging settings:', error);
      // Set default values if loading fails
      setLogging({ pss: true, obs: false, udp: true });
    } finally {
      setIsLoadingSettings(false);
    }
  };

  // Save logging settings to configuration with retry logic
  const saveLoggingSettings = async (newSettings: Record<LogType, boolean>, retryCount = 0): Promise<boolean> => {
    const maxRetries = 3;
    
    try {
      const result = await configCommands.getSettings();
      if (result.success) {
        const updatedSettings = {
          ...result.data,
          logging: {
            ...result.data.logging,
            subsystems: {
              ...result.data.logging.subsystems,
              pss: { name: 'pss', level: 'info', enabled: newSettings.pss, custom_file: null },
              obs: { name: 'obs', level: 'info', enabled: newSettings.obs, custom_file: null },
              udp: { name: 'udp', level: 'info', enabled: newSettings.udp, custom_file: null },
            },
          },
        };
        
        await configCommands.updateSettings(updatedSettings);
        console.log('Logging settings saved successfully');
        return true;
      } else {
        throw new Error(result.error || 'Failed to get settings');
      }
    } catch (error) {
      console.error(`Failed to save logging settings (attempt ${retryCount + 1}):`, error);
      
      if (retryCount < maxRetries) {
        // Wait before retrying (exponential backoff)
        await new Promise(resolve => setTimeout(resolve, Math.pow(2, retryCount) * 1000));
        return saveLoggingSettings(newSettings, retryCount + 1);
      }
      
      return false;
    }
  };

  // Debounced update function to prevent rapid changes
  const debouncedUpdate = useCallback(
    (() => {
      let timeoutId: NodeJS.Timeout;
      return (key: LogType, newValue: boolean) => {
        clearTimeout(timeoutId);
        timeoutId = setTimeout(() => {
          handleToggle(key, newValue);
        }, 300); // 300ms debounce
      };
    })(),
    []
  );

  // Load settings on component mount
  useEffect(() => {
    loadLoggingSettings();
  }, []);

  const handleToggle = async (key: LogType, forceValue?: boolean) => {
    // Prevent multiple simultaneous updates
    if (isUpdating || loading[key]) {
      console.log(`Update already in progress for ${key}, skipping`);
      return;
    }

    const newValue = forceValue !== undefined ? forceValue : !logging[key];
    
    // Optimistic update
    setLoading(prev => ({ ...prev, [key]: true }));
    setErrors(prev => ({ ...prev, [key]: '' }));
    setLogging(prev => ({ ...prev, [key]: newValue }));
    setIsUpdating(true);
    
    try {
      // First, update backend logging state
      console.log(`Updating backend logging for ${key} to ${newValue}`);
      const result = await diagLogsCommands.setLoggingEnabled(key, newValue);
      
      if (!result.success) {
        // Revert on error
        console.error(`Backend logging update failed for ${key}:`, result.error);
        setLogging(prev => ({ ...prev, [key]: !newValue }));
        setErrors(prev => ({ ...prev, [key]: result.error || 'Failed to update logging' }));
        return;
      }

      // Then, save to configuration system
      console.log(`Saving configuration for ${key}`);
      const newSettings = { ...logging, [key]: newValue };
      const configSaved = await saveLoggingSettings(newSettings);
      
      if (!configSaved) {
        console.error(`Configuration save failed for ${key}`);
        setErrors(prev => ({ ...prev, [key]: 'Failed to save configuration' }));
        // Don't revert the backend change since it succeeded
      } else {
        console.log(`Successfully updated logging for ${key}`);
      }
    } catch (error) {
      console.error(`Unexpected error updating logging for ${key}:`, error);
      // Revert on exception
      setLogging(prev => ({ ...prev, [key]: !newValue }));
      setErrors(prev => ({ ...prev, [key]: `Error: ${error}` }));
    } finally {
      setLoading(prev => ({ ...prev, [key]: false }));
      setIsUpdating(false);
    }
  };

  const handleToggleClick = (key: LogType) => {
    // Use debounced update to prevent rapid changes
    debouncedUpdate(key, !logging[key]);
  };

  if (isLoadingSettings) {
    return (
      <div className="bg-[#181F26] rounded-lg p-4 mb-6 border border-gray-700 shadow">
        <h3 className="text-lg font-semibold mb-2 text-blue-300">Logging</h3>
        <div className="text-sm text-gray-400">Loading settings...</div>
      </div>
    );
  }

  return (
    <div className="bg-[#181F26] rounded-lg p-4 mb-6 border border-gray-700 shadow">
      <h3 className="text-lg font-semibold mb-2 text-blue-300">Logging</h3>
      <div className="flex flex-col gap-3">
        {logTypes.map(type => (
          <div key={type.key} className="flex flex-col gap-1">
            <label className="flex items-center gap-3 cursor-pointer">
              <Checkbox 
                checked={logging[type.key]} 
                onChange={() => handleToggleClick(type.key)}
                disabled={loading[type.key] || isUpdating}
              />
              <span className="text-gray-200 font-medium">{type.label}</span>
              {loading[type.key] && (
                <span className="text-blue-400 text-sm">Updating...</span>
              )}
              {isUpdating && !loading[type.key] && (
                <span className="text-yellow-400 text-sm">Processing...</span>
              )}
            </label>
            {errors[type.key] && (
              <span className="text-red-400 text-sm ml-6">{errors[type.key]}</span>
            )}
          </div>
        ))}
      </div>
      {isUpdating && (
        <div className="mt-3 text-xs text-gray-400">
          Please wait while settings are being updated...
        </div>
      )}
    </div>
  );
};

export default LogToggleGroup; 