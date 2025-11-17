import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

export type OverlayWindowId =
  // Scoreboard windows
  | 'olympic'
  | 'modern'
  | 'arcade'
  | 'olympicScoreboard'
  | 'modernScoreboard'
  | 'arcadeScoreboard'
  // Player introduction overlays
  | 'olympicIntro'
  | 'modernIntro'
  | 'arcadeIntro'
  // Result overlays
  | 'modernResult'
  | 'arcadeResult'
  // Winner overlays
  | 'modernWinner'
  | 'arcadeWinner'
  // Video replay overlays
  | 'olympicVideoReplay'
  | 'modernVideoReplay'
  | 'arcadeVideoReplay';

const LABELS: Record<OverlayWindowId, string> = {
  // Primary scoreboard windows
  olympic: 'overlay_olympic',
  modern: 'overlay_modern',
  arcade: 'overlay_arcade',
  olympicScoreboard: 'overlay_olympic',
  modernScoreboard: 'overlay_modern',
  arcadeScoreboard: 'overlay_arcade',
  // Player introductions
  olympicIntro: 'overlay_olympic_intro',
  modernIntro: 'overlay_modern_intro',
  arcadeIntro: 'overlay_arcade_intro',
  // Results
  modernResult: 'overlay_modern_result',
  arcadeResult: 'overlay_arcade_result',
  // Winners
  modernWinner: 'overlay_modern_winner',
  arcadeWinner: 'overlay_arcade_winner',
  // Video replay
  olympicVideoReplay: 'overlay_olympic_video_replay',
  modernVideoReplay: 'overlay_modern_video_replay',
  arcadeVideoReplay: 'overlay_arcade_video_replay',
};

const URLS: Record<OverlayWindowId, string> = {
  // Scoreboards
  olympic: '/overlays/olympic/scoreboard.html',
  modern: '/overlays/modern/scoreboard.html',
  arcade: '/overlays/arcade/scoreboard.html',
  olympicScoreboard: '/overlays/olympic/scoreboard.html',
  modernScoreboard: '/overlays/modern/scoreboard.html',
  arcadeScoreboard: '/overlays/arcade/scoreboard.html',
  // Player introductions
  olympicIntro: '/overlays/olympic/intro.html',
  modernIntro: '/overlays/modern/intro.html',
  arcadeIntro: '/overlays/arcade/intro.html',
  // Results
  modernResult: '/overlays/modern/result.html',
  arcadeResult: '/overlays/arcade/result.html',
  // Winners
  modernWinner: '/overlays/modern/winner.html',
  arcadeWinner: '/overlays/arcade/winner.html',
  // Video replay
  olympicVideoReplay: '/overlays/olympic/replay.html',
  modernVideoReplay: '/overlays/modern/replay.html',
  arcadeVideoReplay: '/overlays/arcade/replay.html',
};

const TITLES: Record<OverlayWindowId, string> = {
  // Primary scoreboard windows (aliases share titles)
  olympic: 'reStrike VTA – Olympic Scoreboard',
  olympicScoreboard: 'reStrike VTA – Olympic Scoreboard',
  modern: 'reStrike VTA – Modern Scoreboard',
  modernScoreboard: 'reStrike VTA – Modern Scoreboard',
  arcade: 'reStrike VTA – Arcade Scoreboard',
  arcadeScoreboard: 'reStrike VTA – Arcade Scoreboard',
  // Player introductions
  olympicIntro: 'reStrike VTA – Olympic Players',
  modernIntro: 'reStrike VTA – Modern Players',
  arcadeIntro: 'reStrike VTA – Arcade Players',
  // Results
  modernResult: 'reStrike VTA – Modern Match Result',
  arcadeResult: 'reStrike VTA – Arcade Match Result',
  // Winners
  modernWinner: 'reStrike VTA – Modern Winner',
  arcadeWinner: 'reStrike VTA – Arcade Winner',
  // Video replay
  olympicVideoReplay: 'reStrike VTA – Olympic Video Replay',
  modernVideoReplay: 'reStrike VTA – Modern Video Replay',
  arcadeVideoReplay: 'reStrike VTA – Arcade Video Replay',
};

export function resolveOverlayLabel(kind: OverlayWindowId): string {
  return LABELS[kind];
}

export async function openOverlayWindow(kind: OverlayWindowId): Promise<void> {
  if (typeof window === 'undefined') return;

  const label = LABELS[kind];
  const path = URLS[kind];

  let url: string;
  const origin = window.location.origin;
  if (origin && origin.startsWith('http')) {
    url = `${origin}${path}`;
  } else {
    // Packaged app: let Tauri's asset protocol resolve the path.
    url = path;
  }

  // Create a new dedicated overlay window; if this fails, the caller's
  // try/catch will see the error.
  new WebviewWindow(label, {
    url,
    title: TITLES[kind] ?? 'reStrike VTA – Overlay',
    visible: true,
    // Standard OS frame so the user can move/close it.
    decorations: true,
    // Allow resizing; SVG content will scale responsively.
    resizable: true,
    fullscreen: false,
    alwaysOnTop: false,
    // Use a conservative size so it fits on
    // smaller displays and doesn't cover the app.
    width: 1280,
    height: 720,
  });
}
