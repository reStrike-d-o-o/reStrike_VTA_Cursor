import { useEffect, useRef } from 'react';
import { useLiveDataStore, LiveDataWebSocket, parsePssEvent, PssEventData } from '../stores/liveDataStore';
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
      if (data.type === 'pss_event' && data.data) {
        const eventData = data.data;
        
        // Create event directly from structured data instead of parsing raw_data
        const event: PssEventData = {
          id: `${eventData.type}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
          eventType: eventData.type as any,
          eventCode: eventData.event_code || '',
          athlete: eventData.athlete as any,
          round: eventData.round || 1,
          time: eventData.time || '0:00',
          timestamp: eventData.timestamp || new Date().toISOString(),
          rawData: eventData.raw_data || '',
          description: eventData.description || '',
        };
        
        useLiveDataStore.getState().addEvent(event);
        
        // Update round and time if provided
        if (eventData.round) {
          useLiveDataStore.getState().setCurrentRound(eventData.round);
        }
        if (eventData.time) {
          useLiveDataStore.getState().setCurrentTime(eventData.time);
        }
      } else if (data.type === 'connection') {
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