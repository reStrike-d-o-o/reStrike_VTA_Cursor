// Environment detection hook for reStrike VTA

import { useState, useEffect } from 'react';

export interface EnvironmentInfo {
  environment: 'windows' | 'web';
  isWindows: boolean;
  isWeb: boolean;
  tauriAvailable: boolean;
}

/**
 * Hook for detecting the current environment
 */
export const useEnvironment = (): EnvironmentInfo => {
  const [environmentInfo, setEnvironmentInfo] = useState<EnvironmentInfo>({
    environment: 'web',
    isWindows: false,
    isWeb: true,
    tauriAvailable: false,
  });

  useEffect(() => {
    const detectEnvironment = () => {
      // Check for Tauri availability
      const tauriAvailable = typeof window !== 'undefined' && !!window.__TAURI__;
      
      // Check if we're in a Tauri context
      const isTauriContext = tauriAvailable || 
        (typeof window !== 'undefined' && window.location.protocol === 'tauri:') ||
        (typeof window !== 'undefined' && window.location.hostname === 'localhost' && window.location.port === '3000');
      
      // For development mode, if we're on localhost:3000, assume we're in Tauri mode
      // This is because Tauri dev server runs React on localhost:3000
      const isWindows = isTauriContext;
      const isWeb = !isWindows;
      const environment = isWindows ? 'windows' : 'web';

      // Debug logging
      console.log('🔍 Environment Detection:', {
        tauriAvailable,
        isTauriContext,
        windowLocation: typeof window !== 'undefined' ? window.location.href : 'undefined',
        windowTauri: typeof window !== 'undefined' ? !!window.__TAURI__ : 'undefined',
        windowTauriInvoke: typeof window !== 'undefined' && window.__TAURI__ ? !!window.__TAURI__.invoke : 'undefined',
        environment,
        isWindows,
        isWeb,
        isDevMode: typeof window !== 'undefined' && window.location.hostname === 'localhost' && window.location.port === '3000'
      });

      setEnvironmentInfo({
        environment,
        isWindows,
        isWeb,
        tauriAvailable: tauriAvailable, // Use actual Tauri availability, not context detection
      });
    };

    // Initial detection
    detectEnvironment();

    // Re-detect multiple times to ensure Tauri is initialized
    // More aggressive retry schedule for better detection
    const intervals = [50, 100, 200, 500, 1000, 2000, 3000, 5000, 10000];
    const timeouts = intervals.map(delay => setTimeout(detectEnvironment, delay));

    return () => timeouts.forEach(clearTimeout);
  }, []);

  return environmentInfo;
}; 