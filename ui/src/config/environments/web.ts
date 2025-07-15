// Web Environment Configuration
export const webConfig = {
  environment: 'web' as const,
  api: {
    baseUrl: 'http://localhost:1420',
    timeout: 10000,
  },
  obs: {
    useTauriCommands: false,
    useWebSocketDirect: true,
    defaultPort: 4455,
    defaultHost: 'localhost',
  },
  dev: {
    hotReload: true,
    polling: true,
    port: 3000,
  },
  features: {
    tauriCommands: false,
    webSocketDirect: true,
    nativeFileSystem: false,
    systemTray: false,
    autoUpdate: false,
  }
}; 