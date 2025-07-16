import React from 'react';

const StatusbarDock: React.FC = () => {
  return (
    <div className="w-full flex justify-between items-center text-xs text-gray-500 px-2 bg-gray-900">
      <div className="flex items-center space-x-1">
        <span className="w-2 h-2 rounded-full bg-red-500" />
        <span>REC</span>
      </div>
      <div className="flex items-center space-x-1">
        <span className="w-2 h-2 rounded-full bg-red-500" />
        <span>STR</span>
      </div>
      <div className="flex items-center space-x-1">
        <span className="w-2 h-2 rounded-full bg-green-500" />
        <span>CPU 0%</span>
      </div>
    </div>
  );
};

export default StatusbarDock; 