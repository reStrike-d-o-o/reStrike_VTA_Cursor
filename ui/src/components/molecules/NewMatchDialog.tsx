import React, { useState } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import { FlagImage } from '../../utils/flagUtils';
import { useI18n } from '../../i18n/index';

interface NewMatchDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onStartMatch: (matchData: ManualMatchData) => void;
}

export interface ManualMatchData {
  player1: {
    name: string;
    ioc_code: string;
  };
  player2: {
    name: string;
    ioc_code: string;
  };
  match_number: string;
  category: string;
  weight: string;
  division: string;
}

const NewMatchDialog: React.FC<NewMatchDialogProps> = ({
  isOpen,
  onClose,
  onStartMatch
}) => {
  const { t } = useI18n();
  const [matchData, setMatchData] = useState<ManualMatchData>({
    player1: { name: '', ioc_code: '' },
    player2: { name: '', ioc_code: '' },
    match_number: '',
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

  const handlePlayerChange = (player: 'player1' | 'player2', field: 'name' | 'ioc_code', value: string) => {
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
    if (!matchData.player1.name || !matchData.player1.ioc_code ||
        !matchData.player2.name || !matchData.player2.ioc_code ||
        !matchData.match_number || !matchData.category || !matchData.weight || !matchData.division) {
      alert(t('dialog.new_match.required', 'Please fill in all required fields'));
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
          {t('dialog.new_match.title', 'New Match')}
        </h2>
        
        <div className="space-y-6">
          {/* Player 1 Section */}
          <div className="space-y-3">
            <h3 className="text-lg font-medium text-blue-400">{t('dialog.new_match.player1', 'Player 1 (Blue)')}</h3>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">{t('dialog.new_match.name', 'Name')}</label>
                <Input
                  type="text"
                  value={matchData.player1.name}
                  onChange={(e) => handlePlayerChange('player1', 'name', e.target.value)}
                  placeholder={t('dialog.new_match.player1_name', 'Player 1 Name')}
                  className="w-full"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">{t('dialog.new_match.ioc_code', 'IOC Code')}</label>
                <div className="flex items-center space-x-2">
                  <Input
                    type="text"
                    value={matchData.player1.ioc_code}
                    onChange={(e) => handlePlayerChange('player1', 'ioc_code', e.target.value.toUpperCase())}
                    placeholder={t('dialog.new_match.ioc_placeholder', 'USA')}
                    className="w-full"
                    maxLength={3}
                  />
                  {matchData.player1.ioc_code && (
                    <div className="flex-shrink-0">
                      <FlagImage countryCode={matchData.player1.ioc_code} className="w-8 h-6" />
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>

          {/* Player 2 Section */}
          <div className="space-y-3">
            <h3 className="text-lg font-medium text-red-400">{t('dialog.new_match.player2', 'Player 2 (Red)')}</h3>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">{t('dialog.new_match.name', 'Name')}</label>
                <Input
                  type="text"
                  value={matchData.player2.name}
                  onChange={(e) => handlePlayerChange('player2', 'name', e.target.value)}
                  placeholder={t('dialog.new_match.player2_name', 'Player 2 Name')}
                  className="w-full"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">{t('dialog.new_match.ioc_code', 'IOC Code')}</label>
                <div className="flex items-center space-x-2">
                  <Input
                    type="text"
                    value={matchData.player2.ioc_code}
                    onChange={(e) => handlePlayerChange('player2', 'ioc_code', e.target.value.toUpperCase())}
                    placeholder={t('dialog.new_match.ioc_placeholder', 'JPN')}
                    className="w-full"
                    maxLength={3}
                  />
                  {matchData.player2.ioc_code && (
                    <div className="flex-shrink-0">
                      <FlagImage countryCode={matchData.player2.ioc_code} className="w-8 h-6" />
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>

          {/* Match Details Section */}
          <div className="space-y-3">
            <h3 className="text-lg font-medium text-gray-300">{t('dialog.new_match.details', 'Match Details')}</h3>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">{t('dialog.new_match.number', 'Match Number')}</label>
                <Input
                  type="text"
                  value={matchData.match_number}
                  onChange={(e) => handleInputChange('match_number', e.target.value)}
                  placeholder={t('dialog.new_match.number_placeholder', '1254')}
                  className="w-full"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">{t('dialog.new_match.category', 'Category')}</label>
                <Input
                  type="text"
                  value={matchData.category}
                  onChange={(e) => handleInputChange('category', e.target.value)}
                  placeholder={t('dialog.new_match.category_placeholder', 'Round of 16')}
                  className="w-full"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">{t('dialog.new_match.weight', 'Weight Class')}</label>
                <Input
                  type="text"
                  value={matchData.weight}
                  onChange={(e) => handleInputChange('weight', e.target.value)}
                  placeholder={t('dialog.new_match.weight_placeholder', 'M-75kg')}
                  className="w-full"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">{t('dialog.new_match.division', 'Division')}</label>
                <Input
                  type="text"
                  value={matchData.division}
                  onChange={(e) => handleInputChange('division', e.target.value)}
                  placeholder={t('dialog.new_match.division_placeholder', 'Senior')}
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
            {t('common.cancel', 'Cancel')}
          </Button>
          <Button
            type="button"
            variant="primary"
            onClick={handleStartMatch}
            className="px-4 py-2 bg-green-600 hover:bg-green-700"
          >
            {t('dialog.new_match.start', 'Start Match')}
          </Button>
        </div>
      </div>
    </div>
  );
};

export default NewMatchDialog; 