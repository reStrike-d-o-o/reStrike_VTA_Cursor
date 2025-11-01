import { useEffect, useRef } from 'react';
import { useLiveDataStore, LiveDataWebSocket, parsePssEvent, PssEventData } from '../stores/liveDataStore';
import { useAppStore } from '../stores/index';
import { canListenTauri, listenTauri } from '../utils/tauriBridge';
import { pssCommands } from '../utils/tauriCommands';

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
    let cleanupWebSocket: (() => void) | null = null;
    let tauriUnlisten: (() => void) | null = null;
    let mounted = true;

    const teardownAll = () => {
      if (cleanupWebSocket) {
        cleanupWebSocket();
        cleanupWebSocket = null;
      }
      if (tauriUnlisten) {
        tauriUnlisten();
        tauriUnlisten = null;
        useLiveDataStore.getState().setConnectionStatus(false);
      }
    };

    if (isManualModeEnabled) {
      teardownAll();
      if (globalWebSocket) {
        globalWebSocket.disconnect();
        globalWebSocket = null;
      }
      connectionCount = 0;
      useLiveDataStore.getState().setConnectionStatus(false);
      useLiveDataStore.getState().clearEvents();
      return () => teardownAll();
    }

    const processMessage = (data: any) => {
      if (!mounted) {
        return;
      }
      if (data.type === 'pss_event' && data.data) {
        const eventData = data.data;

        if (typeof eventData.event_code !== 'string' || !eventData.event_code) {
          return;
        }

        let normalizedAthlete: 'blue' | 'red' | 'yellow';
        if (eventData.athlete === 'blue' || eventData.athlete === 'red' || eventData.athlete === 'yellow') {
          normalizedAthlete = eventData.athlete;
        } else {
          normalizedAthlete = 'yellow';
        }

        const prevRound = useLiveDataStore.getState().currentRound;
        const incomingRound =
          typeof eventData.round === 'number'
            ? eventData.round
            : typeof eventData.current_round === 'number'
            ? eventData.current_round
            : undefined;
        if (typeof incomingRound === 'number' && incomingRound >= prevRound) {
          useLiveDataStore.getState().setCurrentRound(incomingRound);
        }
        if (eventData.time && eventData.time !== '0:00') {
          useLiveDataStore.getState().setCurrentRoundTime(eventData.time);
        }

        const currentStore = useLiveDataStore.getState();

        if (eventData.event_type === 'fight_ready') {
          useLiveDataStore.getState().clearEvents();
          suppressZeroTimeAfterReady = true;
          return;
        }

        if (eventData.event_type === 'fight_loaded') {
          useLiveDataStore.getState().clearEvents();
        }

        if (eventData.event_type === 'winner') {
          try {
            const current = useLiveDataStore.getState();
            const eventsToStore = current.events;
            if (eventsToStore.length > 0) {
              const { invoke } = require('@tauri-apps/api/core');
              const { usePssMatchStore } = require('../stores/pssMatchStore');
              const matchNum: number | undefined =
                usePssMatchStore.getState().matchData.matchConfig?.number;
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
        }

        if (
          suppressZeroTimeAfterReady &&
          (eventData.event_type === 'round' || eventData.event_type === 'clock') &&
          (!eventData.time || eventData.time === '0:00')
        ) {
          suppressZeroTimeAfterReady = false;
          return;
        }

        const isSystem =
          eventData.event_type === 'clock' ||
          eventData.event_type === 'round' ||
          eventData.event_type === 'fight_ready' ||
          eventData.event_type === 'fight_loaded' ||
          eventData.event_type === 'match_config' ||
          eventData.event_type === 'athletes';
        const clockStarted =
          currentStore.currentRoundTime && currentStore.currentRoundTime !== '0:00';
        if (isSystem && !clockStarted) {
          return;
        }

        const effectiveRoundRaw =
          typeof eventData.round === 'number'
            ? eventData.round
            : typeof eventData.current_round === 'number'
            ? eventData.current_round
            : currentStore.currentRound;
        const effectiveRound = Math.max(currentStore.currentRound, effectiveRoundRaw);

        const event: PssEventData = {
          id: `${eventData.event_type}_${Date.now()}_${Math.random()
            .toString(36)
            .substr(2, 9)}`,
          eventType: eventData.event_type || '',
          eventCode: eventData.event_code || '',
          athlete: normalizedAthlete,
          round: effectiveRound,
          time: currentStore.currentRoundTime,
          timestamp: eventData.timestamp || new Date().toISOString(),
          rawData: eventData.raw_data || '',
          description: eventData.description || '',
          action: eventData.action,
          structuredData: eventData.structured_data,
        };
        suppressZeroTimeAfterReady = false;

        const sig = `${event.eventType}|${event.eventCode}|${event.athlete}|${event.round}|${event.time}|${event.timestamp}`;
        const nowMs = Date.now();
        const lastSeen = recentEventSignatures.get(sig) || 0;
        if (nowMs - lastSeen < DUP_WINDOW_MS) {
          return;
        }
        recentEventSignatures.set(sig, nowMs);
        if (recentEventSignatures.size > 400) {
          const cutoff = nowMs - DUP_WINDOW_MS * 4;
          for (const [k, v] of recentEventSignatures) {
            if (v < cutoff) recentEventSignatures.delete(k);
          }
        }
        useLiveDataStore.getState().addEvent(event);
      } else if (data.type === 'connection') {
        useLiveDataStore.getState().setConnectionStatus(data.connected);
      }
    };

    const attachObsListener = () => {
      if (window.__TAURI__?.event?.listen && !obsEventListenerRegistered) {
        obsEventListenerRegistered = true;
        window.__TAURI__.event.listen('obs_event', (evt: any) => {
          try {
            const payload = evt?.payload;
            if (payload && payload.type === 'RecordStateChanged') {
              const isRecording =
                !!payload.is_recording ||
                !!payload.isRecording ||
                !!payload.isRecordingActive;
              if (isRecording) {
                if (recordStartedTimeoutRef.current)
                  window.clearTimeout(recordStartedTimeoutRef.current);
                recordStartedTimeoutRef.current = window.setTimeout(() => {
                  useLiveDataStore.getState().clearEvents();
                }, 500);
              }
            }
          } catch {}
        });
      }
    };

    const startWebSocket = () => {
      connectionCount++;
      if (!globalWebSocket) {
        globalWebSocket = new LiveDataWebSocket('ws://localhost:3001', processMessage);
        globalWebSocket.connect();
        attachObsListener();
      }
      wsRef.current = globalWebSocket;
      return () => {
        connectionCount--;
        if (connectionCount <= 0 && globalWebSocket) {
          globalWebSocket.disconnect();
          globalWebSocket = null;
          connectionCount = 0;
          useLiveDataStore.getState().setConnectionStatus(false);
        }
      };
    };

    const startTauriListener = async () => {
      if (!canListenTauri()) {
        return false;
      }
      try {
        await pssCommands.setupEventListener();
      } catch (error) {
        console.warn('Failed to setup Tauri PSS listener:', error);
        return false;
      }
      try {
        tauriUnlisten = await listenTauri('pss_event', (event: any) => {
          if (event?.payload) {
            processMessage({ type: 'pss_event', data: event.payload });
          }
        });
        useLiveDataStore.getState().setConnectionStatus(true);
        attachObsListener();
        return true;
      } catch (error) {
        console.warn('Failed to register Tauri listener:', error);
        tauriUnlisten = null;
        return false;
      }
    };

    (async () => {
      const tauriOk = await startTauriListener();
      if (!tauriOk) {
        cleanupWebSocket = startWebSocket();
      }
    })();

    return () => {
      mounted = false;
      teardownAll();
    };
  }, [isManualModeEnabled]);

  const currentState = {
    isConnected: useLiveDataStore.getState().isConnected,
    eventCount: useLiveDataStore.getState().events.length,
  };
  
  return currentState;
}; 
