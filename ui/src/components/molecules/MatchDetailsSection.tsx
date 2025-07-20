import React from 'react';
import { FlagImage } from '../../utils/flagUtils';

const players = [
  { ioc: 'USA', name: 'Benjamin Smith', color: 'red' },
  { ioc: 'JPN', name: 'Kei Tanaka', color: 'blue' },
];

// Function to shorten player names if they're too long
const shortenPlayerName = (fullName: string): string => {
  const nameParts = fullName.split(' ');
  if (nameParts.length >= 2) {
    const firstName = nameParts[0];
    const lastName = nameParts.slice(1).join(' ');
    
    // If the full name is longer than 15 characters, shorten it
    if (fullName.length > 15) {
      return `${firstName.charAt(0)}. ${lastName}`;
    }
  }
  return fullName;
};

const MatchDetailsSection: React.FC = () => (
  <div className="mb-2 w-full flex flex-col items-center space-y-3 pt-4">
    {/* Players VS */}
    <div className="flex items-center space-x-4">
      {/* Red Player */}
      <div className="flex items-center space-x-2">
        <FlagImage countryCode={players[0].ioc} className="w-8 h-6 object-cover rounded-sm shadow-sm" />
        <span className="text-base text-white font-medium">{shortenPlayerName(players[0].name)}</span>
      </div>
      
      {/* VS */}
      <span className="text-lg font-bold text-gray-300">VS</span>
      
      {/* Blue Player */}
      <div className="flex items-center space-x-2">
        <span className="text-base text-white font-medium">{shortenPlayerName(players[1].name)}</span>
        <FlagImage countryCode={players[1].ioc} className="w-8 h-6 object-cover rounded-sm shadow-sm" />
      </div>
    </div>
    
    {/* Match Details */}
    <div className="flex items-center space-x-2 text-sm text-gray-400">
      <span>M-75kg</span>
      <span>|</span>
      <span>Semi-final</span>
    </div>
    
    {/* Score */}
    <div className="text-center">
      <span className="text-3xl font-bold text-red-500">#1254</span>
    </div>
  </div>
);

export default MatchDetailsSection; 