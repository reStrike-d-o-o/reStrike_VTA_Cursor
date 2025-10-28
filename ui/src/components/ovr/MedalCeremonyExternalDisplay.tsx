import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { convertFileSrc } from '@tauri-apps/api/core';
import { AnthemAsset, FlagAnimationAsset, MedalCeremonyDetail, MedalCeremonyDivision } from '../../types';

interface StatePayload {
  detail: MedalCeremonyDetail | null;
  preparedDivisions: MedalCeremonyDivision[];
  flagAssets: FlagAnimationAsset[];
  anthemAssets: AnthemAsset[];
}

interface PlaybackPayload {
  division: MedalCeremonyDivision;
  ceremony: MedalCeremonyDetail['ceremony'] | null;
  flagAssets: FlagAnimationAsset[];
  anthemAssets: AnthemAsset[];
}

type DisplayPhase = 'animation' | 'photo' | 'break';

const resolveFilePath = (path?: string | null): string | undefined => {
  if (!path) return undefined;
  try {
    return convertFileSrc(path);
  } catch {
    return path;
  }
};

const MedalCeremonyExternalDisplay: React.FC = () => {
  const [state, setState] = useState<StatePayload>({
    detail: null,
    preparedDivisions: [],
    flagAssets: [],
    anthemAssets: [],
  });
  const [currentPlayback, setCurrentPlayback] = useState<PlaybackPayload | null>(null);
  const [phase, setPhase] = useState<DisplayPhase>('animation');

  useEffect(() => {
    if (typeof window === 'undefined' || !(window as any).__TAURI__?.event?.listen) {
      return;
    }
    let unlistenState: () => void | Promise<void> = () => {};
    let unlistenPlay: () => void | Promise<void> = () => {};
    (async () => {
      unlistenState = await (window as any).__TAURI__.event.listen('medal-ceremony-state', (event: any) => {
        setState(event.payload as StatePayload);
      });
      unlistenPlay = await (window as any).__TAURI__.event.listen('medal-ceremony-play', (event: any) => {
        const payload = event.payload as PlaybackPayload;
        setCurrentPlayback(payload);
        setPhase('animation');
      });
    })();
    return () => {
      Promise.resolve(unlistenState()).catch(() => {});
      Promise.resolve(unlistenPlay()).catch(() => {});
    };
  }, []);

  useEffect(() => {
    if (!currentPlayback) return;
    const ceremony = currentPlayback.ceremony;
    const animationDuration = ceremony?.animation_duration ?? 45_000;
    const photoDuration = (ceremony?.photo_time ?? 10) * 1000;
    const timers: number[] = [];
    timers.push(
      window.setTimeout(() => {
        setPhase('photo');
      }, animationDuration),
    );
    timers.push(
      window.setTimeout(() => {
        setPhase('break');
      }, animationDuration + photoDuration),
    );
    return () => {
      timers.forEach((timer) => window.clearTimeout(timer));
    };
  }, [currentPlayback]);

  const backgroundUrl = useMemo(() => {
    if (phase === 'break') {
      return resolveFilePath(state.detail?.ceremony.break_path);
    }
    return resolveFilePath(currentPlayback?.ceremony?.background_path ?? state.detail?.ceremony.background_path);
  }, [currentPlayback, phase, state.detail?.ceremony.background_path, state.detail?.ceremony.break_path]);

  const resolveFlag = useCallback(
    (flagId?: string | null) => {
      if (!flagId) return undefined;
      const asset =
        currentPlayback?.flagAssets.find((item) => item.id === flagId) ||
        state.flagAssets.find((item) => item.id === flagId);
      return asset ? resolveFilePath(asset.file_path) : undefined;
    },
    [currentPlayback?.flagAssets, state.flagAssets],
  );

  const division = currentPlayback?.division;

  return (
    <div
      className="h-screen w-screen overflow-hidden text-white"
      style={{
        backgroundImage: backgroundUrl ? `url(${backgroundUrl})` : undefined,
        backgroundSize: 'cover',
        backgroundPosition: 'center',
        backgroundColor: '#000',
      }}
    >
      <div className="flex h-full w-full flex-col bg-black/40">
        <header className="p-8 text-center">
          <h1 className="text-4xl font-bold tracking-wide uppercase">
            {division?.division_name || state.detail?.ceremony.name || 'Medal Ceremony'}
          </h1>
          {phase === 'photo' && (
            <p className="mt-4 text-xl uppercase tracking-widest text-amber-200">
              Photo time
            </p>
          )}
        </header>
        <main className="flex flex-1 flex-col justify-center gap-10 px-16">
          <div className="grid grid-cols-4 gap-6">
            {division?.medalists.map((medalist, idx) => {
              const flag = resolveFlag(medalist.flag_asset);
              const isGold = medalist.medal_type.toLowerCase() === 'gold';
              const isSilver = medalist.medal_type.toLowerCase() === 'silver';
              const isBronze = medalist.medal_type.toLowerCase() === 'bronze';
              const columnClass =
                idx === 0
                  ? 'col-span-1 self-end'
                  : idx === 1
                  ? 'col-span-2 self-start'
                  : 'col-span-1 self-end';
              const medalColor = isGold ? 'text-yellow-200' : isSilver ? 'text-slate-200' : 'text-orange-200';
              return (
                <div
                  key={medalist.id ?? `${medalist.medal_type}-${idx}`}
                  className={`flex flex-col items-center gap-4 ${columnClass} transition duration-700 ${
                    phase === 'break' ? 'opacity-0' : 'opacity-100'
                  }`}
                >
                  <div
                    className={`rounded-full border-4 border-white/30 bg-white/10 px-6 py-2 text-center text-lg font-semibold uppercase ${medalColor}`}
                  >
                    {medalist.medal_type}
                  </div>
                  {flag && (
                    <img
                      src={flag}
                      alt={medalist.athlete_name}
                      className="h-32 w-48 rounded-md border border-white/40 bg-black/60 object-contain shadow-lg"
                    />
                  )}
                  <div className="text-center">
                    <div className="text-2xl font-bold tracking-wide">{medalist.athlete_name}</div>
                    {medalist.ioc_code && (
                      <div className="text-sm text-gray-300">{medalist.ioc_code}</div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </main>
        <footer className="p-6 text-center text-sm text-gray-300">
          {phase === 'break'
            ? 'Returning shortly…'
            : 'Medal ceremony presentation powered by reStrike VTA'}
        </footer>
      </div>
      <style>{`
        body {
          margin: 0;
          background-color: black;
        }
      `}</style>
    </div>
  );
};

export default MedalCeremonyExternalDisplay;
