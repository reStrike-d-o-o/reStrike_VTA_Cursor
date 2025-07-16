import React from 'react';
import MatchInfoSection from '../organisms/MatchInfoSection';
import EventTable from '../organisms/EventTable';
import StatusBar from '../organisms/StatusBar';

const AdvancedPanel: React.FC = () => {
  return (
    <div className="flex-1 bg-[#101820] px-8 py-6 min-w-[360px] max-w-[100%] h-full flex flex-col">
      <MatchInfoSection />
      <EventTable />
      <div className="mt-auto">
        <StatusBar />
      </div>
    </div>
  );
};

export default AdvancedPanel; 