import { useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { canListenTauri } from '../utils/tauriBridge';
import {
  deriveMatchKey,
  useIvrMatchHistoryStore,
  AthleteSide,
  IvrAthleteInfo,
  IvrMatchCard,
  IvrVideoEntry,
} from '../stores/ivrMatchHistoryStore';
import { useLiveDataStore } from '../stores/liveDataStore';

interface ObsCommandResponse<T = unknown> {
  success: boolean;
  data?: T | null;
  error?: string | null;
}

interface SnapshotMatchAthlete {
  position?: number | null;
  country_code?: string | null;
  name?: string | null;
  short_name?: string | null;
}

interface SnapshotMatchVideo {
  id?: number;
  video_type?: string | null;
  file_path?: string | null;
  record_directory?: string | null;
  start_time?: string | null;
  duration_seconds?: number | null;
  created_at?: string | null;
}

interface SnapshotMatch {
  match_db_id?: number;
  match_id?: string | null;
  match_number?: string | null;
  category?: string | null;
  weight?: string | null;
  division?: string | null;
  created_at?: string | null;
  athletes?: SnapshotMatchAthlete[];
  videos?: SnapshotMatchVideo[];
}

interface ReplaySavedPayload {
  recorded_video_id?: number;
  match_db_id?: number;
  match_number?: string | null;
  match_id?: string | null;
  video_type?: string | null;
  file_path?: string | null;
  record_directory?: string | null;
  start_time?: string | null;
  duration_seconds?: number | null;
  created_at?: string | null;
}

type RecordingSavedPayload = ReplaySavedPayload;

const extractDateKey = (...sources: Array<string | null | undefined>): string => {
  for (const source of sources) {
    if (!source) continue;
    const parsed = new Date(source);
    if (!Number.isNaN(parsed.getTime())) {
      return parsed.toISOString().slice(0, 10);
    }
    if (source.length >= 10) {
      return source.slice(0, 10);
    }
  }
  return new Date().toISOString().slice(0, 10);
};

const mapSnapshotMatches = (matches: SnapshotMatch[] | undefined): IvrMatchCard[] => {
  const snapshot = matches ?? [];
  return snapshot.map((match) => {
    const matchKey = deriveMatchKey(match.match_db_id, match.match_number, match.match_id);
    const athleteMap: Record<AthleteSide, IvrAthleteInfo> = {
      blue: {},
      red: {},
    };
    (match.athletes ?? []).forEach((athlete) => {
      const position: AthleteSide = athlete.position === 2 ? 'red' : 'blue';
      athleteMap[position] = {
        name: athlete.name ?? undefined,
        shortName: athlete.short_name ?? undefined,
        flag: athlete.country_code ?? undefined,
      };
    });

    const safeMatchNumber = (match.match_number ?? 'MATCH').toString().trim();
    const videos: IvrVideoEntry[] = (match.videos ?? []).map((video, index) => {
      const type: IvrVideoEntry['type'] =
        video.video_type === 'recording' ? 'recording' : 'replay';
      const label =
        type === 'recording'
          ? (match.match_number ? `${safeMatchNumber}# - FULL MATCH RECORDING` : 'FULL MATCH RECORDING')
          : 'IVR video';
      const stableId = video.id !== undefined ? `snapshot-${video.id}` : `snapshot-temp-${index}`;
      return {
        id: stableId,
        recordedVideoId: video.id,
        type,
        label,
        filePath: video.file_path ?? undefined,
        recordDirectory: video.record_directory ?? undefined,
        startTime: video.start_time ?? undefined,
        durationSeconds: video.duration_seconds ?? undefined,
        createdAt: video.created_at ?? undefined,
      };
    });

    return {
      matchKey,
      matchDbId: match.match_db_id,
      matchId: match.match_id ?? undefined,
      matchNumber: match.match_number ?? undefined,
      category: match.category ?? undefined,
      weight: match.weight ?? undefined,
      division: match.division ?? undefined,
      createdAt: match.created_at ?? undefined,
      athletes: athleteMap,
      videos,
      updatedAt: match.created_at ?? undefined,
    };
  });
};

const matchesEqual = (a: IvrMatchCard[], b: IvrMatchCard[]): boolean => {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i += 1) {
    const left = a[i];
    const right = b[i];
    if (
      left.matchKey !== right.matchKey ||
      left.matchDbId !== right.matchDbId ||
      left.matchId !== right.matchId ||
      left.matchNumber !== right.matchNumber ||
      left.category !== right.category ||
      left.weight !== right.weight ||
      left.division !== right.division
    ) {
      return false;
    }
    if (left.videos.length !== right.videos.length) return false;
    for (let v = 0; v < left.videos.length; v += 1) {
      const lv = left.videos[v];
      const rv = right.videos[v];
      if (
        lv.id !== rv.id ||
        lv.recordedVideoId !== rv.recordedVideoId ||
        lv.type !== rv.type ||
        lv.label !== rv.label ||
        lv.filePath !== rv.filePath ||
        lv.startTime !== rv.startTime ||
        lv.durationSeconds !== rv.durationSeconds
      ) {
        return false;
      }
    }
  }
  return true;
};

