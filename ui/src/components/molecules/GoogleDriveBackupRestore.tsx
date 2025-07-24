import React from 'react';
import ConnectDriveButton from '../atoms/ConnectDriveButton';
import { GoogleDriveManager } from './GoogleDriveManager';

export const GoogleDriveBackupRestore: React.FC = () => {
  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-white">Google Drive Backup & Restore</h2>
          <p className="text-sm text-gray-400">Backup and restore database using Google Drive</p>
        </div>
        <ConnectDriveButton />
      </div>

      {/* Google Drive Manager */}
      <GoogleDriveManager />
    </div>
  );
}; 