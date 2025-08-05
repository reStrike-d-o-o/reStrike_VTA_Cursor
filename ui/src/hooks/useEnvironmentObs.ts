// Environment-aware OBS hook for reStrike VTA

import { useCallback, useEffect, useRef } from 'react';
import { useEnvironment } from './useEnvironment';
import { obsCommands } from '../utils/tauriCommands';
import { ApiResponse, ObsConnection } from '../types';
import { useObsStore } from '../stores/obsStore';

/**
 * Hook for environment-aware OBS operations
 */
export const useEnvironmentObs = () => {
  const { isWindows, tauriAvailable } = useEnvironment();
  const { updateObsStatus } = useObsStore();
  const statusIntervalRef = useRef<NodeJS.Timeout | null>(null);

  const connectToObs = useCallback(async (
    connection: ObsConnection
  ): Promise<ApiResponse> => {
    if (isWindows && tauriAvailable) {
      // Use Tauri OBS commands
      const url = `ws://${connection.host}:${connection.port}`;
      return await obsCommands.connect(url);
    } else {
      // Use web WebSocket
      try {
        const ws = new WebSocket(`ws://${connection.host}:${connection.port}`);
        
        return new Promise((resolve) => {
          ws.onopen = () => resolve({ success: true });
          ws.onerror = () => resolve({ success: false, error: 'WebSocket connection failed' });
        });
      } catch (error) {
        return { success: false, error: String(error) };
      }
    }
  }, [isWindows, tauriAvailable]);

  const disconnectFromObs = useCallback(async (
    connectionName: string
  ): Promise<ApiResponse> => {
    if (isWindows && tauriAvailable) {
      return await obsCommands.disconnect(connectionName);
    } else {
      // Web implementation would close WebSocket
      return { success: true };
    }
  }, [isWindows, tauriAvailable]);

  const getObsStatus = useCallback(async (): Promise<ApiResponse> => {
    if (isWindows && tauriAvailable) {
      return await obsCommands.getStatus();
    } else {
      // Web implementation would query WebSocket
      return { success: true, data: { is_recording: false, is_streaming: false, cpu_usage: 0 } };
    }
  }, [isWindows, tauriAvailable]);

  // Start real-time status polling
  const startStatusPolling = useCallback(() => {
    if (statusIntervalRef.current) {
      clearInterval(statusIntervalRef.current);
    }

    statusIntervalRef.current = setInterval(async () => {
      try {
        const response = await getObsStatus();
        if (response.success && response.data) {
          updateObsStatus(response.data);
        }
      } catch (error) {
        console.error('Failed to fetch OBS status:', error);
      }
    }, 1000); // Poll every second
  }, [getObsStatus, updateObsStatus]);

  // Stop real-time status polling
  const stopStatusPolling = useCallback(() => {
    if (statusIntervalRef.current) {
      clearInterval(statusIntervalRef.current);
      statusIntervalRef.current = null;
    }
  }, []);

  // Setup status listener for real-time updates
  const setupStatusListener = useCallback(async (): Promise<ApiResponse> => {
    console.log('🔧 Setting up OBS status listener...');
    console.log('  - isWindows:', isWindows);
    console.log('  - tauriAvailable:', tauriAvailable);
    
    if (isWindows && tauriAvailable) {
      try {
        console.log('  - Calling obsCommands.setupStatusListener()...');
        const result = await obsCommands.setupStatusListener();
        console.log('  - Result:', result);
        return result;
      } catch (error) {
        console.error('  - Error calling setupStatusListener:', error);
        return { success: false, error: String(error) };
      }
    } else {
      console.log('  - Starting polling instead (web mode)');
      // For web, start polling instead
      startStatusPolling();
      return { success: true };
    }
  }, [isWindows, tauriAvailable, startStatusPolling]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      stopStatusPolling();
    };
  }, [stopStatusPolling]);

  return {
    connectToObs,
    disconnectFromObs,
    getObsStatus,
    setupStatusListener,
    startStatusPolling,
    stopStatusPolling,
    isWindows,
    tauriAvailable,
  };
}; 