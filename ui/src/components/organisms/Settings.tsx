import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { useAppStore } from '../../stores';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Toggle from '../atoms/Toggle';
import Label from '../atoms/Label';
import { StatusDot } from '../atoms/StatusDot';
import { Icon } from '../atoms/Icon';
import { useI18n } from '../../i18n/index';

const Settings: React.FC = () => {
  const {
    overlaySettings,
    updateOverlaySettings,
    toggleOverlayVisibility,
    obsConnections,
    setError,
    clearError,
  } = useAppStore();

  const [activeTab, setActiveTab] = useState<'overlay' | 'performance' | 'data'>('overlay');
  const { t } = useI18n();
  const [isResetting, setIsResetting] = useState(false);

  const handleResetSettings = () => {
    setIsResetting(true);
    // Reset to default settings
    updateOverlaySettings({
      opacity: 0.9,
      position: 'bottom-right',
      scale: 1.0,
      visible: true,
      theme: 'dark',
    });
    setTimeout(() => setIsResetting(false), 1000);
  };

  const tabs = [
    { id: 'overlay', label: t('demo.settings.tab.overlay', 'Overlay Settings'), icon: 'overlay' },
    { id: 'performance', label: t('demo.settings.tab.performance', 'Performance'), icon: 'performance' },
    { id: 'data', label: t('demo.settings.tab.data', 'Data Management'), icon: 'data' },
  ] as const;

  return (
    <div className="p-6 bg-gray-900 text-white rounded-lg">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold">{t('demo.settings.title', 'Settings')}</h2>
        <div className="flex space-x-2">
          <Button
            onClick={toggleOverlayVisibility}
            variant={overlaySettings.visible ? 'success' : 'danger'}
            size="sm"
          >
            <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
            </svg>
            {overlaySettings.visible ? t('demo.settings.hide_overlay', 'Hide Overlay') : t('demo.settings.show_overlay', 'Show Overlay')}
          </Button>
          <Button
            onClick={handleResetSettings}
            disabled={isResetting}
            variant="secondary"
            size="sm"
          >
            {isResetting ? (
              <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor" className="animate-spin">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
            ) : (
              <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
            )}
            {isResetting ? t('common.resetting', 'Resetting...') : t('demo.settings.reset_defaults', 'Reset to Defaults')}
          </Button>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="flex space-x-1 mb-6 bg-gray-800 p-1 rounded-lg">
        {tabs.map((tab) => (
          <Button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            variant={activeTab === tab.id ? 'primary' : 'secondary'}
            size="sm"
          >
            {tab.icon === 'overlay' && (
              <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zM21 5a2 2 0 00-2-2h-4a2 2 0 00-2 2v12a4 4 0 004 4h4a2 2 0 002-2V5z" />
              </svg>
            )}
            {tab.icon === 'performance' && (
              <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
            )}
            {tab.icon === 'data' && (
              <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
              </svg>
            )}
            <span>{tab.label}</span>
          </Button>
        ))}
      </div>

      {/* Tab Content */}
      <motion.div
        key={activeTab}
        initial={{ opacity: 0, x: 20 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.2 }}
      >
        {activeTab === 'overlay' && (
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {/* Position */}
              <div>
                <Label htmlFor="overlay-position">{t('demo.settings.overlay.position', 'Overlay Position')}</Label>
                <select
                  id="overlay-position"
                  value={overlaySettings.position}
                  onChange={(e) => updateOverlaySettings({ 
                    position: e.target.value as any 
                  })}
                  className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
                  title={t('demo.settings.option', 'Settings option')}
                  aria-label={t('demo.settings.option', 'Settings option')}
                >
                  <option value="top-left">{t('demo.settings.pos.top_left', 'Top Left')}</option>
                  <option value="top-right">{t('demo.settings.pos.top_right', 'Top Right')}</option>
                  <option value="bottom-left">{t('demo.settings.pos.bottom_left', 'Bottom Left')}</option>
                  <option value="bottom-right">{t('demo.settings.pos.bottom_right', 'Bottom Right')}</option>
                  <option value="center">{t('demo.settings.pos.center', 'Center')}</option>
                </select>
              </div>

              {/* Theme */}
              <div>
                <Label htmlFor="overlay-theme">{t('demo.settings.overlay.theme', 'Theme')}</Label>
                <select
                  id="overlay-theme"
                  value={overlaySettings.theme}
                  onChange={(e) => updateOverlaySettings({ 
                    theme: e.target.value as any 
                  })}
                  className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
                  title={t('demo.settings.option', 'Settings option')}
                  aria-label={t('demo.settings.option', 'Settings option')}
                >
                  <option value="dark">{t('demo.settings.theme.dark', 'Dark')}</option>
                  <option value="light">{t('demo.settings.theme.light', 'Light')}</option>
                  <option value="transparent">{t('demo.settings.theme.transparent', 'Transparent')}</option>
                </select>
              </div>

              {/* Opacity */}
              <div>
                <Label htmlFor="overlay-opacity">
                  {t('demo.settings.overlay.opacity', 'Opacity')}: {Math.round(overlaySettings.opacity * 100)}%
                </Label>
                <input
                  type="range"
                  id="overlay-opacity"
                  min="0.1"
                  max="1"
                  step="0.1"
                  value={overlaySettings.opacity}
                  onChange={(e) => updateOverlaySettings({ 
                    opacity: parseFloat(e.target.value) 
                  })}
                  className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer slider"
                  title={t('demo.settings.overlay.opacity_tip', 'Adjust the opacity of the overlay')}
                />
              </div>

              {/* Scale */}
              <div>
                <Label htmlFor="overlay-scale">
                  {t('demo.settings.overlay.scale', 'Scale')}: {Math.round(overlaySettings.scale * 100)}%
                </Label>
                <input
                  type="range"
                  id="overlay-scale"
                  min="0.5"
                  max="2"
                  step="0.1"
                  value={overlaySettings.scale}
                  onChange={(e) => updateOverlaySettings({ 
                    scale: parseFloat(e.target.value) 
                  })}
                  className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer slider"
                  title={t('demo.settings.overlay.scale_tip', 'Adjust the scale of the overlay')}
                />
              </div>
            </div>

            {/* Preview */}
            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">{t('demo.settings.preview', 'Preview')}</h3>
              <div className="relative w-64 h-48 bg-gray-700 rounded-lg overflow-hidden">
                <div
                  className={`absolute inset-2 rounded border-2 ${overlaySettings.theme === 'light' ? 'bg-white text-gray-900' : 'bg-gray-900 text-white'}`}
                  style={{
                    opacity: overlaySettings.opacity,
                    transform: `scale(${overlaySettings.scale})`,
                  }}
                >
                  <div className="p-2 text-xs">
                    <div>{t('demo.settings.preview.player', 'Video Player')}</div>
                    <div>{t('demo.settings.preview.status', 'Status Bar')}</div>
                    <div>{t('demo.settings.preview.controls', 'Controls')}</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'performance' && (
          <div className="space-y-6">
            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">{t('demo.settings.performance.title', 'Performance Settings')}</h3>
              <div className="space-y-4">
                <div>
                  <Label htmlFor="advanced-video-quality">{t('demo.settings.performance.quality', 'Video Quality')}</Label>
                  <select id="advanced-video-quality" className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500" title={t('demo.settings.option', 'Settings option')} aria-label={t('demo.settings.option', 'Settings option')}>
                    <option value="high">{t('demo.settings.quality.high', 'High Quality')}</option>
                    <option value="medium">{t('demo.settings.quality.medium', 'Medium Quality')}</option>
                    <option value="low">{t('demo.settings.quality.low', 'Low Quality')}</option>
                  </select>
                </div>
                <Toggle
                  id="advanced-hardware-acceleration"
                  checked={true}
                  onChange={() => {}}
                  label={t('demo.settings.performance.hw', 'Hardware acceleration')}
                  labelPosition="right"
                />
                <Toggle
                  id="advanced-video-caching"
                  checked={false}
                  onChange={() => {}}
                  label={t('demo.settings.performance.caching', 'Enable video caching')}
                  labelPosition="right"
                />
              </div>
            </div>

            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">{t('demo.settings.shortcuts.title', 'Keyboard Shortcuts')}</h3>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span>{t('demo.settings.shortcuts.toggle_overlay', 'Toggle overlay visibility')}</span>
                  <span className="text-gray-400">Ctrl + Shift + O</span>
                </div>
                <div className="flex justify-between">
                  <span>{t('demo.settings.shortcuts.play_pause', 'Play/pause current clip')}</span>
                  <span className="text-gray-400">Space</span>
                </div>
                <div className="flex justify-between">
                  <span>{t('demo.settings.shortcuts.next', 'Next clip')}</span>
                  <span className="text-gray-400">Right Arrow</span>
                </div>
                <div className="flex justify-between">
                  <span>{t('demo.settings.shortcuts.prev', 'Previous clip')}</span>
                  <span className="text-gray-400">Left Arrow</span>
                </div>
                <div className="flex justify-between">
                  <span>{t('demo.settings.shortcuts.fullscreen', 'Toggle fullscreen')}</span>
                  <span className="text-gray-400">F</span>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'data' && (
          <div className="space-y-6">
            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">{t('demo.settings.data.title', 'Data Management')}</h3>
              <div className="space-y-4">
                <div>
                  <Label htmlFor="data-clip-location">{t('demo.settings.data.clip_location', 'Clip Storage Location')}</Label>
                  <input
                    type="text"
                    id="data-clip-location"
                    defaultValue="/home/user/clips"
                    className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500"
                    title={t('demo.settings.data.clip_location_tip', 'Enter the location where clips are stored')}
                  />
                </div>
                <div className="flex space-x-2">
                  <Button variant="primary" size="sm">
                    {t('demo.settings.data.export', 'Export Settings')}
                  </Button>
                  <Button variant="success" size="sm">
                    {t('demo.settings.data.import', 'Import Settings')}
                  </Button>
                  <Button variant="danger" size="sm">
                    {t('demo.settings.data.clear', 'Clear All Data')}
                  </Button>
                </div>
              </div>
            </div>
          </div>
        )}
      </motion.div>

      {/* Custom CSS for slider */}
      <style>{`
        .slider::-webkit-slider-thumb {
          appearance: none;
          height: 16px;
          width: 16px;
          border-radius: 50%;
          background: #3b82f6;
          cursor: pointer;
        }
        
        .slider::-moz-range-thumb {
          height: 16px;
          width: 16px;
          border-radius: 50%;
          background: #3b82f6;
          cursor: pointer;
          border: none;
        }
      `}</style>
    </div>
  );
};

export default Settings; 