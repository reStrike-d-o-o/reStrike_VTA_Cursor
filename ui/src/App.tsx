import React, { useEffect } from 'react';
import { motion } from 'framer-motion';
import { useAppStore } from './stores';
import Overlay from './components/Overlay';
import ObsWebSocketManager from './components/ObsWebSocketManager';
import VideoClips from './components/VideoClips';
import Settings from './components/Settings';
import SidebarTest from './components/SidebarTest';

function App() {
  const { currentView, setCurrentView } = useAppStore();

  useEffect(() => {
    console.log('🚀 reStrike VTA - Windows Desktop Application Starting...');
    initializeWindowsFeatures();
  }, []);

  const initializeWindowsFeatures = async () => {
    try {
      if (window.__TAURI__) {
        console.log('✅ Tauri environment detected');
        // Initialize Windows-only features here
      }
    } catch (error) {
      console.error('❌ Failed to initialize Windows features:', error);
    }
  };

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
      default:
        return <SidebarTest />;
    }
  };

  const navigationItems = [
    { id: 'sidebar-test', label: 'Sidebar', icon: '📊' },
    { id: 'overlay', label: 'Overlay', icon: '🎥' },
    { id: 'clips', label: 'Video Clips', icon: '🎬' },
    { id: 'obs-manager', label: 'OBS Manager', icon: '🎛️' },
    { id: 'settings', label: 'Settings', icon: '⚙️' },
  ];

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <header className="bg-gray-800 border-b border-gray-700">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-4">
            <div className="flex items-center space-x-4">
              <h1 className="text-xl font-bold">reStrike VTA - Windows Desktop</h1>
              <span className="px-2 py-1 bg-blue-600 text-xs rounded">Windows Native</span>
            </div>
            <div className="flex items-center space-x-4">
              <span className="text-sm text-gray-400">Windows Desktop Application</span>
            </div>
          </div>
        </div>
      </header>
      <nav className="bg-gray-800 border-b border-gray-700">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex space-x-8">
            {navigationItems.map((item) => (
              <button
                key={item.id}
                onClick={() => setCurrentView(item.id as any)}
                className={`py-4 px-1 border-b-2 font-medium text-sm transition-colors ${
                  currentView === item.id
                    ? 'border-blue-500 text-blue-400'
                    : 'border-transparent text-gray-300 hover:text-gray-100 hover:border-gray-300'
                }`}
              >
                <span className="mr-2">{item.icon}</span>
                {item.label}
              </button>
            ))}
          </div>
        </div>
      </nav>
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <motion.div
          key={currentView}
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3 }}
        >
          {renderCurrentView()}
        </motion.div>
      </main>
    </div>
  );
}

export default App;
