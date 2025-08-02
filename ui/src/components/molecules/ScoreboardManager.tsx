import React, { useState, useEffect, useRef } from 'react';
import Button from '../atoms/Button';
import Toggle from '../atoms/Toggle';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import StatusDot from '../atoms/StatusDot';
import { usePssMatchStore } from '../../stores/pssMatchStore';
import { useAppStore } from '../../stores';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';

// Use the proper Tauri v2 invoke function with fallback
const invoke = async (command: string, args?: any) => {
  try {
    // Try the proper Tauri v2 API first
    return await tauriInvoke(command, args);
  } catch (error) {
    // If that fails, try the global (window as any).__TAURI__.core.invoke
    if (typeof window !== 'undefined' && (window as any).__TAURI__ && (window as any).__TAURI__.core) {
      return await (window as any).__TAURI__.core.invoke(command, args);
    }
    throw new Error('Tauri v2 core module not available - ensure app is running in desktop mode');
  }
};


interface ScoreboardManagerProps {
  className?: string;
}

interface OverlaySettings {
  type: 'scoreboard' | 'introduction' | 'winner' | 'results' | 'victory';
  visible: boolean;
}

const ScoreboardManager: React.FC<ScoreboardManagerProps> = ({ className = '' }) => {
  // Overlay settings state
  const [overlaySettings, setOverlaySettings] = useState<OverlaySettings>({
    type: 'scoreboard',
    visible: false,
  });

  // PSS data from store
  const athlete1 = usePssMatchStore((state) => state.getAthlete1());
  const athlete2 = usePssMatchStore((state) => state.getAthlete2());
  const matchNumber = usePssMatchStore((state) => state.getMatchNumber());
  const matchCategory = usePssMatchStore((state) => state.getMatchCategory());
  const matchWeight = usePssMatchStore((state) => state.getMatchWeight());
  const matchDivision = usePssMatchStore((state) => state.getMatchDivision());
  const totalScore = usePssMatchStore((state) => state.getTotalScore());
  const isLoaded = usePssMatchStore((state) => state.matchData.isLoaded);

  // Store reference
  const { overlaySettings: appOverlaySettings } = useAppStore();

  // Update overlay settings
  const updateOverlaySettings = (updates: Partial<OverlaySettings>) => {
    const newSettings = { ...overlaySettings, ...updates };
    setOverlaySettings(newSettings);
  };





  // Update overlay content with PSS data
  const updateOverlayContent = (svg: SVGElement, settings: OverlaySettings) => {
    if (!isLoaded) return;

    // Update player names
    updateElement(svg, 'bluePlayerName', athlete1?.long || 'BLUE PLAYER');
    updateElement(svg, 'redPlayerName', athlete2?.long || 'RED PLAYER');

    // Update match information
    updateElement(svg, 'matchCategory', matchCategory || 'MEN\'S -58KG');
    updateElement(svg, 'matchNumber', matchNumber?.toString() || '1');
    
    // Update scores
    if (totalScore) {
      updateElement(svg, 'bluePlayerScore', totalScore.athlete1.toString());
      updateElement(svg, 'redPlayerScore', totalScore.athlete2.toString());
    }
  };

  // Helper function to update SVG elements
  const updateElement = (svg: SVGElement, id: string, value: string) => {
    const element = (svg as SVGSVGElement).getElementById(id);
    if (element) {
      element.textContent = value;
    }
  };







  // Overlay type options
  const overlayTypes = [
    { value: 'scoreboard', label: 'Live Scoreboard', description: 'Real-time match scoreboard' },
    { value: 'introduction', label: 'Player Introduction', description: 'Player introduction overlay' },
    { value: 'winner', label: 'Winner Announcement', description: 'Winner announcement overlay' },
    { value: 'results', label: 'Previous Results', description: 'Player match history' },
    { value: 'victory', label: 'Victory Ceremony', description: '4-player medal ceremony' },
  ];

  return (
    <div className={`space-y-6 ${className}`}>

      {/* Overlay Type Selection */}
      <div className="p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
        <h4 className="text-md font-semibold text-gray-100 mb-4">Overlay Type</h4>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {overlayTypes.map((type) => (
            <div
              key={type.value}
              className={`p-3 rounded-lg border cursor-pointer transition-all duration-200 ${
                overlaySettings.type === type.value
                  ? 'border-blue-500 bg-blue-900/20'
                  : 'border-gray-600 bg-gray-700/30 hover:bg-gray-700/50'
              }`}
              onClick={() => updateOverlaySettings({ type: type.value as any })}
            >
              <div className="font-medium text-gray-200">{type.label}</div>
              <div className="text-xs text-gray-400">{type.description}</div>
            </div>
          ))}
        </div>
      </div>







      {/* OBS Integration */}
      <div className="p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
        <h4 className="text-md font-semibold text-gray-100 mb-4">OBS Integration</h4>
        <div className="space-y-4">
          {/* HTML Overlay URLs */}
          <div className="p-3 bg-blue-900/20 rounded-lg border border-blue-500/30">
            <Label className="text-sm text-blue-300 mb-2">HTML Overlay URLs (Real-time PSS Updates)</Label>
            <div className="space-y-3">
              <div>
                <Label className="text-xs text-gray-300">Scoreboard Overlay</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    value={`${window.location.origin}/scoreboard-overlay.html`}
                    readOnly
                    className="flex-1 text-xs"
                  />
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => navigator.clipboard.writeText(`${window.location.origin}/scoreboard-overlay.html`)}
                  >
                    Copy
                  </Button>
                </div>
              </div>
              
              <div>
                <Label className="text-xs text-gray-300">Player Introduction Overlay</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    value={`${window.location.origin}/player-introduction-overlay.html`}
                    readOnly
                    className="flex-1 text-xs"
                  />
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => navigator.clipboard.writeText(`${window.location.origin}/player-introduction-overlay.html`)}
                  >
                    Copy
                  </Button>
                </div>
              </div>
            </div>
            <p className="text-xs text-blue-400 mt-2">
              ✨ These HTML overlays support real-time PSS data updates via WebSocket events
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <Label className="text-sm text-gray-300">OBS Settings</Label>
              <div className="text-xs text-gray-400 space-y-1 mt-1">
                <div>Width: 1920</div>
                <div>Height: 1080</div>
                <div>Refresh: When scene becomes active</div>
              </div>
            </div>
            <div>
              <Label className="text-sm text-gray-300">Network Access</Label>
              <div className="text-xs text-gray-400 space-y-1 mt-1">
                <div>Local: http://localhost:3000</div>
                <div>Network: http://[your-ip]:3000</div>
                <div>Use "npm run dev:network" for network access</div>
                <div>WebSocket: ws://[your-ip]:3001</div>
              </div>
            </div>
          </div>
        </div>
      </div>



      {/* PSS Data Status */}
      <div className="p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
        <h4 className="text-md font-semibold text-gray-100 mb-4">PSS Data Status</h4>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <Label className="text-sm text-gray-300">Connection Status</Label>
            <div className="flex items-center space-x-2 mt-1">
              <StatusDot color={isLoaded ? 'green' : 'red'} />
              <span className="text-sm text-gray-200">
                {isLoaded ? 'Connected' : 'Disconnected'}
              </span>
            </div>
          </div>
          
          <div>
            <Label className="text-sm text-gray-300">Current Match</Label>
            <div className="text-sm text-gray-200 mt-1">
              {matchCategory || 'No match loaded'}
            </div>
          </div>

          {isLoaded && (
            <>
              <div>
                <Label className="text-sm text-gray-300">Blue Player</Label>
                <div className="text-sm text-gray-200 mt-1">
                  {athlete1?.long} ({athlete1?.short})
                </div>
              </div>
              
              <div>
                <Label className="text-sm text-gray-300">Red Player</Label>
                <div className="text-sm text-gray-200 mt-1">
                  {athlete2?.long} ({athlete2?.short})
                </div>
              </div>

              {totalScore && (
                <div>
                  <Label className="text-sm text-gray-300">Current Score</Label>
                  <div className="text-sm text-gray-200 mt-1">
                    {totalScore.athlete1} - {totalScore.athlete2}
                  </div>
                </div>
              )}
            </>
          )}
        </div>
      </div>


    </div>
  );
};

export default ScoreboardManager; 