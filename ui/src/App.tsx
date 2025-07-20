import React from 'react';
import DockBar from './components/layouts/DockBar';
import AdvancedPanel from './components/layouts/AdvancedPanel';
import { useAppStore } from './stores';
import { useEnvironment } from './hooks/useEnvironment';

const App: React.FC = () => {
  const isAdvancedPanelOpen = useAppStore((state) => state.isAdvancedPanelOpen);
  const { tauriAvailable, environment, isLoading } = useEnvironment();
  
  // Debug environment detection
  React.useEffect(() => {
    console.log('🌍 App Environment Detection:');
    console.log('  - Tauri Available:', tauriAvailable);
    console.log('  - Environment:', environment);
    console.log('  - Is Loading:', isLoading);
    console.log('  - Window Tauri:', typeof window !== 'undefined' ? window.__TAURI__ : 'N/A');
  }, [tauriAvailable, environment, isLoading]);
  
  return (
    <div className="h-screen flex flex-col bg-gradient-to-br from-gray-900 via-gray-800 to-black text-white overflow-hidden">
      {/* Subtle background pattern overlay */}
      <div className="absolute inset-0 bg-gradient-to-br from-gray-800/20 to-gray-900/30 opacity-50"></div>
      
      {/* Main content area: DockBar (left) + AdvancedPanel (right) */}
      <div className="flex flex-1 min-h-0 relative z-10">
        {/* DockBar (left) - fixed width, full height with enhanced styling */}
        <div className="w-[450px] flex-shrink-0 relative z-20">
          <div className="absolute inset-0 bg-gradient-to-r from-gray-900/95 to-gray-800/90 backdrop-blur-sm border-r border-gray-700/50 shadow-2xl"></div>
          <div className="relative z-10 h-full">
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
