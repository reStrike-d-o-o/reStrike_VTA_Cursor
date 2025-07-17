import React from 'react';
import { StatusDot } from '../atoms/StatusDot';

const StatusbarDock: React.FC = () => {
  return (
    <div className="w-full h-[4.5rem] bg-[#181F26] flex items-center justify-start text-xs text-gray-500 px-8">
      <div className="flex items-center space-x-8">
        <div className="flex items-center space-x-1">
          <StatusDot color="bg-red-500" />
          <span>REC</span>
        </div>
        <div className="flex items-center space-x-1">
          <StatusDot color="bg-red-500" />
          <span>STR</span>
        </div>
        <div className="flex items-center space-x-1">
          <StatusDot color="bg-green-500" />
          <span>CPU 0%</span>
        </div>
      </div>
    </div>
  );
};

export default StatusbarDock; 