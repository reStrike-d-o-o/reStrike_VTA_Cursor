// Tauri v2 global type declarations

declare global {
  interface Window {
    __TAURI__: {
      core: {
        invoke: (command: string, args?: any) => Promise<any>;
      };
      app: any;
      window: any;
      event: any;
      path: any;
      fs: any;
      shell: any;
      dialog: any;
      http: any;
      notification: any;
      globalShortcut: any;
      menu: any;
      tray: any;
      webview: any;
      webviewWindow: any;
      dpi: any;
      image: any;
      mocks: any;
    };
  }
}

export {}; 