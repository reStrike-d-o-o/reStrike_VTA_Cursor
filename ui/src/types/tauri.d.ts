
declare global {
  interface Window {
    __TAURI__?: {
      core?: {
        invoke: <T = any>(cmd: string, args?: Record<string, any>) => Promise<T>;
      };
    };
  }
}
