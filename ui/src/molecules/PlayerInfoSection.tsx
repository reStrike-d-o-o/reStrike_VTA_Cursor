import React from 'react';
import { FlagImage } from '../utils/flagUtils';

const players = [
  { ioc: 'USA', name: 'Benjamin Smith' },
  { ioc: 'JPN', name: 'Kei Tanaka' },
];

const PlayerInfoSection: React.FC = () => (
  <div className="mb-2 w-full">
    {players.map((athlete, idx) => (
      <div key={idx} className="flex items-center space-x-2 mb-1">
        <FlagImage countryCode={athlete.ioc} className="w-8 h-6 object-cover rounded-sm shadow-sm" />
        <span className="text-base text-white font-medium">{athlete.name}</span>
      </div>
    ))}
  </div>
);

export default PlayerInfoSection; 