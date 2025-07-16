import React from 'react';

const category = 'M-75kg';
const stage = 'Semi-final';
const matchNumber = '1254';

const MatchDetailsSection: React.FC = () => (
  <div className="flex items-end justify-between mb-2">
    <div className="flex flex-col">
      <span className="text-gray-400 text-base">{category}</span>
      <span className="text-gray-400 text-base">{stage}</span>
    </div>
    <span className="text-4xl font-bold text-red-400">{matchNumber}</span>
  </div>
);

export default MatchDetailsSection; 