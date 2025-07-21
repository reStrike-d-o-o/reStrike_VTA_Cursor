import { useEffect, useRef } from 'react';
import { usePssMatchStore } from '../stores/pssMatchStore';
import { handlePssEvent } from '../utils/pssEventHandler';
import { pssCommands } from '../utils/tauriCommands';

// Tauri v2 invoke function
const invoke = async (command: string, args?: any) => {
  try {
    if (typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.core) {
      return await window.__TAURI__.core.invoke(command, args);
    }
    throw new Error('Tauri v2 core module not available');
  } catch (error) {
    console.error('Tauri invoke failed:', error);
    throw error;
  }
};

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

  // Set up real-time PSS event listener using Tauri v2 (OBS pattern)
  const setupEventListener = async () => {
    if (isListeningRef.current) {
      console.log('🔄 PSS event listener already active');
      return;
    }

    try {
      console.log('🎯 Setting up PSS event listener (OBS pattern)...');

      // Ensure backend starts pushing events
      await invoke('pss_setup_event_listener');

      // Subscribe to events WITHOUT awaiting the promise (OBS pattern)
      const promise = window.__TAURI__?.event.listen('pss_event', (event: any) => {
        console.log('📡 Received PSS event:', event.payload);
        handlePssEvent(event.payload);
      });

      if (promise) {
        listenerRef.current = promise; // store promise for cleanup
        isListeningRef.current = true;
        console.log('✅ PSS event listener registration promise stored');
      } else {
        throw new Error('Tauri event API not available');
      }
    } catch (error) {
      console.error('❌ Failed to setup PSS event listener:', error);
    }
  };

  // Clean up event listener
  const cleanupEventListener = () => {
    if (listenerRef.current) {
      listenerRef.current.then((unlisten: () => void) => {
        try {
          unlisten();
          console.log('🧹 PSS event listener cleaned up');
        } catch (error) {
          console.error('❌ Error cleaning up PSS event listener:', error);
        }
      });
      listenerRef.current = null;
      isListeningRef.current = false;
    }
  };

  // Fetch any pending events (fallback for missed events)
  const fetchPendingEvents = async () => {
    try {
      console.log('📥 Fetching pending PSS events...');
      const result = await pssCommands.getEvents();
      console.log('📦 Received pending events result:', result);
      
      if (result.success && result.data && Array.isArray(result.data)) {
        result.data.forEach((event: PssEvent) => {
          console.log('🔄 Processing pending event:', event);
          handlePssEvent(event);
        });
      } else {
        console.log('📭 No pending events found');
      }
    } catch (error) {
      console.error('❌ Error fetching pending events:', error);
    }
  };

  // Manual event emission (for testing)
  const emitTestEvent = async (eventData: any) => {
    try {
      await pssCommands.emitEvent(eventData);
      console.log('📤 Test event emitted:', eventData);
    } catch (error) {
      console.error('❌ Error emitting test event:', error);
    }
  };

  // Emit pending events to frontend
  const emitPendingEvents = async () => {
    try {
      await pssCommands.emitPendingEvents();
      console.log('📤 Pending events emitted to frontend');
    } catch (error) {
      console.error('❌ Error emitting pending events:', error);
    }
  };

  useEffect(() => {
    // Set up event listener when component mounts
    setupEventListener();

    // Clean up when component unmounts
    return () => {
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