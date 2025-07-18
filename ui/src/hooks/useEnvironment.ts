// Environment detection hook for reStrike VTA

import { useState, useEffect } from 'react';
// Tauri v2 API import
import { invoke } from '@tauri-apps/api/core';

// Fallback invoke function for compatibility
const safeInvoke = async (command: string, args?: any) => {
  try {
    // Try the proper Tauri v2 API first
    return await invoke(command, args);
  } catch (error) {
    // If that fails, try the global window.__TAURI__.invoke
    if (typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.invoke) {
      return await window.__TAURI__.invoke(command, args);
    }
    throw new Error('Tauri invoke method not available - ensure app is running in desktop mode');
  }
};

export interface EnvironmentInfo {
  tauriAvailable: boolean;
  isLoading: boolean;
  environment: 'windows' | 'web';
  isWindows: boolean;
  isWeb: boolean;
}

export const useEnvironment = (): EnvironmentInfo => {
  const [tauriAvailable, setTauriAvailable] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

    useEffect(() => {
    const checkTauriAvailability = async () => {
      try {
        console.log('🔍 Checking Tauri availability...');
        console.log('window.__TAURI__:', typeof window !== 'undefined' ? window.__TAURI__ : 'window not available');
        
        // Check if Tauri API is available
        if (typeof window !== 'undefined' && window.__TAURI__) {
          console.log('✅ Tauri global object found, testing command...');
          // Test Tauri command invocation using the proper Tauri v2 API
          const result = await safeInvoke('get_app_status');
          console.log('✅ Tauri command successful:', result);
          setTauriAvailable(true);
        } else {
          console.log('❌ Tauri global object not found');
          setTauriAvailable(false);
        }
      } catch (error) {
        console.warn('❌ Tauri API not available:', error);
        setTauriAvailable(false);
      } finally {
        setIsLoading(false);
      }
    };

    checkTauriAvailability();
  }, []);

  return {
    tauriAvailable,
    isLoading,
    environment: tauriAvailable ? 'windows' : 'web',
    isWindows: tauriAvailable,
    isWeb: !tauriAvailable && !isLoading
  };
}; 