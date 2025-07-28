import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import ConnectDriveButton from '../atoms/ConnectDriveButton';
import { GoogleDriveManager } from './GoogleDriveManager';
import Button from '../atoms/Button';
import TabGroup from './TabGroup';
import TabIcons from '../atoms/TabIcons';
import LottieIcon from '../atoms/LottieIcon';
import { floppyDiscAnimation, downloadAnimation } from '../../assets/icons/json';

interface BackupFileInfo {
  name: string;
  path: string;
  size: number;
  modified: string;
}

export const GoogleDriveBackupRestore: React.FC = () => {
  const [activeTab, setActiveTab] = useState('local');
  const [backupFiles, setBackupFiles] = useState<BackupFileInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Load backup files on component mount
  useEffect(() => {
    loadBackupFiles();
  }, []);

  const loadBackupFiles = async () => {
    try {
      const result = await invoke<BackupFileInfo[]>('list_backup_files');
      setBackupFiles(result);
    } catch (error) {
      console.error('Failed to load backup files:', error);
      setError('Failed to load backup files');
    }
  };

  const createJsonBackup = async () => {
    try {
      const result = await invoke<{ success: boolean; message?: string; error?: string }>('create_json_backup');
      if (result.success) {
        setSuccess(result.message || 'Backup created successfully');
        await loadBackupFiles(); // Refresh the backup files list
      } else {
        setError(result.error || 'Failed to create backup');
      }
    } catch (error) {
      console.error('Error creating backup:', error);
      setError('Failed to create backup');
    }
  };

  const handleRestoreBackup = async (backupPath: string) => {
    if (!confirm('Are you sure you want to restore from this backup? This will overwrite current settings.')) {
      return;
    }

    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      const result = await invoke<{ success: boolean; message?: string; error?: string }>('restore_from_backup', {
        backupPath
      });
      
      if (result.success) {
        setSuccess(result.message || 'Backup restored successfully');
      } else {
        setError(result.error || 'Failed to restore backup');
      }
    } catch (error) {
      console.error('Error restoring backup:', error);
      setError('Failed to restore backup');
    } finally {
      setLoading(false);
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-white">Backup & Restore</h2>
          <p className="text-sm text-gray-400">Manage local backups and Google Drive integration</p>
        </div>
      </div>

      {/* Error/Success Messages */}
      {error && (
        <div className="bg-red-900/20 border border-red-500/50 rounded-lg p-3">
          <span className="text-red-400 font-medium">Error</span>
          <p className="text-red-300 mt-1 text-sm">{error}</p>
        </div>
      )}

      {success && (
        <div className="bg-green-900/20 border border-green-500/50 rounded-lg p-3">
          <span className="text-green-400 font-medium">Success</span>
          <p className="text-green-300 mt-1 text-sm">{success}</p>
        </div>
      )}

      {/* Tab Navigation */}
      <TabGroup
        tabs={[
          {
            id: 'local',
            label: 'Local Backup',
            icon: <LottieIcon animationData={floppyDiscAnimation} size={32} />,
            content: (
              <div className="space-y-4">
                {/* Local Backup Controls */}
                <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg p-6 border border-gray-600/30 shadow-lg">
                  <div className="flex justify-between items-center mb-4">
                    <h3 className="text-lg font-semibold text-blue-300">Local Backup</h3>
                    <Button
                      onClick={async () => {
                        setLoading(true);
                        setError(null);
                        setSuccess(null);
                        try {
                          await createJsonBackup();
                        } catch (error) {
                          setError('Failed to create backup');
                          console.error('❌ Error creating backup:', error);
                        } finally {
                          setLoading(false);
                        }
                      }}
                      disabled={loading}
                      variant="primary"
                      size="sm"
                    >
                      {loading ? 'Creating...' : 'Create Backup'}
                    </Button>
                  </div>

                  {/* Backup Files List */}
                  <div className="max-h-64 overflow-y-auto border border-gray-700 rounded">
                    <table className="min-w-full text-left text-sm text-gray-200">
                      <thead className="bg-[#101820] sticky top-0 z-10">
                        <tr>
                          <th className="px-3 py-2 font-semibold">File Name</th>
                          <th className="px-3 py-2 font-semibold">Size</th>
                          <th className="px-3 py-2 font-semibold">Modified</th>
                          <th className="px-3 py-2 font-semibold">Action</th>
                        </tr>
                      </thead>
                      <tbody>
                        {backupFiles.length === 0 ? (
                          <tr>
                            <td colSpan={4} className="px-3 py-2 text-gray-400 text-center">
                              No backup files found
                              <br />
                              <span className="text-xs">Create a backup to see files here</span>
                            </td>
                          </tr>
                        ) : (
                          backupFiles.map((file, index) => (
                            <tr
                              key={index}
                              className="hover:bg-blue-900 transition-colors"
                            >
                              <td className="px-3 py-2 whitespace-nowrap">
                                {file.name}
                              </td>
                              <td className="px-3 py-2 whitespace-nowrap">
                                {formatFileSize(file.size)}
                              </td>
                              <td className="px-3 py-2 whitespace-nowrap">
                                {file.modified}
                              </td>
                              <td className="px-3 py-2 whitespace-nowrap">
                                <Button
                                  onClick={() => handleRestoreBackup(file.path)}
                                  variant="ghost"
                                  size="sm"
                                  className="text-blue-400 hover:text-blue-300"
                                  disabled={loading}
                                >
                                  {loading ? 'Restoring...' : 'Restore'}
                                </Button>
                              </td>
                            </tr>
                          ))
                        )}
                      </tbody>
                    </table>
                  </div>
                </div>
              </div>
            )
          },
          {
            id: 'google-drive',
            label: 'Google Drive',
            icon: <LottieIcon animationData={downloadAnimation} size={32} />,
            content: (
              <div className="space-y-4">
                {/* Google Drive Header */}
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="text-lg font-semibold text-blue-300">Google Drive Integration</h3>
                    <p className="text-sm text-gray-400">Backup and restore using Google Drive</p>
        </div>
        <ConnectDriveButton />
      </div>

      {/* Google Drive Manager */}
      <GoogleDriveManager />
              </div>
            )
          }
        ]}
        activeTab={activeTab}
        onTabChange={setActiveTab}
      />
    </div>
  );
}; 