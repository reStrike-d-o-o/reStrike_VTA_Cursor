import { useEffect, useRef } from 'react';
import { useLiveDataStore, LiveDataWebSocket, parsePssEvent } from '../stores/liveDataStore';
import { useAppStore } from '../stores/index';

export const useLiveDataEvents = () => {
  const wsRef = useRef<LiveDataWebSocket | null>(null);
  const isManualModeRef = useRef<boolean>(false);
  
  const { isManualModeEnabled } = useAppStore();
  
  // Update the ref when manual mode changes
  useEffect(() => {
    isManualModeRef.current = isManualModeEnabled;
  }, [isManualModeEnabled]);

  useEffect(() => {
    // Only connect when manual mode is OFF
    if (isManualModeEnabled) {
      if (wsRef.current) {
        wsRef.current.disconnect();
        wsRef.current = null;
        // Update connection status using direct store access
        useLiveDataStore.getState().setConnectionStatus(false);
      }
      // Clear events using direct store access
      useLiveDataStore.getState().clearEvents();
      return;
    }

    // Create WebSocket connection with inline message handler
    const ws = new LiveDataWebSocket('ws://localhost:3001', (data: any) => {
      console.log('📡 Received WebSocket message:', data);
      
      // Handle different message types using direct store access
      if (data.type === 'pss_event') {
        const event = parsePssEvent(data.raw_data, data.timestamp);
        if (event) {
          useLiveDataStore.getState().addEvent(event);
          
          // Update round and time if provided
          if (data.round) {
            useLiveDataStore.getState().setCurrentRound(data.round);
          }
          if (data.time) {
            useLiveDataStore.getState().setCurrentTime(data.time);
          }
        }
      } else if (data.type === 'connection_status') {
        useLiveDataStore.getState().setConnectionStatus(data.connected);
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
        // Update connection status using direct store access
        useLiveDataStore.getState().setConnectionStatus(false);
      }
    };
  }, [isManualModeEnabled]); // Only depend on isManualModeEnabled

  return {
    isConnected: useLiveDataStore.getState().isConnected,
    eventCount: useLiveDataStore.getState().events.length,
  };
}; 