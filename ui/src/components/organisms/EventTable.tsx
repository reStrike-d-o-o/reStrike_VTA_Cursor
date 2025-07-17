import React from 'react';
import Button from '../atoms/Button';
import { StatusDot } from '../atoms/StatusDot';

const dummyEvents = [
  { round: 'R1', time: '02.00.343', event: 'Punch', color: 'red' },
  { round: 'R2', time: '02.10.343', event: 'Head Kick', color: 'blue' },
  { round: 'R3', time: '02.20.343', event: 'Referee', color: 'yellow' },
  { round: 'R1', time: '02.30.343', event: 'Kick', color: 'green' },
  { round: 'R2', time: '02.40.343', event: 'Punch', color: 'red' }
];

const EventTable: React.FC = () => {
  return (
    <div className="mb-4 relative">
      {/* Header */}
      <div className="grid grid-cols-12 gap-2 text-xs text-gray-400 mb-2 border-b border-gray-700 pb-1">
        <div className="col-span-2">RND</div>
        <div className="col-span-4">TIME</div>
        <div className="col-span-6">EVENT</div>
      </div>
      {/* Event Rows */}
      <div className="space-y-1 max-h-32 overflow-y-auto">
        {dummyEvents.map((event, index) => (
          <div key={index} className="grid grid-cols-12 gap-2 text-xs">
            <div className="col-span-2 text-gray-300">{event.round}</div>
            <div className="col-span-4 text-gray-300">{event.time}</div>
            <div className="col-span-6 flex items-center space-x-1">
              <StatusDot color={event.color} />
              <span className="text-gray-300">{event.event}</span>
            </div>
          </div>
        ))}
      </div>
      {/* Go to Top Arrow */}
      <div className="absolute bottom-0 right-0">
        <Button variant="secondary" size="sm">↑</Button>
      </div>
    </div>
  );
};

export default EventTable; 