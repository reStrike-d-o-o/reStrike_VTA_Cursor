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
    console.log('🔄 useLiveDataEvents effect - Manual mode state:', isManualModeEnabled);
    console.log('🔌 Current connection count:', connectionCount);
    
    // Only connect when manual mode is OFF
    if (isManualModeEnabled) {
      console.log('🚫 Manual mode enabled - disconnecting WebSocket and clearing events');
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
    console.log('🔌 Connection count incremented to:', connectionCount);

    // Only create WebSocket if it doesn't exist
    if (!globalWebSocket) {
      console.log('🔌 Creating new WebSocket connection to ws://localhost:3001');
      
      // Create WebSocket connection with inline message handler
      globalWebSocket = new LiveDataWebSocket('ws://localhost:3001', (data: any) => {
        console.log('📡 Received WebSocket message:', data);
        console.log('📡 Message type:', data.type);
        console.log('📡 Message data:', data.data);
        
        // Handle different message types using direct store access
        if (data.type === 'pss_event' && data.data) {
          const eventData = data.data;
          console.log('📊 Processing PSS event:', eventData);
          
          // Always use backend event_code for live events
          if (typeof eventData.event_code !== 'string' || !eventData.event_code) {
            console.warn('⚠️ Live event missing event_code from backend:', eventData);
          }
          
          // Validate and normalize athlete field
          let normalizedAthlete: 'blue' | 'red' | 'yellow';
          if (eventData.athlete === 'blue' || eventData.athlete === 'red' || eventData.athlete === 'yellow') {
            normalizedAthlete = eventData.athlete;
          } else {
            // Default to yellow for unknown values
            normalizedAthlete = 'yellow';
          }
          
          // Create event directly from structured data instead of parsing raw_data
          const event: PssEventData = {
            id: `${eventData.event_type}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
            eventType: eventData.event_type || '',
            eventCode: eventData.event_code || '', // Always from backend
            athlete: normalizedAthlete,
            round: eventData.round || 1,
            time: eventData.time || '0:00',
            timestamp: eventData.timestamp || new Date().toISOString(),
            rawData: eventData.raw_data || '',
            description: eventData.description || '',
            action: eventData.action,
            structuredData: eventData.structured_data,
          };
          
          // Debug logging for event creation
          console.log('🔍 Event creation debug:', {
            receivedAthlete: eventData.athlete,
            receivedEventCode: eventData.event_code,
            receivedEventType: eventData.event_type,
            receivedTime: eventData.time,
            receivedRound: eventData.round,
            createdEvent: event
          });
          
          console.log('📊 Adding event to store:', event);
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
        } else {
          // Log other message types for debugging
          console.log('📡 Other message type:', data.type, data);
        }
      });

      globalWebSocket.connect();
    } else {
      console.log('🔌 Reusing existing WebSocket connection');
    }

    wsRef.current = globalWebSocket;

    // Cleanup function
    return () => {
      connectionCount--;
      console.log('🔌 Connection count decremented to:', connectionCount);
      
      // Only disconnect if no more components are using the connection
      if (connectionCount <= 0 && globalWebSocket) {
        console.log('🔌 No more components using WebSocket - disconnecting');
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
  
  console.log('📊 useLiveDataEvents current state:', currentState);
  
  return currentState;
}; 