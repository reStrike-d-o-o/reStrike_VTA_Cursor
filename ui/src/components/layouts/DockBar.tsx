import React from 'react';
import SidebarSmall from '../SidebarSmall';
import SidebarBig from '../SidebarBig';
import StatusbarDock from './StatusbarDock';

const DockBar: React.FC = () => {
  return (
    <div className="flex flex-col w-full h-[40vh] min-h-0 bg-[#181F26] shadow-lg">
      <div className="flex flex-row flex-1 min-h-0">
        <SidebarSmall />
        <SidebarBig />
      </div>
      <StatusbarDock />
    </div>
  );
};

export default DockBar; 