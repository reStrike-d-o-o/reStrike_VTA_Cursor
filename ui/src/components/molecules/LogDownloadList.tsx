import React, { useState, useEffect } from 'react';
import { diagLogsCommands } from '../../utils/tauriCommands';

const logTypes = [
  { key: 'pss', label: 'PSS' },
  { key: 'obs', label: 'OBS' },
  { key: 'udp', label: 'UDP' },
];

type LogType = 'pss' | 'obs' | 'udp';

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

  const fetchLogs = async () => {
    setLoading(true);
    setError('');
    try {
      const res = await diagLogsCommands.listLogFiles(selectedType);
      if (res.success && Array.isArray(res.data)) {
        setLogs(res.data);
      } else {
        setLogs([]);
        setError(res.error || 'Failed to fetch log files');
      }
    } catch (err) {
      setLogs([]);
      setError(`Error fetching logs: ${err}`);
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
        alert(`Failed to download ${logName}: ${res.error || 'Unknown error'}`);
      }
    } catch (err) {
      alert(`Error downloading ${logName}: ${err}`);
    } finally {
      setDownloading('');
    }
  };

  return (
    <div className="bg-[#181F26] rounded-lg p-4 mb-6 border border-gray-700 shadow">
      <h3 className="text-lg font-semibold mb-2 text-blue-300">Download Logs</h3>
      <div className="flex flex-row items-start gap-6">
        <div className="flex flex-col min-w-[120px]">
          <span className="text-gray-200 font-medium mb-1" id="log-type-label">Type:</span>
          <select
            className="bg-[#101820] border border-gray-700 rounded px-2 py-1 text-gray-100"
            value={selectedType}
            onChange={e => setSelectedType(e.target.value as LogType)}
            aria-labelledby="log-type-label"
            title="Select log type"
            aria-label="Select log type"
            disabled={loading}
          >
            {logTypes.map(type => (
              <option key={type.key} value={type.key}>{type.label}</option>
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
                Retry
              </button>
            </div>
          )}
          <table className="min-w-full text-left text-sm text-gray-200 border border-gray-700 rounded">
            <thead className="bg-[#101820]">
              <tr>
                <th className="px-3 py-2 font-semibold">File Name</th>
                <th className="px-3 py-2 font-semibold">Size</th>
                <th className="px-3 py-2 font-semibold">Modified</th>
              </tr>
            </thead>
            <tbody>
              {loading ? (
                <tr>
                  <td colSpan={3} className="px-3 py-2 text-blue-400">Loading logs...</td>
                </tr>
              ) : logs.length > 0 ? logs.map(log => (
                <tr
                  key={log.name}
                  className="hover:bg-blue-900 cursor-pointer transition-colors"
                  onDoubleClick={() => handleDownload(log.name)}
                  title="Double-click to download"
                >
                  <td className="px-3 py-2 whitespace-nowrap">
                    {log.name}
                    {downloading === log.name && (
                      <span className="ml-2 text-blue-400 text-xs">Downloading...</span>
                    )}
                  </td>
                  <td className="px-3 py-2 whitespace-nowrap">{(log.size / 1024).toFixed(1)} KB</td>
                  <td className="px-3 py-2 whitespace-nowrap">{log.modified ? new Date(log.modified).toLocaleString() : ''}</td>
                </tr>
              )) : (
                <tr>
                  <td colSpan={3} className="px-3 py-2 text-gray-400">
                    {error ? 'No logs found due to error' : 'No logs found'}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};

export default LogDownloadList; 