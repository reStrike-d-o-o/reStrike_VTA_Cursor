import React, { useState, useEffect } from 'react';
import { diagLogsCommands } from '../../utils/tauriCommands';
import { useI18n } from '../../i18n/index';

const logTypes = [
  { key: 'pss', label: 'PSS' },
  { key: 'obs', label: 'OBS' },
  { key: 'udp', label: 'UDP' },
  { key: 'websocket', label: 'WebSocket' },
  { key: 'db', label: 'Database' },
  { key: 'app', label: 'Application' },
  { key: 'arc', label: 'Archive' },
];

type LogType = 'pss' | 'obs' | 'udp' | 'websocket' | 'db' | 'app' | 'arc';

type LogFileInfo = {
  name: string;
  size: number;
  modified: string;
  subsystem: string;
};

const LogDownloadList: React.FC = () => {
  const [selectedType, setSelectedType] = useState<LogType>('pss');
  const [logs, setLogs] = useState<LogFileInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>('');
  const [downloading, setDownloading] = useState<string>('');
  const { t } = useI18n();

  const fetchLogs = async () => {
    setLoading(true);
    setError('');
    try {
      let res;
      if (selectedType === 'arc') {
        // For archives, use the listArchives command
        res = await diagLogsCommands.listArchives();
        if (res.success && Array.isArray(res.data)) {
          // Transform archive data to match LogFileInfo format
          const archiveLogs = res.data.map((archiveName: string) => ({
            name: archiveName,
            size: 0, // Archive size not provided by backend yet
            modified: '', // Archive date not provided by backend yet
            subsystem: 'archive'
          }));
          setLogs(archiveLogs);
        } else {
          setLogs([]);
          setError(res.error || t('logs.error.fetch_archives', 'Failed to fetch archive files'));
        }
      } else {
        // For regular logs, use the listLogFiles command
        res = await diagLogsCommands.listLogFiles(selectedType);
        if (res.success && Array.isArray(res.data)) {
          setLogs(res.data);
        } else {
          setLogs([]);
          setError(res.error || t('logs.error.fetch_logs', 'Failed to fetch log files'));
        }
      }
    } catch (err) {
      setLogs([]);
      const errorMessage = err instanceof Error ? err.message : String(err);
      if (errorMessage.includes('timeout') || errorMessage.includes('timed out')) {
        setError(t('logs.error.timeout', 'Command timed out. The backend may be busy or unresponsive. Please try again.'));
      } else if (errorMessage.includes('Cannot read properties of undefined')) {
        setError(t('logs.error.tauri_missing', 'Tauri not available. Please ensure the app is running in desktop mode.'));
      } else {
        setError(t('logs.error.generic_fetch', 'Error fetching {what}: {msg}', { what: selectedType === 'arc' ? 'archives' : 'logs', msg: errorMessage }));
      }
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchLogs();
  }, [selectedType]);

  const handleDownload = async (logName: string) => {
    setDownloading(logName);
    try {
      if (selectedType === 'arc') {
        // For archives, download the ZIP file
        const res = await diagLogsCommands.downloadArchive(logName);
        if (res.success && res.data) {
          const blob = new Blob([new Uint8Array(res.data)], { type: 'application/zip' });
          const url = window.URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = logName;
          document.body.appendChild(a);
          a.click();
          a.remove();
          window.URL.revokeObjectURL(url);
        } else {
          const errorMsg = res.error || t('common.unknown_error', 'Unknown error');
          if (errorMsg.includes('timeout') || errorMsg.includes('timed out')) {
            alert(t('logs.error.download_timeout', 'Download timed out for {name}. Please try again.', { name: logName }));
          } else {
            alert(t('logs.error.download_failed', 'Failed to download {name}: {msg}', { name: logName, msg: errorMsg }));
          }
        }
      } else {
        // For regular logs, download them
        const res = await diagLogsCommands.downloadLogFile(logName);
        if (res.success && res.data) {
          const blob = new Blob([new Uint8Array(res.data)], { type: 'text/plain' });
          const url = window.URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = logName;
          document.body.appendChild(a);
          a.click();
          a.remove();
          window.URL.revokeObjectURL(url);
        } else {
          const errorMsg = res.error || t('common.unknown_error', 'Unknown error');
          if (errorMsg.includes('timeout') || errorMsg.includes('timed out')) {
            alert(t('logs.error.download_timeout', 'Download timed out for {name}. Please try again.', { name: logName }));
          } else {
            alert(t('logs.error.download_failed', 'Failed to download {name}: {msg}', { name: logName, msg: errorMsg }));
          }
        }
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      if (errorMessage.includes('Cannot read properties of undefined')) {
        alert(t('logs.error.tauri_no_download', 'Tauri not available. Cannot download {name} in web mode.', { name: logName }));
      } else if (errorMessage.includes('timeout') || errorMessage.includes('timed out')) {
        alert(t('logs.error.download_timeout', 'Download timed out for {name}. Please try again.', { name: logName }));
      } else {
        alert(t('logs.error.download_error', 'Error downloading {name}: {msg}', { name: logName, msg: errorMessage }));
      }
    } finally {
      setDownloading('');
    }
  };

  return (
  <div className="theme-card p-4 shadow-lg">
      <h3 className="text-lg font-semibold mb-2 text-blue-300">{t('logs.title', 'Download Logs')}</h3>
      <div className="flex flex-row items-start gap-6">
        <div className="flex flex-col min-w-[120px]">
          <span className="text-gray-200 font-medium mb-1" id="log-type-label">{t('logs.type', 'Type:')}</span>
          <select
            className="theme-surface-2 rounded px-2 py-1"
            value={selectedType}
            onChange={e => setSelectedType(e.target.value as LogType)}
            aria-labelledby="log-type-label"
            title={t('logs.select_type', 'Select log type')}
            aria-label={t('logs.select_type', 'Select log type')}
            disabled={loading}
          >
            {logTypes.map(type => (
              <option key={type.key} value={type.key}>{t(`logs.types.${type.key}`, type.label)}</option>
            ))}
          </select>
        </div>
        <div className="flex-1 overflow-x-auto">
          {error && (
            <div className="mb-3 p-2 bg-red-900/20 border border-red-700 rounded text-red-400 text-sm">
              {error}
              <button 
                onClick={fetchLogs}
                className="ml-2 text-blue-400 hover:text-blue-300 underline"
              >
                {t('common.retry', 'Retry')}
              </button>
            </div>
          )}
          <div className="max-h-64 overflow-y-auto border border-gray-700 rounded">
            <table className="min-w-full text-left text-sm text-gray-200">
            <thead className="theme-surface-2 sticky top-0 z-10">
              <tr>
                <th className="px-3 py-2 font-semibold">{t('logs.table.file', 'File Name')}</th>
                <th className="px-3 py-2 font-semibold">{t('logs.table.size', 'Size')}</th>
                <th className="px-3 py-2 font-semibold">{t('logs.table.modified', 'Modified')}</th>
              </tr>
            </thead>
            <tbody>
              {loading ? (
                <tr>
                  <td colSpan={3} className="px-3 py-2 text-blue-400">{t('logs.loading', 'Loading logs...')}</td>
                </tr>
              ) : logs.length > 0 ? logs.map(log => (
                <tr
                  key={log.name}
                  className="hover:bg-blue-900 cursor-pointer transition-colors"
                  onDoubleClick={() => handleDownload(log.name)}
                  title={t('logs.double_click', 'Double-click to download')}
                >
                  <td className="px-3 py-2 whitespace-nowrap">
                    {log.name}
                    {downloading === log.name && (
                      <span className="ml-2 text-blue-400 text-xs">{t('logs.downloading', 'Downloading...')}</span>
                    )}
                  </td>
                  <td className="px-3 py-2 whitespace-nowrap">
                    {selectedType === 'arc' ? t('logs.archive', 'Archive') : `${(log.size / 1024).toFixed(1)} KB`}
                  </td>
                  <td className="px-3 py-2 whitespace-nowrap">{log.modified ? require('../../utils/format').formatDateTime(log.modified) : ''}</td>
                </tr>
              )) : (
                <tr>
                  <td colSpan={3} className="px-3 py-2 text-gray-400">
                    {error ? t('logs.none_error', 'No logs found due to error') : t('logs.none', 'No logs found')}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
          </div>
        </div>
      </div>
    </div>
  );
};

export default LogDownloadList; 