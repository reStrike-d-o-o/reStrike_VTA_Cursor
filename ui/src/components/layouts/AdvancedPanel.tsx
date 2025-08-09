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
import ObsIntegrationPanel from '../molecules/ObsIntegrationPanel';
import IvrReplaySettings from '../molecules/IvrReplaySettings';
import Toggle from '../atoms/Toggle';
import TabGroup from '../molecules/TabGroup';
import TabIcons from '../atoms/TabIcons';
import LottieIcon from '../atoms/LottieIcon';
import { useAppStore } from '../../stores';
import { configCommands } from '../../utils/tauriCommands';
import { flowChartAnimation, spyAnimation, plansAnimation, watcherAnimation, taekwondoAnimation, liveStreamingAnimation, settingsAnimation, robotAnimation, noConnectionAnimation, businessAnimation, tournamentAnimation, mixerAnimation } from '../../assets/icons/json';

type AdvancedPanelProps = React.ComponentProps<'div'>;



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
  
  const drawer = DRAWERS.find(d => d.key === activeDrawer);

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
                content: <WebSocketManager mode="local" />
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
                content: <ObsIntegrationPanel />
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
          <div className="theme-card p-6 text-gray-200 shadow-lg">
            <IvrReplaySettings />
          </div>
        )}
        {drawer?.key === 'ai' && (
          <div className="theme-card p-6 text-gray-200 shadow-lg">[AI report creation and data analyzer will be implemented here]</div>
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
              <div className="theme-card p-6 shadow-lg">
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
              <div className="theme-card p-6 shadow-lg">
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