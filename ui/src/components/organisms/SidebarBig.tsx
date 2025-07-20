import React from 'react';
import PlayerInfoSection from '../molecules/PlayerInfoSection';
import MatchDetailsSection from '../molecules/MatchDetailsSection';
import EventTableSection from '../molecules/EventTableSection';

const SidebarBig: React.FC = () => {
  return (
    <div className="flex-1 h-full min-h-0 flex flex-col p-0 text-white overflow-hidden">
      {/* Main content card with enhanced styling */}
      <div className="flex-1 flex flex-col bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg shadow-xl border border-gray-600/30 p-1 space-y-1 overflow-hidden">
        {/* Player Info and Match Details */}
        <div className="flex-shrink-0 space-y-2">
          <PlayerInfoSection />
          <MatchDetailsSection />
        </div>
        
        {/* Enhanced Divider */}
        <div className="flex-shrink-0 border-t border-gray-600/50 bg-gradient-to-r from-transparent via-gray-600/30 to-transparent h-px"></div>
        
        {/* Event Table Section */}
        <div className="flex-1 min-h-0 overflow-hidden">
          <EventTableSection />
        </div>
      </div>
    </div>
  );
};

export default SidebarBig; 