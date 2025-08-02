import { useEffect, useRef } from 'react';
import { useLiveDataStore, LiveDataWebSocket, parsePssEvent } from '../stores/liveDataStore';
import { useAppStore } from '../stores/index';

export const useLiveDataEvents = () => {
  const wsRef = useRef<LiveDataWebSocket | null>(null);
  const { 
    addEvent, 
    setCurrentRound, 
    setCurrentTime, 
    setConnectionStatus,
    clearEvents 
  } = useLiveDataStore();
  
  const { isManualModeEnabled } = useAppStore();

  useEffect(() => {
    // Only connect when manual mode is OFF
    if (isManualModeEnabled) {
      if (wsRef.current) {
        wsRef.current.disconnect();
        wsRef.current = null;
        setConnectionStatus(false);
      }
      return;
    }

    // Create WebSocket connection
    const ws = new LiveDataWebSocket('ws://localhost:8080', (data) => {
      console.log('📡 Received WebSocket message:', data);
      
      // Handle different message types
      if (data.type === 'pss_event') {
        const event = parsePssEvent(data.raw_data, data.timestamp);
        if (event) {
          addEvent(event);
          
          // Update round and time if provided
          if (data.round) {
            setCurrentRound(data.round);
          }
          if (data.time) {
            setCurrentTime(data.time);
          }
        }
      } else if (data.type === 'connection_status') {
        setConnectionStatus(data.connected);
      } else if (data.type === 'error') {
        console.error('WebSocket error:', data.message);
      }
    });

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
  }, [isManualModeEnabled, addEvent, setCurrentRound, setCurrentTime, setConnectionStatus]);

  // Clear events when manual mode is enabled
  useEffect(() => {
    if (isManualModeEnabled) {
      clearEvents();
    }
  }, [isManualModeEnabled, clearEvents]);

  return {
    isConnected: useLiveDataStore.getState().isConnected,
    eventCount: useLiveDataStore.getState().events.length,
  };
}; 