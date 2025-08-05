import { useEffect, useRef } from 'react';
import { useLiveDataStore, LiveDataWebSocket, parsePssEvent, PssEventData } from '../stores/liveDataStore';
import { useAppStore } from '../stores/index';

// Singleton WebSocket instance to prevent multiple connections
let globalWebSocket: LiveDataWebSocket | null = null;
let connectionCount = 0;

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
      if (globalWebSocket) {
        globalWebSocket.disconnect();
        globalWebSocket = null;
        connectionCount = 0;
        // Update connection status using direct store access
        useLiveDataStore.getState().setConnectionStatus(false);
      }
      // Clear events using direct store access
      useLiveDataStore.getState().clearEvents();
      return;
    }

    // Increment connection count
    connectionCount++;

    // Only create WebSocket if it doesn't exist
    if (!globalWebSocket) {
      
      // Create WebSocket connection with inline message handler
      globalWebSocket = new LiveDataWebSocket('ws://localhost:3001', (data: any) => {
        // Handle different message types using direct store access
        if (data.type === 'pss_event' && data.data) {
          const eventData = data.data;
          
          // Always use backend event_code for live events
          if (typeof eventData.event_code !== 'string' || !eventData.event_code) {
            // Silently handle missing event_code
          }
          
          // Validate and normalize athlete field
          let normalizedAthlete: 'blue' | 'red' | 'yellow';
          if (eventData.athlete === 'blue' || eventData.athlete === 'red' || eventData.athlete === 'yellow') {
            normalizedAthlete = eventData.athlete;
          } else {
            // Default to yellow for unknown values
            normalizedAthlete = 'yellow';
          }
          
          // Update round and time FIRST if provided, so the event gets the current values
          if (eventData.round) {
            useLiveDataStore.getState().setCurrentRound(eventData.round);
          }
          // Only update time if it's a valid time (not "0:00" or empty)
          if (eventData.time && eventData.time !== '0:00') {
            useLiveDataStore.getState().setCurrentRoundTime(eventData.time);
          }
          
          // Get current time and round from store for this event (AFTER updating)
          const currentStore = useLiveDataStore.getState();
          
          // Handle fight_ready event - clear Event Table events
          if (eventData.event_type === 'fight_ready') {
            console.log('🎯 Fight ready event received via WebSocket - clearing Event Table events');
            useLiveDataStore.getState().clearEvents();
            return; // Don't add fight_ready events to the Event Table
          }
          
          // Create event directly from structured data instead of parsing raw_data
          const event: PssEventData = {
            id: `${eventData.event_type}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
            eventType: eventData.event_type || '',
            eventCode: eventData.event_code || '', // Always from backend
            athlete: normalizedAthlete,
            round: eventData.round || currentStore.currentRound,
            time: currentStore.currentRoundTime, // ALWAYS use current store time
            timestamp: eventData.timestamp || new Date().toISOString(),
            rawData: eventData.raw_data || '',
            description: eventData.description || '',
            action: eventData.action,
            structuredData: eventData.structured_data,
          };
          

          

          useLiveDataStore.getState().addEvent(event);
        } else if (data.type === 'connection') {
          useLiveDataStore.getState().setConnectionStatus(data.connected);
        } else if (data.type === 'error') {
          // Silently handle WebSocket errors
        }
      });

      globalWebSocket.connect();
    }

    wsRef.current = globalWebSocket;

    // Cleanup function
    return () => {
      connectionCount--;
      
      // Only disconnect if no more components are using the connection
      if (connectionCount <= 0 && globalWebSocket) {
        globalWebSocket.disconnect();
        globalWebSocket = null;
        connectionCount = 0;
        useLiveDataStore.getState().setConnectionStatus(false);
      }
    };
  }, [isManualModeEnabled]);

  const currentState = {
    isConnected: useLiveDataStore.getState().isConnected,
    eventCount: useLiveDataStore.getState().events.length,
  };
  
  return currentState;
}; 