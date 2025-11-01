import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import ConnectDriveButton from '../atoms/ConnectDriveButton';
import { GoogleDriveManager } from './GoogleDriveManager';
import Button from '../atoms/Button';
import TabGroup from './TabGroup';
import TabIcons from '../atoms/TabIcons';
import LottieIcon from '../atoms/LottieIcon';
import { floppyDiscAnimation, downloadAnimation } from '../../assets/icons/json';
import { useI18n } from '../../i18n/index';

interface SqliteBackupInfo {
  name: string;
  path: string;
  size: number;
  modified: string;
}

export const GoogleDriveBackupRestore: React.FC = () => {
  const { t } = useI18n();
  const [activeTab, setActiveTab] = useState('local');
  const [sqliteBackups, setSqliteBackups] = useState<SqliteBackupInfo[]>([]);
  const [sqliteLoading, setSqliteLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Load backup files on component mount
  useEffect(() => {
    loadSqliteBackups();
  }, []);

  const loadSqliteBackups = async () => {
    try {
      setSqliteLoading(true);
      const result = await invoke<{ success: boolean; backups?: SqliteBackupInfo[]; error?: string }>('db_list_sqlite_backups');
      if (result?.success) {
        setSqliteBackups(result.backups || []);
      } else {
        setError(result?.error || t('backup.error.load_files', 'Failed to load backup files'));
      }
    } catch (err) {
      console.error('Failed to load SQLite backups:', err);
      setError(t('backup.error.load_files', 'Failed to load backup files'));
    } finally {
      setSqliteLoading(false);
    }
  };

  const createSqliteBackup = async () => {
    try {
      setSqliteLoading(true);
      setError(null);
      setSuccess(null);
      const result = await invoke<{ success: boolean; path?: string; error?: string }>('db_create_sqlite_backup', { name: null });
      if (result?.success) {
        setSuccess(t('backup.create_ok', 'Backup created successfully'));
        await loadSqliteBackups();
      } else {
        setError(result?.error || t('backup.create_failed', 'Failed to create backup'));
      }
    } catch (error) {
      console.error('Error creating SQLite backup:', error);
      setError(t('backup.create_failed', 'Failed to create backup'));
    } finally {
      setSqliteLoading(false);
    }
  };

  const handleRestoreSqliteBackup = async (backupPath: string) => {
    if (!confirm(t('backup.confirm_restore', 'Are you sure you want to restore from this backup? This will overwrite current settings.'))) {
      return;
    }

    setSqliteLoading(true);
    setError(null);
    setSuccess(null);

    try {
      const result = await invoke<{ success: boolean; error?: string }>('db_restore_sqlite_backup', {
        backupPath
      });
      
      if (result?.success) {
        setSuccess(t('backup.restore_ok', 'Backup restored successfully'));
      } else {
        setError(result?.error || t('backup.restore_failed', 'Failed to restore backup'));
      }
    } catch (error) {
      console.error('Error restoring SQLite backup:', error);
      setError(t('backup.restore_failed', 'Failed to restore backup'));
    } finally {
      setSqliteLoading(false);
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
          <h2 className="text-xl font-semibold text-white">{t('backup.title', 'Backup & Restore')}</h2>
          <p className="text-sm text-gray-400">{t('backup.subtitle', 'Manage local backups and Google Drive integration')}</p>
        </div>
      </div>

      {/* Error/Success Messages */}
      {error && (
        <div className="bg-red-900/20 border border-red-500/50 rounded-lg p-3">
          <span className="text-red-400 font-medium">{t('common.error', 'Error')}</span>
          <p className="text-red-300 mt-1 text-sm">{error}</p>
        </div>
      )}

      {success && (
        <div className="bg-green-900/20 border border-green-500/50 rounded-lg p-3">
          <span className="text-green-400 font-medium">{t('common.success', 'Success')}</span>
          <p className="text-green-300 mt-1 text-sm">{success}</p>
        </div>
      )}

      {/* Tab Navigation */}
      <TabGroup
        tabs={[
          {
            id: 'local',
            label: t('backup.tabs.local', 'Local Backup'),
            icon: <LottieIcon animationData={floppyDiscAnimation} size={32} />,
            content: (
              <div className="space-y-4">
                {/* Local Backup Controls */}
                <div className="theme-card p-6 shadow-lg space-y-6">
                  <div className="flex justify-between items-center">
                    <h3 className="text-lg font-semibold text-blue-300">{t('backup.local.title', 'Local Backup')}</h3>
                    <Button
                      onClick={createSqliteBackup}
                      disabled={sqliteLoading}
                      variant="primary"
                      size="sm"
                    >
                      {sqliteLoading ? t('common.creating', 'Creating...') : t('backup.local.create', 'Create Backup')}
                    </Button>
                  </div>

                  <div>
                    <h4 className="text-sm font-semibold text-gray-200 mb-2 uppercase tracking-wide">
                      {t('backup.local.sqlite_heading', 'Full Database Backups')}
                    </h4>
                    <div className="max-h-64 overflow-y-auto border border-gray-700 rounded">
                      <table className="min-w-full text-left text-sm text-gray-200">
                        <thead className="theme-surface-2 sticky top-0 z-10">
                          <tr>
                            <th className="px-3 py-2 font-semibold">{t('backup.table.file', 'File Name')}</th>
                            <th className="px-3 py-2 font-semibold">{t('backup.table.size', 'Size')}</th>
                            <th className="px-3 py-2 font-semibold">{t('backup.table.modified', 'Modified')}</th>
                            <th className="px-3 py-2 font-semibold">{t('backup.table.action', 'Action')}</th>
                          </tr>
                        </thead>
                        <tbody>
                          {sqliteLoading && sqliteBackups.length === 0 ? (
                            <tr>
                              <td colSpan={4} className="px-3 py-2 text-gray-400 text-center">
                                {t('common.loading', 'Loading...')}
                              </td>
                            </tr>
                          ) : sqliteBackups.length === 0 ? (
                            <tr>
                              <td colSpan={4} className="px-3 py-2 text-gray-400 text-center">
                                {t('backup.none', 'No backup files found')}
                                <br />
                                <span className="text-xs">{t('backup.none_hint', 'Create a backup to see files here')}</span>
                              </td>
                            </tr>
                          ) : (
                            sqliteBackups.map((file, index) => (
                              <tr
                                key={`${file.path}-${index}`}
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
                                    onClick={() => handleRestoreSqliteBackup(file.path)}
                                    variant="ghost"
                                    size="sm"
                                    className="text-blue-400 hover:text-blue-300"
                                    disabled={sqliteLoading}
                                  >
                                    {sqliteLoading ? t('backup.restoring', 'Restoring...') : t('backup.restore', 'Restore')}
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
              </div>
            )
          },
          {
            id: 'google-drive',
            label: t('backup.tabs.drive', 'Google Drive'),
            icon: <LottieIcon animationData={downloadAnimation} size={32} />,
            content: (
              <div className="space-y-4">
                {/* Google Drive Header */}
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="text-lg font-semibold text-blue-300">{t('backup.drive.title', 'Google Drive Integration')}</h3>
                    <p className="text-sm text-gray-400">{t('backup.drive.subtitle', 'Backup and restore using Google Drive')}</p>
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
