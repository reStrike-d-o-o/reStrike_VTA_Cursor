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
      const tauriAvailable = typeof window !== 'undefined' && !!window.__TAURI__;
      const isWindows = tauriAvailable;
      const isWeb = !isWindows;
      const environment = isWindows ? 'windows' : 'web';

      setEnvironmentInfo({
        environment,
        isWindows,
        isWeb,
        tauriAvailable,
      });
    };

    detectEnvironment();
  }, []);

  return environmentInfo;
}; 