// Environment-aware API hook for reStrike VTA

import { useCallback } from 'react';
import { useEnvironment } from './useEnvironment';
import { executeTauriCommand } from '../utils/tauriCommands';
import { ApiResponse } from '../types';

/**
 * Hook for environment-aware API calls
 */
export const useEnvironmentApi = () => {
  const { isWindows, tauriAvailable } = useEnvironment();

  const apiCall = useCallback(async <T = any>(
    command: string,
    args: any = {}
  ): Promise<ApiResponse<T>> => {
    if (isWindows && tauriAvailable) {
      // Use Tauri commands
      return await executeTauriCommand<T>(command, args);
    } else {
      // Use web API calls
      try {
        const response = await fetch(`/api/${command}`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(args),
        });
        
        const data = await response.json();
        return { success: response.ok, data, error: response.ok ? undefined : data.error };
      } catch (error) {
        return { success: false, error: String(error) };
      }
    }
  }, [isWindows, tauriAvailable]);

  return { apiCall, isWindows, tauriAvailable };
}; 