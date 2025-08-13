import { useEffect, useRef } from 'react';
import { usePssMatchStore } from '../stores/pssMatchStore';
import { handlePssEvent } from '../utils/pssEventHandler';
import { pssCommands } from '../utils/tauriCommands';

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
          // Event listener already set up, skipping...
    return;
  }

  // Setting up PSS event listener...

    try {
      // Check if Tauri is available
      if (typeof window === 'undefined') {
        console.warn('⚠️ Window is undefined, cannot set up event listener');
        return;
      }

      if (!window.__TAURI__) {
        console.warn('⚠️ Tauri is not available, cannot set up event listener');
        return;
      }

      if (!window.__TAURI__.event) {
        console.warn('⚠️ Tauri event system is not available');
        return;
      }

      // Tauri environment is available

      // Set up the event listener on the backend
      // Setting up backend event listener...
      await pssCommands.setupEventListener();
      // Backend event listener setup complete
      
      // Listen for PSS events from the backend
      // Setting up frontend PSS event listener...
      const unlisten = await window.__TAURI__.event.listen('pss_event', (event: any) => {
        // Received PSS event
        
        // Ensure we have a valid payload
        if (event.payload && typeof event.payload === 'object') {
          // Valid payload received, processing event...
          handlePssEvent(event.payload);
        } else {
          // Invalid PSS event payload
        }
      });
      
      // Listen for log events from the backend
      // Setting up frontend log event listener...
      const logUnlisten = await window.__TAURI__.event.listen('log_event', (event: any) => {
        // Log event
        // You can add log event handling here for the Live Data panel
      });

      // NOTE: We only listen to the dedicated custom event to avoid double prompts

      // Also listen to a dedicated event to ensure delivery
      const pathDecisionUnlisten2 = await window.__TAURI__.event.listen('obs_path_decision_needed', async (event: any) => {
        try {
          const payload = event.payload;
          if (!payload || typeof payload !== 'object') return;
          const cont = payload.continue;
          const nw = payload.new;
          const body = `Continue with ${cont.tournament} / ${cont.day}\n\nOr create ${nw.tournament} / ${nw.day}?`;
          const ok = await (await import('../stores/messageCenter')).useMessageCenter.getState().confirm({
            title: 'Select recording path context',
            body,
            confirmText: 'Continue',
            cancelText: 'New Tournament',
          });
          const { obsObwsCommands } = await import('../utils/tauriCommandsObws');
          if (ok) {
            await obsObwsCommands.applyPathDecision(cont.tournament, cont.day);
          } else {
            await obsObwsCommands.applyPathDecision(nw.tournament, nw.day);
          }
        } catch {}
      });
      
      listenerRef.current = () => {
        // Cleaning up event listeners...
        unlisten();
        logUnlisten();
        pathDecisionUnlisten2();
      };
      isListeningRef.current = true;
      // PSS event listener setup complete
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