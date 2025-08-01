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
      console.log('🎯 Event listener already set up, skipping...');
      return;
    }

    console.log('🎯 Setting up PSS event listener...');

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

      console.log('✅ Tauri environment is available');

      // Set up the event listener on the backend
      console.log('🎯 Setting up backend event listener...');
      await pssCommands.setupEventListener();
      console.log('✅ Backend event listener setup complete');
      
      // Listen for PSS events from the backend
      console.log('🎯 Setting up frontend PSS event listener...');
      const unlisten = await window.__TAURI__.event.listen('pss_event', (event: any) => {
        console.log('📡 Received PSS event:', event);
        console.log('📡 Event payload:', event.payload);
        console.log('📡 Event payload type:', typeof event.payload);
        console.log('📡 Event payload keys:', Object.keys(event.payload || {}));
        
        // Ensure we have a valid payload
        if (event.payload && typeof event.payload === 'object') {
          console.log('✅ Valid payload received, processing event...');
          handlePssEvent(event.payload);
        } else {
          console.warn('⚠️ Invalid PSS event payload:', event.payload);
        }
      });
      
      // Listen for log events from the backend
      console.log('🎯 Setting up frontend log event listener...');
      const logUnlisten = await window.__TAURI__.event.listen('log_event', (event: any) => {
        console.log('📋 Log event:', event.payload);
        // You can add log event handling here for the Live Data panel
      });
      
      listenerRef.current = () => {
        console.log('🧹 Cleaning up event listeners...');
        unlisten();
        logUnlisten();
      };
      isListeningRef.current = true;
      console.log('✅ PSS event listener setup complete');
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
      console.log('📋 Fetching pending events...');
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

  // Emit a test event to verify the system is working
  const emitTestEvent = async (eventData: any) => {
    try {
      console.log('🧪 Emitting test event:', eventData);
      await pssCommands.emitEvent(eventData);
      console.log('✅ Test event emitted successfully');
    } catch (error) {
      console.error('❌ Failed to emit test event:', error);
    }
  };

  // Emit any pending events
  const emitPendingEvents = async () => {
    try {
      console.log('📤 Emitting pending events...');
      await pssCommands.emitPendingEvents();
      console.log('✅ Pending events emitted');
    } catch (error) {
      console.error('❌ Error emitting pending events:', error);
    }
  };

  // Set up event listener on mount
  useEffect(() => {
    console.log('🎯 usePssEvents hook mounted');
    setupEventListener();
    
    // Clean up on unmount
    return () => {
      console.log('🎯 usePssEvents hook unmounting');
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