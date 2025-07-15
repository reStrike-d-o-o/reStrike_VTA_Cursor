import { useState, useEffect } from 'react';
import { env, useEnvironment as useEnvConfig } from '../config/environment';

// React hook for environment-aware components
export const useEnvironment = () => {
  const [isClient, setIsClient] = useState(false);
  const envConfig = useEnvConfig();

  useEffect(() => {
    setIsClient(true);
  }, []);

  return {
    ...envConfig,
    isClient,
    // Environment-specific utilities
    canUseTauri: isClient && envConfig.isWindows,
    canUseWebSocket: isClient && envConfig.isWeb,
    canUseNativeFeatures: isClient && envConfig.isWindows,
    
    // Environment-specific rendering
    renderForEnvironment: (windowsComponent: React.ReactNode, webComponent: React.ReactNode) => {
      if (!isClient) return null;
      return envConfig.isWindows ? windowsComponent : webComponent;
    },
    
    // Conditional feature rendering
    renderIfWindows: (component: React.ReactNode) => {
      if (!isClient || !envConfig.isWindows) return null;
      return component;
    },
    
    renderIfWeb: (component: React.ReactNode) => {
      if (!isClient || !envConfig.isWeb) return null;
      return component;
    },
    
    // Environment-specific class names
    getEnvironmentClass: (baseClass: string) => {
      return `${baseClass} ${baseClass}--${envConfig.environment}`;
    },
    
    // Environment-specific styles
    getEnvironmentStyles: (baseStyles: React.CSSProperties, webOverrides?: React.CSSProperties, windowsOverrides?: React.CSSProperties) => {
      if (envConfig.isWeb && webOverrides) {
        return { ...baseStyles, ...webOverrides };
      }
      if (envConfig.isWindows && windowsOverrides) {
        return { ...baseStyles, ...windowsOverrides };
      }
      return baseStyles;
    }
  };
};

// Hook for environment-specific API calls
export const useEnvironmentApi = () => {
  const { isWindows, isWeb } = useEnvironment();
  
  const apiCall = async (endpoint: string, options?: RequestInit): Promise<any> => {
    if (isWindows) {
      // Use Tauri commands for Windows
      const { invokeTauri } = await import('../config/environment');
      return await invokeTauri(endpoint, options);
    } else {
      // Use HTTP for web
      const { getApiBaseUrl } = await import('../config/environment');
      const response = await fetch(`${getApiBaseUrl()}${endpoint}`, {
        ...options,
        headers: {
          'Content-Type': 'application/json',
          ...options?.headers,
        },
      });
      
      if (!response.ok) {
        throw new Error(`API call failed: ${response.statusText}`);
      }
      
      return await response.json();
    }
  };

  return { apiCall };
};

// Hook for environment-specific OBS operations
export const useEnvironmentObs = () => {
  const { isWindows, isWeb } = useEnvironment();
  
  const obsOperation = async (operation: string, params?: any): Promise<any> => {
    if (isWindows) {
      // Use Tauri commands for Windows
      const { invokeTauri } = await import('../config/environment');
      return await invokeTauri(`obs_${operation}`, params);
    } else {
      // Use direct WebSocket for web
      // This would need to be implemented based on your WebSocket setup
      throw new Error(`OBS operation '${operation}' not implemented for web environment`);
    }
  };

  return { obsOperation };
};

// Hook for environment-specific file operations
export const useEnvironmentFileSystem = () => {
  const { isWindows, isWeb } = useEnvironment();
  
  const fileOperation = async (operation: string, params?: any): Promise<any> => {
    if (isWindows) {
      // Use Tauri commands for Windows
      const { invokeTauri } = await import('../config/environment');
      return await invokeTauri(`file_${operation}`, params);
    } else {
      // Use browser APIs for web
      switch (operation) {
        case 'read':
          // Use FileReader API
          return new Promise((resolve, reject) => {
            const input = document.createElement('input');
            input.type = 'file';
            input.onchange = (e) => {
              const file = (e.target as HTMLInputElement).files?.[0];
              if (file) {
                const reader = new FileReader();
                reader.onload = () => resolve(reader.result);
                reader.onerror = reject;
                reader.readAsText(file);
              } else {
                reject(new Error('No file selected'));
              }
            };
            input.click();
          });
        case 'save':
          // Use download link
          const { data, filename } = params;
          const blob = new Blob([data], { type: 'text/plain' });
          const url = URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = filename;
          a.click();
          URL.revokeObjectURL(url);
          return { success: true };
        default:
          throw new Error(`File operation '${operation}' not implemented for web environment`);
      }
    }
  };

  return { fileOperation };
}; 