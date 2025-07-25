import React, { useState } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';

interface ManualModeDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onToggle: (code: string) => boolean;
  isEnabled: boolean;
}

const ManualModeDialog: React.FC<ManualModeDialogProps> = ({
  isOpen,
  onClose,
  onToggle,
  isEnabled
}) => {
  const [code, setCode] = useState('');
  const [error, setError] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    
    const isValid = onToggle(code);
    if (isValid) {
      setCode('');
      onClose();
    } else {
      setError('Wrong code. Please try again.');
    }
  };

  const handleCancel = () => {
    setCode('');
    setError('');
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm border border-gray-600/30 rounded-lg p-6 w-96 max-w-[90vw] shadow-2xl">
        <h2 className="text-xl font-semibold text-white mb-2">
          Manual Mode {isEnabled ? 'Disable' : 'Enable'}
        </h2>
        <p className="text-gray-300 mb-4">
          For manual mode please write: "el Manuel"
        </p>
        
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <Input
              type="text"
              value={code}
              onChange={(e) => setCode(e.target.value)}
              placeholder="Enter code"
              className="w-full"
              autoFocus
            />
          </div>
          
          {error && (
            <div className="text-red-400 text-sm">{error}</div>
          )}
          
          <div className="flex gap-3 justify-end">
            <Button
              type="button"
              variant="secondary"
              onClick={handleCancel}
              className="px-4 py-2"
            >
              Cancel
            </Button>
            <Button
              type="submit"
              variant="primary"
              className="px-4 py-2"
            >
              {isEnabled ? 'Disable' : 'Enable'}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default ManualModeDialog; 