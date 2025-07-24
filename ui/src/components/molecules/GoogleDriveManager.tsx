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
    setOperationStatus({
      type: 'test',
      loading: true,
      message: 'Testing Google Drive connection...',
    });

    try {
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
    setOperationStatus({
      type: 'list_all',
      loading: true,
      message: 'Listing all Google Drive files...',
    });

    try {
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
    try {
      const result = await invoke<{ connected: boolean }>('drive_get_connection_status');
      setDriveStatus(prev => ({ ...prev, connected: result.connected }));
      
      if (result.connected) {
        await listFiles();
      }
    } catch (error) {
      console.error('Failed to check connection status:', error);
    }
  };

  const listFiles = async () => {
    setDriveStatus(prev => ({ ...prev, loading: true, error: undefined }));
    try {
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
    setOperationStatus({
      type: 'upload',
      loading: true,
      message: 'Creating backup archive...',
    });

    try {
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
    <div className="space-y-6">
      {/* Connection Status */}
      <div className="flex items-center justify-between p-4 bg-[#1a1a1a] rounded-lg border border-gray-700">
        <div className="flex items-center space-x-3">
          <StatusDot color={driveStatus.connected ? 'green' : 'red'} />
          <div>
            <h3 className="text-lg font-semibold text-white">Google Drive</h3>
            <p className="text-sm text-gray-400">
              {driveStatus.connected ? 'Connected' : 'Not connected'}
            </p>
          </div>
        </div>
        <Button
          onClick={listFiles}
          disabled={!driveStatus.connected || driveStatus.loading}
          className="px-4 py-2"
        >
          {driveStatus.loading ? 'Loading...' : 'Refresh'}
        </Button>
      </div>

      {/* Upload Section */}
      <div className="p-4 bg-[#1a1a1a] rounded-lg border border-gray-700">
        <h4 className="text-md font-semibold text-white mb-3">Create & Upload Backup</h4>
        <div className="flex items-center space-x-3">
          <Button
            onClick={uploadBackupArchive}
            disabled={!driveStatus.connected || operationStatus.loading}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700"
          >
            {operationStatus.type === 'upload' && operationStatus.loading ? 'Uploading...' : 'Create & Upload Archive'}
          </Button>
          {operationStatus.type === 'upload' && (
            <span className="text-sm text-gray-400">
              {operationStatus.message}
            </span>
          )}
        </div>
        {operationStatus.type === 'upload' && operationStatus.error && (
          <p className="text-sm text-red-400 mt-2">{operationStatus.error}</p>
        )}
      </div>

      {/* Debug Section */}
      <div className="p-4 bg-[#1a1a1a] rounded-lg border border-gray-700">
        <h4 className="text-md font-semibold text-white mb-3">Debug & Testing</h4>
        <div className="flex items-center space-x-3">
          <Button
            onClick={testConnection}
            disabled={!driveStatus.connected || operationStatus.loading}
            className="px-4 py-2 bg-green-600 hover:bg-green-700"
          >
            {operationStatus.type === 'test' && operationStatus.loading ? 'Testing...' : 'Test Connection'}
          </Button>
          <Button
            onClick={listAllFiles}
            disabled={!driveStatus.connected || operationStatus.loading}
            className="px-4 py-2 bg-yellow-600 hover:bg-yellow-700"
          >
            {operationStatus.type === 'list_all' && operationStatus.loading ? 'Listing...' : 'List All Files'}
          </Button>
        </div>
        {(operationStatus.type === 'test' || operationStatus.type === 'list_all') && (
          <span className="text-sm text-gray-400 mt-2 block">
            {operationStatus.message}
          </span>
        )}
        {(operationStatus.type === 'test' || operationStatus.type === 'list_all') && operationStatus.error && (
          <p className="text-sm text-red-400 mt-2">{operationStatus.error}</p>
        )}
      </div>

      {/* Backup Files List */}
      <div className="p-4 bg-[#1a1a1a] rounded-lg border border-gray-700">
        <h4 className="text-md font-semibold text-white mb-3">
          Backup Archives ({backupFiles.length})
        </h4>
        
        {driveStatus.error && (
          <p className="text-sm text-red-400 mb-3">{driveStatus.error}</p>
        )}

        {backupFiles.length === 0 ? (
          <p className="text-sm text-gray-400">No backup archives found</p>
        ) : (
          <div className="space-y-2 max-h-64 overflow-y-auto">
            {backupFiles.map((file) => (
              <div
                key={file.id}
                className={`p-3 border rounded-lg cursor-pointer transition-colors ${
                  selectedFile === file.id
                    ? 'border-blue-500 bg-blue-900/20'
                    : 'border-gray-600 hover:border-gray-500'
                }`}
                onClick={() => setSelectedFile(file.id)}
              >
                <div className="flex items-center justify-between">
                  <div className="flex-1 min-w-0">
                    <h5 className="text-sm font-medium text-white truncate">
                      {file.name}
                    </h5>
                    <div className="flex items-center space-x-4 text-xs text-gray-400 mt-1">
                      <span>Size: {file.size ? formatFileSize(parseInt(file.size)) : 'Unknown'}</span>
                      <span>Created: {formatDate(file.createdTime)}</span>
                      <span>Modified: {formatDate(file.modifiedTime)}</span>
                    </div>
                  </div>
                  
                  <div className="flex items-center space-x-2 ml-4">
                    <Button
                      onClick={(e: React.MouseEvent) => {
                        e.stopPropagation();
                        downloadBackupArchive(file.id);
                      }}
                      disabled={operationStatus.loading}
                      className="px-3 py-1 text-xs bg-green-600 hover:bg-green-700"
                    >
                      {operationStatus.type === 'download' && operationStatus.loading ? 'Downloading...' : 'Download'}
                    </Button>
                    
                    <Button
                      onClick={(e: React.MouseEvent) => {
                        e.stopPropagation();
                        restoreFromArchive(file.id);
                      }}
                      disabled={operationStatus.loading}
                      className="px-3 py-1 text-xs bg-blue-600 hover:bg-blue-700"
                    >
                      {operationStatus.type === 'restore' && operationStatus.loading ? 'Restoring...' : 'Restore'}
                    </Button>
                    
                    <Button
                      onClick={(e: React.MouseEvent) => {
                        e.stopPropagation();
                        deleteBackupArchive(file.id);
                      }}
                      disabled={operationStatus.loading}
                      className="px-3 py-1 text-xs bg-red-600 hover:bg-red-700"
                    >
                      Delete
                    </Button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Operation Status */}
      {(operationStatus.type === 'download' || operationStatus.type === 'restore') && (
        <div className="p-4 bg-[#1a1a1a] rounded-lg border border-gray-700">
          <h4 className="text-md font-semibold text-white mb-2">Operation Status</h4>
          <p className="text-sm text-gray-400">{operationStatus.message}</p>
          {operationStatus.error && (
            <p className="text-sm text-red-400 mt-2">{operationStatus.error}</p>
          )}
        </div>
      )}
    </div>
  );
}; 