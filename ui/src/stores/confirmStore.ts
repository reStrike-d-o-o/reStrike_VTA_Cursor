import { create } from 'zustand';

interface ConfirmState {
  isOpen: boolean;
  title: string;
  message: string;
  delayMs: number;
  remaining: number;
  autoDisable: boolean;
  action?: () => void;
  open: (opts: { title: string; message: string; delayMs: number; action: () => void }) => void;
  cancel: () => void;
  confirm: () => void;
}

export const useConfirmStore = create<ConfirmState>((set, get) => ({
  isOpen: false,
  title: '',
  message: '',
  delayMs: 5000,
  remaining: 0,
  autoDisable: false,

  open({ title, message, delayMs, action }) {
    const { confirmationsEnabled, safetyDelayMs } = require('../stores/settingsStore').useSettingsStore.getState();
    if (!confirmationsEnabled) {
      action();
      return;
    }
    const effectiveDelay = delayMs ?? safetyDelayMs;

    set({ isOpen: true, title, message, delayMs: effectiveDelay, remaining: Math.ceil(effectiveDelay / 1000), action });
    // countdown timer
    let secondsLeft = Math.ceil(delayMs / 1000);
    const interval = setInterval(() => {
      secondsLeft -= 1;
      if (secondsLeft <= 0) {
        clearInterval(interval);
        set({ remaining: 0 });
      } else {
        set({ remaining: secondsLeft });
      }
    }, 1000);
  },

  cancel() {
    set({ isOpen: false, action: undefined });
  },

  confirm() {
    const { action } = get();
    set({ isOpen: false, action: undefined });
    action?.();
  },
}));