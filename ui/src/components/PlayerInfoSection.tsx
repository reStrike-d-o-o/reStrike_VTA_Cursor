import React from 'react';

const players = [
  { flag: '🇺🇸', name: 'Benjamin Smith' },
  { flag: '🇯🇵', name: 'Kei Tanaka' },
];

const PlayerInfoSection: React.FC = () => (
  <div className="mb-2 w-full">
    {players.map((athlete, idx) => (
      <div key={idx} className="flex items-center space-x-2 mb-1">
        <span className="text-xl">{athlete.flag}</span>
        <span className="text-base text-white font-medium">{athlete.name}</span>
      </div>
    ))}
  </div>
);

export default PlayerInfoSection; 