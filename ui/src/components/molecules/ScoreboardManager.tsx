import React, { useState, useEffect, useRef } from 'react';
import Button from '../atoms/Button';
import Toggle from '../atoms/Toggle';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import StatusDot from '../atoms/StatusDot';
import { usePssMatchStore } from '../../stores/pssMatchStore';
import { useAppStore } from '../../stores';


interface ScoreboardManagerProps {
  className?: string;
}

interface OverlaySettings {
  type: 'scoreboard' | 'introduction' | 'winner' | 'results' | 'victory';
  theme: 'default' | 'olympic' | 'dark' | 'bright';
  visible: boolean;
  showNames: boolean; // true for full names, false for country codes
}

const ScoreboardManager: React.FC<ScoreboardManagerProps> = ({ className = '' }) => {
  // Overlay settings state
  const [overlaySettings, setOverlaySettings] = useState<OverlaySettings>({
    type: 'scoreboard',
    theme: 'default',
    visible: false,
    showNames: true,
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



  // Apply theme to SVG
  const applyThemeToSvg = (svg: SVGElement, theme: string) => {
    const root = svg;
    
    switch (theme) {
      case 'olympic':
        root.style.setProperty('--header-color', '#1e3a8a');
        root.style.setProperty('--accent-color', '#14b8a6');
        break;
      case 'dark':
        root.style.setProperty('--header-color', '#111827');
        root.style.setProperty('--accent-color', '#6b7280');
        break;
      case 'bright':
        root.style.setProperty('--header-color', '#3b82f6');
        root.style.setProperty('--accent-color', '#fbbf24');
        break;
      default:
        root.style.setProperty('--header-color', '#1e3a8a');
        root.style.setProperty('--accent-color', '#14b8a6');
    }
  };

  // Update overlay content with PSS data
  const updateOverlayContent = (svg: SVGElement, settings: OverlaySettings) => {
    if (!isLoaded) return;

    // Update player names/countries
    if (settings.showNames) {
      updateElement(svg, 'bluePlayerName', athlete1?.long || 'BLUE PLAYER');
      updateElement(svg, 'redPlayerName', athlete2?.long || 'RED PLAYER');
    } else {
      updateElement(svg, 'bluePlayerCountry', athlete1?.iocCode || 'BLU');
      updateElement(svg, 'redPlayerCountry', athlete2?.iocCode || 'RED');
    }

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





  // Color theme options
  const themeOptions = [
    { value: 'default', label: 'Default', description: 'Classic blue and teal' },
    { value: 'olympic', label: 'Olympic', description: 'Olympic blue and gold' },
    { value: 'dark', label: 'Dark', description: 'Dark theme with grey accents' },
    { value: 'bright', label: 'Bright', description: 'Bright blue and yellow' },
  ];

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

      {/* Color Theme Selection */}
      <div className="p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
        <h4 className="text-md font-semibold text-gray-100 mb-4">Color Theme</h4>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {themeOptions.map((theme) => (
            <div
              key={theme.value}
              className={`p-3 rounded-lg border cursor-pointer transition-all duration-200 ${
                overlaySettings.theme === theme.value
                  ? 'border-blue-500 bg-blue-900/20'
                  : 'border-gray-600 bg-gray-700/30 hover:bg-gray-700/50'
              }`}
              onClick={() => updateOverlaySettings({ theme: theme.value as any })}
            >
              <div className="font-medium text-gray-200">{theme.label}</div>
              <div className="text-xs text-gray-400">{theme.description}</div>
            </div>
          ))}
        </div>
      </div>

      {/* Display Options */}
      <div className="p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
        <h4 className="text-md font-semibold text-gray-100 mb-4">Display Options</h4>
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <Label className="text-sm text-gray-300">Player Names</Label>
              <p className="text-xs text-gray-400">
                {overlaySettings.showNames ? 'Show full names' : 'Show country codes'}
              </p>
            </div>
            <Toggle
              id="show-names"
              checked={overlaySettings.showNames}
              onChange={(e) => updateOverlaySettings({ showNames: e.target.checked })}
              label=""
              labelPosition="left"
            />
          </div>


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
                  {overlaySettings.showNames ? athlete1?.long : athlete1?.iocCode} ({athlete1?.short})
                </div>
              </div>
              
              <div>
                <Label className="text-sm text-gray-300">Red Player</Label>
                <div className="text-sm text-gray-200 mt-1">
                  {overlaySettings.showNames ? athlete2?.long : athlete2?.iocCode} ({athlete2?.short})
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