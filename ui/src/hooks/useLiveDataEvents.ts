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
    console.log('🔄 useLiveDataEvents effect - Manual mode state:', isManualModeEnabled);
    
    // Only connect when manual mode is OFF
    if (isManualModeEnabled) {
      console.log('🚫 Manual mode enabled - disconnecting WebSocket and clearing events');
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

    console.log('🔌 Attempting to connect to WebSocket at ws://localhost:3001');
    
    // Create WebSocket connection with inline message handler
    const ws = new LiveDataWebSocket('ws://localhost:3001', (data: any) => {
      console.log('📡 Received WebSocket message:', data);
      console.log('📡 Message type:', data.type);
      console.log('📡 Message data:', data.data);
      
      // Handle different message types using direct store access
      if (data.type === 'pss_event' && data.data) {
        const eventData = data.data;
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
          id: `${eventData.type}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
          eventType: eventData.type as any,
          eventCode: eventData.event_code || '', // Always from backend
          athlete: normalizedAthlete,
          round: eventData.round || 1,
          time: eventData.time || '0:00',
          timestamp: eventData.timestamp || new Date().toISOString(),
          rawData: eventData.raw_data || '',
          description: eventData.description || '',
        };
        
        // Debug logging for event creation
        console.log('🔍 Event creation debug:', {
          receivedAthlete: eventData.athlete,
          receivedEventCode: eventData.event_code,
          receivedEventType: eventData.type,
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

  const currentState = {
    isConnected: useLiveDataStore.getState().isConnected,
    eventCount: useLiveDataStore.getState().events.length,
  };
  
  console.log('📊 useLiveDataEvents current state:', currentState);
  
  return currentState;
}; 