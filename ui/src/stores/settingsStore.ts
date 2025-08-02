import { create } from 'zustand';

interface SettingsState {
  safetyDelayMs: number;
  confirmationsEnabled: boolean;
  setSafetyDelay: (ms: number) => void;
  toggleConfirmations: (enabled: boolean) => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  safetyDelayMs: Number(localStorage.getItem('safetyDelayMs') || '5000'),
  confirmationsEnabled: localStorage.getItem('confirmationsEnabled') !== 'false',

  setSafetyDelay(ms: number) {
    set({ safetyDelayMs: ms });
    localStorage.setItem('safetyDelayMs', String(ms));
  },

  toggleConfirmations(enabled: boolean) {
    set({ confirmationsEnabled: enabled });
    localStorage.setItem('confirmationsEnabled', String(enabled));
  },
}));