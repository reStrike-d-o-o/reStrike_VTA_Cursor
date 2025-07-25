import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import Button from '../atoms/Button';
import { StatusDot } from '../atoms/StatusDot';

interface DriveFile {
  id: string;
  name: string;
  mimeType: string;
  size?: string;
  createdTime: string;
  modifiedTime: string;
}

interface DriveStatus {
  connected: boolean;
  files: DriveFile[];
  loading: boolean;
  error?: string;
}

export const GoogleDriveManager: React.FC = () => {
  const [driveStatus, setDriveStatus] = useState<DriveStatus>({
    connected: false,
    files: [],
    loading: false,
  });
  const [selectedFile, setSelectedFile] = useState<string>('');
  const [operationStatus, setOperationStatus] = useState<{
    type: 'upload' | 'download' | 'restore' | 'test' | 'list_all' | null;
    loading: boolean;
    message: string;
    error?: string;
  }>({
    type: null,
    loading: false,
    message: '',
  });

  // Check connection status on component mount
  useEffect(() => {
    checkConnectionStatus();
  }, []);

  const testConnection = async () => {
    console.log('🔍 Test Connection button clicked');
    setOperationStatus({
      type: 'test',
      loading: true,
      message: 'Testing Google Drive connection...',
    });

    try {
      console.log('🔍 Calling drive_test_connection...');
      const result = await invoke<{ success: boolean; message?: string; error?: string; file_count?: number }>('drive_test_connection');
      
      if (result.success) {
        setOperationStatus({
          type: 'test',
          loading: false,
          message: result.message || 'Connection test successful!',
        });
      } else {
        console.error('Connection test failed:', result.error);
        setOperationStatus({
          type: 'test',
          loading: false,
          message: 'Connection test failed',
          error: result.error || 'Unknown error occurred during connection test',
        });
      }
    } catch (error) {
      console.error('Error testing connection:', error);
      setOperationStatus({
        type: 'test',
        loading: false,
        message: 'Connection test failed',
        error: error instanceof Error ? error.message : 'Unknown error occurred during connection test',
      });
    }
  };

  const listAllFiles = async () => {
    console.log('🔍 List All Files button clicked');
    setOperationStatus({
      type: 'list_all',
      loading: true,
      message: 'Listing all Google Drive files...',
    });

    try {
      console.log('🔍 Calling drive_list_all_files...');
      const result = await invoke<{ success: boolean; files?: DriveFile[]; error?: string }>('drive_list_all_files');
      
      if (result.success && result.files) {
        setOperationStatus({
          type: 'list_all',
          loading: false,
          message: `Found ${result.files.length} files in Google Drive`,
        });
        console.log('All Google Drive files:', result.files);
      } else {
        console.error('Failed to list all files:', result.error);
        setOperationStatus({
          type: 'list_all',
          loading: false,
          message: 'Failed to list all files',
          error: result.error || 'Unknown error occurred while listing all files',
        });
      }
    } catch (error) {
      console.error('Error listing all files:', error);
      setOperationStatus({
        type: 'list_all',
        loading: false,
        message: 'Failed to list all files',
        error: error instanceof Error ? error.message : 'Unknown error occurred while listing all files',
      });
    }
  };

  const checkConnectionStatus = async () => {
    console.log('🔍 Checking connection status...');
    try {
      console.log('🔍 Calling drive_get_connection_status...');
      const result = await invoke<{ success: boolean; connected: boolean; message?: string; error?: string }>('drive_get_connection_status');
      console.log('🔍 Connection status result:', result);
      
      if (result.success) {
        setDriveStatus(prev => ({ ...prev, connected: result.connected }));
        
        if (result.connected) {
          await listFiles();
        }
      } else {
        console.error('Connection status check failed:', result.error);
        setDriveStatus(prev => ({ ...prev, connected: false, error: result.error }));
      }
    } catch (error) {
      console.error('Failed to check connection status:', error);
      setDriveStatus(prev => ({ ...prev, connected: false, error: error instanceof Error ? error.message : 'Unknown error' }));
    }
  };

  const listFiles = async () => {
    console.log('🔍 Refresh button clicked');
    setDriveStatus(prev => ({ ...prev, loading: true, error: undefined }));
    try {
      console.log('🔍 Calling drive_list_files...');
      const result = await invoke<{ success: boolean; files?: DriveFile[]; error?: string }>('drive_list_files');
      
      if (result.success && result.files) {
        setDriveStatus(prev => ({
          ...prev,
          files: result.files!,
          loading: false,
          error: undefined,
        }));
      } else {
        setDriveStatus(prev => ({
          ...prev,
          files: [],
          loading: false,
          error: result.error || 'Failed to list files',
        }));
      }
    } catch (error) {
      console.error('Error listing files:', error);
      setDriveStatus(prev => ({
        ...prev,
        loading: false,
        error: error instanceof Error ? error.message : 'Unknown error occurred while listing files',
      }));
    }
  };

  const uploadBackupArchive = async () => {
    console.log('🔍 Create & Upload Archive button clicked');
    setOperationStatus({
      type: 'upload',
      loading: true,
      message: 'Creating backup archive...',
    });

    try {
      console.log('🔍 Calling drive_upload_backup_archive...');
      const result = await invoke<{ success: boolean; message?: string; error?: string }>('drive_upload_backup_archive');
      
      if (result.success) {
        setOperationStatus({
          type: 'upload',
          loading: false,
          message: result.message || 'Backup archive uploaded successfully!',
        });
        await listFiles(); // Refresh file list
      } else {
        console.error('Upload failed:', result.error);
        setOperationStatus({
          type: 'upload',
          loading: false,
          message: 'Upload failed',
          error: result.error || 'Unknown error occurred during upload',
        });
      }
    } catch (error) {
      console.error('Error uploading backup:', error);
      setOperationStatus({
        type: 'upload',
        loading: false,
        message: 'Upload failed',
        error: error instanceof Error ? error.message : 'Unknown error occurred during upload',
      });
    }
  };

  const downloadBackupArchive = async (fileId: string) => {
    setOperationStatus({
      type: 'download',
      loading: true,
      message: 'Downloading backup archive...',
    });

    try {
      const result = await invoke<{ success: boolean; message?: string; error?: string }>('drive_download_backup_archive', {
        fileId,
      });
      
      if (result.success) {
        setOperationStatus({
          type: 'download',
          loading: false,
          message: result.message || 'Backup archive downloaded successfully!',
        });
      } else {
        setOperationStatus({
          type: 'download',
          loading: false,
          message: 'Download failed',
          error: result.error || 'Unknown error',
        });
      }
    } catch (error) {
      setOperationStatus({
        type: 'download',
        loading: false,
        message: 'Download failed',
        error: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  };

  const restoreFromArchive = async (fileId: string) => {
    setOperationStatus({
      type: 'restore',
      loading: true,
      message: 'Restoring from backup archive...',
    });

    try {
      const result = await invoke<{ success: boolean; message?: string; error?: string }>('drive_restore_from_archive', {
        fileId,
      });
      
      if (result.success) {
        setOperationStatus({
          type: 'restore',
          loading: false,
          message: result.message || 'Restore completed successfully!',
        });
      } else {
        setOperationStatus({
          type: 'restore',
          loading: false,
          message: 'Restore failed',
          error: result.error || 'Unknown error',
        });
      }
    } catch (error) {
      setOperationStatus({
        type: 'restore',
        loading: false,
        message: 'Restore failed',
        error: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  };

  const deleteBackupArchive = async (fileId: string) => {
    if (!confirm('Are you sure you want to delete this backup archive?')) {
      return;
    }

    try {
      const result = await invoke<{ success: boolean; error?: string }>('drive_delete_backup_archive', {
        fileId,
      });
      
      if (result.success) {
        await listFiles(); // Refresh file list
      } else {
        alert(`Failed to delete archive: ${result.error || 'Unknown error'}`);
      }
    } catch (error) {
      alert(`Failed to delete archive: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatDate = (dateString: string): string => {
    return new Date(dateString).toLocaleString();
  };

  const backupFiles = driveStatus.files.filter(file => 
    file.name.endsWith('.zip') && file.name.includes('backup')
  );

  return (
    <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg p-6 border border-gray-600/30 shadow-lg">
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Left Column - Connection, Upload, Debug */}
        <div className="space-y-4">
          {/* Connection Status, Title, Subtitle, Connect Button */}
          <div className="bg-[#101820] rounded-lg p-4 border border-gray-700 mb-2">
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center space-x-3">
                <StatusDot color={driveStatus.connected ? 'green' : 'red'} />
                <div>
                  <h3 className="text-lg font-semibold text-blue-300">Google Drive Integration</h3>
                  <p className="text-sm text-gray-400">Backup and restore using Google Drive</p>
                </div>
              </div>
              <Button
                onClick={listFiles}
                disabled={!driveStatus.connected || driveStatus.loading}
                size="sm"
                variant="secondary"
              >
                {driveStatus.connected ? (driveStatus.loading ? 'Loading...' : 'Refresh') : 'Connect Google Drive'}
              </Button>
            </div>
            <p className="text-xs text-gray-400">
              {driveStatus.connected ? 'Connected' : 'Not connected'}
            </p>
          </div>

          {/* Upload Section */}
          <div className="bg-[#101820] rounded-lg p-4 border border-gray-700">
            <h4 className="text-md font-semibold text-blue-300 mb-3">Create & Upload Backup</h4>
            <div className="flex items-center space-x-2">
              <Button
                onClick={uploadBackupArchive}
                disabled={!driveStatus.connected || operationStatus.loading}
                size="sm"
                className="bg-blue-600 hover:bg-blue-700"
              >
                {operationStatus.type === 'upload' && operationStatus.loading ? 'Uploading...' : 'Upload Archive'}
              </Button>
              {operationStatus.type === 'upload' && (
                <span className="text-xs text-gray-400">
                  {operationStatus.message}
                </span>
              )}
            </div>
            {operationStatus.type === 'upload' && operationStatus.error && (
              <p className="text-xs text-red-400 mt-2">{operationStatus.error}</p>
            )}
          </div>

          {/* Debug Section */}
          <div className="bg-[#101820] rounded-lg p-4 border border-gray-700">
            <h4 className="text-md font-semibold text-blue-300 mb-3">Debug & Testing</h4>
            <div className="flex flex-wrap gap-2">
              <Button
                onClick={testConnection}
                disabled={!driveStatus.connected || operationStatus.loading}
                size="sm"
                variant="secondary"
              >
                {operationStatus.type === 'test' && operationStatus.loading ? 'Testing...' : 'Test Connection'}
              </Button>
              <Button
                onClick={listAllFiles}
                disabled={!driveStatus.connected || operationStatus.loading}
                size="sm"
                variant="secondary"
              >
                {operationStatus.type === 'list_all' && operationStatus.loading ? 'Listing...' : 'List All Files'}
              </Button>
            </div>
            {(operationStatus.type === 'test' || operationStatus.type === 'list_all') && (
              <span className="text-xs text-gray-400 mt-2 block">
                {operationStatus.message}
              </span>
            )}
            {(operationStatus.type === 'test' || operationStatus.type === 'list_all') && operationStatus.error && (
              <p className="text-xs text-red-400 mt-2">{operationStatus.error}</p>
            )}
          </div>

          {/* Operation Status */}
          {(operationStatus.type === 'download' || operationStatus.type === 'restore') && (
            <div className="bg-[#101820] rounded-lg p-4 border border-gray-700">
              <h4 className="text-md font-semibold text-blue-300 mb-2">Operation Status</h4>
              <p className="text-xs text-gray-400">{operationStatus.message}</p>
              {operationStatus.error && (
                <p className="text-xs text-red-400 mt-2">{operationStatus.error}</p>
              )}
            </div>
          )}
        </div>

        {/* Right Column - Backup Archives */}
        <div className="bg-[#101820] rounded-lg p-4 border border-gray-700">
          <h4 className="text-md font-semibold text-blue-300 mb-3">
            Backup Archives ({backupFiles.length})
          </h4>
          
          {driveStatus.error && (
            <p className="text-xs text-red-400 mb-3">{driveStatus.error}</p>
          )}

          {backupFiles.length === 0 ? (
            <p className="text-xs text-gray-400">No backup archives found</p>
          ) : (
            <div className="max-h-64 overflow-y-auto border border-gray-700 rounded">
              <table className="min-w-full text-left text-xs text-gray-200">
                <thead className="bg-[#0a0f14] sticky top-0 z-10">
                  <tr>
                    <th className="px-2 py-2 font-semibold">File Name</th>
                    <th className="px-2 py-2 font-semibold">Size</th>
                    <th className="px-2 py-2 font-semibold">Created</th>
                    <th className="px-2 py-2 font-semibold">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {backupFiles.map((file) => (
                    <tr
                      key={file.id}
                      className={`hover:bg-blue-900 transition-colors cursor-pointer ${
                        selectedFile === file.id ? 'bg-blue-900/20' : ''
                      }`}
                      onClick={() => setSelectedFile(file.id)}
                    >
                      <td className="px-2 py-2 whitespace-nowrap max-w-32 truncate">
                        {file.name}
                      </td>
                      <td className="px-2 py-2 whitespace-nowrap">
                        {file.size ? formatFileSize(parseInt(file.size)) : 'Unknown'}
                      </td>
                      <td className="px-2 py-2 whitespace-nowrap text-xs">
                        {formatDate(file.createdTime)}
                      </td>
                      <td className="px-2 py-2 whitespace-nowrap">
                        <div className="flex items-center space-x-1">
                          <Button
                            onClick={(e: React.MouseEvent) => {
                              e.stopPropagation();
                              downloadBackupArchive(file.id);
                            }}
                            disabled={operationStatus.loading}
                            variant="ghost"
                            size="sm"
                            className="text-green-400 hover:text-green-300 text-xs px-1 py-1"
                          >
                            {operationStatus.type === 'download' && operationStatus.loading ? 'DL...' : 'DL'}
                          </Button>
                          
                          <Button
                            onClick={(e: React.MouseEvent) => {
                              e.stopPropagation();
                              restoreFromArchive(file.id);
                            }}
                            disabled={operationStatus.loading}
                            variant="ghost"
                            size="sm"
                            className="text-blue-400 hover:text-blue-300 text-xs px-1 py-1"
                          >
                            {operationStatus.type === 'restore' && operationStatus.loading ? 'RS...' : 'RS'}
                          </Button>
                          
                          <Button
                            onClick={(e: React.MouseEvent) => {
                              e.stopPropagation();
                              deleteBackupArchive(file.id);
                            }}
                            disabled={operationStatus.loading}
                            variant="ghost"
                            size="sm"
                            className="text-red-400 hover:text-red-300 text-xs px-1 py-1"
                          >
                            DEL
                          </Button>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}; 