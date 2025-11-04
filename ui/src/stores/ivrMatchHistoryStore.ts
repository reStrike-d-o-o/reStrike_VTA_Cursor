import { create } from 'zustand';

export type AthleteSide = 'blue' | 'red';

export interface IvrAthleteInfo {
  name?: string | null;
  shortName?: string | null;
  flag?: string | null;
  position?: AthleteSide;
}

export type IvrVideoType = 'replay' | 'recording';

export interface IvrVideoEntry {
  id: string;
  recordedVideoId?: number;
  type: IvrVideoType;
  label: string;
  round?: number | null;
  clock?: string | null;
  filePath?: string | null;
  recordDirectory?: string | null;
  startTime?: string | null;
  durationSeconds?: number | null;
  createdAt?: string | null;
}

export interface IvrMatchCard {
  matchKey: string;
  matchDbId?: number;
  matchId?: string | null;
  matchNumber?: string | null;
  category?: string | null;
  weight?: string | null;
  division?: string | null;
  createdAt?: string | null;
  updatedAt?: string | null;
  athletes: Record<AthleteSide, IvrAthleteInfo>;
  videos: IvrVideoEntry[];
}

const toDateKey = (value: Date | string): string => {
  if (value instanceof Date) {
    return value.toISOString().slice(0, 10);
  }
  if (value && typeof value === 'string') {
    const parsed = new Date(value);
    if (!Number.isNaN(parsed.getTime())) {
      return parsed.toISOString().slice(0, 10);
    }
    if (value.length >= 10) {
      return value.slice(0, 10);
    }
  }
  return new Date().toISOString().slice(0, 10);
};

const todayKey = () => toDateKey(new Date());

const createEmptyAthlete = (): IvrAthleteInfo => ({});

const createMatchCard = (matchKey: string): IvrMatchCard => ({
  matchKey,
  athletes: {
    blue: createEmptyAthlete(),
    red: createEmptyAthlete(),
  },
  videos: [],
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
});

const upsertMatchArray = (matches: IvrMatchCard[], card: IvrMatchCard): IvrMatchCard[] => {
  const existingIdx = matches.findIndex((m) => m.matchKey === card.matchKey);
  if (existingIdx === -1) {
    return [card, ...matches];
  }
  const next = [...matches];
  next[existingIdx] = card;
  return next;
};

const normalizeVideo = (video: IvrVideoEntry): IvrVideoEntry => ({ ...video });

const normalizeMatch = (match: IvrMatchCard): IvrMatchCard => ({
  ...match,
  videos: match.videos.map(normalizeVideo),
});

const areVideosEqual = (a: IvrVideoEntry[], b: IvrVideoEntry[]): boolean => {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i += 1) {
    const left = a[i];
    const right = b[i];
    if (
      left.id !== right.id ||
      left.recordedVideoId !== right.recordedVideoId ||
      left.type !== right.type ||
      left.label !== right.label ||
      left.filePath !== right.filePath ||
      left.startTime !== right.startTime ||
      left.durationSeconds !== right.durationSeconds
    ) {
      return false;
    }
  }
  return true;
};

const areMatchesEqual = (current: IvrMatchCard[], incoming: IvrMatchCard[]): boolean => {
  if (current.length !== incoming.length) return false;
  for (let i = 0; i < current.length; i += 1) {
    const left = current[i];
    const right = incoming[i];
    if (
      left.matchKey !== right.matchKey ||
      left.matchDbId !== right.matchDbId ||
      left.matchId !== right.matchId ||
      left.matchNumber !== right.matchNumber ||
      left.category !== right.category ||
      left.weight !== right.weight ||
      left.division !== right.division ||
      !areVideosEqual(left.videos, right.videos)
    ) {
      return false;
    }
  }
  return true;
};

export interface EnsureMatchPayload {
  date: string;
  matchKey: string;
}

export interface UpdateMatchMetadataPayload {
  date: string;
  matchKey: string;
  matchDbId?: number;
  matchId?: string | null;
  matchNumber?: string | null;
  category?: string | null;
  weight?: string | null;
  division?: string | null;
}

export interface UpdateAthletesPayload {
  date: string;
  matchKey: string;
  blue?: IvrAthleteInfo;
  red?: IvrAthleteInfo;
}

export interface AppendVideoPayload {
  date: string;
  matchKey: string;
  matchDbId?: number;
  matchNumber?: string | null;
  entry: IvrVideoEntry;
}

interface IvrMatchHistoryState {
  matchesByDate: Record<string, IvrMatchCard[]>;
  selectedDate: string;
  today: string;
  searchTerm: string;
  setSearchTerm: (term: string) => void;
  setSelectedDate: (date: string) => void;
  resetToToday: () => void;
  setSnapshotForDate: (date: string, matches: IvrMatchCard[]) => void;
  ensureMatchExists: (payload: EnsureMatchPayload) => void;
  upsertMatchMetadata: (payload: UpdateMatchMetadataPayload) => void;
  updateAthletes: (payload: UpdateAthletesPayload) => void;
  appendVideo: (payload: AppendVideoPayload) => void;
  getMatchesForDate: (date: string) => IvrMatchCard[];
}

