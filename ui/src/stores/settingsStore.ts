/**
 * settingsStore
 * - UI settings and persistence flags
 */
import { create } from 'zustand';

interface SettingsState {
  safetyDelayMs: number;
  confirmationsEnabled: boolean;
  theme: 'dark' | 'light';
  sharp: boolean;
  setSafetyDelay: (ms: number) => void;
  toggleConfirmations: (enabled: boolean) => void;
  setTheme: (theme: 'dark' | 'light') => void;
  setSharp: (sharp: boolean) => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  safetyDelayMs: Number(localStorage.getItem('safetyDelayMs') || '5000'),
  confirmationsEnabled: localStorage.getItem('confirmationsEnabled') !== 'false',
  theme: (localStorage.getItem('theme') as 'dark' | 'light') || 'dark',
  sharp: localStorage.getItem('sharp') === 'true',

  setSafetyDelay(ms: number) {
    set({ safetyDelayMs: ms });
    localStorage.setItem('safetyDelayMs', String(ms));
  },

  toggleConfirmations(enabled: boolean) {
    set({ confirmationsEnabled: enabled });
    localStorage.setItem('confirmationsEnabled', String(enabled));
  },

  setTheme(theme: 'dark' | 'light') {
    set({ theme });
    localStorage.setItem('theme', theme);
    document.documentElement.setAttribute('data-theme', theme);
  },

  setSharp(sharp: boolean) {
    set({ sharp });
    localStorage.setItem('sharp', String(sharp));
    document.documentElement.setAttribute('data-sharp', sharp ? 'true' : 'false');
  },
}));