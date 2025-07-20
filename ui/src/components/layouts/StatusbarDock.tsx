import React from 'react';
import { StatusDot } from '../atoms/StatusDot';

const StatusbarDock: React.FC = () => {
  return (
    <div className="w-full h-[4.5rem] bg-gradient-to-r from-gray-800/80 to-gray-900/90 backdrop-blur-sm flex items-center justify-center text-xs text-gray-300 px-8 border-t border-gray-600/30">
      <div className="flex items-center space-x-8">
        {/* REC Status */}
        <div className="flex items-center space-x-2 px-3 py-2 bg-red-500/10 rounded-lg border border-red-500/20 backdrop-blur-sm">
          <StatusDot color="bg-red-500" className="animate-pulse shadow-lg shadow-red-500/50" />
          <span className="font-medium">REC</span>
        </div>
        
        {/* STR Status */}
        <div className="flex items-center space-x-2 px-3 py-2 bg-orange-500/10 rounded-lg border border-orange-500/20 backdrop-blur-sm">
          <StatusDot color="bg-orange-500" className="animate-pulse shadow-lg shadow-orange-500/50" />
          <span className="font-medium">STR</span>
        </div>
        
        {/* CPU Status */}
        <div className="flex items-center space-x-2 px-3 py-2 bg-green-500/10 rounded-lg border border-green-500/20 backdrop-blur-sm">
          <StatusDot color="bg-green-500" className="shadow-lg shadow-green-500/50" />
          <span className="font-medium">CPU 0%</span>
        </div>
      </div>
    </div>
  );
};

export default StatusbarDock; 