/**
 * PasswordDialog
 * - Prompt for Advanced mode authentication
 */
import React, { useState } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';

interface PasswordDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onAuthenticate: (password: string) => boolean;
  title?: string;
  message?: string;
}

const PasswordDialog: React.FC<PasswordDialogProps> = ({
  isOpen,
  onClose,
  onAuthenticate,
  title = 'Advanced Mode Authentication',
  message = 'Please enter the password to enable Advanced mode:'
}) => {
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    
    const isValid = onAuthenticate(password);
    if (isValid) {
      setPassword('');
      onClose();
    } else {
      setError('Wrong password. Please try again.');
    }
  };

  const handleCancel = () => {
    setPassword('');
    setError('');
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm border border-gray-600/30 rounded-lg p-6 w-96 max-w-[90vw] shadow-2xl">
        <h2 className="text-xl font-semibold text-white mb-2">{title}</h2>
        <p className="text-gray-300 mb-4">{message}</p>
        
        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Hidden username field for accessibility */}
          <input type="text" autoComplete="username" className="hidden" aria-hidden="true" tabIndex={-1} />
          
          <div>
            <Input
              type="password"
              autoComplete="new-password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Enter password"
              className="w-full"
              autoFocus
              aria-label="Password"
              title="Password"
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
            <Button type="submit" className="px-4 py-2">
              Confirm
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default PasswordDialog; 