import React, { useRef, useEffect, useState } from 'react';

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
    <div className="relative flex flex-row items-end w-full">
      {/* Table (left) */}
      <div className="w-[14rem] min-w-0">
        {/* Table Header */}
        <div className="grid grid-cols-12 gap-2 text-xs text-gray-400 mb-2 border-b border-gray-700 pb-1">
          <div className="col-span-2">RND</div>
          <div className="col-span-6">TIME</div>
          <div className="col-span-4">EVENT</div>
        </div>
        {/* Event Rows */}
        <div ref={tableRef} className="space-y-1 max-h-48 overflow-y-auto">
          {filteredEvents.map((event, idx) => (
            <div
              key={idx}
              className={`grid grid-cols-12 gap-2 text-sm cursor-pointer transition-colors duration-150 ${
                selectedIdx === idx ? 'bg-blue-900/60 border border-blue-400' : 'hover:bg-gray-700'
              }`}
              onClick={() => setSelectedIdx(idx)}
            >
              <div className="col-span-2 text-gray-300 font-bold">{event.round}</div>
              <div className="col-span-6 text-gray-300">{event.time}</div>
              <div className="col-span-4 flex items-center space-x-2">
                <span className={`w-3 h-3 rounded-full ${
                  event.color === 'red' ? 'bg-red-500' :
                  event.color === 'blue' ? 'bg-blue-500' :
                  event.color === 'yellow' ? 'bg-yellow-400' :
                  'bg-green-500'
                }`}></span>
                <span className="text-white">{event.event}</span>
              </div>
            </div>
          ))}
        </div>
      </div>
      {/* Sidebar (right) */}
      <div className="flex flex-col items-end ml-8 h-full justify-end w-[8rem]">
        {/* Color buttons row with up arrow as first button */}
        <div className="flex flex-row mb-2 space-x-1">
          <button
            className="w-7 h-7 rounded-md bg-gray-700 border-2 border-gray-600 flex items-center justify-center text-white hover:bg-gray-600 transition-all"
            onClick={handleScrollTopAndClear}
            title="Scroll to top and clear filters"
          >
            ↑
          </button>
          {colorOptions.map((opt, i) => (
            <button
              key={opt.color}
              className={`w-7 h-7 rounded-md ${opt.class} border-2 ${colorFilter === opt.color ? 'border-white ring-2 ring-white' : 'border-gray-600'} transition-all`}
              onClick={() => setColorFilter(colorFilter === opt.color ? null : opt.color)}
              title={opt.color.charAt(0).toUpperCase() + opt.color.slice(1)}
            />
          ))}
        </div>
        {/* Event type buttons stack */}
        <div className="flex flex-col space-y-2">
          {eventTypeOptions.map(type => (
            <button
              key={type.value}
              className={`w-[7.75rem] h-7 rounded-md bg-gray-800 border-2 ${eventTypeFilter === type.value ? 'border-white ring-2 ring-white' : 'border-gray-600'} text-white flex items-center justify-center transition-all`}
              onClick={() => setEventTypeFilter(eventTypeFilter === type.value ? null : type.value)}
              title={type.label}
            >
              {type.label}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
};

export default EventTableSection; 