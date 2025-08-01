import { create } from 'zustand';

// Fallback-safe invoke
const invoke = async <T = any>(cmd: string, args?: Record<string, any>): Promise<T> => {
  if (typeof window !== 'undefined' && (window as any).__TAURI__?.core?.invoke) {
    return (window as any).__TAURI__.core.invoke(cmd, args);
  }
  return Promise.resolve(undefined as unknown as T);
};

export type TriggerType = 'scene' | 'overlay' | 'both';

export interface TriggerRow {
  id?: number;
  event_type: string;
  trigger_type: TriggerType;
  obs_scene_id?: number;
  overlay_template_id?: number;
  is_enabled: boolean;
  priority: number;
}

export interface ObsScene {
  id: number;
  scene_name: string;
  scene_id: string;
  is_active: boolean;
}

export interface OverlayTemplate {
  id: number;
  name: string;
  theme: string;
}

interface TriggersStore {
  _obsListenerRegistered?: boolean;
  loading: boolean;
  events: string[];
  scenes: ObsScene[];
  overlays: OverlayTemplate[];
  triggers: TriggerRow[];
  dirty: boolean;

  fetchData: (tournamentId?: number, dayId?: number) => Promise<void>;
  updateTrigger: (event: string, partial: Partial<TriggerRow>) => void;
  saveChanges: () => Promise<void>;
}

export const useTriggersStore = create<TriggersStore>((set, get) => ({
  loading: false,
  events: [],
  scenes: [],
  overlays: [],
  triggers: [],
  dirty: false,
  _obsListenerRegistered: false,

  async fetchData(tournamentId?: number, dayId?: number) {
    set({ loading: true, dirty: false });
    try {
      const defaultEvents = ['pre','rdy','rnd','sup','wrd','wmh'];
      const [eventsResp, scenesResp, overlaysResp, triggersResp] = await Promise.all([
        invoke<string[]>('triggers_list_pss_events'),
        invoke<ObsScene[]>('triggers_list_obs_scenes'),
        invoke<OverlayTemplate[]>('triggers_list_active_overlays'),
        invoke<TriggerRow[]>('triggers_get', { tournamentId, dayId }),
      ]);
      const events = Array.isArray(eventsResp) && eventsResp.length ? eventsResp : defaultEvents;
      const scenes = Array.isArray(scenesResp) ? scenesResp : [];
      const overlays = Array.isArray(overlaysResp) ? overlaysResp : [];
      const triggers = Array.isArray(triggersResp) ? triggersResp : [];
      set(state => ({ ...state, events, scenes, overlays, triggers, loading: false }));

      if (!get()._obsListenerRegistered && typeof window !== 'undefined' && window.__TAURI__?.event?.listen) {
        window.__TAURI__.event.listen('obs_scenes_updated', async () => {
          const scenesLatest = await invoke<ObsScene[]>('triggers_list_obs_scenes');
          set({ scenes: scenesLatest });
        });
        set({ _obsListenerRegistered: true });
      }
    } catch (err) {
      console.error(err);
      set({ loading: false });
    }
  },

  updateTrigger(eventType: string, partial: Partial<TriggerRow>) {
    set((state: TriggersStore) => {
      const idx = state.triggers.findIndex((t: TriggerRow) => t.event_type === eventType);
      const newTriggers = [...state.triggers];
      if (idx >= 0) {
        newTriggers[idx] = { ...newTriggers[idx], ...partial } as TriggerRow;
      } else {
        newTriggers.push({
          event_type: eventType,
          trigger_type: partial.trigger_type ?? 'scene',
          is_enabled: true,
          priority: 0,
          ...partial,
        });
      }
      return { triggers: newTriggers, dirty: true };
    });
  },

  async saveChanges() {
    const { triggers, dirty } = get();
    if (!dirty) return;
    try {
      await invoke('triggers_save', { payload: triggers });
      set({ dirty: false });
    } catch (err) {
      console.error(err);
    }
  },
}));
