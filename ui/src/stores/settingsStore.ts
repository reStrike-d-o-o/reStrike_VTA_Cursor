import { create } from 'zustand';

interface SettingsState {
  safetyDelayMs: number;
  confirmationsEnabled: boolean;
  theme: 'dark' | 'light';
  setSafetyDelay: (ms: number) => void;
  toggleConfirmations: (enabled: boolean) => void;
  setTheme: (theme: 'dark' | 'light') => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  safetyDelayMs: Number(localStorage.getItem('safetyDelayMs') || '5000'),
  confirmationsEnabled: localStorage.getItem('confirmationsEnabled') !== 'false',
  theme: (localStorage.getItem('theme') as 'dark' | 'light') || 'dark',

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
}));