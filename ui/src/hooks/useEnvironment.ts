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
      // If window.__TAURI__ exists, we're likely in native mode
      // The API test is just a verification, but the presence of __TAURI__ is the main indicator
      const isTauriContext = tauriAvailable;
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
        tauriAvailable: tauriAvailable,
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