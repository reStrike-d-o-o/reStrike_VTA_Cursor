/**
 * MatchDetailsSection
 * - Displays selected match details and quick actions
 */
import React, { useEffect, useState } from 'react';
import { FlagImage } from '../../utils/flagUtils';
import { usePssMatchStore } from '../../stores';

// Fallback data when no PSS data is available
const fallbackPlayers = [
  { ioc: 'USA', name: 'Benjamin Smith', color: 'blue' },
  { ioc: 'JPN', name: 'Kei Tanaka', color: 'red' },
];

// Fallback match details
const fallbackMatchDetails = {
  weight: 'M-75kg',
  category: 'Round of 16',
  division: 'Senior',
  number: '1254'
};

// Function to calculate optimal font size based on text length
const getFontSizeClass = (text: string): string => {
  const length = text.length;
  
  if (length <= 15) {
    return 'text-base'; // 16px - default size (increased from 12)
  } else if (length <= 21) {
    return 'text-sm'; // 14px (increased from 16)
  } else if (length <= 26) {
    return 'text-xs'; // 12px (increased from 20)
  } else if (length <= 32) {
    return 'text-xs'; // 12px with additional scaling (increased from 25)
  } else {
    return 'text-xs'; // 12px with maximum scaling
  }
};

// Function to shorten player names if they're too long
const shortenPlayerName = (fullName: string): string => {
  const nameParts = fullName.split(' ');
  if (nameParts.length >= 2) {
    const firstName = nameParts[0];
    const lastName = nameParts.slice(1).join(' ');
    
    // If the full name is longer than 20 characters, shorten it
    if (fullName.length > 20) {
      return `${firstName.charAt(0)}. ${lastName}`;
    }
  }
  return fullName;
};

const MatchDetailsSection: React.FC = () => {
  // Get data from PSS store - use direct property selectors to avoid infinite loops
  const matchData = usePssMatchStore((state) => state.matchData);
  const athlete1 = matchData.athletes?.athlete1;
  const athlete2 = matchData.athletes?.athlete2;
  const matchNumber = matchData.matchConfig?.number;
  const matchCategory = matchData.matchConfig?.category;
  const matchWeight = matchData.matchConfig?.weight;
  const matchDivision = matchData.matchConfig?.division;
  const isLoaded = matchData.isLoaded;

  // Debug logging
  React.useEffect(() => {
    console.log('🎯 MatchDetailsSection updated:', {
      athlete1,
      athlete2,
      matchNumber,
      matchCategory,
      matchWeight,
      matchDivision,
      isLoaded
    });
    
    // Check if data is undefined
    if (!athlete1 || !athlete2) {
      console.warn('⚠️ Athletes data is undefined:', { athlete1, athlete2 });
    }
    
    if (!matchNumber || !matchCategory || !matchWeight || !matchDivision) {
      console.warn('⚠️ Match config data is undefined:', { 
        matchNumber, 
        matchCategory, 
        matchWeight, 
        matchDivision 
      });
    }
    
    if (!isLoaded) {
      console.warn('⚠️ Match is not loaded');
    }
  }, [athlete1, athlete2, matchNumber, matchCategory, matchWeight, matchDivision, isLoaded]);

  // Use PSS data if available, otherwise fallback to dummy data
  const player1 = athlete1 ? {
    ioc: athlete1.iocCode,
    name: athlete1.long,
    color: 'blue'
  } : fallbackPlayers[0];

  const player2 = athlete2 ? {
    ioc: athlete2.iocCode,
    name: athlete2.long,
    color: 'red'
  } : fallbackPlayers[1];

  // Process player names
  const player1Name = shortenPlayerName(player1.name);
  const player2Name = shortenPlayerName(player2.name);
  
  // Get the longer name to determine font size for both
  const longerName = player1Name.length >= player2Name.length ? player1Name : player2Name;
  const sharedFontSize = getFontSizeClass(longerName);

  return (
    <div className="mb-2 w-full flex flex-col items-center space-y-3 pt-4">
      {/* Players VS */}
      <div className="flex items-center space-x-2">
        {/* Blue Player Flag */}
        <FlagImage countryCode={player1.ioc} className="w-8 h-6 object-cover rounded-sm shadow-sm" />
        
        {/* Combined Player Names */}
        <span className={`${sharedFontSize} text-white font-medium whitespace-nowrap`}>
          {player1Name} VS {player2Name}
        </span>
        
        {/* Red Player Flag */}
        <FlagImage countryCode={player2.ioc} className="w-8 h-6 object-cover rounded-sm shadow-sm" />
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