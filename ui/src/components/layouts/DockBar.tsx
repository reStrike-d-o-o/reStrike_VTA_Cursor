import React, { useState, useEffect } from 'react';
import MatchDetailsSection from '../molecules/MatchDetailsSection';
import EventTableSection from '../molecules/EventTableSection';
import Button from '../atoms/Button';
import ReplayButton from '../atoms/ReplayButton';
import Toggle from '../atoms/Toggle';
import StatusbarDock from './StatusbarDock';
import PasswordDialog from '../molecules/PasswordDialog';
import ManualModeDialog from '../molecules/ManualModeDialog';
import NewMatchDialog from '../molecules/NewMatchDialog';
import { useAppStore } from '../../stores';
import { windowCommands } from '../../utils/tauriCommands';
import { useEnvironment } from '../../hooks/useEnvironment';
import { invoke } from '@tauri-apps/api/core';
import { usePssMatchStore } from '../../stores/pssMatchStore';
import { useLiveDataStore } from '../../stores/liveDataStore';
import { PssAthleteInfo, PssMatchConfig } from '../../types';
import { useLiveDataEvents } from '../../hooks/useLiveDataEvents';
import LiveOrchestratorModal from '../molecules/LiveOrchestratorModal';
import { useI18n } from '../../i18n/index';
import { licenseCommands } from '../../utils/tauriCommands';

