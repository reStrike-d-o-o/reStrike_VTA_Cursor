import React, { useRef, useEffect, useState } from 'react';
import Button from '../atoms/Button';
import { StatusDot } from '../atoms/StatusDot';

const events = [
  { round: 'R1', time: '02.00.123', event: 'H', color: 'blue' },
  { round: 'R1', time: '01.45.456', event: 'P', color: 'red' },
  { round: 'R1', time: '01.30.789', event: 'R', color: 'yellow' },
  { round: 'R1', time: '01.15.234', event: 'K', color: 'blue' },
  { round: 'R1', time: '01.00.567', event: 'S', color: 'red' },
  { round: 'R2', time: '02.00.321', event: 'H', color: 'red' },
  { round: 'R2', time: '01.50.654', event: 'P', color: 'blue' },
  { round: 'R2', time: '01.40.987', event: 'R', color: 'yellow' },
  { round: 'R2', time: '01.30.210', event: 'K', color: 'red' },
  { round: 'R2', time: '01.20.543', event: 'R', color: 'yellow' },
  { round: 'R2', time: '01.10.876', event: 'S', color: 'blue' },
  { round: 'R3', time: '02.00.111', event: 'H', color: 'blue' },
  { round: 'R3', time: '01.55.222', event: 'P', color: 'red' },
  { round: 'R3', time: '01.50.333', event: 'R', color: 'yellow' },
  { round: 'R3', time: '01.45.444', event: 'K', color: 'red' },
  { round: 'R3', time: '01.40.555', event: 'S', color: 'blue' },
  { round: 'R3', time: '01.35.666', event: 'R', color: 'yellow' },
  { round: 'R3', time: '01.30.777', event: 'R', color: 'yellow' },
];

const colorOptions = [
  { color: 'red', class: 'bg-red-500' },
  { color: 'blue', class: 'bg-blue-500' },
  { color: 'yellow', class: 'bg-yellow-400' },
];
const eventTypeOptions = [
  { label: 'HEAD', value: 'H' },
  { label: 'KICK', value: 'K' },
  { label: 'PUNCH', value: 'P' },
  { label: 'SPINN', value: 'S' },
  { label: 'REFEREE', value: 'R' },
];

const EventTableSection: React.FC = () => {
  const tableRef = useRef<HTMLDivElement>(null);
  const [selectedIdx, setSelectedIdx] = useState<number | null>(null);
  const [colorFilter, setColorFilter] = useState<string | null>(null);
  const [eventTypeFilter, setEventTypeFilter] = useState<string | null>(null);

  useEffect(() => {
    if (tableRef.current) {
      tableRef.current.scrollTop = tableRef.current.scrollHeight;
    }
  }, []);

  const handleScrollTopAndClear = () => {
    if (tableRef.current) {
      tableRef.current.scrollTop = 0;
    }
    setColorFilter(null);
    setEventTypeFilter(null);
  };

  // Filtering logic
  const filteredEvents = events.filter(e => {
    const colorMatch = colorFilter ? e.color === colorFilter : true;
    const eventTypeMatch = eventTypeFilter ? e.event === eventTypeFilter : true;
    return colorMatch && eventTypeMatch;
  });

  return (
    <div className="flex flex-col space-y-4 h-full overflow-hidden">
      {/* Section Title */}
      <div className="flex-shrink-0 text-lg font-semibold text-gray-200">Event Table</div>
      
      {/* Event Table and Filter Stack Container */}
      <div className="flex flex-row items-start space-x-2 flex-1 min-h-0 overflow-hidden">
        {/* Event Table (left) */}
        <div className="flex-1 min-w-0 flex flex-col overflow-hidden">
          {/* Table Header */}
          <div className="flex-shrink-0 grid grid-cols-12 gap-2 text-xs text-gray-400 mb-3 border-b border-gray-600 pb-2">
            <div className="col-span-2 font-semibold pl-5">RND</div>
            <div className="col-span-4 font-semibold pl-8">TIME</div>
            <div className="col-span-6 font-semibold pl-5 pr-3 flex items-center space-x-2">
              <span>EVENT</span>
              {colorOptions.map((opt) => (
                <button
                  key={opt.color}
                  className={`
                    ${colorFilter === opt.color ? 'ring-2 ring-white' : ''}
                    transition-all duration-200 cursor-pointer p-0.5
                  `}
                  onClick={() => setColorFilter(colorFilter === opt.color ? null : opt.color)}
                  title={opt.color.charAt(0).toUpperCase() + opt.color.slice(1)}
                  type="button"
                >
                  <StatusDot color={opt.class} size="w-4 h-4" />
                </button>
              ))}
            </div>
          </div>
          {/* Event Rows */}
          <div ref={tableRef} className="flex-1 space-y-1 overflow-y-auto bg-[#1a2a3a] rounded-md p-3 min-h-0">
            {filteredEvents.map((event, idx) => (
              <div
                key={idx}
                className={`grid grid-cols-12 gap-2 text-sm cursor-pointer transition-colors duration-150 rounded px-3 py-2 ${
                  selectedIdx === idx ? 'bg-blue-900/60 border border-blue-400' : 'hover:bg-gray-700/50'
                }`}
                onClick={() => setSelectedIdx(idx)}
              >
                <div className="col-span-2 text-gray-300 font-bold">{event.round}</div>
                <div className="col-span-4 text-gray-300">{event.time}</div>
                <div className="col-span-6 flex items-center space-x-2">
                  <StatusDot color={`bg-${event.color}-500`} size="w-4 h-4" />
                  <span className="text-white font-medium">{event.event}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
        
        {/* Filter Stack (right) */}
        <div className="flex flex-col items-end space-y-3 w-24 flex-shrink-0">
          {/* Clear Filter Button (Up Arrow) */}
          <Button
            variant="secondary"
            size="sm"
            onClick={handleScrollTopAndClear}
            title="Scroll to top and clear filters"
            className="w-10 h-10 p-0 flex items-center justify-center text-lg"
          >
            ↑
          </Button>
          

          
          {/* Event Type Filter Buttons */}
          <div className="flex flex-col space-y-2">
            {eventTypeOptions.map(type => (
              <Button
                key={type.value}
                variant={eventTypeFilter === type.value ? 'primary' : 'secondary'}
                size="sm"
                onClick={() => setEventTypeFilter(eventTypeFilter === type.value ? null : type.value)}
                title={type.label}
                className="w-10 h-10 p-0 text-sm font-bold"
              >
                {type.label.charAt(0)}
              </Button>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

export default EventTableSection; 