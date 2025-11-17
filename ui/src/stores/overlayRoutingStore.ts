import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import type {
  OverlayId,
  OverlayRoutingConfig,
  OverlayRoutingRule,
  OverlayTriggerType,
} from '../types';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';

const invoke = async (command: string, args?: any) => {
  try {
    return await tauriInvoke(command, args);
  } catch (error) {
    if (typeof window !== 'undefined' && (window as any).__TAURI__ && (window as any).__TAURI__.core) {
      return await (window as any).__TAURI__.core.invoke(command, args);
    }
    throw error;
  }
};

export interface OverlayRoutingState {
  rules: OverlayRoutingRule[];
}

export interface OverlayRoutingActions {
  setRules: (rules: OverlayRoutingRule[]) => void;
  updateRule: (index: number, rule: OverlayRoutingRule) => void;
  resetToDefaults: () => void;
  loadFromBackend: () => Promise<void>;
  saveToBackend: () => Promise<void>;
}

export type OverlayRoutingStore = OverlayRoutingState & OverlayRoutingActions;

const defaultRules: OverlayRoutingRule[] = [
  {
    trigger: 'fight_loaded',
    overlay: 'olympicScoreboard',
    action: 'show',
  },
  {
    trigger: 'winner',
    overlay: 'modernWinner',
    action: 'show',
  },
] ;

export const defaultOverlayRoutingConfig: OverlayRoutingConfig = {
  rules: defaultRules,
};

export const useOverlayRoutingStore = create<OverlayRoutingStore>()(
  devtools(
    (set, get) => ({
      rules: defaultRules,

      setRules(rules: OverlayRoutingRule[]) {
        set({ rules: [...rules] });
      },

      updateRule(index: number, rule: OverlayRoutingRule) {
        set((state) => {
          if (index < 0 || index >= state.rules.length) {
            return state;
          }
          const next = [...state.rules];
          next[index] = rule;
          return { rules: next };
        });
      },

      resetToDefaults() {
        set({ rules: defaultRules });
      },

      async loadFromBackend() {
        try {
          const result = await invoke('get_overlay_routing_config');
          const payload = result as OverlayRoutingConfig;
          if (payload && Array.isArray(payload.rules) && payload.rules.length > 0) {
            set({ rules: payload.rules });
          }
        } catch (error) {
          console.error('Failed to load overlay routing config, using defaults:', error);
          set({ rules: defaultRules });
        }
      },

      async saveToBackend() {
        try {
          const rules = get().rules;
          const payload: OverlayRoutingConfig = { rules };
          await invoke('set_overlay_routing_config', { config: payload });
        } catch (error) {
          console.error('Failed to save overlay routing config:', error);
        }
      },
    }),
    { name: 'overlay-routing-store' },
  ),
);
