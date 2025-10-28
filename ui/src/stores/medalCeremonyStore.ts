import { create } from 'zustand';
import { AnthemAsset, FlagAnimationAsset, MedalCeremonyAthleteOption, MedalCeremonyDetail, MedalCeremonyDivision, MedalCeremonyDivisionOption, MedalCeremonyMedalist, MedalCeremonyRecord, MedalCeremonySummary, MedalType } from '../types';
import { medalCeremonyCommands } from '../utils/tauriCommands';

type DetailUpdater = (detail: MedalCeremonyDetail) => MedalCeremonyDetail;

const DEFAULT_ANIMATION_DURATION = 45_000;
const DEFAULT_ANIMATION_SPEED = 1.0;
const DEFAULT_PHOTO_TIME = 10;

const createMedalist = (type: MedalType, rank: number): MedalCeremonyMedalist => ({
  id: null,
  medal_type: type,
  medal_rank: rank,
  athlete_id: null,
  athlete_name: '',
  athlete_short_name: null,
  ioc_code: null,
  flag_asset: null,
  anthem_asset: null,
});

export const createEmptyDivision = (order: number = 1): MedalCeremonyDivision => ({
  id: null,
  division_id: null,
  division_name: '',
  order_index: order,
  played_at: null,
  medalists: [
    createMedalist('gold', 1),
    createMedalist('silver', 2),
    createMedalist('bronze', 3),
    createMedalist('bronze', 4),
  ],
});

export const createDraftCeremony = (): MedalCeremonyDetail => ({
  ceremony: {
    id: null,
    tournament_id: null,
    name: '',
    background_path: null,
    break_path: null,
    animation_duration: DEFAULT_ANIMATION_DURATION,
    animation_speed: DEFAULT_ANIMATION_SPEED,
    photo_time: DEFAULT_PHOTO_TIME,
    prepared_at: null,
    prepared_version: 0,
    show_external: false,
    created_at: null,
    updated_at: null,
  },
  divisions: [createEmptyDivision()],
});

export interface MedalCeremonyStore {
  ceremonies: MedalCeremonySummary[];
  selectedId: string | null;
  detail: MedalCeremonyDetail | null;
  flagAssets: FlagAnimationAsset[];
  anthemAssets: AnthemAsset[];
  assetsInitialized: boolean;
  preparedDivisions: MedalCeremonyDivision[];
  divisionOptions: MedalCeremonyDivisionOption[];
  athleteOptions: Record<string, MedalCeremonyAthleteOption[]>;
  loading: boolean;
  saving: boolean;
  error: string | null;

  // Mutators
  setDetail: (updater: DetailUpdater) => void;
  setError: (message: string | null) => void;

  // Async actions
  loadCeremonies: () => Promise<void>;
  selectCeremony: (id: string | null) => Promise<void>;
  refreshDetail: () => Promise<void>;
  saveCeremony: () => Promise<string | null>;
  deleteCeremony: (id: string) => Promise<void>;
  prepareCeremony: (id?: string) => Promise<MedalCeremonyDivision[]>;
  markDivisionPlayed: (divisionId: string) => Promise<void>;
  resetPlayback: (id?: string) => Promise<void>;
  toggleExternalDisplay: (enabled: boolean) => Promise<void>;
  loadAssets: (force?: boolean) => Promise<void>;
  loadDivisionOptions: () => Promise<void>;
  loadAthletesForDivision: (division: string) => Promise<MedalCeremonyAthleteOption[]>;
  saveFlagAsset: (asset: FlagAnimationAsset) => Promise<string | null>;
  deleteFlagAsset: (assetId: string) => Promise<void>;
  saveAnthemAsset: (asset: AnthemAsset) => Promise<string | null>;
  deleteAnthemAsset: (assetId: string) => Promise<void>;
  syncExternalState: () => void;
  emitPlaybackEvent: (division: MedalCeremonyDivision) => Promise<void>;
}

const emitTauriEvent = async (event: string, payload: unknown) => {
  if (typeof window === 'undefined') return;
  const api = (window as any).__TAURI__?.event;
  if (!api?.emit) return;
  try {
    await api.emit(event, payload);
  } catch (error) {
    console.warn(`Failed to emit ${event}:`, error);
  }
};

const broadcastStateSnapshot = (state: MedalCeremonyStore) => {
  void emitTauriEvent('medal-ceremony-state', {
    detail: state.detail,
    preparedDivisions: state.preparedDivisions,
    flagAssets: state.flagAssets,
    anthemAssets: state.anthemAssets,
  });
};

