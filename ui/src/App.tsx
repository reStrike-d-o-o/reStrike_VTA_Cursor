import React from 'react';
import DockBar from './components/layouts/DockBar';
import TaskBar from './components/layouts/TaskBar';
import AdvancedPanel from './components/layouts/AdvancedPanel';
import StatusbarDock from './components/layouts/StatusbarDock';
import StatusbarAdvanced from './components/layouts/StatusbarAdvanced';

const App: React.FC = () => {
  return (
    <div className="min-h-screen flex flex-col bg-gray-900 text-white">
      {/* Task Bar at the top */}
      <TaskBar />
      {/* Main content area: DockBar (left) + AdvancedPanel (right) */}
      <div className="flex flex-1 h-full">
        {/* DockBar (left) */}
        <div className="flex flex-col h-full">
          <DockBar />
          <StatusbarDock />
        </div>
        {/* AdvancedPanel (right) */}
        <div className="flex flex-col flex-1 h-full">
          <AdvancedPanel />
          <StatusbarAdvanced />
        </div>
      </div>
    </div>
  );
};

export default App;
