import React, { useState, useEffect } from 'react';
import SettingsDrawerTabs from '../molecules/SettingsDrawerTabs';
import LogDownloadList from '../molecules/LogDownloadList';
import LiveDataPanel from '../molecules/LiveDataPanel';
import WebSocketManager from '../molecules/WebSocketManager';
import { CpuMonitoringSection } from '../molecules/CpuMonitoringSection';
import PssDrawer from '../molecules/PssDrawer';
import Toggle from '../atoms/Toggle';
import { useAppStore } from '../../stores';
import { configCommands } from '../../utils/tauriCommands';

type AdvancedPanelProps = React.ComponentProps<'div'>;

// OBS Integration Settings interface
interface ObsIntegrationSettings {
  autoConnectOnStartup: boolean;
  showStatusInOverlay: boolean;
  autoRecordOnClipPlay: boolean;
  saveReplayBufferOnClipCreation: boolean;
}

const DRAWERS = [
  {
    key: 'pss',
    label: 'PSS',
    icon: (
      <svg width="24" height="24" fill="none" viewBox="0 0 24 24">
        <rect x="6" y="3" width="12" height="6" rx="2" stroke="currentColor" strokeWidth="2"/>
        <path d="M6 9l6 12 6-12" stroke="currentColor" strokeWidth="2"/>
        <path d="M9 9v3l3 3 3-3V9" stroke="currentColor" strokeWidth="2"/>
      </svg>
    ),
    description: 'UDP server, PSS protocol, event database, etc.'
  },
  {
    key: 'obs',
    label: 'OBS',
    icon: (
      <svg width="24" height="24" fill="none" viewBox="0 0 24 24">
        <rect x="3" y="7" width="15" height="10" rx="2" stroke="currentColor" strokeWidth="2"/>
        <rect x="16" y="10" width="5" height="4" rx="1" stroke="currentColor" strokeWidth="2"/>
        <circle cx="10.5" cy="12" r="2.5" stroke="currentColor" strokeWidth="2"/>
      </svg>
    ),
    description: 'OBS connection management and options.'
  },
  {
    key: 'video',
    label: 'Video',
    icon: (
      <svg width="24" height="24" fill="none" viewBox="0 0 24 24"><rect x="3" y="5" width="18" height="14" rx="2" stroke="currentColor" strokeWidth="2"/><polygon points="10,9 16,12 10,15" fill="currentColor"/></svg>
    ),
    description: 'mpv video integration and controls.'
  },
  {
    key: 'ai',
    label: 'AI',
    icon: (
      <svg width="24" height="24" fill="none" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="2"/><path d="M8 12a4 4 0 1 1 8 0 4 4 0 0 1-8 0z" stroke="currentColor" strokeWidth="2"/></svg>
    ),
    description: 'AI report creation and data analyzer.'
  },
  {
    key: 'settings',
    label: 'Settings',
    icon: (
      <svg width="24" height="24" fill="none" viewBox="0 0 24 24"><circle cx="12" cy="12" r="3" stroke="currentColor" strokeWidth="2"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06-.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h.09A1.65 1.65 0 0 0 9 3.09V3a2 2 0 1 1 4 0v.09c0 .66.39 1.26 1 1.51h.09a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.09c.66 0 1.26.39 1.51 1H21a2 2 0 1 1 0 4h-.09c-.66 0-1.26.39-1.51 1z" stroke="currentColor" strokeWidth="2"/></svg>
    ),
    description: 'All settings, including Diagnostics & Logs Manager.'
  },
];

