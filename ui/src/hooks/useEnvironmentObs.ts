// Environment-aware OBS hook for reStrike VTA

import { useCallback } from 'react';
import { useEnvironment } from './useEnvironment';
import { obsCommands } from '../utils/tauriCommands';
import { ApiResponse, ObsConnection } from '../types';

/**
 * Hook for environment-aware OBS operations
 */
export const useEnvironmentObs = () => {
  const { isWindows, tauriAvailable } = useEnvironment();

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

  return {
    connectToObs,
    disconnectFromObs,
    getObsStatus,
    isWindows,
    tauriAvailable,
  };
}; 