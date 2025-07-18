import React from 'react';
import PlayerInfoSection from '../molecules/PlayerInfoSection';
import MatchDetailsSection from '../molecules/MatchDetailsSection';
import EventTableSection from '../molecules/EventTableSection';

const SidebarBig: React.FC = () => {
  return (
    <div className="flex-1 h-full min-h-0 flex flex-col p-6 text-white">
      {/* Main content card with dark background and rounded corners */}
      <div className="flex-1 flex flex-col bg-[#18232e] rounded-lg shadow-lg p-6 space-y-6">
        {/* Player Info and Match Details */}
        <div className="space-y-4">
          <PlayerInfoSection />
          <MatchDetailsSection />
        </div>
        
        {/* Divider */}
        <div className="border-t border-gray-700"></div>
        
        {/* Event Table Section */}
        <div className="flex-1 min-h-0">
          <EventTableSection />
        </div>
      </div>
    </div>
  );
};

export default SidebarBig; 