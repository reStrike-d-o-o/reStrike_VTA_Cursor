// reStrike VTA - Frontend Library
// Main export file for all frontend modules and utilities

// Core components
export { default as App } from '../App';
export { default as SidebarTest } from '../organisms/SidebarTest';
export { default as Overlay } from '../organisms/Overlay';
export { default as VideoClips } from '../organisms/VideoClips';
export { default as ObsWebSocketManager } from '../organisms/ObsWebSocketManager';
export { default as Settings } from '../organisms/Settings';

// Hooks
export { useAppStore } from '../stores';
export { useEnvironment } from '../hooks/useEnvironment';
export { useEnvironmentApi } from '../hooks/useEnvironmentApi';
export { useEnvironmentObs } from '../hooks/useEnvironmentObs';

// Utilities
export * from '../utils/flagUtils';
export * from '../utils/tauriCommands';
export * from '../utils/videoUtils';
export * from '../utils/obsUtils';

// Types
export * from '../types';

// Constants
export const APP_NAME = 'reStrike VTA';
export const APP_VERSION = '0.1.0';
export const IS_WINDOWS = typeof window !== 'undefined' && window.__TAURI__;

// Application initialization
export const initializeApp = async () => {
  console.log(`🚀 Initializing ${APP_NAME} v${APP_VERSION}`);
  
  if (IS_WINDOWS) {
    console.log('✅ Windows environment detected');
    // Initialize Windows-specific features
  } else {
    console.log('🌐 Web environment detected');
    // Initialize web-specific features
  }
  
  console.log(`✅ ${APP_NAME} initialized successfully`);
}; 