/**
 * Settings panel (demo)
 * - Overlay settings, performance, and data management tabs
 */
import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { useAppStore } from '../../stores';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Toggle from '../atoms/Toggle';
import Label from '../atoms/Label';
import { StatusDot } from '../atoms/StatusDot';
import { Icon } from '../atoms/Icon';

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
    { id: 'overlay', label: 'Overlay Settings', icon: 'overlay' },
    { id: 'performance', label: 'Performance', icon: 'performance' },
    { id: 'data', label: 'Data Management', icon: 'data' },
  ] as const;

  const previewOpacityClass = () => {
    const o = Math.max(0, Math.min(1, overlaySettings.opacity));
    if (o >= 0.95) return 'opacity-100';
    if (o >= 0.85) return 'opacity-90';
    if (o >= 0.75) return 'opacity-80';
    if (o >= 0.65) return 'opacity-70';
    if (o >= 0.55) return 'opacity-60';
    return 'opacity-50';
  };

  const previewScaleClass = () => {
    const s = Math.max(0.5, Math.min(2, overlaySettings.scale));
    if (s >= 1.75) return 'scale-150';
    if (s >= 1.35) return 'scale-125';
    if (s >= 1.15) return 'scale-110';
    if (s >= 0.95) return 'scale-100';
    if (s >= 0.85) return 'scale-90';
    if (s >= 0.65) return 'scale-75';
    return 'scale-50';
  };

  return (
    <div className="p-6 bg-gray-900 text-white rounded-lg">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold">Settings</h2>
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
            {overlaySettings.visible ? 'Hide Overlay' : 'Show Overlay'}
          </Button>
          <Button
            onClick={handleResetSettings}
            variant="secondary"
            size="sm"
            disabled={isResetting}
          >
            {isResetting ? 'Resetting…' : 'Reset to Defaults'}
          </Button>
        </div>
      </div>

      <div className="space-y-8">
        {activeTab === 'overlay' && (
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {/* Opacity */}
              <div>
                <Label htmlFor="overlay-opacity">Opacity</Label>
                <input
                  type="range"
                  id="overlay-opacity"
                  min="0"
                  max="1"
                  step="0.05"
                  value={overlaySettings.opacity}
                  onChange={(e) => updateOverlaySettings({ opacity: parseFloat(e.target.value) })}
                  className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer slider"
                  title="Adjust the overlay opacity"
                />
              </div>

              {/* Position */}
              <div>
                <Label htmlFor="overlay-position">Position</Label>
                <select
                  id="overlay-position"
                  value={overlaySettings.position}
                  onChange={(e) => updateOverlaySettings({ position: e.target.value as any })}
                  className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500"
                  title="Select overlay position"
                >
                  <option value="top-left">Top Left</option>
                  <option value="top-right">Top Right</option>
                  <option value="bottom-left">Bottom Left</option>
                  <option value="bottom-right">Bottom Right</option>
                  <option value="center">Center</option>
                </select>
              </div>

              {/* Scale */}
              <div>
                <Label htmlFor="overlay-scale">
                  Scale: {Math.round(overlaySettings.scale * 100)}%
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
                  title="Adjust the scale of the overlay"
                />
              </div>
            </div>

            {/* Preview */}
            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">Preview</h3>
              <div className="relative w-64 h-48 bg-gray-700 rounded-lg overflow-hidden">
                <div
                  className={`absolute inset-2 rounded border-2 ${overlaySettings.theme === 'light' ? 'bg-white text-gray-900' : 'bg-gray-900 text-white'} transform ${previewOpacityClass()} ${previewScaleClass()}`}
                >
                  <div className="p-2 text-xs">
                    <div>Video Player</div>
                    <div>Status Bar</div>
                    <div>Controls</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'performance' && (
          <div className="space-y-6">
            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">Performance Settings</h3>
              <div className="space-y-4">
                <div>
                  <Label htmlFor="advanced-video-quality">Video Quality</Label>
                  <select id="advanced-video-quality" className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500" title="Settings option" aria-label="Settings option">
                    <option value="high">High Quality</option>
                    <option value="medium">Medium Quality</option>
                    <option value="low">Low Quality</option>
                  </select>
                </div>
                <Toggle
                  id="advanced-hardware-acceleration"
                  checked
                  onChange={() => {}}
                >
                  Enable Hardware Acceleration
                </Toggle>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'data' && (
          <div className="space-y-6">
            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">Data Management</h3>
              <p className="text-sm text-gray-400">Import/export and housekeeping tools will appear here.</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default Settings; 