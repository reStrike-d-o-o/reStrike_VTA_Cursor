import React from 'react';
import PlayerInfoSection from './PlayerInfoSection';
import MatchDetailsSection from './MatchDetailsSection';
import EventTableSection from './EventTableSection';

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