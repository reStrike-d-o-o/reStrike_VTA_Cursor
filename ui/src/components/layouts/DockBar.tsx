import React from 'react';
import SidebarSmall from '../organisms/SidebarSmall';
import SidebarBig from '../organisms/SidebarBig';
import StatusbarDock from './StatusbarDock';

const DockBar: React.FC = () => {
  return (
    <div className="flex flex-col w-full h-full min-h-0 bg-gradient-to-b from-gray-900/95 to-gray-800/90 backdrop-blur-sm border-r border-gray-600/30 shadow-xl overflow-hidden">
      {/* Main content area */}
      <div className="flex flex-row flex-1 min-h-0 overflow-hidden">
        <SidebarSmall />
        <SidebarBig />
      </div>
      
      {/* Status bar with enhanced styling */}
      <div className="flex-shrink-0 border-t border-gray-600/30 bg-gray-800/50 backdrop-blur-sm">
        <StatusbarDock />
      </div>
    </div>
  );
};

export default DockBar; 