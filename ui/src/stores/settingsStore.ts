import { create } from 'zustand';

interface SettingsState {
  safetyDelayMs: number;
  confirmationsEnabled: boolean;
  setSafetyDelay: (ms: number) => void;
  toggleConfirmations: (enabled: boolean) => void;
}

export const useSettingsStore = create<SettingsState>((set) => {
  const storedDelay = typeof window !== 'undefined' ? Number(localStorage.getItem('safetyDelayMs') || '5000') : 5000;
  const storedEnabled = typeof window !== 'undefined' ? localStorage.getItem('confirmationsEnabled') !== 'false' : true;
  return {
    safetyDelayMs: storedDelay || 5000,
    confirmationsEnabled: storedEnabled,
    setSafetyDelay(ms) {
      set({ safetyDelayMs: ms });
      if (typeof window !== 'undefined') localStorage.setItem('safetyDelayMs', String(ms));
    },
    toggleConfirmations(enabled) {
      set({ confirmationsEnabled: enabled });
      if (typeof window !== 'undefined') localStorage.setItem('confirmationsEnabled', String(enabled));
    },
  };
});