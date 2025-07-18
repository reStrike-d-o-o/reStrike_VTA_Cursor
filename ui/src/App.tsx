import React from 'react';
import DockBar from './components/layouts/DockBar';
import TaskBar from './components/layouts/TaskBar';
import AdvancedPanel from './components/layouts/AdvancedPanel';
import StatusbarAdvanced from './components/layouts/StatusbarAdvanced';
import { useAppStore } from './stores';

const App: React.FC = () => {
  const isAdvancedPanelOpen = useAppStore((state) => state.isAdvancedPanelOpen);
  return (
    <div className="h-screen flex flex-col bg-gray-900 text-white">
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
