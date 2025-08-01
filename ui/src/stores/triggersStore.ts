import { create } from 'zustand';

// --------------------
// Helper: safe invoke that works in non-Tauri environments (Storybook, tests)
// --------------------
const invoke = async <T = any>(cmd: string, args?: Record<string, any>): Promise<T> => {
  if (typeof window !== 'undefined' && (window as any).__TAURI__?.core?.invoke) {
    return (window as any).__TAURI__.core.invoke(cmd, args);
  }
  return Promise.resolve(undefined as unknown as T);
};

export type TriggerType = 'scene' | 'overlay' | 'both';
export type TriggerRowKind = 'event' | 'delay';

export interface TriggerRowBase {
  id?: number; // from DB when persisted
  priority: number; // execution order (row 0 = highest priority)
}

export interface EventTriggerRow extends TriggerRowBase {
  kind: 'event';
  event_type: string; // e.g. pre,rdy,rnd, etc.
  action: 'show' | 'hide';
  target_type: 'scene' | 'overlay';
  obs_scene_id?: number;
  overlay_template_id?: number;
  is_enabled: boolean;
}

export interface DelayTriggerRow extends TriggerRowBase {
  kind: 'delay';
  delay_ms: number;
}

export type TriggerRow = EventTriggerRow | DelayTriggerRow;

export interface ObsScene {
  id: number;
  scene_name: string;
  scene_id: string;
  is_active: boolean;
  connection_name?: string; // optional, populated from OBS plugin
}

export interface OverlayTemplate {
  id: number;
  name: string;
  theme: string | null;
  type?: string; // additional metadata
}

interface TriggersStore {
  // meta / helpers
  loading: boolean;
  dirty: boolean;
  _obsListenerRegistered?: boolean;

  // data
  eventsCatalog: string[]; // canonical list of event ids coming from backend
  scenes: ObsScene[];
  overlays: OverlayTemplate[];
  rows: TriggerRow[]; // ordered list – first executes first
  selectedIndex: number | null;

  // pause / resume helpers
  resumeDelay: number; // milliseconds; default 2000

  // actions
  fetchData: (tournamentId?: number, dayId?: number) => Promise<void>;
  addRow: () => void;
  deleteSelectedRow: () => void;
  selectRow: (index: number | null) => void;
  updateRow: (index: number, partial: Partial<TriggerRow>) => void;
  setResumeDelay: (delay: number) => void;
  saveChanges: () => Promise<void>;
}

export const useTriggersStore = create<TriggersStore>((set, get) => ({
  // ------------ initial state ------------
  loading: false,
  dirty: false,

  eventsCatalog: [],
  scenes: [],
  overlays: [],
  rows: [],
  selectedIndex: null,
  resumeDelay: (() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('triggerResumeDelay');
      if (saved) return Number(saved) || 2000;
    }
    return 2000;
  })(),

  // ------------ actions ------------
  async fetchData(tournamentId?: number, dayId?: number) {
    set({ loading: true, dirty: false });
    try {
      const defaultEvents = ['pre', 'rdy', 'rnd', 'sup', 'wrd', 'wmh'];
      const [eventsResp, scenesResp, overlaysResp, triggersResp] = await Promise.all([
        invoke<string[]>('triggers_list_pss_events'),
        invoke<ObsScene[]>('triggers_list_obs_scenes'),
        invoke<OverlayTemplate[]>('triggers_list_active_overlays'),
        invoke<any[]>('triggers_get', { tournamentId, dayId }),
      ]);

      const eventsCatalog = Array.isArray(eventsResp) && eventsResp.length ? eventsResp : defaultEvents;
      const scenes = Array.isArray(scenesResp) ? scenesResp : [];
      const overlays = Array.isArray(overlaysResp) ? overlaysResp : [];

      // Convert DB triggers to TriggerRow list (kind = event)
      const rows: TriggerRow[] = Array.isArray(triggersResp)
        ? triggersResp.map((t, i) => ({
            kind: t.event_type === 'delay' ? 'delay' : 'event',
            id: t.id,
            priority: i,
            event_type: t.event_type,
            action: t.action ?? 'show',
            target_type: t.target_type ?? 'scene',
            obs_scene_id: t.obs_scene_id,
            overlay_template_id: t.overlay_template_id,
            is_enabled: t.is_enabled ?? true,
            delay_ms: t.delay_ms ?? 300,
          }))
        : [];

      set(state => ({ ...state, loading: false, eventsCatalog, scenes, overlays, rows }));

      // subscribe to obs scene updates once
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

  addRow() {
    set(state => {
      const newRow: EventTriggerRow = {
        kind: 'event',
        event_type: state.eventsCatalog[0] ?? 'pre',
        action: 'show',
        target_type: 'scene',
        obs_scene_id: undefined,
        overlay_template_id: undefined,
        is_enabled: true,
        priority: 0,
      };
      const newRows = [newRow, ...state.rows];
      // re-index priorities
      newRows.forEach((r, idx) => (r.priority = idx));
      return { rows: newRows, dirty: true, selectedIndex: 0 };
    });
  },

  deleteSelectedRow() {
    set(state => {
      if (state.selectedIndex === null) return {};
      const newRows = state.rows.filter((_, idx) => idx !== state.selectedIndex!);
      newRows.forEach((r, idx) => (r.priority = idx));
      return { rows: newRows, dirty: true, selectedIndex: null };
    });
  },

  selectRow(index: number | null) {
    set({ selectedIndex: index });
  },

  updateRow(index: number, partial: Partial<TriggerRow>) {
    set(state => {
      if (!(index in state.rows)) return {};
      const updated: TriggerRow = { ...state.rows[index], ...partial } as TriggerRow;
      const newRows = [...state.rows];
      newRows[index] = updated;
      return { rows: newRows, dirty: true };
    });
  },

  setResumeDelay(ms: number) {
    set({ resumeDelay: ms, dirty: true });
    if (typeof window !== 'undefined') {
      localStorage.setItem('triggerResumeDelay', String(ms));
    }
  },

  async saveChanges() {
    const { rows, resumeDelay, dirty } = get();
    if (!dirty) return;

    // Prepare payload for backend: convert rows into DB triggers format
    const payload = rows.map((row, idx) => {
      if (row.kind === 'delay') {
        return {
          id: row.id,
          event_type: 'delay',
          delay_ms: (row as DelayTriggerRow).delay_ms ?? 300,
          trigger_type: 'scene',
          priority: idx,
          is_enabled: true,
        };
      }
      const evRow = row as EventTriggerRow;
      return {
        id: evRow.id,
        event_type: evRow.event_type,
        action: evRow.action,
        target_type: evRow.target_type,
        obs_scene_id: evRow.obs_scene_id,
        overlay_template_id: evRow.overlay_template_id,
        priority: idx,
        is_enabled: evRow.is_enabled,
      };
    });

    try {
      await invoke('triggers_save', { payload, resumeDelay });
      set({ dirty: false });
    } catch (err) {
      console.error(err);
    }
  },
}));
