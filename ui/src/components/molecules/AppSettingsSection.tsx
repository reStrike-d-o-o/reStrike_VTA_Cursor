import React, { useState } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import { useAppStore } from '../../stores';
import { useSettingsStore } from '../../stores/settingsStore';
import { windowCommands } from '../../utils/tauriCommands';
import { useEnvironment } from '../../hooks/useEnvironment';
import { logger, setLogLevel, LogLevel } from '../../utils/logger';

const AppSettingsSection: React.FC = () => {
  const { tauriAvailable } = useEnvironment();
  const windowSettings = useAppStore((state) => state.windowSettings);
  const updateWindowSettings = useAppStore((state) => state.updateWindowSettings);
  const resetWindowSettings = useAppStore((state) => state.resetWindowSettings);
  const saveWindowSettings = useAppStore((state) => state.saveWindowSettings);

  const [isLoading, setIsLoading] = useState(false);
  const [message, setMessage] = useState('');
  const [logLevel, setLevel] = useState<LogLevel>('info');
  const theme = useSettingsStore((s) => s.theme);
  const setTheme = useSettingsStore((s) => s.setTheme);
  const sharp = useSettingsStore((s)=> (s as any).sharp);
  const setSharp = useSettingsStore((s)=> (s as any).setSharp);

  const handleApplySettings = async () => {
    if (!tauriAvailable) {
      setMessage('Tauri not available - settings saved but not applied');
      return;
    }

    setIsLoading(true);
    setMessage('');

    try {
      // Save settings first
      await saveWindowSettings();
      
      // Apply compact size
      await windowCommands.setCustomSize(windowSettings.compactWidth, windowSettings.compactHeight);
      setMessage('Window settings applied and saved successfully!');
    } catch (error) {
      setMessage(`Error applying settings: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleReset = () => {
    resetWindowSettings();
    setMessage('Settings reset to defaults');
  };

  const applyLogLevel = (lvl: LogLevel) => {
    setLevel(lvl);
    setLogLevel(lvl);
    logger.info('Log level set to', lvl);
  };

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold text-white mb-4">Window Settings</h3>
        <p className="text-gray-300 text-sm mb-4">
          Configure the window dimensions for compact and fullscreen modes.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Compact Mode Settings */}
        <div className="space-y-4">
          <h4 className="text-md font-medium text-white">Compact Mode (Default)</h4>
          <div className="space-y-3">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">
                Width (px)
              </label>
              <Input
                type="number"
                value={windowSettings.compactWidth}
                onChange={(e) => updateWindowSettings({ compactWidth: Number(e.target.value) })}
                min="200"
                max="800"
                className="w-full"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">
                Height (px)
              </label>
              <Input
                type="number"
                value={windowSettings.compactHeight}
                onChange={(e) => updateWindowSettings({ compactHeight: Number(e.target.value) })}
                min="400"
                max="2000"
                className="w-full"
              />
            </div>
          </div>
        </div>

        {/* Fullscreen Mode Settings */}
        <div className="space-y-4">
          <h4 className="text-md font-medium text-white">Fullscreen Mode (Advanced)</h4>
          <div className="space-y-3">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">
                Width (px)
              </label>
              <Input
                type="number"
                value={windowSettings.fullscreenWidth}
                onChange={(e) => updateWindowSettings({ fullscreenWidth: Number(e.target.value) })}
                min="800"
                max="4000"
                className="w-full"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">
                Height (px)
              </label>
              <Input
                type="number"
                value={windowSettings.fullscreenHeight}
                onChange={(e) => updateWindowSettings({ fullscreenHeight: Number(e.target.value) })}
                min="600"
                max="3000"
                className="w-full"
              />
            </div>
          </div>
        </div>
      </div>

      {/* Quick Presets */}
      <div className="space-y-3">
        <h4 className="text-md font-medium text-white">Quick Presets</h4>
        <div className="flex flex-wrap gap-2">
          <Button
            variant="secondary"
            size="sm"
            onClick={() => updateWindowSettings({ compactWidth: 350, compactHeight: 1200 })}
            className="text-xs"
          >
            1920x1200 (Compact)
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => updateWindowSettings({ compactWidth: 350, compactHeight: 1080 })}
            className="text-xs"
          >
            1920x1080 (Compact)
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => updateWindowSettings({ fullscreenWidth: 1920, fullscreenHeight: 1200 })}
            className="text-xs"
          >
            1920x1200 (Full)
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => updateWindowSettings({ fullscreenWidth: 1920, fullscreenHeight: 1080 })}
            className="text-xs"
          >
            1920x1080 (Full)
          </Button>
        </div>
      </div>

      {/* Theme & Log verbosity */}
      <div className="space-y-3">
        <h4 className="text-md font-medium text-white">Appearance</h4>
        <div className="flex flex-wrap items-center gap-3">
          <span className="text-sm text-gray-300">Theme</span>
          <Button size="sm" variant={theme==='dark' ? 'primary' : 'secondary'} onClick={() => setTheme('dark')}>Dark</Button>
          <Button size="sm" variant={theme==='light' ? 'primary' : 'secondary'} onClick={() => setTheme('light')}>Light</Button>
          <span className="text-sm text-gray-300 ml-4">Corners</span>
          <Button size="sm" variant={sharp ? 'primary' : 'secondary'} onClick={() => setSharp(true)}>Square</Button>
          <Button size="sm" variant={!sharp ? 'primary' : 'secondary'} onClick={() => setSharp(false)}>Rounded</Button>
        </div>
        <h4 className="text-md font-medium text-white mt-4">Log verbosity</h4>
        <div className="flex flex-wrap gap-2">
          {(['silent','error','warn','info','debug'] as LogLevel[]).map((lvl) => (
            <Button
              key={lvl}
              size="sm"
              className={`${logLevel===lvl ? 'bg-blue-600' : 'bg-gray-600 hover:bg-gray-700'}`}
              onClick={() => applyLogLevel(lvl)}
            >
              {lvl}
            </Button>
          ))}
        </div>
        <p className="text-xs text-gray-400">Lower levels reduce console noise in production.</p>
      </div>

      {/* Actions */}
      <div className="flex gap-3 pt-4 border-t border-gray-600">
        <Button
          variant="primary"
          onClick={handleApplySettings}
          disabled={isLoading}
          className="flex-1"
        >
          {isLoading ? 'Applying...' : 'Apply Settings'}
        </Button>
        <Button
          variant="secondary"
          onClick={handleReset}
          className="px-4"
        >
          Reset
        </Button>
      </div>

      {/* Message */}
      {message && (
        <div className={`p-3 rounded-lg text-sm ${
          message.includes('Error') 
            ? 'bg-red-500/20 border border-red-500/30 text-red-300' 
            : 'bg-green-500/20 border border-green-500/30 text-green-300'
        }`}>
          {message}
        </div>
      )}

      {/* Info */}
      <div className="bg-blue-500/10 border border-blue-500/20 rounded-lg p-3">
        <p className="text-blue-300 text-sm">
          <strong>Note:</strong> Compact mode is used when the app starts and when Advanced mode is disabled. 
          Fullscreen mode is used when Advanced mode is enabled.
        </p>
      </div>
    </div>
  );
};

export default AppSettingsSection; 