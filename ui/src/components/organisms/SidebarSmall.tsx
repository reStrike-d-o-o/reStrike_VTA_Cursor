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
    <div className="flex flex-col items-center justify-between py-8 px-4 w-[12.5rem] border-r border-gray-800 h-full min-h-0 flex-1">
      <div className="flex flex-col items-center space-y-8">
        {/* Replay Button */}
        <ReplayButton onClick={() => { /* TODO: Implement Replay action */ }}>
          REPLAY
        </ReplayButton>
        {/* Manual Mode Toggle */}
        <div className="flex flex-col items-center space-y-3">
          <Checkbox
            checked={manualMode}
            onChange={() => setManualMode((v) => !v)}
            label="Manual Mode"
            labelPosition="bottom"
            className="scale-110"
          />
        </div>
        {/* Advanced Button */}
        <Button
          variant={isAdvancedPanelOpen ? 'primary' : 'secondary'}
          size="sm"
          onClick={toggleAdvancedPanel}
        >
          Advanced
        </Button>
      </div>
    </div>
  );
};

export default SidebarSmall; 