const initialToday = todayKey();

export const useIvrMatchHistoryStore = create<IvrMatchHistoryState>((set, get) => ({
  matchesByDate: {},
  selectedDate: initialToday,
  today: initialToday,
  searchTerm: '',

  setSearchTerm: (term: string) => set({ searchTerm: term }),

  setSelectedDate: (date: string) => {
    const normalized = toDateKey(date);
    set({ selectedDate: normalized });
  },

  resetToToday: () => {
    const today = todayKey();
    set({ selectedDate: today, today });
  },

  setSnapshotForDate: (date, matches) => {
    const normalized = toDateKey(date);
    set((state) => ({
      matchesByDate: {
        ...state.matchesByDate,
        [normalized]: matches.map((match) => ({
          ...match,
          updatedAt: match.updatedAt ?? match.createdAt ?? undefined,
          videos: match.videos.map((video) => ({ ...video })),
        })),
      },
    }));
  },

  ensureMatchExists: ({ date, matchKey }) => {
    const normalized = toDateKey(date);
    set((state) => {
      const matches = state.matchesByDate[normalized] ?? [];
      const exists = matches.some((match) => match.matchKey === matchKey);
      if (exists) {
        return state;
      }
      const next = [createMatchCard(matchKey), ...matches];
      return {
        matchesByDate: {
          ...state.matchesByDate,
          [normalized]: next,
        },
      };
    });
  },

  upsertMatchMetadata: ({ date, matchKey, matchDbId, matchId, matchNumber, category, weight, division }) => {
    const normalized = toDateKey(date);
    set((state) => {
      const matches = [...(state.matchesByDate[normalized] ?? [])];
      const idx = matches.findIndex((m) => m.matchKey === matchKey);
      let card = idx === -1 ? createMatchCard(matchKey) : { ...matches[idx] };
      if (matchDbId) card.matchDbId = matchDbId;
      if (matchId !== undefined) card.matchId = matchId;
      if (matchNumber !== undefined) card.matchNumber = matchNumber;
      if (category !== undefined) card.category = category;
      if (weight !== undefined) card.weight = weight;
      if (division !== undefined) card.division = division;
      card.updatedAt = new Date().toISOString();
      const next = upsertMatchArray(matches, card);
      return {
        matchesByDate: {
          ...state.matchesByDate,
          [normalized]: next,
        },
      };
    });
  },

  updateAthletes: ({ date, matchKey, blue, red }) => {
    const normalized = toDateKey(date);
    set((state) => {
      const matches = [...(state.matchesByDate[normalized] ?? [])];
      const idx = matches.findIndex((m) => m.matchKey === matchKey);
      let card = idx === -1 ? createMatchCard(matchKey) : { ...matches[idx] };
      card.athletes = {
        blue: { ...card.athletes.blue, ...(blue ?? {}) },
        red: { ...card.athletes.red, ...(red ?? {}) },
      };
      card.updatedAt = new Date().toISOString();
      const next = upsertMatchArray(matches, card);
      return {
        matchesByDate: {
          ...state.matchesByDate,
          [normalized]: next,
        },
      };
    });
  },

  appendVideo: ({ date, matchKey, matchDbId, matchNumber, entry }) => {
    const normalized = toDateKey(date);
    set((state) => {
      const matches = [...(state.matchesByDate[normalized] ?? [])];
      const idx = matches.findIndex((m) => m.matchKey === matchKey);
      let card = idx === -1 ? createMatchCard(matchKey) : { ...matches[idx] };
      if (matchDbId && !card.matchDbId) {
        card.matchDbId = matchDbId;
      }
      if (matchNumber && !card.matchNumber) {
        card.matchNumber = matchNumber;
      }
      const alreadyExists = entry.recordedVideoId
        ? card.videos.some((video) => video.recordedVideoId === entry.recordedVideoId)
        : card.videos.some((video) => video.id === entry.id);
      if (!alreadyExists) {
        card.videos = [...card.videos, entry];
      }
      card.updatedAt = new Date().toISOString();
      const next = upsertMatchArray(matches, card);
      return {
        matchesByDate: {
          ...state.matchesByDate,
          [normalized]: next,
        },
      };
    });
  },

  getMatchesForDate: (date: string) => {
    const normalized = toDateKey(date);
    return get().matchesByDate[normalized] ?? [];
  },
}));

export const deriveMatchKey = (
  matchDbId?: number | null,
  matchNumber?: string | number | null,
  matchId?: string | null,
): string => {
  if (typeof matchDbId === 'number' && !Number.isNaN(matchDbId)) {
    return `db-${matchDbId}`;
  }
  if (matchNumber !== undefined && matchNumber !== null) {
    const asString = String(matchNumber).trim();
    if (asString.length > 0) {
      return `no-${asString}`;
    }
  }
  if (matchId && matchId.trim().length > 0) {
    return matchId.trim();
  }
  return `match-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
};

export const selectMatchesForCurrentDate = (state: IvrMatchHistoryState) =>
  state.matchesByDate[state.selectedDate] ?? [];

export const selectMatchesForDate = (date: string) =>
  (state: IvrMatchHistoryState) => state.matchesByDate[toDateKey(date)] ?? [];
