import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { useAppStore } from '../../stores';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Checkbox from '../atoms/Checkbox';
import Label from '../atoms/Label';
import { StatusDot } from '../atoms/StatusDot';
import { Icon } from '../atoms/Icon';
import WebSocketManager from '../molecules/WebSocketManager';

const Settings: React.FC = () => {
  const {
    overlaySettings,
    updateOverlaySettings,
    toggleOverlayVisibility,
    obsConnections,
    setError,
    clearError,
  } = useAppStore();

  const [activeTab, setActiveTab] = useState<'overlay' | 'obs' | 'websocket' | 'advanced'>('overlay');
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
    { id: 'overlay', label: 'Overlay Settings', icon: '🎨' },
    { id: 'obs', label: 'OBS Integration', icon: '🎥' },
    { id: 'websocket', label: 'WebSocket', icon: '🔌' },
    { id: 'advanced', label: 'Advanced', icon: '⚙️' },
  ] as const;

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
            <Icon name="👁️" /> {overlaySettings.visible ? 'Hide Overlay' : 'Show Overlay'}
          </Button>
          <Button
            onClick={handleResetSettings}
            disabled={isResetting}
            variant="secondary"
            size="sm"
          >
            {isResetting ? '🔄 Resetting...' : '🔄 Reset to Defaults'}
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
            <Icon name={tab.icon} />
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
                <Label htmlFor="overlay-position">Overlay Position</Label>
                <select
                  id="overlay-position"
                  value={overlaySettings.position}
                  onChange={(e) => updateOverlaySettings({ 
                    position: e.target.value as any 
                  })}
                  className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
                  title="Settings option"
                  aria-label="Settings option"
                >
                  <option value="top-left">Top Left</option>
                  <option value="top-right">Top Right</option>
                  <option value="bottom-left">Bottom Left</option>
                  <option value="bottom-right">Bottom Right</option>
                  <option value="center">Center</option>
                </select>
              </div>

              {/* Theme */}
              <div>
                <Label htmlFor="overlay-theme">Theme</Label>
                <select
                  id="overlay-theme"
                  value={overlaySettings.theme}
                  onChange={(e) => updateOverlaySettings({ 
                    theme: e.target.value as any 
                  })}
                  className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
                  title="Settings option"
                  aria-label="Settings option"
                >
                  <option value="dark">Dark</option>
                  <option value="light">Light</option>
                  <option value="transparent">Transparent</option>
                </select>
              </div>

              {/* Opacity */}
              <div>
                <Label htmlFor="overlay-opacity">
                  Opacity: {Math.round(overlaySettings.opacity * 100)}%
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
                  title="Adjust the opacity of the overlay"
                />
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
                  className={`absolute inset-2 rounded border-2 ${overlaySettings.theme === 'light' ? 'bg-white text-gray-900' : 'bg-gray-900 text-white'}`}
                  style={{
                    opacity: overlaySettings.opacity,
                    transform: `scale(${overlaySettings.scale})`,
                  }}
                >
                  <div className="p-2 text-xs">
                    <div>🎬 Video Player</div>
                    <div>📊 Status Bar</div>
                    <div>🎮 Controls</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'obs' && (
          <div className="space-y-6">
            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">OBS Integration Settings</h3>
              <div className="space-y-4">
                <div>
                  <Label htmlFor="obs-auto-connect">
                    <Checkbox
                      checked={true}
                      onChange={() => {}}
                    />
                    <span>Auto-connect to OBS on startup</span>
                  </Label>
                </div>
                <div>
                  <Label htmlFor="obs-show-status">
                    <Checkbox
                      checked={true}
                      onChange={() => {}}
                    />
                    <span>Show OBS status in overlay</span>
                  </Label>
                </div>
                <div>
                  <Label htmlFor="obs-auto-record">
                    <Checkbox
                      checked={false}
                      onChange={() => {}}
                    />
                    <span>Auto-record when playing clips</span>
                  </Label>
                </div>
                <div>
                  <Label htmlFor="obs-save-replay">
                    <Checkbox
                      checked={true}
                      onChange={() => {}}
                    />
                    <span>Save replay buffer on clip creation</span>
                  </Label>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'websocket' && (
          <WebSocketManager />
        )}

        {activeTab === 'advanced' && (
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
                <div>
                  <Label htmlFor="advanced-hardware-acceleration">
                    <Checkbox
                      checked={true}
                      onChange={() => {}}
                    />
                    <span>Hardware acceleration</span>
                  </Label>
                </div>
                <div>
                  <Label htmlFor="advanced-video-caching">
                    <Checkbox
                      checked={false}
                      onChange={() => {}}
                    />
                    <span>Enable video caching</span>
                  </Label>
                </div>
              </div>
            </div>

            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">Keyboard Shortcuts</h3>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span>Toggle overlay visibility</span>
                  <span className="text-gray-400">Ctrl + Shift + O</span>
                </div>
                <div className="flex justify-between">
                  <span>Play/pause current clip</span>
                  <span className="text-gray-400">Space</span>
                </div>
                <div className="flex justify-between">
                  <span>Next clip</span>
                  <span className="text-gray-400">Right Arrow</span>
                </div>
                <div className="flex justify-between">
                  <span>Previous clip</span>
                  <span className="text-gray-400">Left Arrow</span>
                </div>
                <div className="flex justify-between">
                  <span>Toggle fullscreen</span>
                  <span className="text-gray-400">F</span>
                </div>
              </div>
            </div>

            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">Data Management</h3>
              <div className="space-y-4">
                <div>
                  <Label htmlFor="data-clip-location">Clip Storage Location</Label>
                  <input
                    type="text"
                    id="data-clip-location"
                    defaultValue="/home/user/clips"
                    className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500"
                    title="Enter the location where clips are stored"
                  />
                </div>
                <div className="flex space-x-2">
                  <Button variant="primary" size="sm">
                    Export Settings
                  </Button>
                  <Button variant="success" size="sm">
                    Import Settings
                  </Button>
                  <Button variant="danger" size="sm">
                    Clear All Data
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