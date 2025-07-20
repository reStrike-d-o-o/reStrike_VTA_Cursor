import { useEffect, useRef } from 'react';
import { pssCommands } from '../utils/tauriCommands';
import { handlePssEvent, processPssEvents } from '../utils/pssEventHandler';
import { usePssMatchStore } from '../stores';

/**
 * Hook to listen for PSS events and update the match store
 * Automatically fetches and processes PSS events from the backend
 * Only starts polling when a match is loaded (FightLoaded/FightReady events)
 */
export const usePssEvents = () => {
  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  const resetMatchData = usePssMatchStore((state) => state.resetMatchData);
  const isLoaded = usePssMatchStore((state) => state.matchData.isLoaded);

  // Function to fetch and process PSS events
  const fetchAndProcessEvents = async () => {
    try {
      const result = await pssCommands.getEvents();
      if (result.success && result.data && Array.isArray(result.data)) {
        // Process all events
        processPssEvents(result.data);
      }
    } catch (error) {
      console.error('Error fetching PSS events:', error);
    }
  };

  // Set up event listener
  useEffect(() => {
    // Initial fetch to check for existing match data
    fetchAndProcessEvents();

    // Only start polling if a match is already loaded
    if (isLoaded) {
      intervalRef.current = setInterval(fetchAndProcessEvents, 2000);
    }

    // Cleanup on unmount
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    };
  }, [isLoaded]);

  // Reset match data when component unmounts or when explicitly called
  useEffect(() => {
    return () => {
      resetMatchData();
    };
  }, [resetMatchData]);

  // Function to start polling (called when match is loaded)
  const startPolling = () => {
    if (!intervalRef.current) {
      intervalRef.current = setInterval(fetchAndProcessEvents, 2000);
    }
  };

  // Function to stop polling
  const stopPolling = () => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  };

  return {
    isLoaded,
    fetchAndProcessEvents,
    resetMatchData,
    startPolling,
    stopPolling,
  };
};

/**
 * Hook to manually trigger PSS event processing
 * Useful for testing or manual updates
 */
export const usePssEventProcessor = () => {
  const processEvent = (event: any) => {
    handlePssEvent(event);
  };

  const processEvents = (events: any[]) => {
    processPssEvents(events);
  };

  return {
    processEvent,
    processEvents,
  };
}; 