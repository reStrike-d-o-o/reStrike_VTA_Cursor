import React, { useState, useEffect } from 'react';
import { diagLogsCommands } from '../../utils/tauriCommands';
import Button from '../atoms/Button';
import Toggle from '../atoms/Toggle';

interface AutoArchiveConfig {
  enabled: boolean;
  schedule: 'Weekly' | 'Monthly' | 'Quarterly' | 'Biannual' | 'Annual';
  upload_to_drive: boolean;
  delete_after_upload: boolean;
  last_archive_time?: string;
}

interface ArchiveStatus {
  should_archive: boolean;
  next_archive_time?: string;
  schedule: string;
  enabled: boolean;
}

const LogArchiveManager: React.FC = () => {
  const [config, setConfig] = useState<AutoArchiveConfig>({
    enabled: false,
    schedule: 'Monthly',
    upload_to_drive: false,
    delete_after_upload: false,
    last_archive_time: undefined,
  });
  
  const [status, setStatus] = useState<ArchiveStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>('');
  const [success, setSuccess] = useState<string>('');

  // Load configuration on component mount
  useEffect(() => {
    loadConfig();
  }, []);

  // Check status when config changes
  useEffect(() => {
    if (config.enabled) {
      checkStatus();
    }
  }, [config]);

  const loadConfig = async () => {
    try {
      const response = await diagLogsCommands.getAutoArchiveConfig();
      if (response.success && response.data) {
        setConfig(response.data);
      } else {
        setError(response.error || 'Failed to load configuration');
      }
    } catch (err) {
      setError(`Failed to load configuration: ${err}`);
    }
  };

  const saveConfig = async (newConfig: AutoArchiveConfig) => {
    setLoading(true);
    setError('');
    setSuccess('');
    
    try {
      const response = await diagLogsCommands.setAutoArchiveConfig(newConfig);
      if (response.success) {
        setConfig(newConfig);
        setSuccess('Configuration saved successfully');
        setTimeout(() => setSuccess(''), 3000);
      } else {
        setError(response.error || 'Failed to save configuration');
      }
    } catch (err) {
      setError(`Failed to save configuration: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const checkStatus = async () => {
    try {
      const response = await diagLogsCommands.checkAutoArchiveStatus(config);
      if (response.success && response.data) {
        setStatus(response.data);
      }
    } catch (err) {
      console.error('Failed to check status:', err);
    }
  };

  const handleCreateArchive = async () => {
    setLoading(true);
    setError('');
    setSuccess('');
    
    try {
      const response = await diagLogsCommands.createCompleteLogArchive();
      if (response.success) {
        setSuccess(`Archive created successfully: ${response.data?.name}`);
      } else {
        setError(response.error || 'Failed to create archive');
      }
    } catch (err) {
      setError(`Failed to create archive: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleUploadArchive = async () => {
    setLoading(true);
    setError('');
    setSuccess('');
    
    try {
      const response = await diagLogsCommands.createAndUploadLogArchive();
      if (response.success) {
        setSuccess(response.message || 'Archive uploaded successfully');
      } else {
        setError(response.error || 'Failed to upload archive');
      }
    } catch (err) {
      setError(`Failed to upload archive: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleUploadAndCleanup = async () => {
    setLoading(true);
    setError('');
    setSuccess('');
    
    try {
      const response = await diagLogsCommands.createUploadAndCleanupLogArchive();
      if (response.success) {
        setSuccess(response.message || 'Archive uploaded and cleaned up successfully');
      } else {
        setError(response.error || 'Failed to upload and cleanup archive');
      }
    } catch (err) {
      setError(`Failed to upload and cleanup archive: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handlePerformAutoArchive = async () => {
    setLoading(true);
    setError('');
    setSuccess('');
    
    try {
      const response = await diagLogsCommands.performAutoArchive(config);
      if (response.success) {
        setSuccess(response.message || 'Auto-archive completed successfully');
        if (response.updated_config) {
          setConfig(response.updated_config);
        }
      } else {
        setError(response.error || 'Auto-archive failed');
      }
    } catch (err) {
      setError(`Auto-archive failed: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const updateConfig = (updates: Partial<AutoArchiveConfig>) => {
    const newConfig = { ...config, ...updates };
    setConfig(newConfig);
    saveConfig(newConfig);
  };

  return (
  <div className="theme-card p-6 shadow-lg">
      <h3 className="text-lg font-semibold mb-4 text-gray-100 flex items-center">
        <svg className="w-5 h-5 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
        </svg>
        Log Archive Manager
      </h3>

      {/* Manual Actions + Inline Auto Toggle */}
      <div className="mb-3">
        <div className="flex flex-wrap items-center gap-3">
          <Button
            onClick={handleCreateArchive}
            disabled={loading}
            className="px-3"
            variant="secondary"
            title="Create local archive"
          >
            {loading ? 'Creating…' : 'Create'}
          </Button>
          <Button
            onClick={handleUploadArchive}
            disabled={loading}
            className="px-3"
            variant="primary"
            title="Create and upload to Drive"
          >
            {loading ? 'Uploading…' : 'Create+Upload'}
          </Button>
          <Button
            onClick={handleUploadAndCleanup}
            disabled={loading}
            className="px-3"
            variant="primary"
            title="Upload and delete local copy"
          >
            {loading ? 'Processing…' : 'Upload+Delete'}
          </Button>

          <div className="flex-1" />
          <Toggle
            id="auto-archive-enabled"
            checked={config.enabled}
            onChange={(e) => updateConfig({ enabled: e.target.checked })}
            label="Auto-Archive"
            labelPosition="left"
            className="self-center"
          />
        </div>
      </div>

      {config.enabled && (
        <div className="mt-3 space-y-3">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3 items-center">
            <div>
              <label className="block text-xs font-medium text-gray-300 mb-1">Schedule</label>
              <select
                value={config.schedule}
                onChange={(e) => updateConfig({
                  schedule: e.target.value as AutoArchiveConfig['schedule'],
                })}
                className="w-full px-2 py-1.5 bg-gray-700 border border-gray-600 rounded-md text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                title="Archive schedule frequency"
              >
                <option value="Weekly">Weekly</option>
                <option value="Monthly">Monthly</option>
                <option value="Quarterly">Every 3 months</option>
                <option value="Biannual">Every 6 months</option>
                <option value="Annual">Annually</option>
              </select>
            </div>
            <div className="flex items-center gap-4 justify-center md:justify-center">
              <Toggle
                id="upload-to-drive"
                checked={config.upload_to_drive}
                onChange={(e) => updateConfig({ upload_to_drive: e.target.checked })}
                label="Upload"
                labelPosition="right"
                className="self-center"
              />
              {config.upload_to_drive && (
                <Toggle
                  id="delete-after-upload"
                  checked={config.delete_after_upload}
                  onChange={(e) => updateConfig({ delete_after_upload: e.target.checked })}
                  label="Delete"
                  labelPosition="right"
                  className="self-center"
                />
              )}
            </div>
            <div className="text-right">
              {status?.should_archive && (
                <Button
                  onClick={handlePerformAutoArchive}
                  disabled={loading}
                  variant="primary"
                >
                  {loading ? 'Running…' : 'Run Now'}
                </Button>
              )}
            </div>
          </div>
          {status && (
            <div className="bg-gray-700/40 rounded-md p-2 text-xs text-gray-300">
              <div className="flex flex-wrap gap-4">
                <div>Status: {status.enabled ? 'Enabled' : 'Disabled'}</div>
                {status.next_archive_time && <div>Next: {status.next_archive_time}</div>}
                <div>Schedule: {status.schedule}</div>
                {status.should_archive && (
                  <div className="text-yellow-400">⚠️ Archive is due</div>
                )}
              </div>
            </div>
          )}
        </div>
      )}


      {/* Status Messages */}
      {error && (
        <div className="mb-4 p-3 bg-red-900/50 border border-red-600/50 rounded-lg">
          <div className="text-red-200 text-sm">{error}</div>
        </div>
      )}

      {success && (
        <div className="mb-4 p-3 bg-green-900/50 border border-green-600/50 rounded-lg">
          <div className="text-green-200 text-sm">{success}</div>
        </div>
      )}
    </div>
  );
};

export default LogArchiveManager; 