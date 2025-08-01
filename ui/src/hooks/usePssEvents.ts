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
      return;
    }

    try {
      // Set up the event listener on the backend
      await pssCommands.setupEventListener();
      
      // Listen for PSS events from the backend
      if (typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.event) {
        const unlisten = await window.__TAURI__.event.listen('pss_event', (event: any) => {
          console.log('📡 Received PSS event:', event);
          console.log('📡 Event payload:', event.payload);
          console.log('📡 Event payload type:', typeof event.payload);
          console.log('📡 Event payload keys:', Object.keys(event.payload || {}));
          
          // Ensure we have a valid payload
          if (event.payload && typeof event.payload === 'object') {
            handlePssEvent(event.payload);
          } else {
            console.warn('⚠️ Invalid PSS event payload:', event.payload);
          }
        });
        
        // Listen for log events from the backend
        const logUnlisten = await window.__TAURI__.event.listen('log_event', (event: any) => {
          console.log('📋 Log event:', event.payload);
          // You can add log event handling here for the Live Data panel
        });
        
        listenerRef.current = () => {
          unlisten();
          logUnlisten();
        };
        isListeningRef.current = true;
        console.log('✅ PSS event listener setup complete');
      }
    } catch (error) {
      console.error('❌ Failed to setup PSS event listener:', error);
    }
  };

  // Clean up event listener
  const cleanupEventListener = () => {
    if (listenerRef.current) {
      try {
        listenerRef.current();
        listenerRef.current = null;
        isListeningRef.current = false;
        console.log('🧹 PSS event listener cleaned up');
      } catch (error) {
        console.error('❌ Error cleaning up PSS event listener:', error);
      }
    }
  };

  // Fetch any pending events (fallback for missed events)
  const fetchPendingEvents = async () => {
    try {
      const result = await pssCommands.getEvents();
      
      if (result && result.success && result.data && Array.isArray(result.data)) {
        console.log('📋 Fetching pending events:', result.data.length);
        result.data.forEach((event: PssEvent) => {
          handlePssEvent(event);
        });
      } else {
        console.log('📋 No pending events to fetch or invalid response:', result);
      }
    } catch (error) {
      console.error('❌ Error fetching pending events:', error);
    }
  };

  // Manual event emission (for testing)
  const emitTestEvent = async (eventData: any) => {
    try {
      await pssCommands.emitEvent(eventData);
    } catch (error) {
      console.error('❌ Error emitting test event:', error);
    }
  };

  // Emit pending events to frontend
  const emitPendingEvents = async () => {
    try {
      await pssCommands.emitPendingEvents();
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