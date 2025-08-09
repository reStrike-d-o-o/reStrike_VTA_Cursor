import React from 'react';
import DockBar from './components/layouts/DockBar';
import AdvancedPanel from './components/layouts/AdvancedPanel';
import { useAppStore } from './stores';
import { useEnvironment } from './hooks/useEnvironment';
import { usePssEvents } from './hooks/usePssEvents';
import { useLiveDataEvents } from './hooks/useLiveDataEvents';
import { useEnvironmentObs } from './hooks/useEnvironmentObs';
import { invoke } from '@tauri-apps/api/core';

import { useTriggersStore } from './stores/triggersStore';
import PausedOverlay from './components/molecules/PausedOverlay';
import DestructiveConfirmModal from './components/molecules/DestructiveConfirmModal';
import { useSettingsStore } from './stores/settingsStore';

const App: React.FC = () => {
  const isAdvancedPanelOpen = useAppStore((state) => state.isAdvancedPanelOpen);
  const windowSettings = useAppStore((state) => state.windowSettings);
  const loadWindowSettings = useAppStore((state) => state.loadWindowSettings);
  const { tauriAvailable, environment, isLoading } = useEnvironment();
  
  const paused = useTriggersStore((s) => s.paused);
  const theme = useSettingsStore((s)=>s.theme);
  // Initialize PSS event listener for real-time events
  const { setupEventListener, fetchPendingEvents } = usePssEvents();
  
  // Initialize live data events for Event Table
  const { isConnected: liveDataConnected, eventCount } = useLiveDataEvents();
  
  // Initialize OBS status listener for real-time status updates
  const { setupStatusListener } = useEnvironmentObs();
  
  // Debug environment detection
  React.useEffect(() => {
    // console.log('🌍 App Environment Detection:');
    // console.log('  - Tauri Available:', tauriAvailable);
    // console.log('  - Environment:', environment);
    // console.log('  - Is Loading:', isLoading);
    // console.log('  - Window Tauri:', typeof window !== 'undefined' ? window.__TAURI__ : 'N/A');
  }, [tauriAvailable, environment, isLoading]);

  // Debug live data connection
  React.useEffect(() => {
    console.log('📡 Live Data Events Status:', { 
      isConnected: liveDataConnected, 
      eventCount 
    });
  }, [liveDataConnected, eventCount]);

  // Load window settings and set startup position
  React.useEffect(() => {
    if (tauriAvailable && !isLoading) {
      loadWindowSettings();
      
      // Set window to startup position (x=1, y=1)
      invoke('set_window_startup_position').catch((error) => {
        console.error('Failed to set window startup position:', error);
      });
    }
  }, [tauriAvailable, isLoading, loadWindowSettings]);

  // Apply theme attribute
  React.useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  }, [theme]);

  // Apply dock width CSS variable (avoid inline width styling on elements)
  React.useEffect(() => {
    document.documentElement.style.setProperty('--dock-width', `${windowSettings.compactWidth}px`);
  }, [windowSettings.compactWidth]);

  // Set up PSS event listener and OBS status listener when Tauri is available (run once)
  const hasInitRef = React.useRef(false);
  React.useEffect(() => {
    if (hasInitRef.current) return;

    if (tauriAvailable && !isLoading) {
      // console.log('🚀 Setting up PSS event system...');
      setupEventListener();
      fetchPendingEvents();
      
      // Setup OBS status listener for real-time status updates
      setupStatusListener().catch((error) => {
        console.error('Failed to setup OBS status listener:', error);
      });
      
      hasInitRef.current = true;
    }
  }, [tauriAvailable, isLoading, setupEventListener, fetchPendingEvents, setupStatusListener]);
  
  return (
    <div className="h-screen flex flex-col bg-gradient-to-br from-gray-900 via-gray-800 to-black text-white overflow-hidden">
      {paused && <PausedOverlay />}
      <DestructiveConfirmModal />

      {/* Subtle background pattern overlay */}
      <div className="absolute inset-0 bg-gradient-to-br from-gray-800/20 to-gray-900/30 opacity-50"></div>
      
      {/* Main content area: DockBar (left) + AdvancedPanel (right) */}
      <div className="flex flex-1 min-h-0 relative z-10">
        {/* DockBar (left) - dynamic width from settings, full height with enhanced styling */}
        <div 
          className="flex-shrink-0 relative z-20 w-[var(--dock-width)]"
        >
          <div className="absolute inset-0 w-[var(--dock-width)] bg-gradient-to-r from-gray-900/95 to-gray-800/90 backdrop-blur-sm border-r border-gray-700/50 shadow-2xl"></div>
          <div className="relative z-10 h-full w-[var(--dock-width)]">
            <DockBar />
          </div>
        </div>
        
        {/* AdvancedPanel (right) - flexible width with enhanced styling */}
        <div className="flex-1 min-h-0 relative z-10">
          {isAdvancedPanelOpen ? (
            <>
              <div className="absolute inset-0 bg-gradient-to-br from-gray-800/90 to-gray-900/95 backdrop-blur-sm shadow-inner"></div>
              <div className="relative z-10 h-full">
                <AdvancedPanel className="h-full" />
              </div>
            </>
          ) : (
            <div className="h-full flex items-center justify-center text-gray-500">
              <div className="text-center">
                <div className="text-2xl mb-2">Click "Advanced" to open settings</div>
                <div className="text-sm">WebSocket connections, protocol settings, and more</div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default App;
