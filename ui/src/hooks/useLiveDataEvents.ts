import { useEffect, useRef, useCallback } from 'react';
import { useLiveDataStore, LiveDataWebSocket, parsePssEvent } from '../stores/liveDataStore';
import { useAppStore } from '../stores/index';

export const useLiveDataEvents = () => {
  const wsRef = useRef<LiveDataWebSocket | null>(null);
  const storeRef = useRef<{
    addEvent: any;
    setCurrentRound: any;
    setCurrentTime: any;
    setConnectionStatus: any;
    clearEvents: any;
  } | null>(null);
  
  const { 
    addEvent, 
    setCurrentRound, 
    setCurrentTime, 
    setConnectionStatus,
    clearEvents 
  } = useLiveDataStore();
  
  const { isManualModeEnabled } = useAppStore();

  // Update the ref with latest store functions
  useEffect(() => {
    storeRef.current = {
      addEvent,
      setCurrentRound,
      setCurrentTime,
      setConnectionStatus,
      clearEvents
    };
  }, [addEvent, setCurrentRound, setCurrentTime, setConnectionStatus, clearEvents]);

  // Memoize the WebSocket message handler to prevent infinite loops
  const handleWebSocketMessage = useCallback((data: any) => {
    console.log('📡 Received WebSocket message:', data);
    
    const store = storeRef.current;
    if (!store) return;
    
    // Handle different message types
    if (data.type === 'pss_event') {
      const event = parsePssEvent(data.raw_data, data.timestamp);
      if (event) {
        store.addEvent(event);
        
        // Update round and time if provided
        if (data.round) {
          store.setCurrentRound(data.round);
        }
        if (data.time) {
          store.setCurrentTime(data.time);
        }
      }
    } else if (data.type === 'connection_status') {
      store.setConnectionStatus(data.connected);
    } else if (data.type === 'error') {
      console.error('WebSocket error:', data.message);
    }
  }, []); // Empty dependency array - no dependencies

  useEffect(() => {
    // Only connect when manual mode is OFF
    if (isManualModeEnabled) {
      if (wsRef.current) {
        wsRef.current.disconnect();
        wsRef.current = null;
        setConnectionStatus(false);
      }
      // Clear events when manual mode is enabled
      clearEvents();
      return;
    }

    // Create WebSocket connection
    const ws = new LiveDataWebSocket('ws://localhost:3001', handleWebSocketMessage);

    wsRef.current = ws;
    ws.connect();

    // Cleanup on unmount or when manual mode is enabled
    return () => {
      if (wsRef.current) {
        wsRef.current.disconnect();
        wsRef.current = null;
        setConnectionStatus(false);
      }
    };
  }, [isManualModeEnabled]); // Removed all function dependencies

  return {
    isConnected: useLiveDataStore.getState().isConnected,
    eventCount: useLiveDataStore.getState().events.length,
  };
}; 