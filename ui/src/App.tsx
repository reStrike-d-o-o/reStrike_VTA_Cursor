import React from 'react';
import DockBar from './components/layouts/DockBar';
import AdvancedPanel from './components/layouts/AdvancedPanel';
import { useAppStore } from './stores';
import { useEnvironment } from './hooks/useEnvironment';
import { usePssEvents } from './hooks/usePssEvents';
import { useLiveDataEvents } from './hooks/useLiveDataEvents';
import { useEnvironmentObs } from './hooks/useEnvironmentObs';
import { invoke } from '@tauri-apps/api/core';
import { listenTauri, canListenTauri } from './utils/tauriBridge';
import { ObsHealthSnapshot, PssStatsSnapshot } from './types';

import { useTriggersStore } from './stores/triggersStore';
import PausedOverlay from './components/molecules/PausedOverlay';
import GlobalModals from './components/molecules/GlobalModals';
import { useSettingsStore } from './stores/settingsStore';
import MedalCeremonyExternalDisplay from './components/ovr/MedalCeremonyExternalDisplay';
import { useOverlayController } from './hooks/useOverlayController';
import { useOverlayRoutingStore } from './stores/overlayRoutingStore';

const App: React.FC = () => {
  const isAdvancedPanelOpen = useAppStore((state) => state.isAdvancedPanelOpen);
  const windowSettings = useAppStore((state) => state.windowSettings);
  const loadWindowSettings = useAppStore((state) => state.loadWindowSettings);
  const updateObsHealth = useAppStore((state) => state.updateObsHealth);
  const updatePssStats = useAppStore((state) => state.updatePssStats);
  const { tauriAvailable, environment, isLoading } = useEnvironment();

  const paused = useTriggersStore((s) => s.paused);
  const theme = useSettingsStore((s) => s.theme);
  const sharp = useSettingsStore((s) => s.sharp);
  const [externalMode, setExternalMode] = React.useState<'medal' | null>(null);
  const loadOverlayRouting = useOverlayRoutingStore((s) => s.loadFromBackend);
  // Initialize PSS event listener for real-time events
  const { setupEventListener, fetchPendingEvents } = usePssEvents();

  // Initialize live data events for Event Table
  const { isConnected: liveDataConnected, eventCount } = useLiveDataEvents();
  // Initialize overlay controller (routes PSS triggers to overlay windows)
  useOverlayController();

  // Initialize OBS status listener for real-time status updates
  const { setupStatusListener } = useEnvironmentObs();

  React.useEffect(() => {
    if (typeof window === 'undefined') return;
    const params = new URLSearchParams(window.location.search);
    const external = params.get('external');
    if (external === 'medal') {
      setExternalMode('medal');
    }
  }, []);

  // Debug environment detection
  React.useEffect(() => {
    // console.log('🌍 App Environment Detection:');
    // console.log('  - Tauri Available:', tauriAvailable);
    // console.log('  - Environment:', environment);
    // console.log('  - Is Loading:', isLoading);
    // console.log('  - Window Tauri:', typeof window !== 'undefined' ? window.__TAURI__ : 'N/A');
  }, [tauriAvailable, environment, isLoading]);

  React.useEffect(() => {
    if (!tauriAvailable || !canListenTauri()) {
      return;
    }

    let unlistenHealth: (() => void) | undefined;
    let unlistenPss: (() => void) | undefined;

    const setup = async () => {
      try {
        unlistenHealth = await listenTauri('obs_health', (event: any) => {
          const connections: unknown = event?.payload?.connections;
          if (Array.isArray(connections)) {
            updateObsHealth(connections as ObsHealthSnapshot[]);
          } else {
            updateObsHealth([]);
          }
        });
      } catch (error) {
        console.error('Failed to listen for obs_health events:', error);
      }

      try {
        unlistenPss = await listenTauri('pss_stats', (event: any) => {
          const snapshot: unknown = event?.payload?.stats;
          if (snapshot) {
            updatePssStats(snapshot as PssStatsSnapshot);
          }
        });
      } catch (error) {
        console.error('Failed to listen for pss_stats events:', error);
      }
    };

    setup();

    return () => {
      if (unlistenHealth) {
        try {
          unlistenHealth();
        } catch (error) {
          console.warn('Failed to unlisten obs_health:', error);
        }
      }
      if (unlistenPss) {
        try {
          unlistenPss();
        } catch (error) {
          console.warn('Failed to unlisten pss_stats:', error);
        }
      }
    };
  }, [tauriAvailable, updateObsHealth, updatePssStats]);
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
      loadOverlayRouting();

      // Set window to startup position (x=1, y=1)
      invoke('set_window_startup_position').catch((error) => {
        console.error('Failed to set window startup position:', error);
      });
    }
  }, [tauriAvailable, isLoading, loadWindowSettings, loadOverlayRouting]);

  // Apply theme attribute
  React.useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  }, [theme]);

  React.useEffect(() => {
    document.documentElement.setAttribute('data-sharp', sharp ? 'true' : 'false');
  }, [sharp]);

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

  if (externalMode === 'medal') {
    return <MedalCeremonyExternalDisplay />;
  }

  return (
    <div className="h-screen flex flex-col bg-gradient-to-br from-gray-900 via-gray-800 to-black text-white overflow-hidden">
      {paused && <PausedOverlay />}
      <GlobalModals />

      {/* Subtle background pattern overlay */}
      <div className="absolute inset-0 bg-gradient-to-br from-gray-800/20 to-gray-900/30 opacity-50"></div>

      {/* Main content area: DockBar (left) + AdvancedPanel (right) */}
      <div className="flex flex-1 min-h-0 relative z-10">
        {/* DockBar (left) - fixed 350px width */}
        <div
          className="flex-shrink-0 relative z-20 w-[350px]"
        >
          <div className="absolute inset-0 w-[350px] bg-gradient-to-r from-gray-900/95 to-gray-800/90 backdrop-blur-sm border-r border-gray-700/50 shadow-2xl"></div>
          <div className="relative z-10 h-full w-[350px]">
            <DockBar />
          </div>
        </div>

        {/* AdvancedPanel (right) - only shown when panel is open */}
        {isAdvancedPanelOpen && (
          <div className="flex-1 min-h-0 relative z-10">
            <>
              <div className="absolute inset-0 bg-gradient-to-br from-gray-800/90 to-gray-900/95 backdrop-blur-sm shadow-inner"></div>
              <div className="relative z-10 h-full">
                <AdvancedPanel className="h-full" />
              </div>
            </>
          </div>
        )}
      </div>
    </div>
  );
};

export default App;
