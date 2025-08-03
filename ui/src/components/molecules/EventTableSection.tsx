import React, { useRef, useEffect, useState } from 'react';
import Button from '../atoms/Button';
import { StatusDot } from '../atoms/StatusDot';
import { useLiveDataStore, PssEventData } from '../../stores/liveDataStore';
import { useAppStore } from '../../stores/index';

const colorOptions = [
  { color: 'red', class: 'bg-red-500' },
  { color: 'blue', class: 'bg-blue-500' },
  { color: 'referee', class: 'bg-yellow-400' },
];

// Event type filter buttons (order: H, K, P, TH, TB, R)
const eventTypeOptions = [
  { label: 'HEAD', value: 'H' },
  { label: 'KICK', value: 'K' },
  { label: 'PUNCH', value: 'P' },
  { label: 'TECH HEAD', value: 'TH' },
  { label: 'TECH BODY', value: 'TB' },
  { label: 'REFEREE', value: 'R' },
];

const EventTableSection: React.FC = () => {
  const tableRef = useRef<HTMLDivElement>(null);
  const [selectedIdx, setSelectedIdx] = useState<number | null>(null);
  const [colorFilter, setColorFilter] = useState<string | null>(null);
  const [eventTypeFilter, setEventTypeFilter] = useState<string | null>(null);
  
  // Get real-time data from stores
  const { 
    events, 
    currentRound, 
    currentTime, 
    isConnected, 
    getFilteredEvents,
    clearEvents 
  } = useLiveDataStore();
  
  const { isManualModeEnabled } = useAppStore();

  // Auto-scroll to top when new events arrive (since newest are now at top)
  useEffect(() => {
    if (tableRef.current) {
      tableRef.current.scrollTop = 0;
    }
  }, [events]);

  const handleScrollTopAndClear = () => {
    if (tableRef.current) {
      tableRef.current.scrollTop = tableRef.current.scrollHeight;
    }
    setColorFilter(null);
    setEventTypeFilter(null);
  };

  // Filtering logic - only show events when manual mode is OFF
  const filteredEvents = isManualModeEnabled ? [] : getFilteredEvents(colorFilter, eventTypeFilter);

  // Format time for display (convert from "m:ss" to "mm.ss.000" format)
  const formatTimeForDisplay = (time: string): string => {
    if (!time || time === '0:00') return '00.00.000';
    
    const parts = time.split(':');
    if (parts.length === 2) {
      const minutes = parts[0].padStart(2, '0');
      const seconds = parts[1].padStart(2, '0');
      return `${minutes}.${seconds}.000`;
    }
    
    return time;
  };

  // Get athlete color for display
  const getAthleteColor = (athlete: string): string => {
    switch (athlete) {
      case 'blue': return 'bg-blue-500';
      case 'red': return 'bg-red-500';
      case 'referee': return 'bg-yellow-400';
      default: return 'bg-gray-500';
    }
  };

  return (
    <div className="flex flex-col space-y-4 overflow-hidden pt-4">
      {/* Section Title */}
      <div className="flex-shrink-0 flex items-center justify-between">
        <div className="text-lg font-semibold text-gray-200">Event Table</div>
        <div className="flex items-center space-x-2">
          {isManualModeEnabled && (
            <span className="text-xs text-yellow-400 bg-yellow-900/20 px-2 py-1 rounded">
              Manual Mode
            </span>
          )}
        </div>
      </div>
      
      {/* Centered wrapper for Event Table and Filter Stack */}
      <div className="flex justify-center flex-1 min-h-0 overflow-hidden">
        <div className="flex flex-row items-start">
          {/* Event Table (left) */}
          <div className="w-80 flex flex-col overflow-hidden mr-3">
            {/* Table Header */}
            <div className="flex-shrink-0 flex text-xs text-gray-400 mb-3 border-b border-gray-600 pb-2 items-center mt-2">
              <div className="w-16 font-semibold text-white px-1 rounded mr-2 ml-12">RND</div>
              <div className="w-24 font-semibold mr-2 ml-1">TIME</div>
              <div className="w-32 font-semibold flex items-center space-x-1 -ml-8">
                <span>EVENT</span>
                {colorOptions.map((opt) => (
                  <button
                    key={opt.color}
                    className={`
                      ${colorFilter === opt.color ? 'ring-2 ring-white rounded-full' : 'rounded-full'}
                      transition-all duration-200 cursor-pointer p-0.5 w-5 h-5 flex items-center justify-center
                    `}
                    onClick={() => setColorFilter(colorFilter === opt.color ? null : opt.color)}
                    title={opt.color.charAt(0).toUpperCase() + opt.color.slice(1)}
                    type="button"
                  >
                    <StatusDot color={opt.class} size="w-3 h-3" />
                  </button>
                ))}
              </div>
            </div>
            
            {/* Event Rows */}
            <div ref={tableRef} className="h-64 space-y-1 overflow-y-auto bg-[#1a2a3a] rounded-md p-3">
              {isManualModeEnabled ? (
                <div className="flex items-center justify-center h-full text-gray-500 text-sm">
                  Manual Mode Active - Real-time events disabled
                </div>
              ) : filteredEvents.length === 0 ? (
                <div className="flex items-center justify-center h-full text-gray-500 text-sm">
                  {isConnected ? 'Waiting for events...' : 'Not connected to PSS'}
                </div>
              ) : (
                filteredEvents.map((event, idx) => (
                  <div
                    key={event.id}
                    className={`flex text-sm cursor-pointer transition-colors duration-150 rounded px-2 py-2 ${
                      selectedIdx === idx ? 'bg-blue-900/60 border border-blue-400' : 'hover:bg-gray-700/50'
                    }`}
                    onClick={() => setSelectedIdx(idx)}
                  >
                    <div className="w-16 text-gray-300 font-bold mr-2 ml-8">
                      R{event.round}
                    </div>
                    <div className="w-24 text-gray-300 mr-2">
                      {formatTimeForDisplay(event.time)}
                    </div>
                    <div className="w-32 flex items-center space-x-2 pl-10">
                      <StatusDot color={getAthleteColor(event.athlete)} size="w-4 h-4" />
                      <span className="text-white font-medium">{event.eventCode}</span>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
          
          {/* Filter Stack (right) */}
          <div className="relative w-16 flex-shrink-0 h-64 pt-2">
            {/* Clear Filter Button (Up Arrow) - positioned at top */}
            <Button
              variant="secondary"
              size="sm"
              onClick={handleScrollTopAndClear}
              title="Scroll to bottom (oldest events) and clear filters"
              className="absolute top-0 w-8 h-8 p-0 flex items-center justify-center text-base"
            >
              ↑
            </Button>
            

            
            {/* Event Type Filter Buttons - positioned to align with table bottom */}
            <div className="absolute bottom-0 flex flex-col space-y-2" style={{ transform: 'translateY(50px)' }}>
              {eventTypeOptions.map(type => (
                <Button
                  key={type.value}
                  variant={eventTypeFilter === type.value ? 'primary' : 'secondary'}
                  size="sm"
                  onClick={() => setEventTypeFilter(eventTypeFilter === type.value ? null : type.value)}
                  title={type.label}
                  className="w-8 h-8 p-0 text-xs font-bold"
                  disabled={isManualModeEnabled}
                >
                  {type.value}
                </Button>
              ))}
            </div>
          </div>
        </div>
      </div>
      
      {/* Current Status Bar */}
      <div className="flex-shrink-0 flex items-center justify-between text-xs text-gray-400 border-t border-gray-600 pt-2">
        <div className="flex items-center space-x-4">
          <span>Round: {currentRound}</span>
          <span>Time: {currentTime}</span>
        </div>
        <div className="flex items-center space-x-2">
          <span>Events: {events.length}</span>
          <span>Filtered: {filteredEvents.length}</span>
        </div>
      </div>
    </div>
  );
};

export default EventTableSection; 