const computeSignature = (matches: IvrMatchCard[]): string =>
  matches
    .map((match) => {
      const base =
        `${match.matchKey ?? ''}|${match.matchDbId ?? ''}|${match.matchId ?? ''}|${match.matchNumber ?? ''}|` +
        `${match.category ?? ''}|${match.weight ?? ''}|${match.division ?? ''}`;
      const videoHash = match.videos
        .map(
          (video) =>
            `${video.type}:${video.recordedVideoId ?? video.id}:${video.label ?? ''}:${video.filePath ?? ''}:${video.startTime ?? ''}:${video.durationSeconds ?? ''}`,
        )
        .join(',');
      return `${base}|${videoHash}`;
    })
    .join('||');

export const useIvrMatchHistorySync = () => {
  const selectedDate = useIvrMatchHistoryStore((state) => state.selectedDate);
  const hydrationStateRef = useRef<Record<string, { status: 'hydrating' | 'hydrated'; signature?: string }>>({});

  useEffect(() => {
    if (!canListenTauri()) {
      return;
    }

    const hydrationState = hydrationStateRef.current[selectedDate];
    const storeSnapshot = useIvrMatchHistoryStore.getState();
    const existingMatches = storeSnapshot.getMatchesForDate(selectedDate);
    const existingSignature = computeSignature(existingMatches);

    if (existingMatches.length > 0 && hydrationState?.signature === existingSignature) {
      hydrationStateRef.current[selectedDate] = { status: 'hydrated', signature: existingSignature };
      return undefined;
    }

    if (hydrationState?.status === 'hydrating') {
      return undefined;
    }

    let cancelled = false;
    hydrationStateRef.current[selectedDate] = { status: 'hydrating' };

    const hydrate = async () => {
      try {
        const result = await invoke<ObsCommandResponse<{ matches?: SnapshotMatch[] }>>(
          'ivr_match_history_snapshot',
          { limit: 80, date: selectedDate },
        );
        if (cancelled || !result?.success || !result.data?.matches) {
          if (cancelled) {
            delete hydrationStateRef.current[selectedDate];
          }
          return;
        }

        const matches = mapSnapshotMatches(result.data.matches);
        const signature = computeSignature(matches);
        const nextStore = useIvrMatchHistoryStore.getState();
        const current = nextStore.getMatchesForDate(selectedDate);
        if (!matchesEqual(current, matches)) {
          nextStore.setSnapshotForDate(selectedDate, matches);
        }
        hydrationStateRef.current[selectedDate] = { status: 'hydrated', signature };
      } catch (error) {
        console.warn('Failed to hydrate IVR match history snapshot:', error);
        delete hydrationStateRef.current[selectedDate];
      }
    };

    hydrate();

    return () => {
      cancelled = true;
      const state = hydrationStateRef.current[selectedDate];
      if (state?.status === 'hydrating') {
        delete hydrationStateRef.current[selectedDate];
      }
    };
  }, [selectedDate]);

  useEffect(() => {
    if (!canListenTauri()) {
      return;
    }

    let unlistenReplay: (() => void) | undefined;
    let unlistenRecording: (() => void) | undefined;

    const registerListeners = async () => {
      try {
        unlistenReplay = await listen<ReplaySavedPayload>('ivr_replay_saved', (event) => {
          const payload = event.payload;
          if (!payload) return;

          const eventDate = extractDateKey(payload.start_time, payload.created_at);
          const matchKey = deriveMatchKey(
            payload.match_db_id,
            payload.match_number ?? payload.match_id,
            payload.match_id,
          );
          const historyStore = useIvrMatchHistoryStore.getState();
          const { currentRound, currentRoundTime } = useLiveDataStore.getState();

          historyStore.ensureMatchExists({ date: eventDate, matchKey });
          historyStore.upsertMatchMetadata({
            date: eventDate,
            matchKey,
            matchDbId: payload.match_db_id,
            matchId: payload.match_id ?? undefined,
            matchNumber: payload.match_number ?? undefined,
          });
          historyStore.appendVideo({
            date: eventDate,
            matchKey,
            matchDbId: payload.match_db_id,
            matchNumber: payload.match_number ?? undefined,
            entry: {
              id: `replay-${payload.recorded_video_id ?? Date.now()}`,
              recordedVideoId: payload.recorded_video_id,
              type: 'replay',
              label: 'IVR video',
              round: Number.isFinite(currentRound) ? currentRound : undefined,
              clock: currentRoundTime,
              filePath: payload.file_path ?? undefined,
              recordDirectory: payload.record_directory ?? undefined,
              startTime: payload.start_time ?? undefined,
              durationSeconds: payload.duration_seconds ?? undefined,
              createdAt: payload.created_at ?? new Date().toISOString(),
            },
          });
        });

        unlistenRecording = await listen<RecordingSavedPayload>('ivr_recording_saved', (event) => {
          const payload = event.payload;
          if (!payload) return;

          const eventDate = extractDateKey(payload.start_time, payload.created_at);
          const matchKey = deriveMatchKey(
            payload.match_db_id,
            payload.match_number ?? payload.match_id,
            payload.match_id,
          );
          const historyStore = useIvrMatchHistoryStore.getState();
          historyStore.ensureMatchExists({ date: eventDate, matchKey });
          historyStore.upsertMatchMetadata({
            date: eventDate,
            matchKey,
            matchDbId: payload.match_db_id,
            matchId: payload.match_id ?? undefined,
            matchNumber: payload.match_number ?? undefined,
          });
          const matchNumber = payload.match_number ? String(payload.match_number).trim() : null;
          historyStore.appendVideo({
            date: eventDate,
            matchKey,
            matchDbId: payload.match_db_id,
            matchNumber: payload.match_number ?? undefined,
            entry: {
              id: `recording-${payload.recorded_video_id ?? Date.now()}`,
              recordedVideoId: payload.recorded_video_id,
              type: 'recording',
              label: matchNumber ? `${matchNumber}# - FULL MATCH RECORDING` : 'FULL MATCH RECORDING',
              filePath: payload.file_path ?? undefined,
              recordDirectory: payload.record_directory ?? undefined,
              startTime: payload.start_time ?? undefined,
              durationSeconds: payload.duration_seconds ?? undefined,
              createdAt: payload.created_at ?? new Date().toISOString(),
            },
          });
        });
      } catch (error) {
        console.warn('Failed to register IVR history listeners:', error);
      }
    };

    registerListeners();

    return () => {
      if (unlistenReplay) unlistenReplay();
      if (unlistenRecording) unlistenRecording();
    };
  }, []);
};
