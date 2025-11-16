import { useEffect } from 'react';
import { useOverlayRoutingStore } from '../stores/overlayRoutingStore';
import type { OverlayId, OverlayRoutingRule, OverlayTriggerType } from '../types';
import { openOverlayWindow } from '../utils/overlayWindows';

async function applyOverlayAction(overlay: OverlayId, action: OverlayRoutingRule['action']) {
  switch (overlay) {
    case 'olympic':
      if (action === 'show' || action === 'toggle') {
        await openOverlayWindow('olympic');
      }
      break;
    case 'modern':
      if (action === 'show' || action === 'toggle') {
        await openOverlayWindow('modern');
      }
      break;
    case 'arcade':
      if (action === 'show' || action === 'toggle') {
        await openOverlayWindow('arcade');
      }
      break;
    case 'none':
    default:
      break;
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
