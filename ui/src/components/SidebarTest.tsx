import React, { useState, useEffect, useMemo, useRef } from 'react';
import { motion } from 'framer-motion';
import { useAppStore, ObsStatusInfo } from '../stores';
import { FlagImage } from '../utils/flagUtils';
import { isWindows, invokeTauri, logError } from '../config/environment';

// Event data structure
interface EventData {
  id: string;
  round: string;
  timestamp: string;
  player: 'red' | 'blue' | 'yellow';
  eventType: 'head' | 'punch' | 'kick' | 'spinning kick' | 'foul';
}

const SidebarTest: React.FC = () => {
  const { 
    obsStatus, 
    updateObsStatus, 
    obsConnections 
  } = useAppStore();
  
  const [manualMode, setManualMode] = useState(false);
  const [activeFilters, setActiveFilters] = useState<{
    players: Set<'red' | 'blue' | 'yellow'>;
    events: Set<'head' | 'punch' | 'kick' | 'spinning kick' | 'foul'>;
  }>({
    players: new Set(),
    events: new Set()
  });
  
  const eventTableRef = useRef<HTMLDivElement>(null);

  // OBS Status Polling - Uses existing connections from OBS Manager
  useEffect(() => {
    const pollObsStatus = async () => {
      try {
        // Check if we have any active OBS connections
        const activeConnections = obsConnections.filter(
          conn => conn.status === 'Connected' || conn.status === 'Authenticated'
        );

        if (activeConnections.length === 0) {
          // No active connections, set default status
          updateObsStatus({
            is_recording: false,
            is_streaming: false,
            cpu_usage: 0,
          });
          return;
        }

        if (isWindows()) {
          // Use Tauri commands for Windows environment
          const status = await invokeTauri('obs_get_status');
          if (status.success && status.data) {
            updateObsStatus(status.data);
          }
        } else {
          // For web environment, get status from active WebSocket connections
          // The status should be updated by the OBS WebSocket Manager
          // We can get the latest status from the store
          if (obsStatus) {
            // Status is already available from OBS Manager
            // No need to make additional requests
          }
        }
      } catch (error) {
        logError('OBS status polling failed', error);
      }
    };

    // Poll immediately
    pollObsStatus();

    // Then poll every 3 seconds
    const interval = setInterval(pollObsStatus, 3000);

    return () => clearInterval(interval);
  }, [updateObsStatus, obsConnections, obsStatus]);

  // Sample event data
  const allEvents: EventData[] = [
    { id: '1', round: 'R1', timestamp: '01.05.123', player: 'red', eventType: 'punch' },
    { id: '2', round: 'R1', timestamp: '01.18.456', player: 'blue', eventType: 'kick' },
    { id: '3', round: 'R1', timestamp: '01.32.789', player: 'yellow', eventType: 'foul' },
    { id: '4', round: 'R1', timestamp: '01.45.234', player: 'red', eventType: 'head' },
    { id: '5', round: 'R1', timestamp: '02.00.343', player: 'blue', eventType: 'spinning kick' },
    { id: '6', round: 'R1', timestamp: '02.15.127', player: 'red', eventType: 'kick' },
    { id: '7', round: 'R1', timestamp: '02.32.891', player: 'yellow', eventType: 'foul' },
    { id: '8', round: 'R1', timestamp: '02.45.234', player: 'blue', eventType: 'head' },
    { id: '9', round: 'R1', timestamp: '03.12.567', player: 'red', eventType: 'punch' },
    { id: '10', round: 'R2', timestamp: '03.25.890', player: 'blue', eventType: 'kick' },
    { id: '11', round: 'R2', timestamp: '03.38.123', player: 'red', eventType: 'spinning kick' },
    { id: '12', round: 'R2', timestamp: '03.52.456', player: 'yellow', eventType: 'foul' },
    { id: '13', round: 'R2', timestamp: '04.05.789', player: 'blue', eventType: 'head' },
    { id: '14', round: 'R2', timestamp: '04.18.234', player: 'red', eventType: 'kick' },
    { id: '15', round: 'R3', timestamp: '04.32.567', player: 'blue', eventType: 'punch' },
  ];

  // Filter events based on active filters
  const filteredEvents = useMemo(() => {
    return allEvents.filter(event => {
      const playerMatch = activeFilters.players.size === 0 || activeFilters.players.has(event.player);
      const eventMatch = activeFilters.events.size === 0 || activeFilters.events.has(event.eventType);
      return playerMatch && eventMatch;
    });
  }, [activeFilters]);

  // Filter toggle functions
  const togglePlayerFilter = (player: 'red' | 'blue' | 'yellow') => {
    setActiveFilters(prev => {
      const newPlayers = new Set(prev.players);
      if (newPlayers.has(player)) {
        newPlayers.delete(player);
      } else {
        newPlayers.add(player);
      }
      return { ...prev, players: newPlayers };
    });
  };

  const toggleEventFilter = (eventType: 'head' | 'punch' | 'kick' | 'spinning kick' | 'foul') => {
    setActiveFilters(prev => {
      const newEvents = new Set(prev.events);
      if (newEvents.has(eventType)) {
        newEvents.delete(eventType);
      } else {
        newEvents.add(eventType);
      }
      return { ...prev, events: newEvents };
    });
  };

  const clearAllFilters = () => {
    // Only clear filters if any are active
    if (activeFilters.players.size > 0 || activeFilters.events.size > 0) {
      setActiveFilters({
        players: new Set(),
        events: new Set()
      });
    }
    
    // Focus on the first row of the event table
    if (eventTableRef.current) {
      const firstEventRow = eventTableRef.current.querySelector('[data-event-row]') as HTMLElement;
      if (firstEventRow) {
        firstEventRow.focus();
      } else {
        // If no events, focus on the event table container
        eventTableRef.current.focus();
      }
    }
  };

  // Helper function to get dot color
  const getDotColor = (player: 'red' | 'blue' | 'yellow') => {
    switch (player) {
      case 'red': return 'bg-red-500';
      case 'blue': return 'bg-blue-500';
      case 'yellow': return 'bg-yellow-400';
      default: return 'bg-gray-500';
    }
  };

  // Get CPU color based on usage
  const getCpuColor = (cpuUsage: number) => {
    if (cpuUsage <= 30) return 'bg-green-500';
    if (cpuUsage <= 70) return 'bg-yellow-500';
    return 'bg-red-500';
  };

  // Get recording status color
  const getRecordingColor = (isRecording: boolean) => {
    return isRecording ? 'bg-red-500' : 'bg-green-500';
  };

  // Get streaming status color
  const getStreamingColor = (isStreaming: boolean) => {
    return isStreaming ? 'bg-red-500' : 'bg-green-500';
  };

  // Get status from OBS Manager connections
  const isRecording = obsStatus?.is_recording ?? false;
  const isStreaming = obsStatus?.is_streaming ?? false;
  const cpuUsage = obsStatus?.cpu_usage ?? 0;
  const hasObsConnections = obsConnections.some(
    conn => conn.status === 'Connected' || conn.status === 'Authenticated'
  );

  return (
    <div className="min-h-screen flex bg-[#101820] text-white" style={{ fontFamily: 'Segoe UI, Roboto, sans-serif' }}>
      {/* Left Control Column */}
      <div className="flex flex-col items-center justify-between py-8 px-4 bg-[#181F26] w-40 shadow-lg">
        <div className="flex flex-col items-center space-y-8">
          {/* REPLAY Button */}
          <button
            className="w-32 h-32 rounded-full bg-red-600 shadow-2xl flex items-center justify-center text-xl font-bold text-white border-4 border-red-700 hover:bg-red-700 focus:outline-none mb-4 transition-all duration-200"
            style={{ 
              boxShadow: '0 0 20px rgba(220, 38, 38, 0.6), 0 0 0 4px #2B2B2B',
              animation: 'pulse 2s infinite'
            }}
          >
            REPLAY
          </button>

          {/* Manual Mode Toggle */}
          <div className="flex flex-col items-center space-y-3">
            <span className="text-sm text-gray-300 font-medium">Manual Mode</span>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={manualMode}
                onChange={() => setManualMode((v) => !v)}
                className="sr-only peer"
              />
              <div className="w-12 h-6 bg-gray-700 rounded-full peer peer-focus:ring-2 peer-focus:ring-blue-500 transition-all duration-200 border border-gray-600" />
              <div
                className={`absolute left-0.5 top-0.5 w-5 h-5 rounded-full transition-transform duration-200 ${manualMode ? 'translate-x-6 bg-blue-500 shadow-lg' : 'bg-gray-400'}`}
              />
            </label>
          </div>

          {/* Advanced Button */}
          <button className="w-32 h-10 mt-6 bg-gray-800 hover:bg-gray-700 border border-gray-600 rounded-lg text-sm text-gray-200 font-medium transition-all duration-200 hover:border-gray-500">
            Advanced
          </button>
        </div>
        
        {/* Status Bar (bottom left) */}
        <div className="w-full flex justify-between items-center text-xs text-gray-500 mt-8 px-2">
          {/* OBS Recording Status */}
          <div className="flex items-center space-x-1">
            <span 
              className={`w-2 h-2 rounded-full ${getRecordingColor(isRecording)} ${
                !hasObsConnections ? 'opacity-50' : ''
              }`}
              title={hasObsConnections ? `Recording: ${isRecording ? 'ON' : 'OFF'}` : 'No OBS connection'}
            ></span>
            <span className={!hasObsConnections ? 'opacity-50' : ''}>REC</span>
          </div>
          
          {/* OBS Streaming Status */}
          <div className="flex items-center space-x-1">
            <span 
              className={`w-2 h-2 rounded-full ${getStreamingColor(isStreaming)} ${
                !hasObsConnections ? 'opacity-50' : ''
              }`}
              title={hasObsConnections ? `Streaming: ${isStreaming ? 'ON' : 'OFF'}` : 'No OBS connection'}
            ></span>
            <span className={!hasObsConnections ? 'opacity-50' : ''}>STR</span>
          </div>
          
          {/* OBS CPU Usage */}
          <div className="flex items-center space-x-1">
            <span 
              className={`w-2 h-2 rounded-full ${getCpuColor(cpuUsage)} ${
                !hasObsConnections ? 'opacity-50' : ''
              }`}
              title={hasObsConnections ? `CPU: ${cpuUsage.toFixed(1)}%` : 'No OBS connection'}
            ></span>
            <span className={!hasObsConnections ? 'opacity-50' : ''}>
              CPU {hasObsConnections ? `${cpuUsage.toFixed(0)}%` : '--'}
            </span>
          </div>
        </div>
      </div>

      {/* Right Info Column */}
      <div className="flex-1 flex flex-col justify-between bg-[#101820] px-8 py-6 min-w-[360px] max-w-[500px]">
        <div>
          {/* Athlete Info and Match Number */}
          <div className="flex items-center justify-between mb-4">
            <div className="flex flex-col space-y-1">
              <div className="flex items-center space-x-3 text-lg font-semibold">
                <FlagImage countryCode="USA" />
                <span className="text-white">Benjamin Smith</span>
              </div>
              <div className="flex items-center space-x-3 text-lg font-semibold">
                <FlagImage countryCode="JPN" />
                <span className="text-white">Kei Tanaka</span>
              </div>
              <div className="flex flex-col text-gray-400 text-sm mt-1">
                <span className="font-medium">M-75kg</span>
                <span>Semi-final</span>
              </div>
            </div>
            <div className="text-6xl font-bold text-white tracking-tight -ml-[1000px] flex items-center justify-center w-[200px]">1254</div>
          </div>
          <hr className="border-gray-700 my-6" />
          {/* Event Table with Filters */}
          <div className="flex gap-3">
            {/* Event Table */}
            <div ref={eventTableRef} className="flex-1 flex flex-col h-64 overflow-y-auto" tabIndex={-1}>
              <div className="flex items-center justify-between text-gray-400 text-xs font-medium mb-3 px-1 sticky top-0 bg-[#101820] py-2 z-10">
                <span className="w-8">RND</span>
                <span className="w-20 text-center">TIME</span>
                <span className="flex-1">EVENT</span>
                <span className="text-xs text-gray-500 ml-2">
                  {filteredEvents.length}/{allEvents.length}
                </span>
              </div>
              {/* Event Rows - Dynamic based on filters */}
              {filteredEvents.length > 0 ? (
                filteredEvents.map((event) => (
                  <div 
                    key={event.id} 
                    data-event-row
                    className="flex items-center justify-between py-2 px-1 hover:bg-gray-800 rounded transition-colors focus:outline-none focus:bg-gray-800 focus:ring-2 focus:ring-blue-500"
                    tabIndex={0}
                  >
                    <span className="font-bold text-white w-8">{event.round}</span>
                    <span className="text-gray-300 w-20 text-center text-sm">{event.timestamp}</span>
                    <span className="flex items-center space-x-3 flex-1">
                      <span className={`w-3 h-3 rounded-full ${getDotColor(event.player)} inline-block shadow-sm`}></span>
                      <span className="text-white text-sm capitalize">{event.eventType}</span>
                    </span>
                  </div>
                ))
              ) : (
                <div className="flex items-center justify-center py-8 text-gray-500 text-sm">
                  No events match the current filters
                </div>
              )}
            </div>
            
            {/* Filter Buttons Stack */}
            <div className="flex flex-col gap-1">
              {/* Top Row: Clear Filter + Player Filter Buttons */}
              <div className="flex gap-1">
                {/* Clear Filter Button (Up Arrow) */}
                <button 
                  onClick={clearAllFilters}
                  className={`w-8 h-8 rounded flex items-center justify-center transition-all duration-200 ${
                    activeFilters.players.size === 0 && activeFilters.events.size === 0 
                      ? 'bg-gray-700 hover:bg-gray-600 hover:scale-105 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-1 focus:ring-offset-gray-800' 
                      : 'bg-gray-700 hover:bg-gray-600 hover:scale-105 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-1 focus:ring-offset-gray-800'
                  }`}
                  title={activeFilters.players.size === 0 && activeFilters.events.size === 0 ? "Move focus to first event row" : "Clear all filters and move to first event row"}
                >
                  <svg className="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 15l7-7 7 7" />
                  </svg>
                </button>
                
                {/* Player Filter Buttons */}
                <button 
                  onClick={() => togglePlayerFilter('red')}
                  className={`w-8 h-8 rounded transition-colors ${
                    activeFilters.players.has('red') 
                      ? 'bg-red-700 ring-2 ring-red-400' 
                      : 'bg-red-600 hover:bg-red-500'
                  }`}
                  title="Filter Red player events"
                ></button>
                <button 
                  onClick={() => togglePlayerFilter('blue')}
                  className={`w-8 h-8 rounded transition-colors ${
                    activeFilters.players.has('blue') 
                      ? 'bg-blue-700 ring-2 ring-blue-400' 
                      : 'bg-blue-600 hover:bg-blue-500'
                  }`}
                  title="Filter Blue player events"
                ></button>
                <button 
                  onClick={() => togglePlayerFilter('yellow')}
                  className={`w-8 h-8 rounded transition-colors ${
                    activeFilters.players.has('yellow') 
                      ? 'bg-yellow-600 ring-2 ring-yellow-400' 
                      : 'bg-yellow-500 hover:bg-yellow-400'
                  }`}
                  title="Filter Referee events"
                ></button>
              </div>
              
              {/* Bottom Row: Event Type Filter Buttons (Full Width) */}
              <div className="flex flex-col gap-1">
                <button 
                  onClick={() => toggleEventFilter('head')}
                  className={`w-[140px] h-8 rounded text-white text-xs font-bold transition-colors ${
                    activeFilters.events.has('head') 
                      ? 'bg-blue-600 ring-2 ring-blue-400' 
                      : 'bg-gray-700 hover:bg-gray-600'
                  }`}
                  title="Filter Head events"
                >
                  Head
                </button>
                <button 
                  onClick={() => toggleEventFilter('punch')}
                  className={`w-[140px] h-8 rounded text-white text-xs font-bold transition-colors ${
                    activeFilters.events.has('punch') 
                      ? 'bg-blue-600 ring-2 ring-blue-400' 
                      : 'bg-gray-700 hover:bg-gray-600'
                  }`}
                  title="Filter Punch events"
                >
                  Punch
                </button>
                <button 
                  onClick={() => toggleEventFilter('kick')}
                  className={`w-[140px] h-8 rounded text-white text-xs font-bold transition-colors ${
                    activeFilters.events.has('kick') 
                      ? 'bg-blue-600 ring-2 ring-blue-400' 
                      : 'bg-gray-700 hover:bg-gray-600'
                  }`}
                  title="Filter Kick events"
                >
                  Kick
                </button>
                <button 
                  onClick={() => toggleEventFilter('spinning kick')}
                  className={`w-[140px] h-8 rounded text-white text-xs font-bold transition-colors ${
                    activeFilters.events.has('spinning kick') 
                      ? 'bg-blue-600 ring-2 ring-blue-400' 
                      : 'bg-gray-700 hover:bg-gray-600'
                  }`}
                  title="Filter Spinning Kick events"
                >
                  SPIN
                </button>
              </div>
            </div>
          </div>
        </div>
        {/* Status Bar (bottom right) - already handled in left column for this design */}
      </div>
    </div>
  );
};

export default SidebarTest; 