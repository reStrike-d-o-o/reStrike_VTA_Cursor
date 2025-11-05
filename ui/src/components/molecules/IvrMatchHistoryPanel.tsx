import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  useIvrMatchHistoryStore,
  IvrMatchCard,
  IvrVideoEntry,
  deriveMatchKey,
} from '../../stores/ivrMatchHistoryStore';
import Input from '../atoms/Input';
import Button from '../atoms/Button';
import { FlagImage } from '../../utils/flagUtils';
import { canListenTauri } from '../../utils/tauriBridge';

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

interface ObsCommandResponse<T = unknown> {
  success: boolean;
  data?: T | null;
  error?: string | null;
}

const mapSnapshotMatches = (matches: SnapshotMatch[] | undefined): IvrMatchCard[] => {
  const snapshot = matches ?? [];
  return snapshot.map((match) => {
    const resolvedKey = deriveMatchKey(match.match_db_id, match.match_number, match.match_id);

    const athleteMap: IvrMatchCard['athletes'] = {
      blue: {},
      red: {},
    };
    (match.athletes ?? []).forEach((athlete) => {
      const position = athlete.position === 2 ? 'red' : 'blue';
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
      matchKey: resolvedKey,
      matchDbId: match.match_db_id,
      matchId: match.match_id ?? undefined,
      matchNumber: match.match_number ?? undefined,
      category: match.category ?? undefined,
      weight: match.weight ?? undefined,
      division: match.division ?? undefined,
      createdAt: match.created_at ?? undefined,
      updatedAt: match.created_at ?? undefined,
      athletes: athleteMap,
      videos,
    };
  });
};

const renderAthleteBlock = (athlete?: { name?: string | null; flag?: string | null }, fallback?: string) => {
  const displayName = athlete?.name || fallback || 'Unknown';
  const flagCode = athlete?.flag ? athlete.flag.toUpperCase() : null;

  return (
    <span className="inline-flex items-center gap-2">
      {flagCode ? (
        <FlagImage countryCode={flagCode} className="w-7 h-5 object-cover rounded-sm shadow-sm border border-white/10" />
      ) : (
        <span className="w-7 h-5 flex items-center justify-center rounded-sm bg-gray-700 text-xs text-gray-300">IOC</span>
      )}
      <span className="text-sm font-semibold text-gray-100 truncate max-w-[120px]" title={displayName}>
        {displayName}
      </span>
    </span>
  );
};

const formatMetadataLine = (match: IvrMatchCard) => {
  const parts = [match.weight, match.category, match.division].filter(Boolean);
  return parts.join(' | ') || '—';
};

const getReplayBadge = (entry: IvrVideoEntry) => {
  if (entry.type !== 'replay') {
    return null;
  }
  if (typeof entry.round === 'number' && entry.round > 0) {
    return `R${entry.round}`;
  }
  return 'R?';
};

const getReplayTime = (entry: IvrVideoEntry) => {
  if (entry.type !== 'replay') {
    return null;
  }
  if (entry.clock && entry.clock.trim().length > 0) {
    return entry.clock;
  }
  return '--:--';
};

const handleOpenVideo = async (entry: IvrVideoEntry) => {
  if (!entry.filePath) {
    return;
  }
  try {
    await invoke('ivr_open_video_file', {
      filePath: entry.filePath,
      offsetSeconds: 0,
    });
  } catch (error) {
    console.warn('Failed to open IVR video file:', error);
  }
};

const MatchCard: React.FC<{ match: IvrMatchCard }> = ({ match }) => {
  const matchNumberDisplay = match.matchNumber ? String(match.matchNumber).trim() : '—';
  return (
    <div
      key={match.matchKey}
      className="inline-block w-[300px] theme-card p-4 mb-6 shadow-lg border border-white/5 bg-gradient-to-br from-slate-900/80 to-slate-800/70"
      style={{ breakInside: 'avoid' }}
    >
      <div className="flex flex-col gap-2">
        <div className="flex flex-wrap items-center gap-2 text-sm font-semibold text-gray-100">
          <span>{matchNumberDisplay}</span>
          <span className="text-gray-500 font-normal">|</span>
          {renderAthleteBlock(match.athletes.blue, 'Blue Athlete')}
          <span className="text-xs font-medium text-gray-500 tracking-wide uppercase">VS</span>
          {renderAthleteBlock(match.athletes.red, 'Red Athlete')}
        </div>
        <div className="text-xs text-gray-400 border-t border-white/10 pt-2">
          {formatMetadataLine(match)}
        </div>
      </div>

      <div className="mt-4 space-y-2">
        {match.videos.length === 0 && (
          <div className="text-xs text-gray-500 italic py-4 text-center border border-dashed border-white/10 rounded">
            No replays saved for this match yet.
          </div>
        )}
        {match.videos.map((entry) => {
          if (entry.type === 'recording') {
            return (
              <button
                key={entry.id}
                onClick={() => handleOpenVideo(entry)}
                className="w-full rounded border border-blue-500/40 bg-blue-500/10 px-3 py-2 text-sm font-semibold text-blue-100 hover:bg-blue-500/20 transition focus:outline-none focus-visible:ring-2 focus-visible:ring-blue-400/70"
              >
                {entry.label}
              </button>
            );
          }
          return (
            <button
              key={entry.id}
              onClick={() => handleOpenVideo(entry)}
              className="w-full flex items-center justify-between gap-2 rounded border border-white/10 px-3 py-2 text-sm text-gray-100 hover:bg-white/10 transition focus:outline-none focus-visible:ring-2 focus-visible:ring-white/40"
            >
              <span className="w-10 text-left text-xs font-semibold text-gray-300">
                {getReplayBadge(entry)}
              </span>
              <span className="flex-1 text-xs text-gray-300">{getReplayTime(entry)}</span>
              <span className="text-xs font-semibold text-blue-300">{entry.label}</span>
            </button>
          );
        })}
      </div>
    </div>
  );
};

export const IvrMatchHistoryPanel: React.FC = () => {
  const hydrationStatusRef = useRef<Map<string, 'hydrating' | 'hydrated'>>(new Map());
  const inflightDatesRef = useRef<Set<string>>(new Set());
  const [loadingDate, setLoadingDate] = useState<string | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const {
    matches,
    searchTerm,
    setSearchTerm,
    selectedDate,
    setSelectedDate,
    resetToToday,
    today,
  } = useIvrMatchHistoryStore((state) => ({
    matches: state.getMatchesForDate(state.selectedDate),
    searchTerm: state.searchTerm,
    setSearchTerm: state.setSearchTerm,
    selectedDate: state.selectedDate,
    setSelectedDate: state.setSelectedDate,
    resetToToday: state.resetToToday,
    today: state.today,
  }));

  useEffect(() => {
    if (matches.length > 0) {
      hydrationStatusRef.current.set(selectedDate, 'hydrated');
    }
  }, [matches, selectedDate]);

  const loadMatchesForDate = useCallback(
    async (date: string, force = false) => {
      if (!canListenTauri()) {
        return;
      }

      if (!force && hydrationStatusRef.current.get(date) === 'hydrated') {
        return;
      }

      if (inflightDatesRef.current.has(date)) {
        return;
      }

      inflightDatesRef.current.add(date);
      hydrationStatusRef.current.set(date, 'hydrating');
      setLoadingDate(date);
      setLoadError(null);

      try {
        const result = await invoke<ObsCommandResponse<{ matches?: SnapshotMatch[] }>>(
          'ivr_match_history_snapshot',
          { limit: 80, date },
        );

        if (result?.success && result.data?.matches) {
          const parsed = mapSnapshotMatches(result.data.matches);
          useIvrMatchHistoryStore.getState().setSnapshotForDate(date, parsed);
        }

        hydrationStatusRef.current.set(date, 'hydrated');
      } catch (error) {
        console.warn('Failed to hydrate IVR match history snapshot:', error);
        setLoadError('Failed to load match history snapshot.');
        hydrationStatusRef.current.delete(date);
      } finally {
        inflightDatesRef.current.delete(date);
        setLoadingDate((current) => (current === date ? null : current));
      }
    },
    [],
  );

  useEffect(() => {
    if (hydrationStatusRef.current.get(selectedDate) !== 'hydrated') {
      void loadMatchesForDate(selectedDate);
    }
  }, [selectedDate, loadMatchesForDate]);

  const handleDateChange = useCallback(
    (value: string) => {
      hydrationStatusRef.current.delete(value);
      setSelectedDate(value);
      void loadMatchesForDate(value);
    },
    [setSelectedDate, loadMatchesForDate],
  );

  const handleManualReload = useCallback(() => {
    hydrationStatusRef.current.delete(selectedDate);
    void loadMatchesForDate(selectedDate, true);
  }, [selectedDate, loadMatchesForDate]);

  const filteredMatches = useMemo(() => {
    if (!searchTerm) {
      return matches;
    }
    const query = searchTerm.toLowerCase();
    return matches.filter((match) => {
      const numberMatch = match.matchNumber ? String(match.matchNumber).toLowerCase().includes(query) : false;
      const athleteNames = [
        match.athletes.blue?.name ?? '',
        match.athletes.red?.name ?? '',
      ]
        .join(' ')
        .toLowerCase();
      return numberMatch || athleteNames.includes(query);
    });
  }, [matches, searchTerm]);

  return (
    <div className="flex flex-col gap-4 h-full overflow-hidden">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div className="flex-1">
          <Input
            value={searchTerm}
            onChange={(event: React.ChangeEvent<HTMLInputElement>) => setSearchTerm(event.target.value)}
            placeholder="Filter matches by number or athlete name..."
            className="bg-slate-900/60 border-slate-700/80 text-sm"
          />
        </div>
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2 text-xs text-gray-400">
            <span>Date:</span>
            <Input
              type="date"
              value={selectedDate}
              onChange={(event: React.ChangeEvent<HTMLInputElement>) => handleDateChange(event.target.value)}
              className="bg-slate-900/60 border-slate-700/80 text-xs w-[150px]"
            />
          </div>
          <Button
            variant="secondary"
            size="sm"
            onClick={resetToToday}
            disabled={selectedDate === today}
          >
            Today
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onClick={handleManualReload}
            disabled={loadingDate === selectedDate}
          >
            {loadingDate === selectedDate ? 'Loading…' : 'Reload'}
          </Button>
        </div>
      </div>

      {loadError && (
        <div className="text-xs text-red-400 bg-red-900/20 border border-red-800 rounded px-3 py-2">
          {loadError}
        </div>
      )}

      <div className="flex-1 min-h-0 overflow-x-auto overflow-y-hidden pb-2">
        <div className="min-h-full columns-[320px] gap-6 pr-6">
          {filteredMatches.length === 0 ? (
            <div className="inline-flex items-center justify-center w-full h-48 text-sm text-gray-500 border border-dashed border-white/10 rounded px-4 text-center">
              <span>
                No matches for{' '}
                <span className="font-semibold text-gray-300">{selectedDate}</span>. Once recordings exist, cards will
                appear here.
              </span>
            </div>
          ) : (
            filteredMatches.map((match) => <MatchCard key={match.matchKey} match={match} />)
          )}
        </div>
      </div>
    </div>
  );
};

export default IvrMatchHistoryPanel;
