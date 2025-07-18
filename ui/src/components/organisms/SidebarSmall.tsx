import React, { useState } from 'react';
import Button from '../atoms/Button';
import ReplayButton from '../atoms/ReplayButton';
import Input from '../atoms/Input';
import Checkbox from '../atoms/Checkbox';
import { useAppStore } from '../../stores';

const SidebarSmall: React.FC = () => {
  const [manualMode, setManualMode] = useState(false);
  const isAdvancedPanelOpen = useAppStore((state) => state.isAdvancedPanelOpen);
  const toggleAdvancedPanel = useAppStore((state) => state.toggleAdvancedPanel);

  return (
    <div className="flex flex-col items-center justify-between py-4 px-3 w-full border-r border-gray-800 h-full min-h-0 bg-[#1a2328]">
      {/* Top section with controls */}
      <div className="flex flex-col items-center space-y-4">
        {/* Replay Button */}
        <div className="flex flex-col items-center space-y-1">
          <ReplayButton onClick={() => { /* TODO: Implement Replay action */ }}>
            REPLAY
          </ReplayButton>
        </div>
        
        {/* Manual Mode Toggle */}
        <div className="flex flex-col items-center space-y-1">
          <Checkbox
            checked={manualMode}
            onChange={() => setManualMode((v) => !v)}
            label="Manual Mode"
            labelPosition="bottom"
            className="scale-100"
          />
        </div>
        
        {/* Advanced Button */}
        <div className="flex flex-col items-center space-y-1">
          <Button
            variant={isAdvancedPanelOpen ? 'primary' : 'secondary'}
            size="sm"
            onClick={toggleAdvancedPanel}
            className="w-20"
          >
            Advanced
          </Button>
        </div>
      </div>
      
      {/* Status Indicators at the bottom */}
      <div className="flex flex-col items-center space-y-1 text-xs">
        <div className="flex items-center space-x-1">
          <div className="w-2 h-2 bg-red-500 rounded-full"></div>
          <span className="text-gray-300">REC</span>
        </div>
        <div className="flex items-center space-x-1">
          <div className="w-2 h-2 bg-red-500 rounded-full"></div>
          <span className="text-gray-300">STR</span>
        </div>
        <div className="flex items-center space-x-1">
          <div className="w-2 h-2 bg-green-500 rounded-full"></div>
          <span className="text-gray-300">CPU 0%</span>
        </div>
      </div>
    </div>
  );
};

export default SidebarSmall; 