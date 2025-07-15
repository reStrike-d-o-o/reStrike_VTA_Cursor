import React, { useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useAppStore } from './stores';
import { useEnvironment } from './hooks/useEnvironment';
import { EnvironmentWrapper } from './components/EnvironmentWrapper';
import Overlay from './components/Overlay';
import ObsWebSocketManager from './components/ObsWebSocketManager';
import VideoClips from './components/VideoClips';
import Settings from './components/Settings';
import SidebarTest from './components/SidebarTest';
import EnvironmentTest from './components/EnvironmentTest';

function App() {
  const {
    currentView,
    setCurrentView,
    overlaySettings,
    isLoading,
    error,
    setError,
    clearError,
    obsConnections,
    videoClips,
    currentClip,
    isPlaying,
  } = useAppStore();

  const { environment, isWindows, isWeb, config } = useEnvironment();

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Toggle overlay visibility
      if (e.ctrlKey && e.shiftKey && e.key === 'O') {
        e.preventDefault();
        useAppStore.getState().toggleOverlayVisibility();
      }

      // Play/pause current clip
      if (e.code === 'Space' && currentClip) {
        e.preventDefault();
        useAppStore.getState().setPlaying(!isPlaying);
      }

      // Toggle fullscreen
      if (e.code === 'KeyF') {
        e.preventDefault();
        // Fullscreen logic handled in Overlay component
      }

      // Navigation shortcuts
      if (e.ctrlKey) {
        switch (e.key) {
          case '1':
            e.preventDefault();
            setCurrentView('sidebar-test');
            break;
          case '2':
            e.preventDefault();
            setCurrentView('overlay');
            break;
          case '3':
            e.preventDefault();
            setCurrentView('clips');
            break;
          case '4':
            e.preventDefault();
            setCurrentView('obs-manager');
            break;
          case '5':
            e.preventDefault();
            setCurrentView('settings');
            break;
          case '6':
            e.preventDefault();
            setCurrentView('environment-test');
            break;
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [currentView, setCurrentView, currentClip, isPlaying]);

  // Auto-hide error after 5 seconds
  useEffect(() => {
    if (error) {
      const timer = setTimeout(() => clearError(), 5000);
      return () => clearTimeout(timer);
    }
  }, [error, clearError]);

  const navigationItems = [
    { id: 'sidebar-test', label: 'Sidebar Test', icon: '🧪', shortcut: 'Ctrl+1' },
    { id: 'overlay', label: 'Overlay', icon: '🎬', shortcut: 'Ctrl+2' },
    { id: 'clips', label: 'Video Clips', icon: '📁', shortcut: 'Ctrl+3' },
    { id: 'obs-manager', label: 'OBS Manager', icon: '🎥', shortcut: 'Ctrl+4' },
    { id: 'settings', label: 'Settings', icon: '⚙️', shortcut: 'Ctrl+5' },
    { id: 'environment-test', label: 'Environment Test', icon: '🌐', shortcut: 'Ctrl+6' },
  ] as const;

  const renderCurrentView = () => {
    switch (currentView) {
      case 'sidebar-test':
        return <SidebarTest />;
      case 'overlay':
        return <Overlay />;
      case 'clips':
        return <VideoClips />;
      case 'obs-manager':
        return <ObsWebSocketManager />;
      case 'settings':
        return <Settings />;
      case 'environment-test':
        return <EnvironmentTest />;
      default:
        return <SidebarTest />;
    }
  };

  return (
    <div className="min-h-screen bg-gray-950 text-white">
      {/* Error Toast */}
      {error && (
        <motion.div
          initial={{ opacity: 0, y: -50 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -50 }}
          className="fixed top-4 right-4 z-50 bg-red-600 text-white px-6 py-3 rounded-lg shadow-lg max-w-md"
        >
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <span>🚨</span>
              <span>{error}</span>
            </div>
            <button
              onClick={clearError}
              className="ml-4 text-white hover:text-gray-200"
            >
              ✕
            </button>
          </div>
        </motion.div>
      )}

      {/* Loading Overlay */}
      {isLoading && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
        >
          <div className="bg-gray-900 p-6 rounded-lg flex items-center space-x-3">
            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-500"></div>
            <span>Loading...</span>
          </div>
        </motion.div>
      )}

      {/* Header */}
      <header className="bg-gray-900 border-b border-gray-800">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            {/* Logo and Title */}
            <div className="flex items-center space-x-3">
              <div className="text-2xl">🎯</div>
              <div>
                <h1 className="text-xl font-bold">reStrike VTA Overlay</h1>
                <div className="flex items-center space-x-2">
                  <p className="text-sm text-gray-400">Overlay & Automation Toolkit</p>
                  <span className={`px-2 py-1 text-xs rounded-full font-medium ${
                    isWindows 
                      ? 'bg-blue-600 text-white' 
                      : 'bg-green-600 text-white'
                  }`}>
                    {environment.toUpperCase()}
                  </span>
                  {config.isDevelopment && (
                    <span className="px-2 py-1 text-xs rounded-full font-medium bg-yellow-600 text-black">
                      DEV
                    </span>
                  )}
                </div>
              </div>
            </div>

            {/* Navigation */}
            <nav className="flex space-x-1">
              {navigationItems.map((item) => (
                <button
                  key={item.id}
                  onClick={() => setCurrentView(item.id)}
                  className={`flex items-center space-x-2 px-4 py-2 rounded-lg transition-colors ${
                    currentView === item.id
                      ? 'bg-blue-600 text-white'
                      : 'text-gray-400 hover:text-white hover:bg-gray-800'
                  }`}
                  title={`${item.label} (${item.shortcut})`}
                >
                  <span>{item.icon}</span>
                  <span className="hidden md:block">{item.label}</span>
                  <span className="hidden lg:block text-xs opacity-75">({item.shortcut})</span>
                </button>
              ))}
            </nav>

            {/* Status Indicators */}
            <div className="flex items-center space-x-4">
              {/* OBS Status */}
              <div className="flex items-center space-x-2">
                <span className="text-sm text-gray-400">OBS:</span>
                <div className="flex space-x-1">
                  {obsConnections.map((connection) => (
                    <span
                      key={connection.name}
                      className={`w-2 h-2 rounded-full ${
                        connection.status === 'Connected' || connection.status === 'Authenticated'
                          ? 'bg-green-500'
                          : connection.status === 'Error'
                          ? 'bg-red-500'
                          : 'bg-yellow-500'
                      }`}
                      title={`${connection.name}: ${connection.status}`}
                    />
                  ))}
                </div>
              </div>

              {/* Video Status */}
              {currentClip && (
                <div className="flex items-center space-x-2">
                  <span className="text-sm text-gray-400">Video:</span>
                  <span className="text-sm font-medium">{currentClip.name}</span>
                  {isPlaying && <span className="animate-pulse">▶️</span>}
                </div>
              )}

              {/* Overlay Visibility Toggle */}
              <button
                onClick={() => useAppStore.getState().toggleOverlayVisibility()}
                className={`p-2 rounded-lg transition-colors ${
                  overlaySettings.visible
                    ? 'bg-green-600 hover:bg-green-700'
                    : 'bg-red-600 hover:bg-red-700'
                }`}
                title={`${overlaySettings.visible ? 'Hide' : 'Show'} Overlay (Ctrl+Shift+O)`}
              >
                {overlaySettings.visible ? '👁️' : '👁️‍🗨️'}
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
        <motion.div
          key={currentView}
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -20 }}
          transition={{ duration: 0.2 }}
        >
          {renderCurrentView()}
        </motion.div>
      </main>

      {/* Footer */}
      <footer className="bg-gray-900 border-t border-gray-800 mt-auto">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <div className="flex items-center justify-between text-sm text-gray-400">
            <div className="flex items-center space-x-4">
              <span>reStrike VTA v1.0.0</span>
              <span>•</span>
              <span>Clips: {videoClips.length}</span>
              <span>•</span>
              <span>OBS: {obsConnections.filter(c => c.status === 'Connected').length}/{obsConnections.length}</span>
            </div>
            <div className="flex items-center space-x-4">
              <span>Press Ctrl+Shift+O to toggle overlay</span>
              <span>•</span>
              <span>Space to play/pause</span>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}

export default App;
