import React, { useState } from 'react';
import Button from '../atoms/Button';
import ReplayButton from '../atoms/ReplayButton';
import Input from '../atoms/Input';
import Toggle from '../atoms/Toggle';
import { useAppStore } from '../../stores';

const SidebarSmall: React.FC = () => {
  const [manualMode, setManualMode] = useState(false);
  const isAdvancedPanelOpen = useAppStore((state) => state.isAdvancedPanelOpen);
  const toggleAdvancedPanel = useAppStore((state) => state.toggleAdvancedPanel);

  return (
    <div className="flex flex-col items-center justify-between py-2 px-4 w-40 flex-shrink-0 border-r border-gray-600/30 h-full min-h-0 bg-gradient-to-b from-gray-800/80 to-gray-900/90 backdrop-blur-sm">
      {/* Top section with controls */}
      <div className="flex flex-col items-center space-y-2">
        {/* Replay Button */}
        <div className="flex flex-col items-center space-y-2">
          <div className="relative group">
            <div className="absolute inset-0 bg-blue-500/20 rounded-full blur-sm group-hover:bg-blue-500/30 transition-all duration-300"></div>
            <ReplayButton onClick={() => { /* TODO: Implement Replay action */ }}>
              REPLAY
            </ReplayButton>
          </div>
        </div>
        
        {/* Manual Mode Toggle */}
        <div className="flex flex-col items-center space-y-2">
          <Toggle
            checked={manualMode}
            onChange={() => setManualMode((v) => !v)}
            label="Manual Mode"
            labelPosition="bottom"
            className="scale-100"
          />
        </div>
        
        {/* Advanced Button */}
        <div className="flex flex-col items-center space-y-2">
          <div className="relative group">
            <div className="absolute inset-0 bg-purple-500/20 rounded-lg blur-sm group-hover:bg-purple-500/30 transition-all duration-300"></div>
            <Button
              variant={isAdvancedPanelOpen ? 'primary' : 'secondary'}
              size="sm"
              onClick={toggleAdvancedPanel}
              className="w-32 relative z-10"
            >
              Advanced
            </Button>
          </div>
        </div>
      </div>
      
      {/* Status Indicators at the bottom with enhanced styling */}
      <div className="flex flex-col items-center space-y-2 text-xs">
        {/* REC Status */}
        <div className="flex items-center space-x-2 px-3 py-2 bg-red-500/10 rounded-lg border border-red-500/20 backdrop-blur-sm">
          <div className="w-2 h-2 bg-red-500 rounded-full animate-pulse shadow-lg shadow-red-500/50"></div>
          <span className="text-gray-200 font-medium">REC</span>
        </div>
        
        {/* STR Status */}
        <div className="flex items-center space-x-2 px-3 py-2 bg-orange-500/10 rounded-lg border border-orange-500/20 backdrop-blur-sm">
          <div className="w-2 h-2 bg-orange-500 rounded-full animate-pulse shadow-lg shadow-orange-500/50"></div>
          <span className="text-gray-200 font-medium">STR</span>
        </div>
        
        {/* CPU Status */}
        <div className="flex items-center space-x-2 px-3 py-2 bg-green-500/10 rounded-lg border border-green-500/20 backdrop-blur-sm">
          <div className="w-2 h-2 bg-green-500 rounded-full shadow-lg shadow-green-500/50"></div>
          <span className="text-gray-200 font-medium">CPU 0%</span>
        </div>
      </div>
    </div>
  );
};

export default SidebarSmall; 