import { useEffect, useRef } from 'react';
import { useLiveDataStore, LiveDataWebSocket, parsePssEvent, PssEventData } from '../stores/liveDataStore';
import { useAppStore } from '../stores/index';

// Singleton WebSocket instance to prevent multiple connections
let globalWebSocket: LiveDataWebSocket | null = null;
let connectionCount = 0;
let obsEventListenerRegistered = false;
const recentEventSignatures: Map<string, number> = new Map();
const DUP_WINDOW_MS = 1500;
// Skip the first zero-time round/clock event immediately after a fight_ready clear
let suppressZeroTimeAfterReady = false;

export const useLiveDataEvents = () => {
  const wsRef = useRef<LiveDataWebSocket | null>(null);
  const recordStartedTimeoutRef = useRef<number | null>(null);
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
          
          // Keep currentRound in sync with backend whenever a round number is present
          if (typeof eventData.round === 'number') {
            useLiveDataStore.getState().setCurrentRound(eventData.round);
          } else if (typeof (eventData.current_round) === 'number') {
            useLiveDataStore.getState().setCurrentRound(eventData.current_round);
          } else if (eventData.event_type === 'round' && typeof eventData.round === 'number') {
            // Fallback (kept for completeness)
            useLiveDataStore.getState().setCurrentRound(eventData.round);
          }
          // Only update time if it's a valid time (not "0:00" or empty)
          if (eventData.time && eventData.time !== '0:00') {
            useLiveDataStore.getState().setCurrentRoundTime(eventData.time);
          }
          
          // Get current time and round from store for this event (AFTER updating)
          const currentStore = useLiveDataStore.getState();
          
          // Clear Event Table at start-of-match signals
          // Handle fight_ready event - clear Event Table events and suppress first zero-time tick
          if (eventData.event_type === 'fight_ready') {
            console.log('🎯 Fight ready event received via WebSocket - clearing Event Table events');
            useLiveDataStore.getState().clearEvents();
            suppressZeroTimeAfterReady = true;
            return; // Don't add fight_ready events to the Event Table
          }

          // Also clear on fight_loaded to avoid stale events when a new fight loads before ready
          if (eventData.event_type === 'fight_loaded') {
            console.log('🎯 Fight loaded event - pre-clearing Event Table events');
            useLiveDataStore.getState().clearEvents();
            // do not return; we still want subsequent setup events to display if needed
          }

          // On winner event: persist current Event Table to DB for this match
          if (eventData.event_type === 'winner') {
            try {
              const current = useLiveDataStore.getState();
              const eventsToStore = current.events;
              if (eventsToStore.length > 0) {
                const { invoke } = require('@tauri-apps/api/core');
                // Try to infer match number from PSS store if available
                const { usePssMatchStore } = require('../stores/pssMatchStore');
                const matchNum: number | undefined = usePssMatchStore.getState().matchData.matchConfig?.number;
                const matchId = (matchNum ?? 'current').toString();
                (async () => {
                  for (const ev of [...eventsToStore].reverse()) {
                    try {
                      await invoke('store_pss_event', {
                        eventData: {
                          match_id: matchId,
                          event_code: ev.eventCode,
                          athlete: ev.athlete,
                          round: ev.round,
                          time: ev.time,
                          timestamp: ev.timestamp,
                          raw_data: ev.rawData,
                        },
                      });
                    } catch (err) {
                      console.warn('⚠️ Failed to store event:', err);
                    }
                  }
                })();
              }
            } catch (e) {
              console.warn('⚠️ Failed to persist Event Table on winner:', e);
            }
            // Do not early-return; also add winner event to list below
          }
          
          // Suppress the first zero-time 'round' or 'clock' event following a ready
          if (
            suppressZeroTimeAfterReady &&
            (eventData.event_type === 'round' || eventData.event_type === 'clock') &&
            (!eventData.time || eventData.time === '0:00')
          ) {
            suppressZeroTimeAfterReady = false; // one-time suppression
            return;
          }

          // Create event directly from structured data instead of parsing raw_data
          // Prefer backend-provided round over store to stamp correct RND on rows
          const effectiveRound = (typeof eventData.round === 'number')
            ? eventData.round
            : (typeof (eventData.current_round) === 'number'
              ? eventData.current_round
              : currentStore.currentRound);

          const event: PssEventData = {
            id: `${eventData.event_type}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
            eventType: eventData.event_type || '',
            eventCode: eventData.event_code || '', // Always from backend
            athlete: normalizedAthlete,
            round: effectiveRound,
            time: currentStore.currentRoundTime, // ALWAYS use current store time
            timestamp: eventData.timestamp || new Date().toISOString(),
            rawData: eventData.raw_data || '',
            description: eventData.description || '',
            action: eventData.action,
            structuredData: eventData.structured_data,
          };
          // Clear suppression once a non-zero-time or non-round/clock event arrives
          suppressZeroTimeAfterReady = false;

          // De-duplicate frequent duplicates using a short signature window
          const sig = `${event.eventType}|${event.eventCode}|${event.athlete}|${event.round}|${event.time}|${event.timestamp}`;
          const nowMs = Date.now();
          const lastSeen = recentEventSignatures.get(sig) || 0;
          if (nowMs - lastSeen < DUP_WINDOW_MS) {
            // skip duplicate
          } else {
            recentEventSignatures.set(sig, nowMs);
            if (recentEventSignatures.size > 400) {
              const cutoff = nowMs - DUP_WINDOW_MS * 4;
              for (const [k, v] of recentEventSignatures) {
                if (v < cutoff) recentEventSignatures.delete(k);
              }
            }
            useLiveDataStore.getState().addEvent(event);
          }
        } else if (data.type === 'connection') {
          useLiveDataStore.getState().setConnectionStatus(data.connected);
        } else if (data.type === 'error') {
          // Silently handle WebSocket errors
        }
      });

      globalWebSocket.connect();

      // Listen for OBS events from backend to detect actual recording start
      if (window.__TAURI__?.event?.listen && !obsEventListenerRegistered) {
        obsEventListenerRegistered = true;
        window.__TAURI__.event.listen('obs_event', (evt: any) => {
          try {
            const payload = evt?.payload;
            if (payload && payload.type === 'RecordStateChanged') {
              const isRecording = !!payload.is_recording || !!payload.isRecording || !!payload.isRecordingActive;
              if (isRecording) {
                if (recordStartedTimeoutRef.current) window.clearTimeout(recordStartedTimeoutRef.current);
                recordStartedTimeoutRef.current = window.setTimeout(() => {
                  console.log('🧹 Clearing Event Table 500ms after recording started');
                  useLiveDataStore.getState().clearEvents();
                }, 500);
              }
            }
          } catch {}
        });
      }
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