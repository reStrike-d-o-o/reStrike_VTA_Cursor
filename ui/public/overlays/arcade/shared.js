/* Arcade Overlay Shared Runtime (Full HD 1920x1080)
   - Listens to localStorage 'pss_event' (via ui/src/utils/eventBroadcaster.ts)
   - Fallback: listens to window 'pss-event' custom events (for local testing)
   - Exposes OverlayBus with subscribe(handler)
*/
(function () {
  const subs = new Set();
  function emit(e) {
    subs.forEach((h) => {
      try { h(e); } catch (_) {}
    });
  }
  const OverlayBus = {
    subscribe(handler) { subs.add(handler); return () => subs.delete(handler); },
  };
  // storage listener (from eventBroadcaster)
  window.addEventListener('storage', (e) => {
    if (e.key === 'pss_event' && e.newValue) {
      try {
        const parsed = JSON.parse(e.newValue);
        if (parsed?.type === 'pss_event') emit(parsed.data);
      } catch (_) {}
    }
  });
  // custom event fallback
  window.addEventListener('pss-event', (ev) => {
    if (ev?.detail) emit(ev.detail);
  });
  // Simple mapper to a normalized model
  function mapEvent(e) {
    const t = e?.type;
    switch (t) {
      case 'athletes':
        return { kind: 'athletes', blue: e.athlete1, red: e.athlete2 };
      case 'current_scores':
        return { kind: 'scores', blue: e.athlete1_score || 0, red: e.athlete2_score || 0 };
      case 'clock':
        return { kind: 'clock', time: e.time || '00:00', round: e.round || e.current_round || 1 };
      case 'round':
        return { kind: 'round', round: e.round || e.current_round || 1 };
      case 'points':
        return { kind: 'points', athlete: e.athlete, points: e.points || e.point_type || 1 };
      case 'hit_level':
        return { kind: 'hit', athlete: e.athlete, level: e.level || 0 };
      case 'warnings':
        return { kind: 'warning', athlete: e.athlete };
      case 'winner':
        return { kind: 'winner', athlete: e.athlete, method: e.method || 'WIN' };
      default:
        return { kind: 'raw', raw: e };
    }
  }
  window.__ARCADE_OVERLAY__ = { OverlayBus, mapEvent };
})();


