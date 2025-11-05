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

const EMPTY_MATCHES: IvrMatchCard[] = [];

export const useIvrMatchHistorySync = () => {
  const selectedDate = useIvrMatchHistoryStore((state) => state.selectedDate);
  const matchesForSelected = useIvrMatchHistoryStore(
    (state) => state.matchesByDate[state.selectedDate] ?? EMPTY_MATCHES,
  );
  const inflightRef = useRef<Set<string>>(new Set());
  const hydratedDatesRef = useRef<Set<string>>(new Set());
  const lastFailureRef = useRef<Record<string, number>>({});

  useEffect(() => {
    if (matchesForSelected.length > 0) {
      hydratedDatesRef.current.add(selectedDate);
    }
  }, [matchesForSelected, selectedDate]);

  useEffect(() => {
    if (!canListenTauri()) {
      return;
    }

    if (hydratedDatesRef.current.has(selectedDate)) {
      return;
    }

    if (inflightRef.current.has(selectedDate)) {
      return;
    }

    const lastFailure = lastFailureRef.current[selectedDate];
    if (lastFailure && Date.now() - lastFailure < 1000) {
      return;
    }

    inflightRef.current.add(selectedDate);
    let cancelled = false;

    const hydrate = async () => {
      try {
        const result = await invoke<ObsCommandResponse<{ matches?: SnapshotMatch[] }>>(
          'ivr_match_history_snapshot',
          { limit: 80, date: selectedDate },
        );

        if (cancelled) {
          return;
        }

        if (result?.success && result.data?.matches) {
          const matches = mapSnapshotMatches(result.data.matches);
          useIvrMatchHistoryStore.getState().setSnapshotForDate(selectedDate, matches);
          hydratedDatesRef.current.add(selectedDate);
          delete lastFailureRef.current[selectedDate];
        } else {
          lastFailureRef.current[selectedDate] = Date.now();
        }
      } catch (error) {
        console.warn('Failed to hydrate IVR match history snapshot:', error);
        lastFailureRef.current[selectedDate] = Date.now();
      } finally {
        inflightRef.current.delete(selectedDate);
      }
    };

    hydrate();

    return () => {
      cancelled = true;
      inflightRef.current.delete(selectedDate);
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
