import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { useAppStore } from '../stores';

const Settings: React.FC = () => {
  const {
    overlaySettings,
    updateOverlaySettings,
    toggleOverlayVisibility,
    obsConnections,
    setError,
    clearError,
  } = useAppStore();

  const [activeTab, setActiveTab] = useState<'overlay' | 'obs' | 'advanced'>('overlay');
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
    { id: 'advanced', label: 'Advanced', icon: '⚙️' },
  ] as const;

  return (
    <div className="p-6 bg-gray-900 text-white rounded-lg">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold">Settings</h2>
        <div className="flex space-x-2">
          <button
            onClick={toggleOverlayVisibility}
            className={`px-4 py-2 rounded-lg transition-colors ${
              overlaySettings.visible
                ? 'bg-green-600 hover:bg-green-700'
                : 'bg-red-600 hover:bg-red-700'
            }`}
          >
            {overlaySettings.visible ? '👁️ Hide Overlay' : '👁️ Show Overlay'}
          </button>
          <button
            onClick={handleResetSettings}
            disabled={isResetting}
            className="bg-gray-600 hover:bg-gray-700 px-4 py-2 rounded-lg transition-colors disabled:opacity-50"
          >
            {isResetting ? '🔄 Resetting...' : '🔄 Reset to Defaults'}
          </button>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="flex space-x-1 mb-6 bg-gray-800 p-1 rounded-lg">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex items-center space-x-2 px-4 py-2 rounded-md transition-colors ${
              activeTab === tab.id
                ? 'bg-blue-600 text-white'
                : 'text-gray-400 hover:text-white hover:bg-gray-700'
            }`}
          >
            <span>{tab.icon}</span>
            <span>{tab.label}</span>
          </button>
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
                <label className="block text-sm font-medium mb-2">Overlay Position</label>
                <select
                  value={overlaySettings.position}
                  onChange={(e) => updateOverlaySettings({ 
                    position: e.target.value as any 
                  })}
                  className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
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
                <label className="block text-sm font-medium mb-2">Theme</label>
                <select
                  value={overlaySettings.theme}
                  onChange={(e) => updateOverlaySettings({ 
                    theme: e.target.value as any 
                  })}
                  className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
                >
                  <option value="dark">Dark</option>
                  <option value="light">Light</option>
                  <option value="transparent">Transparent</option>
                </select>
              </div>

              {/* Opacity */}
              <div>
                <label className="block text-sm font-medium mb-2">
                  Opacity: {Math.round(overlaySettings.opacity * 100)}%
                </label>
                <input
                  type="range"
                  min="0.1"
                  max="1"
                  step="0.1"
                  value={overlaySettings.opacity}
                  onChange={(e) => updateOverlaySettings({ 
                    opacity: parseFloat(e.target.value) 
                  })}
                  className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer slider"
                />
              </div>

              {/* Scale */}
              <div>
                <label className="block text-sm font-medium mb-2">
                  Scale: {Math.round(overlaySettings.scale * 100)}%
                </label>
                <input
                  type="range"
                  min="0.5"
                  max="2"
                  step="0.1"
                  value={overlaySettings.scale}
                  onChange={(e) => updateOverlaySettings({ 
                    scale: parseFloat(e.target.value) 
                  })}
                  className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer slider"
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
              <h3 className="text-lg font-semibold mb-3">OBS Connection Status</h3>
              <div className="space-y-2">
                {obsConnections.length === 0 ? (
                  <p className="text-gray-400">No OBS connections configured</p>
                ) : (
                  obsConnections.map((connection) => (
                    <div
                      key={connection.name}
                      className="flex items-center justify-between p-3 bg-gray-700 rounded-lg"
                    >
                      <div>
                        <div className="font-medium">{connection.name}</div>
                        <div className="text-sm text-gray-400">
                          {connection.host}:{connection.port} ({connection.protocol_version})
                        </div>
                      </div>
                      <div className="flex items-center space-x-2">
                        <span
                          className={`w-3 h-3 rounded-full ${
                            connection.status === 'Connected' || connection.status === 'Authenticated'
                              ? 'bg-green-500'
                              : connection.status === 'Error'
                              ? 'bg-red-500'
                              : 'bg-yellow-500'
                          }`}
                        ></span>
                        <span className="text-sm">{connection.status}</span>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>

            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">OBS Integration Settings</h3>
              <div className="space-y-4">
                <div>
                  <label className="flex items-center">
                    <input
                      type="checkbox"
                      defaultChecked
                      className="mr-2"
                    />
                    <span>Auto-connect to OBS on startup</span>
                  </label>
                </div>
                <div>
                  <label className="flex items-center">
                    <input
                      type="checkbox"
                      defaultChecked
                      className="mr-2"
                    />
                    <span>Show OBS status in overlay</span>
                  </label>
                </div>
                <div>
                  <label className="flex items-center">
                    <input
                      type="checkbox"
                      className="mr-2"
                    />
                    <span>Auto-record when playing clips</span>
                  </label>
                </div>
                <div>
                  <label className="flex items-center">
                    <input
                      type="checkbox"
                      defaultChecked
                      className="mr-2"
                    />
                    <span>Save replay buffer on clip creation</span>
                  </label>
                </div>
              </div>
            </div>

            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">Protocol Information</h3>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                <div>
                  <h4 className="font-medium text-blue-400 mb-2">OBS WebSocket v5</h4>
                  <ul className="text-gray-300 space-y-1">
                    <li>• Default port: 4455</li>
                    <li>• SHA256 authentication</li>
                    <li>• Enhanced features</li>
                  </ul>
                </div>
                <div>
                  <h4 className="font-medium text-yellow-400 mb-2">OBS WebSocket v4</h4>
                  <ul className="text-gray-300 space-y-1">
                    <li>• Default port: 4444</li>
                    <li>• Password authentication</li>
                    <li>• Legacy support</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'advanced' && (
          <div className="space-y-6">
            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">Performance Settings</h3>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium mb-2">Video Quality</label>
                  <select className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500">
                    <option value="high">High Quality</option>
                    <option value="medium">Medium Quality</option>
                    <option value="low">Low Quality</option>
                  </select>
                </div>
                <div>
                  <label className="flex items-center">
                    <input
                      type="checkbox"
                      defaultChecked
                      className="mr-2"
                    />
                    <span>Hardware acceleration</span>
                  </label>
                </div>
                <div>
                  <label className="flex items-center">
                    <input
                      type="checkbox"
                      className="mr-2"
                    />
                    <span>Enable video caching</span>
                  </label>
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
                  <label className="block text-sm font-medium mb-2">Clip Storage Location</label>
                  <input
                    type="text"
                    defaultValue="/home/user/clips"
                    className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500"
                  />
                </div>
                <div className="flex space-x-2">
                  <button className="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-lg transition-colors">
                    Export Settings
                  </button>
                  <button className="bg-green-600 hover:bg-green-700 px-4 py-2 rounded-lg transition-colors">
                    Import Settings
                  </button>
                  <button className="bg-red-600 hover:bg-red-700 px-4 py-2 rounded-lg transition-colors">
                    Clear All Data
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}
      </motion.div>

      {/* Custom CSS for slider */}
      <style jsx>{`
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