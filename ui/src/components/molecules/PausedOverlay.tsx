import React from 'react';

const PausedOverlay: React.FC = () => {
  return (
    <div className="fixed inset-0 z-[10000] flex items-center justify-center bg-black/70 text-white pointer-events-auto select-none">
      <h1 className="text-6xl font-bold tracking-wider animate-pulse">PAUSED</h1>
    </div>
  );
};

export default PausedOverlay;