const DockBar: React.FC = () => {
  const { tauriAvailable } = useEnvironment();
  const { t } = useI18n();
  
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
  
  // Real-time event connection
  const { isConnected } = useLiveDataEvents();
  
  // Local state
  const [showPasswordDialog, setShowPasswordDialog] = useState(false);
  const [showManualDialog, setShowManualDialog] = useState(false);
  const [showNewMatchDialog, setShowNewMatchDialog] = useState(false);
  const [isRestoring, setIsRestoring] = useState(false);
  const [liveToggle, setLiveToggle] = useState<'off'|'checking'|'on'>('off');
  const [showLiveModal, setShowLiveModal] = useState(false);
  const [licenseStatus, setLicenseStatus] = useState<any>(null);

  // Removed LIVE orchestrator event hook (button removed)

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
  const handleManualModeToggle = (event: React.ChangeEvent<HTMLInputElement>) => {
    console.log('Manual mode toggle clicked!', { 
      currentState: isManualModeEnabled, 
      eventTarget: event.target.checked 
    });
    // The toggle was clicked, show confirmation dialog
    setShowManualDialog(true);
  };
  
  // Debug manual mode state
  useEffect(() => {
    console.log('🔧 DockBar - Manual mode state:', isManualModeEnabled);
  }, [isManualModeEnabled]);

  // Load license status (poll lightly)
  useEffect(() => {
    let mounted = true;
    const load = async () => {
      try { const res = await licenseCommands.getStatus(); if (mounted && res.success) setLicenseStatus(res.data); } catch {}
    };
    load();
    const id = setInterval(load, 30000);
    return () => { mounted = false; clearInterval(id); };
  }, []);

  // Handle Manual mode confirmation
  const handleManualModeConfirm = () => {
    console.log('Manual mode confirmed!', { 
      currentState: isManualModeEnabled 
    });
    toggleManualMode();
    console.log('Manual mode toggled!', { 
      newState: !isManualModeEnabled 
    });
  };

  // Handle New Match button click
  const handleNewMatchClick = () => {
    setShowNewMatchDialog(true);
  };

  // Handle Restore button click
  const handleRestoreClick = async () => {
    setIsRestoring(true);
    try {
      const result = await invoke('manual_restore_data');
      console.log('Restore result:', result);
      // TODO: Update all stores with restored data
      alert('Data restored successfully!');
    } catch (error) {
      console.error('Failed to restore data:', error);
      alert('Failed to restore data. Please check the console for details.');
    } finally {
      setIsRestoring(false);
    }
  };

  // Handle New Match creation
  const handleNewMatchCreate = async (matchData: any) => {
    console.log('Creating new match with data:', matchData);
    try {
      const result = await invoke('manual_create_match', { matchData });
      console.log('New match created:', result);
      
      // Store current events to database before clearing (if any events exist)
      const { events, storeEventsToDatabase, clearEvents } = useLiveDataStore.getState();
      if (events.length > 0) {
        try {
          // Use the match number as the match ID for storing events
          const matchId = matchData.match_number.toString();
          await storeEventsToDatabase(matchId);
          console.log(`✅ Stored ${events.length} events to database for previous match`);
        } catch (error) {
          console.error('❌ Failed to store events to database:', error);
        }
      }
      
      // Clear the event table for the new match
      clearEvents();
      console.log('✅ Cleared event table for new match');
      
      // Update the PSS match store with the new match data
      const { updateAthletes, updateMatchConfig, setMatchLoaded } = usePssMatchStore.getState();
      
      // Create athlete info objects
      const athlete1: PssAthleteInfo = {
        short: matchData.player1.name.split(' ')[0], // First name as short name
        long: matchData.player1.name,
        country: matchData.player1.ioc_code,
        iocCode: matchData.player1.ioc_code
      };
      
      const athlete2: PssAthleteInfo = {
        short: matchData.player2.name.split(' ')[0], // First name as short name
        long: matchData.player2.name,
        country: matchData.player2.ioc_code,
        iocCode: matchData.player2.ioc_code
      };
      
      // Create match config object
      const matchConfig: PssMatchConfig = {
        number: parseInt(matchData.match_number) || 0,
        category: matchData.category,
        weight: matchData.weight,
        division: matchData.division,
        totalRounds: 3, // Default to 3 rounds
        roundDuration: 120, // Default to 2 minutes
        countdownType: 'standard',
        format: 1
      };
      
      // Update the store
      updateAthletes(athlete1, athlete2);
      updateMatchConfig(matchConfig);
      setMatchLoaded(true);
      
      alert('New match created successfully! Event table cleared and ready for new data.');
      setShowNewMatchDialog(false);
    } catch (error) {
      console.error('Failed to create new match:', error);
      alert('Failed to create new match. Please check the console for details.');
    }
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
                  <div className="relative">
                    <ReplayButton onClick={async () => {
                      try {
                        const res = await (await import('../../utils/tauriCommandsObws')).obsObwsCommands.ivrRoundReplayNow('OBS_REC');
                        if (!res.success) alert(res.error || 'Replay failed');
                      } catch (e) {
                        console.error('Failed to trigger round replay:', e);
                        alert('Failed to trigger replay. Check IVR settings and OBS connection.');
                      }
                    }}>
                      REPLAY
                    </ReplayButton>
                  </div>
                </div>
                
                {/* Manual Mode Toggle and Advanced Button Stack */}
                {/* Manual Mode Toggle and Advanced Button Stack */}
                <div className="flex flex-col items-center space-y-2">
                  {/* Manual Mode Toggle */}
                  <Toggle
                    checked={isManualModeEnabled}
                    onChange={handleManualModeToggle}
                    label={t('dock.manual_mode', 'Manual Mode')}
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
                      {t('dock.advanced', 'Advanced')}
                    </Button>
                  </div>
                </div>
              </div>

              {/* License Block Banner */}
              {licenseStatus && licenseStatus.state !== 'Valid' && (
                <div className="mt-2 mx-2">
                  <div className="relative rounded-lg border-2 border-red-600 bg-gradient-to-r from-red-900 via-red-800 to-red-900 p-3 shadow-[0_0_20px_rgba(220,38,38,0.6)]">
                    <div className="absolute -top-2 -left-2 right-0 h-0.5 bg-red-600/60 rounded-t" />
                    <div className="flex items-center gap-3">
                      <div className="text-red-300 text-2xl">⚠️</div>
                      <div className="flex-1">
                        <div className="text-red-200 font-extrabold text-lg tracking-wide uppercase">
                          {t('license.blocked.title', 'License invalid or clock tampering detected')}
                        </div>
                        <div className="text-red-200/90 text-xs mt-0.5">
                          {t('license.blocked.subtitle', 'Open Settings → App settings → License to activate.')}
                        </div>
                        <div className="text-red-300 text-xs mt-1">
                          {t('license.blocked.status', 'Status')}: <span className="font-semibold">{licenseStatus.state}</span>
                          {licenseStatus.reason ? <> • {t('license.blocked.reason', 'Reason')}: <span className="font-semibold">{licenseStatus.reason}</span></> : null}
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* Visual separation and Manual Mode action row */}
              {isManualModeEnabled && (
                <>
                  <div className="flex-shrink-0 border-t border-gray-600/50 bg-gradient-to-r from-transparent via-gray-600/30 to-transparent h-px" />
                  <div className="flex-shrink-0 flex flex-row items-center justify-center gap-4 p-2">
                    {/* New Match Button */}
                    <div className="relative group">
                      <div className="absolute inset-0 bg-green-500/20 rounded-lg blur-sm group-hover:bg-green-500/30 transition-all duration-300"></div>
                      <Button
                        variant="primary"
                        size="sm"
                        onClick={handleNewMatchClick}
                        className="w-32 relative z-10 bg-green-600 hover:bg-green-700"
                      >
                        {t('dock.new_match', 'New Match')}
                      </Button>
                    </div>
                    {/* Restore Button */}
                    <div className="relative group">
                      <div className="absolute inset-0 bg-red-500/20 rounded-lg blur-sm group-hover:bg-red-500/30 transition-all duration-300"></div>
                      <Button
                        variant="danger"
                        size="sm"
                        onClick={handleRestoreClick}
                        className="w-32 relative z-10"
                        disabled={isRestoring}
                      >
                        {isRestoring ? t('dock.restoring', 'Restoring...') : t('dock.restore', 'Restore')}
                      </Button>
                    </div>
                  </div>
                </>
              )}
            </div>
            
            {/* Copyright Section with Logo */}
            <div className="flex-shrink-0 flex flex-col items-start justify-center py-3 px-4 border-t border-gray-600/30 bg-gray-800/20 backdrop-blur-sm">
              <div className="flex items-center justify-between w-full mb-2">
                <div className="text-xs text-gray-400">All rights reserved ®:</div>
                <div className="text-xs text-gray-400">Inspired by:</div>
              </div>
              <div className="flex items-center justify-between w-full">
                <div className="relative z-10 min-h-[32px] min-w-[32px]">
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
                <div className="text-lg font-mono text-gray-300 text-[32px] leading-[32px] mt-[3px]">
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
        title={t('dock.auth.title', 'Advanced Mode Authentication')}
        message={t('dock.auth.message', 'Please enter the password to enable Advanced mode:')}
      />

      {/* Manual Mode Dialog */}
      <ManualModeDialog
        isOpen={showManualDialog}
        onClose={() => setShowManualDialog(false)}
        onConfirm={handleManualModeConfirm}
        isEnabled={isManualModeEnabled}
      />

      {/* New Match Dialog */}
      <NewMatchDialog
        isOpen={showNewMatchDialog}
        onClose={() => setShowNewMatchDialog(false)}
        onStartMatch={handleNewMatchCreate}
      />

      {/* Live Orchestrator Modal */}
      <LiveOrchestratorModal
        isOpen={showLiveModal}
        onClose={() => { setShowLiveModal(false); if (liveToggle==='checking') setLiveToggle('off'); }}
        onStarted={() => { setShowLiveModal(false); setLiveToggle('on'); alert('All systems ready. Waiting for first match load.'); }}
      />
    </>
  );
};

export default DockBar; 