export {};

declare global {
  interface Window {
    __TAURI__?: {
      fs?: {
        readTextFile?: (path: string) => Promise<string>;
      };
      core?: {
        invoke: (command: string, args?: Record<string, unknown>) => Promise<any>;
      };
      event?: {
        listen?: (
          event: string,
          handler: (payload: any) => void
        ) => Promise<() => void>;
      };
    };
  }
}
