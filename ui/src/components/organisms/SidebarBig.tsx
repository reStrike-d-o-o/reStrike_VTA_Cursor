import React from 'react';
import PlayerInfoSection from '../molecules/PlayerInfoSection';
import MatchDetailsSection from '../molecules/MatchDetailsSection';
import EventTableSection from '../molecules/EventTableSection';

const SidebarBig: React.FC = () => {
  return (
    <div className="flex-1 h-full flex flex-col justify-between p-12 text-white">
      <div>
        <PlayerInfoSection />
        <MatchDetailsSection />
        <div className="mb-8" />
      </div>
      <EventTableSection />
    </div>
  );
};

export default SidebarBig; 