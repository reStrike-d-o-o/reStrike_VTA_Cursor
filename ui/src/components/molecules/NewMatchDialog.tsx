import React, { useState } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import { FlagImage } from '../../utils/flagUtils';

interface NewMatchDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onStartMatch: (matchData: ManualMatchData) => void;
}

export interface ManualMatchData {
  player1: {
    name: string;
    iocCode: string;
  };
  player2: {
    name: string;
    iocCode: string;
  };
  matchNumber: string;
  category: string;
  weight: string;
  division: string;
}

const NewMatchDialog: React.FC<NewMatchDialogProps> = ({
  isOpen,
  onClose,
  onStartMatch
}) => {
  const [matchData, setMatchData] = useState<ManualMatchData>({
    player1: { name: '', iocCode: '' },
    player2: { name: '', iocCode: '' },
    matchNumber: '',
    category: '',
    weight: '',
    division: ''
  });

  const handleInputChange = (field: keyof ManualMatchData, value: string) => {
    setMatchData(prev => ({
      ...prev,
      [field]: value
    }));
  };

  const handlePlayerChange = (player: 'player1' | 'player2', field: 'name' | 'iocCode', value: string) => {
    setMatchData(prev => ({
      ...prev,
      [player]: {
        ...prev[player],
        [field]: value
      }
    }));
  };

  const handleStartMatch = () => {
    // Validate required fields
    if (!matchData.player1.name || !matchData.player1.iocCode ||
        !matchData.player2.name || !matchData.player2.iocCode ||
        !matchData.matchNumber || !matchData.category || !matchData.weight || !matchData.division) {
      alert('Please fill in all required fields');
      return;
    }
    
    onStartMatch(matchData);
    onClose();
  };

  const handleCancel = () => {
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm border border-gray-600/30 rounded-lg p-6 w-[600px] max-w-[90vw] max-h-[90vh] overflow-y-auto shadow-2xl">
        <h2 className="text-xl font-semibold text-white mb-6">
          New Match
        </h2>
        
        <div className="space-y-6">
          {/* Player 1 Section */}
          <div className="space-y-3">
            <h3 className="text-lg font-medium text-blue-400">Player 1 (Blue)</h3>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Name</label>
                <Input
                  type="text"
                  value={matchData.player1.name}
                  onChange={(e) => handlePlayerChange('player1', 'name', e.target.value)}
                  placeholder="Player 1 Name"
                  className="w-full"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">IOC Code</label>
                <div className="flex items-center space-x-2">
                  <Input
                    type="text"
                    value={matchData.player1.iocCode}
                    onChange={(e) => handlePlayerChange('player1', 'iocCode', e.target.value.toUpperCase())}
                    placeholder="USA"
                    className="w-full"
                    maxLength={3}
                  />
                  {matchData.player1.iocCode && (
                    <div className="flex-shrink-0">
                      <FlagImage countryCode={matchData.player1.iocCode} className="w-8 h-6" />
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>

          {/* Player 2 Section */}
          <div className="space-y-3">
            <h3 className="text-lg font-medium text-red-400">Player 2 (Red)</h3>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Name</label>
                <Input
                  type="text"
                  value={matchData.player2.name}
                  onChange={(e) => handlePlayerChange('player2', 'name', e.target.value)}
                  placeholder="Player 2 Name"
                  className="w-full"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">IOC Code</label>
                <div className="flex items-center space-x-2">
                  <Input
                    type="text"
                    value={matchData.player2.iocCode}
                    onChange={(e) => handlePlayerChange('player2', 'iocCode', e.target.value.toUpperCase())}
                    placeholder="JPN"
                    className="w-full"
                    maxLength={3}
                  />
                  {matchData.player2.iocCode && (
                    <div className="flex-shrink-0">
                      <FlagImage countryCode={matchData.player2.iocCode} className="w-8 h-6" />
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>

          {/* Match Details Section */}
          <div className="space-y-3">
            <h3 className="text-lg font-medium text-gray-300">Match Details</h3>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Match Number</label>
                <Input
                  type="text"
                  value={matchData.matchNumber}
                  onChange={(e) => handleInputChange('matchNumber', e.target.value)}
                  placeholder="1254"
                  className="w-full"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Category</label>
                <Input
                  type="text"
                  value={matchData.category}
                  onChange={(e) => handleInputChange('category', e.target.value)}
                  placeholder="Round of 16"
                  className="w-full"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Weight Class</label>
                <Input
                  type="text"
                  value={matchData.weight}
                  onChange={(e) => handleInputChange('weight', e.target.value)}
                  placeholder="M-75kg"
                  className="w-full"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Division</label>
                <Input
                  type="text"
                  value={matchData.division}
                  onChange={(e) => handleInputChange('division', e.target.value)}
                  placeholder="Senior"
                  className="w-full"
                />
              </div>
            </div>
          </div>
        </div>
        
        <div className="flex gap-3 justify-end mt-6">
          <Button
            type="button"
            variant="danger"
            onClick={handleCancel}
            className="px-4 py-2"
          >
            Cancel
          </Button>
          <Button
            type="button"
            variant="primary"
            onClick={handleStartMatch}
            className="px-4 py-2 bg-green-600 hover:bg-green-700"
          >
            Start Match
          </Button>
        </div>
      </div>
    </div>
  );
};

export default NewMatchDialog; 