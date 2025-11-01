import { useEffect, useRef } from 'react';
import { usePssMatchStore } from '../stores/pssMatchStore';
import { handlePssEvent } from '../utils/pssEventHandler';
import { useLiveDataStore } from '../stores/liveDataStore';
import { pssCommands } from '../utils/tauriCommands';
import { canListenTauri, listenTauri } from '../utils/tauriBridge';

export interface PssEvent {
  type: string;
  description: string;
  [key: string]: any;
}

export const usePssEvents = () => {
  const {
    setMatchLoaded,
    updateAthletes,
    updateMatchConfig,
    updateScores,
    updateCurrentScores,
    updateWinnerRounds,
    resetMatchData,
  } = usePssMatchStore();

  const listenerRef = useRef<any>(null);
  const isListeningRef = useRef(false);

  // Set up real-time PSS event listener using Tauri v2
  const setupEventListener = async () => {
  if (isListeningRef.current) {
    return;
  }

  try {
    if (typeof window === 'undefined') {
      console.warn('?? Window is undefined, cannot set up event listener');
      return;
    }

    if (!canListenTauri()) {
      console.warn('?? Tauri event system is not available');
      return;
    }

    await pssCommands.setupEventListener();

    const unlisten = await listenTauri('pss_event', (event: any) => {
      if (event.payload && typeof event.payload === 'object') {
        const payload = event.payload;
        if (payload?.type === 'fight_loaded' || payload?.event === 'FightLoaded') {
          try { useLiveDataStore.getState().clearEvents(); } catch {}
        }
        if (payload?.type === 'fight_ready' || payload?.event === 'FightReady') {
          try { useLiveDataStore.getState().clearEvents(); } catch {}
        }
        if (payload?.type === 'round' && typeof payload?.round === 'number') {
          try { useLiveDataStore.getState().setCurrentRound(payload.round); } catch {}
        }
        if (payload?.type === 'clock' && typeof payload?.time === 'string' && payload.time !== '0:00') {
          try { useLiveDataStore.getState().setCurrentRoundTime(payload.time); } catch {}
        }
        handlePssEvent(payload);
      }
    });

    const logUnlisten = await listenTauri('log_event', (event: any) => {
      try {
        const payload = event?.payload;
        if (!payload || typeof payload !== 'object') {
          return;
        }
        const message = typeof payload.message === 'string' ? payload.message : '';
        if (!message) {
          return;
        }
        const timestamp =
          typeof payload.timestamp === 'number' ? payload.timestamp : Date.now();
        useLiveDataStore.getState().addLog(message, timestamp);
      } catch (error) {
        console.warn('Failed to process log_event payload', error);
      }
    });

    const pathDecisionUnlisten2 = await listenTauri('obs_path_decision_needed', async (event: any) => {
      try {
        const payload = event.payload;
        if (!payload || typeof payload !== 'object') return;
        const cont = payload.continue;
        const nw = payload.new;
        const { useMessageCenter } = await import('../stores/messageCenter');
        const choice = await useMessageCenter.getState().choose({
          title: 'Select recording path context',
          body: 'Would you like to CONTINUE with existent day, start a NEXT day or create a NEW tournament?',
          severity: 'info',
          choices: [
            { text: 'Continue', value: 'continue' },
            { text: 'Next', value: 'next' },
            { text: 'New', value: 'new' },
          ],
        });
        const { obsObwsCommands } = await import('../utils/tauriCommandsObws');
        if (choice === 'continue') {
          await obsObwsCommands.applyPathDecision(cont.tournament, cont.day);
        } else if (choice === 'next') {
          const m = typeof cont?.day === 'string' ? cont.day.match(/Day\s*(\d+)/i) : null;
          const currentNum = m && m[1] ? parseInt(m[1], 10) : 0;
          const nextDay = `Day ${currentNum > 0 ? currentNum + 1 : 1}`;
          await obsObwsCommands.applyPathDecision(cont.tournament, nextDay);
        } else if (choice === 'new') {
          await obsObwsCommands.applyPathDecision(nw.tournament, 'Day 1');
        }
      } catch {}
    });

    listenerRef.current = () => {
      unlisten();
      logUnlisten();
      pathDecisionUnlisten2();
    };
    isListeningRef.current = true;
  } catch (error) {
    // Failed to setup PSS event listener
  }
};

// Clean up event listener
const cleanupEventListener = () => {
    if (listenerRef.current) {
      try {
        listenerRef.current();
        listenerRef.current = null;
        isListeningRef.current = false;
        // PSS event listener cleaned up
      } catch (error) {
        // Error cleaning up PSS event listener
      }
    }
  };

  // Fetch any pending events (fallback for missed events)
  const fetchPendingEvents = async () => {
    try {
      // Fetching pending events...
      const result = await pssCommands.getEvents();
      
      if (result && result.success && result.data && Array.isArray(result.data)) {
        // Fetching pending events
        result.data.forEach((event: PssEvent) => {
          handlePssEvent(event);
        });
      } else {
        // No pending events to fetch or invalid response
      }
    } catch (error) {
      // Error fetching pending events
    }
  };

  // Emit a test event to verify the system is working
  const emitTestEvent = async (eventData: any) => {
    try {
      // Emitting test event
      await pssCommands.emitEvent(eventData);
      // Test event emitted successfully
    } catch (error) {
      // Failed to emit test event
    }
  };

  // Emit any pending events
  const emitPendingEvents = async () => {
    try {
      // Emitting pending events...
      await pssCommands.emitPendingEvents();
      // Pending events emitted
    } catch (error) {
      // Error emitting pending events
    }
  };

  // Set up event listener on mount
  useEffect(() => {
    // usePssEvents hook mounted
    setupEventListener();
    
    // Clean up on unmount
    return () => {
      // usePssEvents hook unmounting
      cleanupEventListener();
    };
  }, []);

  return {
    setupEventListener,
    cleanupEventListener,
    fetchPendingEvents,
    emitTestEvent,
    emitPendingEvents,
  };
}; 



