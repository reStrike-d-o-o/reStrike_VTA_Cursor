// Environment detection hook for reStrike VTA

import { useState, useEffect } from 'react';
import { testTauriApi } from '../utils/tauriCommands';

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
    const detectEnvironment = async () => {
      // Check for Tauri availability
      const tauriAvailable = typeof window !== 'undefined' && !!window.__TAURI__;
      
      // Test Tauri API if available
      let tauriApiWorking = false;
      if (tauriAvailable) {
        tauriApiWorking = await testTauriApi();
      }
      
      // Check if we're in a Tauri context
      const isTauriContext = tauriAvailable && tauriApiWorking;
      
      // For development mode, if we're on localhost:3000, assume we're in Tauri mode
      // This is because Tauri dev server runs React on localhost:3000
      const isWindows = isTauriContext;
      const isWeb = !isWindows;
      const environment = isWindows ? 'windows' : 'web';

      // Debug logging
      console.log('🔍 Environment Detection:', {
        tauriAvailable,
        tauriApiWorking,
        isTauriContext,
        isWindows,
        isWeb,
        environment
      });

      setEnvironmentInfo({
        environment,
        isWindows,
        isWeb,
        tauriAvailable: tauriApiWorking,
      });
    };

    // Initial detection
    detectEnvironment();

    // Set up interval for continuous detection (useful for development)
    const interval = setInterval(detectEnvironment, 5000);

    return () => clearInterval(interval);
  }, []);

  return environmentInfo;
}; 