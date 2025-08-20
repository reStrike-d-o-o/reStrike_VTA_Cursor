import React, { useState, useEffect, useRef } from 'react';
import Button from '../atoms/Button';
import Toggle from '../atoms/Toggle';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import StatusDot from '../atoms/StatusDot';
import { usePssMatchStore } from '../../stores/pssMatchStore';
import { useAppStore } from '../../stores';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { useI18n } from '../../i18n/index';

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

interface OverlayTemplate {
  id?: number;
  name: string;
  description?: string;
  theme: string;
  colors?: string;
  animation_type: string;
  duration_ms: number;
  is_active: boolean;
  url?: string;
  created_at: string;
  updated_at: string;
}

const ScoreboardManager: React.FC<ScoreboardManagerProps> = ({ className = '' }) => {
  const { t } = useI18n();
  // Overlay settings state
  const [overlaySettings, setOverlaySettings] = useState<OverlaySettings>({
    type: 'scoreboard',
    visible: false,
  });

  // Overlay templates state
  const [overlayTemplates, setOverlayTemplates] = useState<OverlayTemplate[]>([]);
  const [isLoadingTemplates, setIsLoadingTemplates] = useState(false);

  // PSS data from store - use direct property selectors to avoid infinite loops
  const matchData = usePssMatchStore((state) => state.matchData);
  const athlete1 = matchData.athletes?.athlete1;
  const athlete2 = matchData.athletes?.athlete2;
  const matchNumber = matchData.matchConfig?.number;
  const matchCategory = matchData.matchConfig?.category;
  const matchWeight = matchData.matchConfig?.weight;
  const matchDivision = matchData.matchConfig?.division;
  const totalScore = matchData.currentScores ? {
    athlete1: matchData.currentScores.athlete1_score,
    athlete2: matchData.currentScores.athlete2_score,
  } : undefined;
  const isLoaded = matchData.isLoaded;

  // Store reference
  const { overlaySettings: appOverlaySettings } = useAppStore();

  // Load overlay templates on component mount
  useEffect(() => {
    loadOverlayTemplates();
  }, []);

  // Load overlay templates from database
  const loadOverlayTemplates = async () => {
    try {
      setIsLoadingTemplates(true);
      const templates = await invoke('overlays_sync_templates', { templates: [] });
      setOverlayTemplates(templates as OverlayTemplate[]);
    } catch (error) {
      console.error('Failed to load overlay templates:', error);
    } finally {
      setIsLoadingTemplates(false);
    }
  };

  // Toggle overlay template active status
  const toggleOverlayActive = async (templateId: number, isActive: boolean) => {
    try {
      const updatedTemplates = overlayTemplates.map(template => 
        template.id === templateId ? { ...template, is_active: isActive } : template
      );
      
      await invoke('overlays_sync_templates', { templates: updatedTemplates });
      setOverlayTemplates(updatedTemplates);
    } catch (error) {
      console.error('Failed to update overlay template:', error);
    }
  };

  // Populate overlay templates from files
  const populateOverlayTemplates = async () => {
    try {
      setIsLoadingTemplates(true);
      const templates = await invoke('overlays_populate_from_files');
      setOverlayTemplates(templates as OverlayTemplate[]);
    } catch (error) {
      console.error('Failed to populate overlay templates:', error);
    } finally {
      setIsLoadingTemplates(false);
    }
  };

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
    updateElement(svg, 'matchCategory', matchCategory || "MEN'S -58KG");
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

  return (
    <div className={`space-y-6 ${className}`}>

      {/* Overlay Type Selection */}
      <div className="p-6 theme-card shadow-lg">
        <div className="flex items-center justify-between mb-4">
          <h4 className="text-md font-semibold text-gray-100">{t('ovr.scoreboard.title', 'Overlay Type')}</h4>
          <div className="flex items-center space-x-2">
            <Button
              size="sm"
              variant="primary"
              onClick={populateOverlayTemplates}
              disabled={isLoadingTemplates}
            >
              {isLoadingTemplates ? t('common.loading', 'Loading...') : t('ovr.templates.populate', 'Populate from Files')}
            </Button>
            <Button
              size="sm"
              variant="secondary"
              onClick={loadOverlayTemplates}
              disabled={isLoadingTemplates}
            >
              {isLoadingTemplates ? t('common.loading', 'Loading...') : t('common.refresh', 'Refresh')}
            </Button>
          </div>
        </div>
        
        {isLoadingTemplates ? (
          <div className="text-sm text-gray-400">{t('ovr.templates.loading', 'Loading overlay templates...')}</div>
        ) : overlayTemplates.length === 0 ? (
          <div className="text-sm text-gray-400">{t('ovr.templates.none', 'No overlay templates found')}</div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {overlayTemplates.map((template) => (
              <div
                key={template.id}
                className={`p-3 rounded-lg border transition-all duration-200 ${
                  template.is_active
                    ? 'border-green-500 bg-green-900/20'
                    : 'border-gray-600 bg-gray-700/30'
                }`}
              >
                <div className="flex items-center justify-between">
                  <div className="flex-1">
                    <div className="flex items-center space-x-2">
                      <span className="text-sm font-medium text-gray-200">
                        {template.name}
                      </span>
                      {template.is_active && (
                        <span className="px-2 py-1 bg-green-900/30 text-green-300 text-xs rounded border border-green-600/30">
                          {t('common.active', 'Active')}
                        </span>
                      )}
                    </div>
                    {template.description && (
                      <p className="text-xs text-gray-400 mt-1">{template.description}</p>
                    )}
                    <div className="flex items-center space-x-4 mt-2 text-xs text-gray-500">
                      <span>{t('ovr.templates.theme', 'Theme')}: {template.theme}</span>
                      <span>{t('ovr.templates.animation', 'Animation')}: {template.animation_type}</span>
                      <span>{t('ovr.templates.duration', 'Duration')}: {template.duration_ms}ms</span>
                      {template.url && (
                        <span className="text-blue-400">URL: {template.url}</span>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    <Toggle
                      id={`overlay-${template.id}`}
                      checked={template.is_active}
                      onChange={(e) => toggleOverlayActive(template.id!, e.target.checked)}
                      label={t('common.active', 'Active')}
                      labelPosition="left"
                    />
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* OBS Integration */}
      <div className="p-6 theme-card shadow-lg">
        <h4 className="text-md font-semibold text-gray-100 mb-4">{t('ovr.obs_integration.title', 'OBS Integration')}</h4>
        <div className="space-y-4">
          {/* HTML Overlay URLs */}
          <div className="p-3 bg-blue-900/20 rounded-lg border border-blue-500/30">
            <Label className="text-sm text-blue-300 mb-2">{t('ovr.urls.title', 'HTML Overlay URLs (Real-time PSS Updates)')}</Label>
            <div className="space-y-3">
              <div>
                <Label className="text-xs text-gray-300">{t('ovr.urls.scoreboard', 'Scoreboard Overlay')}</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    value={`${window.location.origin}/scoreboard-overlay.html`}
                    readOnly
                    className="flex-1 text-xs"
                  />
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => window.open(`${window.location.origin}/scoreboard-overlay.html`, '_blank')}
                  >
                    {t('common.open_browser', 'Open in Browser')}
                  </Button>
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => navigator.clipboard.writeText(`${window.location.origin}/scoreboard-overlay.html`)}
                  >
                    {t('common.copy', 'Copy')}
                  </Button>
                </div>
              </div>
              
              <div>
                <Label className="text-xs text-gray-300">{t('ovr.urls.player_intro', 'Player Introduction Overlay')}</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    value={`${window.location.origin}/player-introduction-overlay.html`}
                    readOnly
                    className="flex-1 text-xs"
                  />
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => window.open(`${window.location.origin}/player-introduction-overlay.html`, '_blank')}
                  >
                    {t('common.open_browser', 'Open in Browser')}
                  </Button>
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => navigator.clipboard.writeText(`${window.location.origin}/player-introduction-overlay.html`)}
                  >
                    {t('common.copy', 'Copy')}
                  </Button>
                </div>
              </div>
              
              {/* Arcade Series (Tekken/Street-Fighter style) */}
              <div className="pt-2">
                <Label className="text-xs text-purple-300">{t('ovr.urls.arcade_scoreboard', 'Arcade Scoreboard (New)')}</Label>
                <div className="flex items-center space-x-2">
                  <Input value={`${window.location.origin}/overlays/arcade/scoreboard.html`} readOnly className="flex-1 text-xs" />
                  <Button size="sm" variant="secondary" onClick={() => window.open(`${window.location.origin}/overlays/arcade/scoreboard.html`, '_blank')}>{t('common.open', 'Open')}</Button>
                  <Button size="sm" variant="secondary" onClick={() => navigator.clipboard.writeText(`${window.location.origin}/overlays/arcade/scoreboard.html`)}>{t('common.copy', 'Copy')}</Button>
                </div>
              </div>
              <div>
                <Label className="text-xs text-purple-300">{t('ovr.urls.arcade_intro', 'Arcade Intro')}</Label>
                <div className="flex items-center space-x-2">
                  <Input value={`${window.location.origin}/overlays/arcade/intro.html`} readOnly className="flex-1 text-xs" />
                  <Button size="sm" variant="secondary" onClick={() => window.open(`${window.location.origin}/overlays/arcade/intro.html`, '_blank')}>{t('common.open', 'Open')}</Button>
                  <Button size="sm" variant="secondary" onClick={() => navigator.clipboard.writeText(`${window.location.origin}/overlays/arcade/intro.html`)}>{t('common.copy', 'Copy')}</Button>
                </div>
              </div>
              <div>
                <Label className="text-xs text-purple-300">{t('ovr.urls.arcade_intermission', 'Arcade Intermission Stats')}</Label>
                <div className="flex items-center space-x-2">
                  <Input value={`${window.location.origin}/overlays/arcade/intermission.html`} readOnly className="flex-1 text-xs" />
                  <Button size="sm" variant="secondary" onClick={() => window.open(`${window.location.origin}/overlays/arcade/intermission.html`, '_blank')}>{t('common.open', 'Open')}</Button>
                  <Button size="sm" variant="secondary" onClick={() => navigator.clipboard.writeText(`${window.location.origin}/overlays/arcade/intermission.html`)}>{t('common.copy', 'Copy')}</Button>
                </div>
              </div>
              <div>
                <Label className="text-xs text-purple-300">{t('ovr.urls.arcade_winner', 'Arcade Winner')}</Label>
                <div className="flex items-center space-x-2">
                  <Input value={`${window.location.origin}/overlays/arcade/winner.html`} readOnly className="flex-1 text-xs" />
                  <Button size="sm" variant="secondary" onClick={() => window.open(`${window.location.origin}/overlays/arcade/winner.html`, '_blank')}>{t('common.open', 'Open')}</Button>
                  <Button size="sm" variant="secondary" onClick={() => navigator.clipboard.writeText(`${window.location.origin}/overlays/arcade/winner.html`)}>{t('common.copy', 'Copy')}</Button>
                </div>
              </div>
              <div>
                <Label className="text-xs text-purple-300">{t('ovr.urls.arcade_specials', 'Arcade Specials')}</Label>
                <div className="flex items-center space-x-2">
                  <Input value={`${window.location.origin}/overlays/arcade/specials.html`} readOnly className="flex-1 text-xs" />
                  <Button size="sm" variant="secondary" onClick={() => window.open(`${window.location.origin}/overlays/arcade/specials.html`, '_blank')}>{t('common.open', 'Open')}</Button>
                  <Button size="sm" variant="secondary" onClick={() => navigator.clipboard.writeText(`${window.location.origin}/overlays/arcade/specials.html`)}>{t('common.copy', 'Copy')}</Button>
                </div>
              </div>
            </div>
            <p className="text-xs text-blue-400 mt-2">
              {t('ovr.urls.realtime_note', '✨ These HTML overlays support real-time PSS data updates via WebSocket events')}
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <Label className="text-sm text-gray-300">{t('ovr.obs_settings', 'OBS Settings')}</Label>
              <div className="text-xs text-gray-400 space-y-1 mt-1">
                <div>Width: 1920</div>
                <div>Height: 1080</div>
                <div>{t('ovr.obs_settings.refresh', 'Refresh: When scene becomes active')}</div>
              </div>
            </div>
            <div>
              <Label className="text-sm text-gray-300">{t('ovr.network_access', 'Network Access')}</Label>
              <div className="text-xs text-gray-400 space-y-1 mt-1">
                <div>Local: http://localhost:3000</div>
                <div>Network: http://[your-ip]:3000</div>
                <div>{t('ovr.network_access.tip', 'Use "npm run dev:network" for network access')}</div>
                <div>WebSocket: ws://[your-ip]:3001</div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* PSS Data Status */}
      <div className="p-6 theme-card shadow-lg">
        <h4 className="text-md font-semibold text-gray-100 mb-4">{t('ovr.pss_status.title', 'PSS Data Status')}</h4>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <Label className="text-sm text-gray-300">{t('ovr.pss_status.connection', 'Connection Status')}</Label>
            <div className="flex items-center space-x-2 mt-1">
              <StatusDot color={isLoaded ? 'green' : 'red'} />
              <span className="text-sm text-gray-200">
                {isLoaded ? t('common.connected', 'Connected') : t('common.disconnected', 'Disconnected')}
              </span>
            </div>
          </div>
          
          <div>
            <Label className="text-sm text-gray-300">{t('ovr.pss_status.current_match', 'Current Match')}</Label>
            <div className="text-sm text-gray-200 mt-1">
              {matchCategory || t('ovr.pss_status.no_match', 'No match loaded')}
            </div>
          </div>

          {isLoaded && (
            <>
              <div>
                <Label className="text-sm text-gray-300">{t('ovr.pss_status.blue', 'Blue Player')}</Label>
                <div className="text-sm text-gray-200 mt-1">
                  {athlete1?.long} ({athlete1?.short})
                </div>
              </div>
              
              <div>
                <Label className="text-sm text-gray-300">{t('ovr.pss_status.red', 'Red Player')}</Label>
                <div className="text-sm text-gray-200 mt-1">
                  {athlete2?.long} ({athlete2?.short})
                </div>
              </div>

              {totalScore && (
                <div>
                  <Label className="text-sm text-gray-300">{t('ovr.pss_status.score', 'Current Score')}</Label>
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