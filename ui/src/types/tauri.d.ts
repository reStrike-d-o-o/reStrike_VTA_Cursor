import type { InvokeArgs } from '@tauri-apps/api/core';

// Global typings for Tauri v2 runtime objects so CRA/Vite TypeScript passes during web build.
declare global {
  interface Window {
    __TAURI__?: {
      core: {
        invoke<T = any>(cmd: string, args?: InvokeArgs | Record<string, unknown>): Promise<T>;
      };
      event?: {
        listen: (event: string, cb: (event: any) => void) => Promise<() => void>;
      };
    };
  }
}

declare module '@tauri-apps/api/tauri' {
  export function invoke<T = any>(cmd: string, args?: Record<string, unknown>): Promise<T>;
}

export {};