export const useMedalCeremonyStore = create<MedalCeremonyStore>((set, get) => ({
  ceremonies: [],
  selectedId: null,
  detail: null,
  flagAssets: [],
  anthemAssets: [],
  assetsInitialized: false,
  preparedDivisions: [],
  divisionOptions: [],
  athleteOptions: {},
  loading: false,
  saving: false,
  error: null,

  setDetail: (updater) => {
    set((state) => {
      if (!state.detail) {
        return state;
      }
      const updated = updater(state.detail);
      return { detail: updated };
    });
  },

  setError: (message) => set({ error: message }),

  loadCeremonies: async () => {
    set({ loading: true, error: null });
    try {
      const data = await medalCeremonyCommands.list();
      set({ ceremonies: data });
      const state = get();
      if (!state.selectedId) {
        if (data.length > 0) {
          await get().selectCeremony(data[0].id);
        } else if (!state.detail) {
          set({ detail: createDraftCeremony(), preparedDivisions: [] });
        }
      }
    } catch (error) {
      console.error('Failed to load medal ceremonies', error);
      set({ error: error instanceof Error ? error.message : String(error) });
    } finally {
      set({ loading: false });
    }
  },

  selectCeremony: async (id) => {
    set({ selectedId: id, loading: true, error: null });
    try {
      if (!id) {
        set({ detail: createDraftCeremony(), preparedDivisions: [] });
        broadcastStateSnapshot(get());
        return;
      }
      const detail = await medalCeremonyCommands.get(id);
      set({
        detail: detail ?? createDraftCeremony(),
        preparedDivisions: [],
      });
      broadcastStateSnapshot(get());
    } catch (error) {
      console.error('Failed to load medal ceremony detail', error);
      set({
        error: error instanceof Error ? error.message : String(error),
        detail: createDraftCeremony(),
      });
    } finally {
      set({ loading: false });
    }
  },

  refreshDetail: async () => {
    const id = get().selectedId;
    if (!id) {
      return;
    }
    await get().selectCeremony(id);
  },

  saveCeremony: async () => {
    const detail = get().detail;
    if (!detail) {
      return null;
    }
    set({ saving: true, error: null });
    try {
      const id = await medalCeremonyCommands.save(detail);
      await get().loadCeremonies();
      await get().selectCeremony(id);
      return id;
    } catch (error) {
      console.error('Failed to save medal ceremony', error);
      set({ error: error instanceof Error ? error.message : String(error) });
      return null;
    } finally {
      set({ saving: false });
    }
  },

  deleteCeremony: async (id) => {
    set({ loading: true, error: null });
    try {
      await medalCeremonyCommands.remove(id);
      const selected = get().selectedId;
      await get().loadCeremonies();
      if (selected === id) {
        set({ selectedId: null, detail: createDraftCeremony(), preparedDivisions: [] });
        broadcastStateSnapshot(get());
      }
    } catch (error) {
      console.error('Failed to delete medal ceremony', error);
      set({ error: error instanceof Error ? error.message : String(error) });
    } finally {
      set({ loading: false });
    }
  },

  prepareCeremony: async (id) => {
    const ceremonyId = id ?? get().selectedId;
    if (!ceremonyId) {
      throw new Error('No ceremony selected');
    }
    set({ loading: true, error: null });
    try {
      const divisions = await medalCeremonyCommands.prepare(ceremonyId);
      set({ preparedDivisions: divisions });
      broadcastStateSnapshot(get());
      return divisions;
    } catch (error) {
      console.error('Failed to prepare medal ceremony', error);
      set({ error: error instanceof Error ? error.message : String(error) });
      throw error;
    } finally {
      set({ loading: false });
    }
  },

  markDivisionPlayed: async (divisionId) => {
    try {
      await medalCeremonyCommands.markDivisionPlayed(divisionId);
      set((state) => ({
        preparedDivisions: state.preparedDivisions.map((division) =>
          division.id === divisionId
            ? { ...division, played_at: new Date().toISOString() }
            : division
        ),
      }));
      broadcastStateSnapshot(get());
    } catch (error) {
      console.error('Failed to mark division played', error);
      set({ error: error instanceof Error ? error.message : String(error) });
    }
  },

  resetPlayback: async (id) => {
    const ceremonyId = id ?? get().selectedId;
    if (!ceremonyId) {
      return;
    }
    set({ loading: true, error: null });
    try {
      await medalCeremonyCommands.resetPlayback(ceremonyId);
      await get().refreshDetail();
      set({ preparedDivisions: [] });
      broadcastStateSnapshot(get());
    } catch (error) {
      console.error('Failed to reset ceremony playback', error);
      set({ error: error instanceof Error ? error.message : String(error) });
    } finally {
      set({ loading: false });
    }
  },

  toggleExternalDisplay: async (enabled) => {
    const detail = get().detail;
    if (!detail?.ceremony.id) {
      return;
    }
    try {
      await medalCeremonyCommands.setShowExternal(detail.ceremony.id, enabled);
      set({
        detail: {
          ...detail,
          ceremony: {
            ...detail.ceremony,
            show_external: enabled,
          } as MedalCeremonyRecord,
        },
      });
      broadcastStateSnapshot(get());
    } catch (error) {
      console.error('Failed to toggle external display', error);
      set({ error: error instanceof Error ? error.message : String(error) });
    }
  },

  loadAssets: async (force = false) => {
    const { assetsInitialized } = get();
    if (assetsInitialized && !force) {
      return;
    }
    set({ loading: true, error: null });
    try {
      const [flags, anthems] = await Promise.all([
        medalCeremonyCommands.listFlagAssets(),
        medalCeremonyCommands.listAnthemAssets(),
      ]);
      set({
        flagAssets: flags,
        anthemAssets: anthems,
        assetsInitialized: true,
      });
      broadcastStateSnapshot(get());
    } catch (error) {
      console.error('Failed to load medal ceremony assets', error);
      set({ error: error instanceof Error ? error.message : String(error) });
    } finally {
      set({ loading: false });
    }
  },

  loadDivisionOptions: async () => {
    try {
      const options = await medalCeremonyCommands.listDivisionOptions();
      set({ divisionOptions: options, error: null });
    } catch (error) {
      console.error('Failed to load medal ceremony divisions', error);
      set({ error: error instanceof Error ? error.message : String(error) });
    }
  },

  loadAthletesForDivision: async (division: string) => {
    const trimmed = division.trim();
    if (!trimmed) {
      return [];
    }
    const key = trimmed.toLowerCase();
    const cached = get().athleteOptions[key];
    if (cached) {
      return cached;
    }
    try {
      const options = await medalCeremonyCommands.listAthletes(trimmed);
      set((state) => ({
        athleteOptions: {
          ...state.athleteOptions,
          [key]: options,
        },
        error: null,
      }));
      return options;
    } catch (error) {
      console.error('Failed to load athletes for division', error);
      set({ error: error instanceof Error ? error.message : String(error) });
      return [];
    }
  },

  saveFlagAsset: async (asset) => {
    set({ saving: true, error: null });
    try {
      const id = await medalCeremonyCommands.saveFlagAsset(asset);
      await get().loadAssets(true);
      return id;
    } catch (error) {
      console.error('Failed to save flag animation asset', error);
      set({ error: error instanceof Error ? error.message : String(error) });
      return null;
    } finally {
      set({ saving: false });
    }
  },

  deleteFlagAsset: async (assetId) => {
    set({ loading: true, error: null });
    try {
      await medalCeremonyCommands.deleteFlagAsset(assetId);
      await get().loadAssets(true);
    } catch (error) {
      console.error('Failed to delete flag animation asset', error);
      set({ error: error instanceof Error ? error.message : String(error) });
    } finally {
      set({ loading: false });
    }
  },

  saveAnthemAsset: async (asset) => {
    set({ saving: true, error: null });
    try {
      const id = await medalCeremonyCommands.saveAnthemAsset(asset);
      await get().loadAssets(true);
      return id;
    } catch (error) {
      console.error('Failed to save anthem asset', error);
      set({ error: error instanceof Error ? error.message : String(error) });
      return null;
    } finally {
      set({ saving: false });
    }
  },

  deleteAnthemAsset: async (assetId) => {
    set({ loading: true, error: null });
    try {
      await medalCeremonyCommands.deleteAnthemAsset(assetId);
      await get().loadAssets(true);
    } catch (error) {
      console.error('Failed to delete anthem asset', error);
      set({ error: error instanceof Error ? error.message : String(error) });
    } finally {
      set({ loading: false });
    }
  },

  syncExternalState: () => {
    broadcastStateSnapshot(get());
  },

  emitPlaybackEvent: async (division) => {
    const state = get();
    await emitTauriEvent('medal-ceremony-play', {
      division,
      ceremony: state.detail?.ceremony ?? null,
      flagAssets: state.flagAssets,
      anthemAssets: state.anthemAssets,
    });
  },
}));
