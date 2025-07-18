// Environment detection hook for reStrike VTA

import { useState, useEffect } from 'react';
// Tauri v2 invoke function that uses the core module
const safeInvoke = async (command: string, args?: any) => {
  try {
    // Check if the global Tauri object is available
    if (typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.core) {
      console.log('✅ Using Tauri v2 core module for command:', command);
      // In Tauri v2, invoke is available through the core module
      return await window.__TAURI__.core.invoke(command, args);
    }
    
    throw new Error('Tauri v2 core module not available - ensure app is running in desktop mode');
  } catch (error) {
    console.error('Tauri invoke failed:', error);
    throw error;
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