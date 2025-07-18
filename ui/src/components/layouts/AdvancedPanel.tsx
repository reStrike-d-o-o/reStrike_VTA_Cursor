import React, { useState } from 'react';
import SettingsDrawerTabs from '../molecules/SettingsDrawerTabs';
import LogToggleGroup from '../molecules/LogToggleGroup';
import LogDownloadList from '../molecules/LogDownloadList';
import LiveDataPanel from '../molecules/LiveDataPanel';
import WebSocketManager from '../molecules/WebSocketManager';

type AdvancedPanelProps = React.ComponentProps<'div'>;

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
      <svg width="24" height="24" fill="none" viewBox="0 0 24 24"><circle cx="12" cy="12" r="3" stroke="currentColor" strokeWidth="2"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h.09A1.65 1.65 0 0 0 9 3.09V3a2 2 0 1 1 4 0v.09c0 .66.39 1.26 1 1.51h.09a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.09c.66 0 1.26.39 1.51 1H21a2 2 0 1 1 0 4h-.09c-.66 0-1.26.39-1.51 1z" stroke="currentColor" strokeWidth="2"/></svg>
    ),
    description: 'All settings, including Diagnostics & Logs Manager.'
  },
];

const AdvancedPanel: React.FC<AdvancedPanelProps> = ({ className = '', ...rest }) => {
  const [activeDrawer, setActiveDrawer] = useState('pss');
  // Settings horizontal drawer state
  const [settingsTab, setSettingsTab] = useState('diagnostics');
  // OBS horizontal drawer state
  const [obsTab, setObsTab] = useState('websocket');
  const drawer = DRAWERS.find(d => d.key === activeDrawer);

  return (
    <div className={`flex h-full min-h-0 min-w-[360px] max-w-[100%] bg-[#101820] shadow-lg overflow-hidden ${className}`} {...rest}>
      {/* Drawer Sidebar */}
      <nav className="h-full min-h-0 flex flex-col justify-center items-center py-0 px-2 bg-[#181F26] border-r border-gray-800 w-20">
        <div className="flex flex-col items-center space-y-4">
          {DRAWERS.map(d => (
            <button
              key={d.key}
              className={`flex flex-col items-center justify-center w-14 h-14 transition-colors ${activeDrawer === d.key ? 'bg-blue-700 text-white shadow-lg' : 'text-gray-400 hover:bg-gray-800 hover:text-blue-300'}`}
              onClick={() => setActiveDrawer(d.key)}
              aria-label={d.label}
            >
              <span className="mb-1">{d.icon}</span>
              <span className="text-xs font-semibold">{d.label}</span>
            </button>
          ))}
        </div>
      </nav>
      {/* Drawer Content */}
      <main className="flex-1 flex flex-col p-8 overflow-y-auto min-h-0">
        {/* Placeholder for each drawer's content */}
        {drawer?.key === 'pss' && (
          <div className="bg-[#18232e] rounded p-6 text-gray-300">[PSS options, UDP server, protocol, event DB will be implemented here]</div>
        )}
        {drawer?.key === 'obs' && (
          <>
            {/* OBS Tab Navigation */}
            <div className="flex space-x-1 mb-6 bg-gray-800 p-1 rounded-lg">
              <button
                onClick={() => setObsTab('websocket')}
                className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                  obsTab === 'websocket' 
                    ? 'bg-blue-600 text-white' 
                    : 'text-gray-300 hover:bg-gray-700'
                }`}
              >
                🔌 WebSocket
              </button>
              <button
                onClick={() => setObsTab('integration')}
                className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                  obsTab === 'integration' 
                    ? 'bg-blue-600 text-white' 
                    : 'text-gray-300 hover:bg-gray-700'
                }`}
              >
                ⚙️ Integration
              </button>
            </div>

            {/* OBS Tab Content */}
            {obsTab === 'websocket' && (
              <WebSocketManager />
            )}
            
            {obsTab === 'integration' && (
              <div className="space-y-6">
                <div className="p-4 bg-gray-800 rounded-lg">
                  <h3 className="text-lg font-semibold mb-3">OBS Integration Settings</h3>
                  <div className="space-y-4">
                    <div className="flex items-center space-x-2">
                      <input
                        type="checkbox"
                        id="obs-auto-connect"
                        defaultChecked={true}
                        className="rounded border-gray-600 bg-gray-700 text-blue-500 focus:ring-blue-500"
                      />
                      <label htmlFor="obs-auto-connect" className="text-sm">Auto-connect to OBS on startup</label>
                    </div>
                    <div className="flex items-center space-x-2">
                      <input
                        type="checkbox"
                        id="obs-show-status"
                        defaultChecked={true}
                        className="rounded border-gray-600 bg-gray-700 text-blue-500 focus:ring-blue-500"
                      />
                      <label htmlFor="obs-show-status" className="text-sm">Show OBS status in overlay</label>
                    </div>
                    <div className="flex items-center space-x-2">
                      <input
                        type="checkbox"
                        id="obs-auto-record"
                        defaultChecked={false}
                        className="rounded border-gray-600 bg-gray-700 text-blue-500 focus:ring-blue-500"
                      />
                      <label htmlFor="obs-auto-record" className="text-sm">Auto-record when playing clips</label>
                    </div>
                    <div className="flex items-center space-x-2">
                      <input
                        type="checkbox"
                        id="obs-save-replay"
                        defaultChecked={true}
                        className="rounded border-gray-600 bg-gray-700 text-blue-500 focus:ring-blue-500"
                      />
                      <label htmlFor="obs-save-replay" className="text-sm">Save replay buffer on clip creation</label>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </>
        )}
        {drawer?.key === 'video' && (
          <div className="bg-[#182e1a] rounded p-6 text-gray-300">[mpv video integration and controls will be implemented here]</div>
        )}
        {drawer?.key === 'ai' && (
          <div className="bg-[#2e1824] rounded p-6 text-gray-300">[AI report creation and data analyzer will be implemented here]</div>
        )}
        {drawer?.key === 'settings' && (
          <>
            <SettingsDrawerTabs activeTab={settingsTab} onTabChange={setSettingsTab} />
            {settingsTab === 'diagnostics' && (
              <div className="flex flex-col gap-6">
                <div className="flex flex-row gap-6">
                  <div className="flex-1 min-w-[220px]">
                    <LogToggleGroup />
                  </div>
                  <div className="flex-[2] min-w-[320px]">
                    <LogDownloadList />
                  </div>
                </div>
                <LiveDataPanel />
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