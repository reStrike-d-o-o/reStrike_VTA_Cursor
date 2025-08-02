import { create } from 'zustand';
import { useSettingsStore } from './settingsStore';

interface ConfirmState {
  isOpen: boolean;
  title: string;
  message: string;
  delayMs: number;
  remaining: number;
  action?: () => void;
  open: (o: { title: string; message: string; delayMs?: number; action: () => void }) => void;
  cancel: () => void;
  confirm: () => void;
}

export const useConfirmStore = create<ConfirmState>((set, get) => ({
  isOpen: false,
  title: '',
  message: '',
  delayMs: 5000,
  remaining: 0,

  open({ title, message, delayMs, action }) {
    const { confirmationsEnabled, safetyDelayMs } = useSettingsStore.getState();
    if (!confirmationsEnabled) {
      action();
      return;
    }
    const eff = delayMs ?? safetyDelayMs;
    set({ isOpen: true, title, message, delayMs: eff, remaining: Math.ceil(eff / 1000), action });
    let secs = Math.ceil(eff / 1000);
    const timer = setInterval(() => {
      secs -= 1;
      if (secs <= 0) {
        clearInterval(timer);
        set({ remaining: 0 });
      } else {
        set({ remaining: secs });
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