const AdvancedPanel: React.FC<AdvancedPanelProps> = ({ className = '', ...rest }) => {
  const activeDrawer = useAppStore((state) => state.activeDrawer);
  const setActiveDrawer = useAppStore((state) => state.setActiveDrawer);
  // Settings horizontal drawer state
  const [settingsTab, setSettingsTab] = useState('diagnostics');
  // OBS horizontal drawer state
  const [obsTab, setObsTab] = useState('websocket');
  
  // OBS Integration Settings state
  const [obsIntegrationSettings, setObsIntegrationSettings] = useState<ObsIntegrationSettings>({
    autoConnectOnStartup: true,
    showStatusInOverlay: true,
    autoRecordOnClipPlay: false,
    saveReplayBufferOnClipCreation: true,
  });
  
  const [isLoadingSettings, setIsLoadingSettings] = useState(false);
  
  const drawer = DRAWERS.find(d => d.key === activeDrawer);

  // Load OBS Integration settings from configuration
  const loadObsIntegrationSettings = async () => {
    try {
      setIsLoadingSettings(true);
      const result = await configCommands.getSettings();
      if (result.success && result.data?.obs?.integration) {
        setObsIntegrationSettings({
          autoConnectOnStartup: result.data.obs.integration.auto_connect_on_startup ?? true,
          showStatusInOverlay: result.data.obs.integration.show_status_in_overlay ?? true,
          autoRecordOnClipPlay: result.data.obs.integration.auto_record_on_clip_play ?? false,
          saveReplayBufferOnClipCreation: result.data.obs.integration.save_replay_buffer_on_clip_creation ?? true,
        });
      }
    } catch (error) {
      console.error('Failed to load OBS integration settings:', error);
    } finally {
      setIsLoadingSettings(false);
    }
  };

  // Save OBS Integration settings to configuration
  const saveObsIntegrationSettings = async (newSettings: ObsIntegrationSettings) => {
    try {
      const result = await configCommands.getSettings();
      if (result.success) {
        const updatedSettings = {
          ...result.data,
          obs: {
            ...result.data.obs,
            integration: {
              auto_connect_on_startup: newSettings.autoConnectOnStartup,
              show_status_in_overlay: newSettings.showStatusInOverlay,
              auto_record_on_clip_play: newSettings.autoRecordOnClipPlay,
              save_replay_buffer_on_clip_creation: newSettings.saveReplayBufferOnClipCreation,
            },
          },
        };
        await configCommands.updateSettings(updatedSettings);
        console.log('OBS integration settings saved successfully');
      }
    } catch (error) {
      console.error('Failed to save OBS integration settings:', error);
    }
  };

  // Handle checkbox change
  const handleObsSettingChange = async (setting: keyof ObsIntegrationSettings, value: boolean) => {
    const newSettings = {
      ...obsIntegrationSettings,
      [setting]: value,
    };
    setObsIntegrationSettings(newSettings);
    await saveObsIntegrationSettings(newSettings);
  };

  // Load settings when OBS tab is opened
  useEffect(() => {
    if (activeDrawer === 'obs' && obsTab === 'integration') {
      loadObsIntegrationSettings();
    }
  }, [activeDrawer, obsTab]);

  return (
    <div className={`flex h-full min-h-0 min-w-[320px] max-w-[100%] bg-gradient-to-br from-gray-900/95 to-gray-800/90 backdrop-blur-sm shadow-xl overflow-hidden z-20 ${className}`} {...rest}>
      {/* Drawer Sidebar */}
      <nav className="h-full min-h-0 flex flex-col justify-center items-center py-0 px-3 bg-gradient-to-b from-gray-800/90 to-gray-900/95 backdrop-blur-sm border-r border-gray-600/30 w-28">
        <div className="flex flex-col items-center space-y-4">
          {DRAWERS.map(d => (
            <button
              key={d.key}
              className={`flex flex-col items-center justify-center w-20 h-20 transition-all duration-300 rounded-lg group relative ${
                activeDrawer === d.key 
                  ? 'bg-gradient-to-br from-blue-600 to-blue-700 text-white shadow-lg shadow-blue-500/25' 
                  : 'text-gray-400 hover:bg-gray-700/50 hover:text-blue-300 hover:shadow-md'
              }`}
              onClick={() => setActiveDrawer(d.key)}
              aria-label={d.label}
            >
              <div className={`absolute inset-0 rounded-lg transition-all duration-300 ${
                activeDrawer === d.key 
                  ? 'bg-blue-500/20 blur-sm' 
                  : 'bg-transparent group-hover:bg-blue-500/10 blur-sm'
              }`}></div>
              <span className="mb-1 relative z-10">{d.icon}</span>
              <span className="text-xs font-semibold relative z-10">{d.label}</span>
            </button>
          ))}
        </div>
      </nav>
      {/* Drawer Content */}
      <main className="flex-1 flex flex-col p-8 overflow-y-auto min-h-0 bg-gradient-to-br from-gray-800/60 to-gray-900/80 backdrop-blur-sm overflow-hidden">
        {/* Placeholder for each drawer's content */}
        {drawer?.key === 'pss' && (
          <PssDrawer />
        )}
        {drawer?.key === 'obs' && (
          <>
            {/* OBS Tab Navigation */}
            <div className="flex space-x-1 mb-6 bg-gradient-to-r from-gray-800/80 to-gray-700/80 backdrop-blur-sm p-1 rounded-lg border border-gray-600/30 shadow-md">
              <button
                onClick={() => setObsTab('websocket')}
                className={`px-4 py-2 rounded-md text-sm font-medium transition-all duration-300 relative group ${
                  obsTab === 'websocket' 
                    ? 'bg-gradient-to-r from-blue-600 to-blue-700 text-white shadow-lg shadow-blue-500/25' 
                    : 'text-gray-300 hover:bg-gray-700/80 hover:text-blue-300 hover:shadow-md'
                }`}
              >
                <div className={`absolute inset-0 rounded-md transition-all duration-300 ${
                  obsTab === 'websocket' 
                    ? 'bg-blue-500/20 blur-sm' 
                    : 'bg-transparent group-hover:bg-blue-500/10 blur-sm'
                }`}></div>
                <span className="relative z-10 flex items-center space-x-2">
                  <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.141 0M1.394 9.393c5.857-5.857 15.355-5.857 21.213 0" />
                  </svg>
                  <span>WebSocket</span>
                </span>
              </button>
              <button
                onClick={() => setObsTab('integration')}
                className={`px-4 py-2 rounded-md text-sm font-medium transition-all duration-300 relative group ${
                  obsTab === 'integration' 
                    ? 'bg-gradient-to-r from-blue-600 to-blue-700 text-white shadow-lg shadow-blue-500/25' 
                    : 'text-gray-300 hover:bg-gray-700/80 hover:text-blue-300 hover:shadow-md'
                }`}
              >
                <div className={`absolute inset-0 rounded-md transition-all duration-300 ${
                  obsTab === 'integration' 
                    ? 'bg-blue-500/20 blur-sm' 
                    : 'bg-transparent group-hover:bg-blue-500/10 blur-sm'
                }`}></div>
                <span className="relative z-10 flex items-center space-x-2">
                  <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <circle cx="12" cy="12" r="3" stroke="currentColor" strokeWidth="2"/>
                    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06-.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h.09A1.65 1.65 0 0 0 9 3.09V3a2 2 0 1 1 4 0v.09c0 .66.39 1.26 1 1.51h.09a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.09c.66 0 1.26.39 1.51 1H21a2 2 0 1 1 0 4h-.09c-.66 0-1.26.39-1.51 1z" stroke="currentColor" strokeWidth="2"/>
                  </svg>
                  <span>Integration</span>
                </span>
              </button>
            </div>

            {/* OBS Tab Content */}
            {obsTab === 'websocket' && (
              <WebSocketManager />
            )}
            
            {obsTab === 'integration' && (
              <div className="space-y-6">
                <div className="p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
                  <h3 className="text-lg font-semibold mb-4 text-gray-100">OBS Integration Settings</h3>
                  {isLoadingSettings ? (
                    <div className="text-sm text-gray-400">Loading settings...</div>
                  ) : (
                    <div className="space-y-4">
                      <Toggle
                        id="obs-auto-connect"
                        checked={obsIntegrationSettings.autoConnectOnStartup}
                        onChange={(e) => handleObsSettingChange('autoConnectOnStartup', e.target.checked)}
                        label="Auto-connect to OBS on startup"
                        labelPosition="right"
                      />
                      <Toggle
                        id="obs-show-status"
                        checked={obsIntegrationSettings.showStatusInOverlay}
                        onChange={(e) => handleObsSettingChange('showStatusInOverlay', e.target.checked)}
                        label="Show OBS status in overlay"
                        labelPosition="right"
                      />
                      <Toggle
                        id="obs-auto-record"
                        checked={obsIntegrationSettings.autoRecordOnClipPlay}
                        onChange={(e) => handleObsSettingChange('autoRecordOnClipPlay', e.target.checked)}
                        label="Auto-record when playing clips"
                        labelPosition="right"
                      />
                      <Toggle
                        id="obs-save-replay"
                        checked={obsIntegrationSettings.saveReplayBufferOnClipCreation}
                        onChange={(e) => handleObsSettingChange('saveReplayBufferOnClipCreation', e.target.checked)}
                        label="Save replay buffer on clip creation"
                        labelPosition="right"
                      />
                    </div>
                  )}
                </div>
              </div>
            )}
          </>
        )}
        {drawer?.key === 'video' && (
          <div className="bg-gradient-to-br from-green-900/20 to-green-800/30 backdrop-blur-sm rounded-lg p-6 text-gray-200 border border-green-600/30 shadow-lg">[mpv video integration and controls will be implemented here]</div>
        )}
        {drawer?.key === 'ai' && (
          <div className="bg-gradient-to-br from-purple-900/20 to-purple-800/30 backdrop-blur-sm rounded-lg p-6 text-gray-200 border border-purple-600/30 shadow-lg">[AI report creation and data analyzer will be implemented here]</div>
        )}
        {drawer?.key === 'settings' && (
          <>
            <SettingsDrawerTabs activeTab={settingsTab} onTabChange={setSettingsTab} />
            {settingsTab === 'diagnostics' && (
              <div className="flex flex-col gap-6">
                <div className="flex flex-row gap-6">
                  <div className="flex-[2] min-w-[320px]">
                    <LogDownloadList />
                  </div>
                </div>
                <div className="space-y-6">
                <LiveDataPanel />
                  <CpuMonitoringSection />
                </div>
              </div>
            )}
            {/* Future settings tabs can be added here */}
          </>
        )}
      </main>
    </div>
  );
};

export default AdvancedPanel; 