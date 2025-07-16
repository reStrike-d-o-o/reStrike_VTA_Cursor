import React from 'react';

const dummyAthletes = [
  { flag: '🇺🇸', name: 'Benjamin Smith' },
  { flag: '🇯🇵', name: 'Kei Tanaka' }
];

const MatchInfoSection: React.FC = () => {
  return (
    <div className="mb-4">
      {/* Athletes */}
      <div className="space-y-1 mb-2">
        {dummyAthletes.map((athlete, index) => (
          <div key={index} className="flex items-center space-x-2">
            <span className="text-lg">{athlete.flag}</span>
            <span className="text-sm text-gray-300">{athlete.name}</span>
          </div>
        ))}
      </div>
      {/* Category and Stage */}
      <div className="space-y-1 mb-3">
        <div className="text-sm text-gray-400">M-75kg</div>
        <div className="text-sm text-gray-400">Semi-final</div>
      </div>
      {/* Match Number */}
      <div className="text-right">
        <span className="text-3xl font-bold text-red-500">1254</span>
      </div>
    </div>
  );
};

export default MatchInfoSection; 