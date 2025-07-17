import React from 'react';
import DockBar from './components/layouts/DockBar';
import TaskBar from './components/layouts/TaskBar';
import AdvancedPanel from './components/layouts/AdvancedPanel';
import StatusbarAdvanced from './components/layouts/StatusbarAdvanced';
import { useAppStore } from './stores';

const App: React.FC = () => {
  const isAdvancedPanelOpen = useAppStore((state) => state.isAdvancedPanelOpen);
  return (
    <div className="min-h-screen flex flex-col bg-gray-900 text-white">
      {/* Task Bar at the top */}
      <TaskBar />
      {/* Main content area: DockBar (left) + AdvancedPanel (right) */}
      <div className="flex flex-1 h-full">
        {/* DockBar (left) */}
        <div className="flex flex-col h-full">
          <DockBar />
        </div>
        {/* AdvancedPanel (right) */}
        <div className="flex flex-col flex-1 h-full">
          {isAdvancedPanelOpen && <AdvancedPanel />}
          <StatusbarAdvanced />
        </div>
      </div>
    </div>
  );
};

export default App;
