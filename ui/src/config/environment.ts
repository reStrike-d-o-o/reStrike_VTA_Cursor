// Global Environment Configuration
// This file controls whether the app runs in web mode or Windows desktop mode

export type Environment = 'web' | 'windows';

// Environment detection and configuration
export class EnvironmentConfig {
  private static instance: EnvironmentConfig;
  private _environment: Environment;
  private _isProduction: boolean;

  private constructor() {
    this._environment = this.detectEnvironment();
    this._isProduction = this.detectProduction();
  }

  public static getInstance(): EnvironmentConfig {
    if (!EnvironmentConfig.instance) {
      EnvironmentConfig.instance = new EnvironmentConfig();
    }
    return EnvironmentConfig.instance;
  }

  // Detect the current environment
  private detectEnvironment(): Environment {
    // Check if we're running in a Tauri environment
    if (typeof window !== 'undefined' && (window as any).__TAURI__) {
      return 'windows';
    }

    // Check for environment variables
    if (process.env.REACT_APP_ENVIRONMENT === 'windows') {
      return 'windows';
    }

    if (process.env.REACT_APP_ENVIRONMENT === 'web') {
      return 'web';
    }

    // Default to web for development
    return 'web';
  }

  // Detect if we're in production
  private detectProduction(): boolean {
    return process.env.NODE_ENV === 'production';
  }

  // Get current environment
  public get environment(): Environment {
    return this._environment;
  }

  // Check if we're in Windows mode
  public get isWindows(): boolean {
    return this._environment === 'windows';
  }

  // Check if we're in web mode
  public get isWeb(): boolean {
    return this._environment === 'web';
  }

  // Check if we're in production
  public get isProduction(): boolean {
    return this._isProduction;
  }

  // Check if we're in development
  public get isDevelopment(): boolean {
    return !this._isProduction;
  }

  // Get environment-specific configuration
  public get config() {
    return {
      environment: this._environment,
      isProduction: this._isProduction,
      isDevelopment: !this._isProduction,
      isWindows: this.isWindows,
      isWeb: this.isWeb,
      
      // API endpoints
      api: {
        baseUrl: this.isWindows ? 'tauri://localhost' : 'http://localhost:1420',
        timeout: this.isWindows ? 30000 : 10000,
      },

      // OBS WebSocket configuration
      obs: {
        useTauriCommands: this.isWindows,
        useWebSocketDirect: this.isWeb,
        defaultPort: this.isWindows ? 4455 : 4455,
        defaultHost: this.isWindows ? 'localhost' : 'localhost',
      },

      // Development settings
      dev: {
        hotReload: this.isWeb,
        polling: this.isWeb,
        port: this.isWeb ? 3000 : 1420,
      },

      // Feature flags
      features: {
        tauriCommands: this.isWindows,
        webSocketDirect: this.isWeb,
        nativeFileSystem: this.isWindows,
        systemTray: this.isWindows,
        autoUpdate: this.isWindows,
      }
    };
  }

  // Override environment (for testing)
  public setEnvironment(env: Environment): void {
    this._environment = env;
  }

  // Get environment info for debugging
  public getInfo() {
    return {
      environment: this._environment,
      isProduction: this._isProduction,
      userAgent: typeof navigator !== 'undefined' ? navigator.userAgent : 'unknown',
      platform: typeof navigator !== 'undefined' ? navigator.platform : 'unknown',
      hasTauri: typeof window !== 'undefined' && !!(window as any).__TAURI__,
      nodeEnv: process.env.NODE_ENV,
      reactAppEnv: process.env.REACT_APP_ENVIRONMENT,
    };
  }
}

// Export singleton instance
export const env = EnvironmentConfig.getInstance();

// Export convenience functions
export const isWindows = () => env.isWindows;
export const isWeb = () => env.isWeb;
export const isProduction = () => env.isProduction;
export const isDevelopment = () => env.isDevelopment;

// Export configuration
export const config = env.config;

// Environment-specific utilities
export const getApiBaseUrl = () => config.api.baseUrl;
export const getObsConfig = () => config.obs;
export const getDevConfig = () => config.dev;
export const getFeatures = () => config.features;

// Tauri command wrapper
export const invokeTauri = async (command: string, args?: any): Promise<any> => {
  if (!isWindows()) {
    throw new Error(`Tauri command '${command}' not available in web environment`);
  }

  if (typeof window !== 'undefined' && (window as any).__TAURI__) {
    return await (window as any).__TAURI__.invoke(command, args);
  }

  throw new Error('Tauri not available');
};

// Environment-aware API calls
export const apiCall = async (endpoint: string, options?: RequestInit): Promise<any> => {
  if (isWindows()) {
    // Use Tauri commands for Windows
    return await invokeTauri(endpoint, options);
  } else {
    // Use HTTP for web
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

// Environment-specific logging
export const log = (message: string, data?: any) => {
  const prefix = `[${env.environment.toUpperCase()}]`;
  if (isDevelopment()) {
    console.log(`${prefix} ${message}`, data || '');
  }
};

export const logError = (message: string, error?: any) => {
  const prefix = `[${env.environment.toUpperCase()}]`;
  console.error(`${prefix} ERROR: ${message}`, error || '');
};

// Environment detection hook for React
export const useEnvironment = () => {
  return {
    environment: env.environment,
    isWindows: env.isWindows,
    isWeb: env.isWeb,
    isProduction: env.isProduction,
    isDevelopment: env.isDevelopment,
    config: env.config,
    info: env.getInfo(),
  };
}; 