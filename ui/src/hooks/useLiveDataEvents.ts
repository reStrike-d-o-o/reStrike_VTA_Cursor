import { useEffect, useRef } from 'react';
import { useLiveDataStore, LiveDataType } from '../stores/liveDataStore';

/**
 * Hook that subscribes to backend "live_data" events via Tauri v2 event system
 * and pushes them into the LiveDataStore. It automatically cleans up on
 * component unmount or when `enabled` flag changes.
 */
export const useLiveDataEvents = (enabled: boolean, selectedType: LiveDataType) => {
  const { addData } = useLiveDataStore();
  const listenerRef = useRef<Promise<() => void> | null>(null);

  useEffect(() => {
    // Helper to attach listener
    const attach = async () => {
      if (!enabled) return;

      try {
        if (typeof window === 'undefined' || !window.__TAURI__?.event) {
          console.warn('Tauri event API not available – running in web mode?');
          return;
        }

        // Listen for generic live_data events (emitted from Rust backend)
        listenerRef.current = window.__TAURI__.event.listen(
          'live_data',
          (event: any) => {
            try {
              const payload = event.payload || {};
              const subsystem: LiveDataType = (payload.subsystem as LiveDataType) || 'pss';
              if (subsystem !== selectedType) return; // ignore other subsystems

              const data: string = payload.data || JSON.stringify(payload);

              addData({ subsystem, data, type: 'info' });
            } catch (err) {
              console.error('Error processing live_data payload:', err);
            }
          }
        );
      } catch (err) {
        console.error('Failed to register live_data listener:', err);
      }
    };

    // Attach on mount / when enabled turns true
    attach();

    // Cleanup on disable/unmount
    return () => {
      if (listenerRef.current) {
        listenerRef.current.then((unlisten) => unlisten()).catch(() => {});
        listenerRef.current = null;
      }
    };
  }, [enabled, selectedType, addData]);
}; 