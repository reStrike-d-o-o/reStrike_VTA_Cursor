import { useEffect } from 'react';
import { useOverlayRoutingStore } from '../stores/overlayRoutingStore';
import type { OverlayId, OverlayRoutingRule, OverlayTriggerType } from '../types';
import { obsObwsCommands } from '../utils/tauriCommandsObws';

// Local visibility cache so "toggle" does not depend on OBS state.
const overlayVisibility: Partial<Record<OverlayId, boolean>> = {};

interface ObsSourceTarget {
  scene: string;
  source: string;
}

function mapOverlayToObsTarget(overlay: OverlayId): ObsSourceTarget | null {
  switch (overlay) {
    // Olympic theme
    case 'olympicScoreboard':
      return { scene: 'OLYMPIC', source: 'Scoreboard' };
    case 'olympicPlayers':
      return { scene: 'OLYMPIC', source: 'Players' };
    case 'olympicMatchResult':
      return { scene: 'OLYMPIC', source: 'MatchResult' };
    case 'olympicWinner':
      return { scene: 'OLYMPIC', source: 'Winner' };
    case 'olympicVideoReplay':
      return { scene: 'OLYMPIC', source: 'VideoReplay' };
    // Modern theme
    case 'modernScoreboard':
      return { scene: 'MODERN', source: 'Scoreboard' };
    case 'modernPlayers':
      return { scene: 'MODERN', source: 'Players' };
    case 'modernMatchResult':
      return { scene: 'MODERN', source: 'MatchResult' };
    case 'modernWinner':
      return { scene: 'MODERN', source: 'Winner' };
    case 'modernVideoReplay':
      return { scene: 'MODERN', source: 'VideoReplay' };
    // Arcade theme
    case 'arcadeScoreboard':
      return { scene: 'ARCADE', source: 'Scoreboard' };
    case 'arcadePlayers':
      return { scene: 'ARCADE', source: 'Players' };
    case 'arcadeMatchResult':
      return { scene: 'ARCADE', source: 'MatchResult' };
    case 'arcadeWinner':
      return { scene: 'ARCADE', source: 'Winner' };
    case 'arcadeVideoReplay':
      return { scene: 'ARCADE', source: 'VideoReplay' };
    case 'none':
    default:
      return null;
  }
}

async function applyOverlayAction(overlay: OverlayId, action: OverlayRoutingRule['action']) {
  if (overlay === 'none') return;
  const target = mapOverlayToObsTarget(overlay);
  if (!target) return;

  const current = overlayVisibility[overlay] ?? false;
  let nextVisible = current;

  if (action === 'show') {
    nextVisible = true;
  } else if (action === 'hide') {
    nextVisible = false;
  } else if (action === 'toggle') {
    nextVisible = !current;
  }

  overlayVisibility[overlay] = nextVisible;
  try {
    await obsObwsCommands.setSourceVisibility(target.scene, target.source, nextVisible);
  } catch {
    // Errors are already logged in the command helper; do not throw into UI loop.
  }
}

export function useOverlayController(): void {
  useEffect(() => {
    if (typeof window === 'undefined') {
      return;
    }

    const handleBrowserPssEvent = (evt: Event) => {
      const custom = evt as CustomEvent<any>;
      const payload = custom.detail;
      if (!payload || typeof payload !== 'object') {
        return;
      }

      const trigger = mapEventToTriggerFromPayload(payload);
      if (!trigger) {
        return;
      }

      const rules = useOverlayRoutingStore.getState().rules;
      const matchingRules = rules.filter((r) => r.trigger === trigger);
      if (!matchingRules.length) {
        return;
      }

      matchingRules.forEach((rule) => {
        void applyOverlayAction(rule.overlay, rule.action);
      });
    };

    window.addEventListener('pss-event', handleBrowserPssEvent as EventListener);

    return () => {
      window.removeEventListener('pss-event', handleBrowserPssEvent as EventListener);
    };
  }, []);
}

function mapEventToTriggerFromPayload(event: any): OverlayTriggerType | null {
  const rawType = String(event.type ?? event.event ?? '').toLowerCase();

  switch (rawType) {
    case 'fight_loaded':
    case 'fightloaded':
      return 'fight_loaded';
    case 'fight_ready':
    case 'fightready':
      return 'fight_ready';
    case 'winner':
      return 'winner';
    case 'injury':
      if (event.visible === true || event.injury_visible === true) {
        return 'injury_show';
      }
      if (event.visible === false || event.injury_visible === false) {
        return 'injury_hide';
      }
      return null;
    case 'round':
      // Map generic round events to round_start for now
      return 'round_start';
    default:
      return null;
  }
}
