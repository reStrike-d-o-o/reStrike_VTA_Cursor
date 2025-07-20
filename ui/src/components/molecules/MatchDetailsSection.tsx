import React from 'react';
import { FlagImage } from '../../utils/flagUtils';
import { usePssMatchStore } from '../../stores';

// Fallback data when no PSS data is available
const fallbackPlayers = [
  { ioc: 'USA', name: 'Benjamin Smith', color: 'red' },
  { ioc: 'JPN', name: 'Kei Tanaka', color: 'blue' },
];

// Fallback match details
const fallbackMatchDetails = {
  weight: 'M-75kg',
  category: 'Round of 16',
  division: 'Senior',
  number: '1254'
};

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

const MatchDetailsSection: React.FC = () => {
  // Get data from PSS store
  const athlete1 = usePssMatchStore((state) => state.getAthlete1());
  const athlete2 = usePssMatchStore((state) => state.getAthlete2());
  const matchNumber = usePssMatchStore((state) => state.getMatchNumber());
  const matchCategory = usePssMatchStore((state) => state.getMatchCategory());
  const matchWeight = usePssMatchStore((state) => state.getMatchWeight());
  const matchDivision = usePssMatchStore((state) => state.getMatchDivision());
  const isLoaded = usePssMatchStore((state) => state.matchData.isLoaded);

  // Use PSS data if available, otherwise fallback to dummy data
  const player1 = athlete1 ? {
    ioc: athlete1.iocCode,
    name: athlete1.long,
    color: 'red'
  } : fallbackPlayers[0];

  const player2 = athlete2 ? {
    ioc: athlete2.iocCode,
    name: athlete2.long,
    color: 'blue'
  } : fallbackPlayers[1];

  return (
    <div className="mb-2 w-full flex flex-col items-center space-y-3 pt-4">
      {/* Players VS */}
      <div className="flex items-center space-x-4">
        {/* Red Player */}
        <div className="flex items-center space-x-2">
          <FlagImage countryCode={player1.ioc} className="w-8 h-6 object-cover rounded-sm shadow-sm" />
          <span className="text-base text-white font-medium">{shortenPlayerName(player1.name)}</span>
        </div>
        
        {/* VS */}
        <span className="text-lg font-bold text-gray-300">VS</span>
        
        {/* Blue Player */}
        <div className="flex items-center space-x-2">
          <span className="text-base text-white font-medium">{shortenPlayerName(player2.name)}</span>
          <FlagImage countryCode={player2.ioc} className="w-8 h-6 object-cover rounded-sm shadow-sm" />
        </div>
      </div>
      
      {/* Match Details */}
      <div className="flex items-center space-x-2 text-sm text-gray-400">
        <span>{matchWeight || fallbackMatchDetails.weight}</span>
        <span>|</span>
        <span>{matchCategory || fallbackMatchDetails.category}</span>
        <span>|</span>
        <span>{matchDivision || fallbackMatchDetails.division}</span>
      </div>
      
      {/* Match Number */}
      <div className="text-center">
        <span className="text-3xl font-bold text-red-500">#{matchNumber || fallbackMatchDetails.number}</span>
      </div>
    </div>
  );
};

export default MatchDetailsSection; 