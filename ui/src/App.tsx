import React from 'react';
import DockBar from './components/layouts/DockBar';
import TaskBar from './components/layouts/TaskBar';
import AdvancedPanel from './components/layouts/AdvancedPanel';
import StatusbarAdvanced from './components/layouts/StatusbarAdvanced';
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
    <div className="h-screen flex flex-col bg-gray-900 text-white">
      {/* Environment Indicator */}
      {!isLoading && (
        <div className={`fixed top-2 right-2 px-3 py-1 rounded-full text-xs font-bold z-50 ${
          tauriAvailable 
            ? 'bg-green-500 text-white' 
            : 'bg-red-500 text-white'
        }`}>
          {tauriAvailable ? '🖥️ NATIVE' : '🌐 WEB'}
        </div>
      )}
      {/* Task Bar at the top */}
      <TaskBar />
      {/* Main content area: DockBar (left) + AdvancedPanel (right) */}
      <div className="flex flex-1 min-h-0">
        {/* DockBar (left) - fixed width, full height */}
        <div className="w-96 flex-shrink-0">
          <DockBar />
        </div>
        {/* AdvancedPanel (right) - flexible width */}
        <div className="flex-1 min-h-0">
          {isAdvancedPanelOpen && <AdvancedPanel className="h-full" />}
        </div>
      </div>
    </div>
  );
};

export default App;
