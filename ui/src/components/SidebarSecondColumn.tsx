import React from 'react';

// Static dummy data
const dummyAthletes = [
  { flag: '🇺🇸', name: 'Benjamin Smith' },
  { flag: '🇯🇵', name: 'Kei Tanaka' }
];

const dummyEvents = [
  { round: 'R1', time: '02.00.343', event: 'Punch', color: 'red' },
  { round: 'R2', time: '02.10.343', event: 'Head Kick', color: 'blue' },
  { round: 'R3', time: '02.20.343', event: 'Referee', color: 'yellow' },
  { round: 'R1', time: '02.30.343', event: 'Kick', color: 'green' },
  { round: 'R2', time: '02.40.343', event: 'Punch', color: 'red' }
];

const MatchInfoSection: React.FC = () => {
  return (
    <div className="mb-4">
      {/* Athletes */}
      <div className="space-y-1 mb-2">
        {dummyAthletes.map((athlete, index) => (
          <div key={index} className="flex items-center text-sm">
            <span className="mr-2">{athlete.flag}</span>
            <span>{athlete.name}</span>
          </div>
        ))}
      </div>
      
      {/* Category and Stage */}
      <div className="flex justify-between items-start">
        <div className="space-y-1">
          <div className="text-sm font-medium">M-75kg</div>
          <div className="text-sm text-gray-400">Semi-final</div>
        </div>
        
        {/* Match Number - Large, right-aligned */}
        <div className="text-4xl font-bold text-red-500">1254</div>
      </div>
    </div>
  );
};

const EventTable: React.FC = () => {
  return (
    <div className="mb-4 relative">
      {/* Table Header */}
      <div className="grid grid-cols-12 gap-2 text-xs text-gray-400 mb-2 border-b border-gray-700 pb-1">
        <div className="col-span-2">RND</div>
        <div className="col-span-4">TIME</div>
        <div className="col-span-6">EVENT</div>
      </div>
      
      {/* Event Rows */}
      <div className="space-y-1 max-h-32 overflow-y-auto">
        {dummyEvents.map((event, index) => (
          <div key={index} className="grid grid-cols-12 gap-2 text-xs">
            <div className="col-span-2">{event.round}</div>
            <div className="col-span-4">{event.time}</div>
            <div className="col-span-6 flex items-center">
              <span 
                className={`w-2 h-2 rounded-full mr-2 ${
                  event.color === 'red' ? 'bg-red-500' :
                  event.color === 'blue' ? 'bg-blue-500' :
                  event.color === 'yellow' ? 'bg-yellow-500' :
                  'bg-green-500'
                }`}
              />
              {event.event}
            </div>
          </div>
        ))}
      </div>
      
      {/* Go to Top Arrow - Bottom right of table */}
      <button className="absolute bottom-0 right-0 p-1 text-gray-400 hover:text-white transition-colors">
        ↑
      </button>
    </div>
  );
};

const StatusBar: React.FC = () => {
  return (
    <div className="flex justify-between items-center text-xs border-t border-gray-700 pt-2">
      {/* OBS Recording Status */}
      <div className="flex items-center">
        <span className="w-2 h-2 rounded-full bg-green-500 mr-1"></span>
        <span>OBS Recording</span>
      </div>
      
      {/* CP % Status */}
      <div className="flex items-center">
        <span className="w-2 h-2 rounded-full bg-yellow-500 mr-1"></span>
        <span>CP 5%</span>
      </div>
      
      {/* PSS Status */}
      <div className="flex items-center">
        <span className="w-2 h-2 rounded-full bg-green-500 mr-1"></span>
        <span>PSS</span>
      </div>
    </div>
  );
};

const SidebarSecondColumn: React.FC = () => {
  return (
    <div className="flex-1 p-3 bg-gray-800 text-white">
      <MatchInfoSection />
      <EventTable />
      <StatusBar />
    </div>
  );
};

export default SidebarSecondColumn; 