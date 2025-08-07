import React, { useState, useEffect } from 'react';
import SettingsDrawerTabs from '../molecules/SettingsDrawerTabs';
import LogDownloadList from '../molecules/LogDownloadList';
import LogArchiveManager from '../molecules/LogArchiveManager';
import LiveDataPanel from '../molecules/LiveDataPanel';
import WebSocketManager from '../molecules/WebSocketManager';
import ControlRoom from '../molecules/ControlRoom';
import { CpuMonitoringSection } from '../molecules/CpuMonitoringSection';
import PssDrawer from '../molecules/PssDrawer';
import AppSettingsSection from '../molecules/AppSettingsSection';
import { DatabaseManagementPanel } from '../molecules/DatabaseManagementPanel';
import { GoogleDriveBackupRestore } from '../molecules/GoogleDriveBackupRestore';
import DatabaseMigrationPanel from '../molecules/DatabaseMigrationPanel';
import FlagManagementPanel from '../molecules/FlagManagementPanel';
import TournamentManagementPanel from '../molecules/TournamentManagementPanel';
import ObsWebSocketManager from '../organisms/ObsWebSocketManager';
import Toggle from '../atoms/Toggle';
import TabGroup from '../molecules/TabGroup';
import TabIcons from '../atoms/TabIcons';
import LottieIcon from '../atoms/LottieIcon';
import { useAppStore } from '../../stores';
import { configCommands } from '../../utils/tauriCommands';
import { flowChartAnimation, spyAnimation, plansAnimation, watcherAnimation, taekwondoAnimation, liveStreamingAnimation, settingsAnimation, robotAnimation, noConnectionAnimation, businessAnimation, tournamentAnimation, mixerAnimation } from '../../assets/icons/json';

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
    icon: <LottieIcon animationData={taekwondoAnimation} size={48} />,
    description: 'UDP server, PSS protocol, event database, etc.'
  },
  {
    key: 'obs',
    label: 'OBS',
    icon: <LottieIcon animationData={liveStreamingAnimation} size={48} />,
    description: 'OBS connection management and options.'
  },
  {
    key: 'ovr',
    label: 'OVR',
    icon: <LottieIcon animationData={plansAnimation} size={48} />,
    description: 'Overlay integration and tournament management.'
  },
  {
    key: 'ivr',
    label: 'IVR',
    icon: <LottieIcon animationData={watcherAnimation} size={48} />,
    description: 'Instant Video Replay integration and controls.'
  },
  {
    key: 'ai',
    label: 'AI',
    icon: <LottieIcon animationData={robotAnimation} size={48} />,
    description: 'AI report creation and data analyzer.'
  },
  {
    key: 'settings',
    label: 'Settings',
    icon: <LottieIcon animationData={settingsAnimation} size={48} />,
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
  // OVR horizontal drawer state
  const [ovrTab, setOvrTab] = useState('integration');
  
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
          <TabGroup
            tabs={[
              {
                id: 'websocket',
                label: 'WebSocket',
                icon: <LottieIcon animationData={noConnectionAnimation} size={32} />,
                content: <ObsWebSocketManager mode="local" />
              },
              {
                id: 'control-room',
                label: 'Control Room',
                icon: <LottieIcon animationData={mixerAnimation} size={32} />,
                content: <ObsWebSocketManager mode="remote" />
              },
                                {
                    id: 'integration',
                    label: 'Integration',
                    icon: <LottieIcon animationData={businessAnimation} size={32} />,
                content: (
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
                )
              }
            ]}
            activeTab={obsTab}
            onTabChange={setObsTab}
          />
        )}
        {drawer?.key === 'ovr' && (
          <TabGroup
            tabs={[
                                {
                    id: 'integration',
                    label: 'Integration',
                    icon: <LottieIcon animationData={businessAnimation} size={32} />,
                content: (
                  <div className="text-gray-400 text-center py-8">
                    OVR Integration Settings will be implemented here
                  </div>
                )
              },
              {
                id: 'tournament',
                label: 'Tournament',
                icon: <LottieIcon animationData={tournamentAnimation} size={32} />,
                content: <TournamentManagementPanel />
              }
            ]}
            activeTab={ovrTab}
            onTabChange={setOvrTab}
          />
        )}
        {drawer?.key === 'ivr' && (
          <div className="bg-gradient-to-br from-green-900/20 to-green-800/30 backdrop-blur-sm rounded-lg p-6 text-gray-200 border border-green-600/30 shadow-lg">[Instant Video Replay integration and controls will be implemented here]</div>
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
                  <div className="flex-[2] min-w-[320px]">
                    <LogArchiveManager />
                  </div>
                </div>
                <div className="space-y-6">
                <LiveDataPanel />
                  <CpuMonitoringSection />
                </div>
              </div>
            )}
            {settingsTab === 'app-settings' && (
              <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg p-6 border border-gray-600/30 shadow-lg">
                <AppSettingsSection />
              </div>
            )}
            {settingsTab === 'database' && (
              <div className="space-y-6">
                <DatabaseManagementPanel />
                <DatabaseMigrationPanel />
              </div>
            )}
            {settingsTab === 'backup-restore' && (
              <div className="space-y-6">
                <GoogleDriveBackupRestore />
              </div>
            )}
            {settingsTab === 'flag-management' && (
              <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg p-6 border border-gray-600/30 shadow-lg">
                <FlagManagementPanel />
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