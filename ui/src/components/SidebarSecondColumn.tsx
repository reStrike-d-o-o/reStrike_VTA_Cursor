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
          <div key={index} className="flex items-center space-x-2">
            <span className="text-lg">{athlete.flag}</span>
            <span className="text-sm text-gray-300">{athlete.name}</span>
          </div>
        ))}
      </div>
      
      {/* Category and Stage */}
      <div className="space-y-1 mb-3">
        <div className="text-sm text-gray-400">M-75kg</div>
        <div className="text-sm text-gray-400">Semi-final</div>
      </div>
      
      {/* Match Number */}
      <div className="text-right">
        <span className="text-3xl font-bold text-red-500">1254</span>
      </div>
    </div>
  );
};

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
              <span 
                className={`w-2 h-2 rounded-full ${
                  event.color === 'red' ? 'bg-red-500' :
                  event.color === 'blue' ? 'bg-blue-500' :
                  event.color === 'yellow' ? 'bg-yellow-500' :
                  'bg-green-500'
                }`}
              ></span>
              <span className="text-gray-300">{event.event}</span>
            </div>
          </div>
        ))}
      </div>
      
      {/* Go to Top Arrow */}
      <div className="absolute bottom-0 right-0">
        <button className="text-gray-400 hover:text-white text-xs">
          ↑
        </button>
      </div>
    </div>
  );
};

const StatusBar: React.FC = () => {
  return (
    <div className="flex items-center justify-between text-xs text-gray-400 border-t border-gray-700 pt-2">
      {/* OBS Recording Status */}
      <div className="flex items-center space-x-1">
        <span className="w-2 h-2 rounded-full bg-green-500"></span>
        <span>OBS Recording</span>
      </div>
      
      {/* CP % Status */}
      <div className="flex items-center space-x-1">
        <span className="w-2 h-2 rounded-full bg-yellow-500"></span>
        <span>CP 5%</span>
      </div>
      
      {/* PSS Status */}
      <div className="flex items-center space-x-1">
        <span className="w-2 h-2 rounded-full bg-green-500"></span>
        <span>PSS</span>
      </div>
    </div>
  );
};

const SidebarSecondColumn: React.FC = () => {
  return (
    <div className="w-64 bg-gray-900 p-4 text-white">
      <MatchInfoSection />
      <EventTable />
      <StatusBar />
    </div>
  );
};

export default SidebarSecondColumn; 