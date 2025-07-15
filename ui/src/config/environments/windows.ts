// Windows Environment Configuration
export const windowsConfig = {
  environment: 'windows' as const,
  api: {
    baseUrl: 'tauri://localhost',
    timeout: 30000,
  },
  obs: {
    useTauriCommands: true,
    useWebSocketDirect: false,
    defaultPort: 4455,
    defaultHost: 'localhost',
  },
  dev: {
    hotReload: false,
    polling: false,
    port: 1420,
  },
  features: {
    tauriCommands: true,
    webSocketDirect: false,
    nativeFileSystem: true,
    systemTray: true,
    autoUpdate: true,
  }
}; 