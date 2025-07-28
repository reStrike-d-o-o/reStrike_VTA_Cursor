import React, { useState } from 'react';
import MatchDetailsSection from '../molecules/MatchDetailsSection';
import EventTableSection from '../molecules/EventTableSection';
import Button from '../atoms/Button';
import ReplayButton from '../atoms/ReplayButton';
import Toggle from '../atoms/Toggle';
import StatusbarDock from './StatusbarDock';
import PasswordDialog from '../molecules/PasswordDialog';
import ManualModeDialog from '../molecules/ManualModeDialog';
import { useAppStore } from '../../stores';
import { windowCommands } from '../../utils/tauriCommands';
import { useEnvironment } from '../../hooks/useEnvironment';

const DockBar: React.FC = () => {
  const { tauriAvailable } = useEnvironment();
  
  // Store state
  const isAdvancedPanelOpen = useAppStore((state) => state.isAdvancedPanelOpen);
  const isAdvancedModeAuthenticated = useAppStore((state) => state.isAdvancedModeAuthenticated);
  const isManualModeEnabled = useAppStore((state) => state.isManualModeEnabled);
  const windowSettings = useAppStore((state) => state.windowSettings);
  
  // Store actions
  const toggleAdvancedPanel = useAppStore((state) => state.toggleAdvancedPanel);
  const authenticateAdvancedMode = useAppStore((state) => state.authenticateAdvancedMode);
  const deauthenticateAdvancedMode = useAppStore((state) => state.deauthenticateAdvancedMode);
  const toggleManualMode = useAppStore((state) => state.toggleManualMode);
  
  // Local state
  const [showPasswordDialog, setShowPasswordDialog] = useState(false);
  const [showManualDialog, setShowManualDialog] = useState(false);

  // Handle Advanced button click
  const handleAdvancedClick = async () => {
    if (!isAdvancedModeAuthenticated) {
      setShowPasswordDialog(true);
      return;
    }

    // Toggle Advanced panel and window size
    if (!isAdvancedPanelOpen) {
      // Opening Advanced panel - go fullscreen with custom dimensions
      if (tauriAvailable) {
        try {
          await windowCommands.setCustomSize(windowSettings.fullscreenWidth, windowSettings.fullscreenHeight);
        } catch (error) {
          console.error('Failed to set fullscreen:', error);
        }
      }
    } else {
      // Closing Advanced panel - go compact with custom dimensions
      if (tauriAvailable) {
        try {
          await windowCommands.setCompact(windowSettings.compactWidth, windowSettings.compactHeight);
        } catch (error) {
          console.error('Failed to set compact mode:', error);
        }
      }
    }
    
    toggleAdvancedPanel();
  };

  // Handle Manual mode toggle
  const handleManualModeToggle = () => {
    setShowManualDialog(true);
  };

  return (
    <>
      <div className="flex flex-col w-full h-full min-h-0 bg-gradient-to-b from-gray-900/95 to-gray-800/90 backdrop-blur-sm border-r border-gray-600/30 shadow-xl overflow-hidden">
        {/* Main content area */}
        <div className="flex flex-row flex-1 min-h-0 overflow-hidden">
          <div className="flex-1 h-full min-h-0 flex flex-col p-0 text-white overflow-hidden">
            {/* Main content card with enhanced styling */}
            <div className="flex-1 flex flex-col bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm shadow-xl border border-gray-600/30 p-2 space-y-1 overflow-hidden">
              {/* Match Info Section */}
              <div className="flex-shrink-0">
                <MatchDetailsSection />
              </div>
              
              {/* Enhanced Divider */}
              <div className="flex-shrink-0 border-t border-gray-600/50 bg-gradient-to-r from-transparent via-gray-600/30 to-transparent h-px"></div>
              
              {/* Event Table Section */}
              <div className="flex-shrink-0 overflow-hidden">
                <EventTableSection />
              </div>
              
              {/* Spacer for 20px gap */}
              <div className="flex-shrink-0 h-5"></div>
              
              {/* Enhanced Divider */}
              <div className="flex-shrink-0 border-t border-gray-600/50 bg-gradient-to-r from-transparent via-gray-600/30 to-transparent h-px"></div>
              
              {/* Controls Section */}
              <div className="flex-shrink-0 flex flex-row items-center justify-center space-x-4 p-2">
                {/* Replay Button */}
                <div className="flex flex-col items-center space-y-2">
                  <div className="relative group">
                    <div className="absolute inset-0 bg-blue-500/20 rounded-full blur-sm group-hover:bg-blue-500/30 transition-all duration-300"></div>
                    <ReplayButton onClick={() => { /* TODO: Implement Replay action */ }}>
                      REPLAY
                    </ReplayButton>
                  </div>
                </div>
                
                {/* Manual Mode Toggle and Advanced Button Stack */}
                <div className="flex flex-col items-center space-y-2">
                  {/* Manual Mode Toggle */}
                  <Toggle
                    checked={isManualModeEnabled}
                    onChange={handleManualModeToggle}
                    label="Manual Mode"
                    labelPosition="bottom"
                    className="scale-100"
                  />
                  
                  {/* Advanced Button */}
                  <div className="relative group">
                    <div className="absolute inset-0 bg-purple-500/20 rounded-lg blur-sm group-hover:bg-purple-500/30 transition-all duration-300"></div>
                    <Button
                      variant={isAdvancedPanelOpen ? 'primary' : 'secondary'}
                      size="sm"
                      onClick={handleAdvancedClick}
                      className="w-32 relative z-10"
                    >
                      Advanced
                    </Button>
                  </div>
                </div>
              </div>
            </div>
            
            {/* Copyright Section with Logo */}
            <div className="flex-shrink-0 flex flex-col items-start justify-center py-3 px-4 border-t border-gray-600/30 bg-gray-800/20 backdrop-blur-sm">
              <div className="flex items-center justify-between w-full mb-2">
                <div className="text-xs text-gray-400">All rights reserved ®:</div>
                <div className="text-xs text-gray-400">Inspired by:</div>
              </div>
              <div className="flex items-center justify-between w-full">
                <div className="relative z-10" style={{ minHeight: '32px', minWidth: '32px' }}>
                  <img 
                    src="/assets/img/logo.png" 
                    alt="reStrike VTA Logo" 
                    className="h-8 w-auto object-contain"
                    onError={(e) => {
                      // console.log('Logo failed to load:', e);
                      e.currentTarget.style.display = 'none';
                    }}
                    onLoad={() => {
                      // console.log('Logo loaded successfully');
                    }}
                  />
                </div>
                <div className="text-lg font-mono text-gray-300" style={{ fontSize: '32px', lineHeight: '32px', marginTop: '3px' }}>
                  E7 88 B1
                </div>
              </div>
            </div>
            
            {/* Status bar with enhanced styling */}
            <div className="flex-shrink-0 border-t border-gray-600/30 bg-gray-800/50 backdrop-blur-sm">
              <StatusbarDock />
            </div>
          </div>
        </div>
      </div>

      {/* Password Dialog */}
      <PasswordDialog
        isOpen={showPasswordDialog}
        onClose={() => setShowPasswordDialog(false)}
        onAuthenticate={authenticateAdvancedMode}
        title="Advanced Mode Authentication"
        message="Please enter the password to enable Advanced mode:"
      />

      {/* Manual Mode Dialog */}
      <ManualModeDialog
        isOpen={showManualDialog}
        onClose={() => setShowManualDialog(false)}
        onToggle={toggleManualMode}
        isEnabled={isManualModeEnabled}
      />
    </>
  );
};

export default DockBar; 