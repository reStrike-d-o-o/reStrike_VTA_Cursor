import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

export type OverlayWindowId = 'olympic' | 'modern' | 'arcade';

const LABELS: Record<OverlayWindowId, string> = {
  olympic: 'overlay_olympic',
  modern: 'overlay_modern',
  arcade: 'overlay_arcade',
};

const URLS: Record<OverlayWindowId, string> = {
  olympic: '/overlays/olympic/scoreboard.html',
  modern: '/overlays/modern/scoreboard.html',
  arcade: '/overlays/arcade/scoreboard.html',
};

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
    title: '',
    visible: true,
    // Use standard OS window frame so the user can
    // easily move and close the overlay.
    decorations: true,
    resizable: false,
    fullscreen: false,
    alwaysOnTop: false,
    // Use a conservative size so it fits on
    // smaller displays and doesn't cover the app.
    width: 1280,
    height: 720,
  });
}
