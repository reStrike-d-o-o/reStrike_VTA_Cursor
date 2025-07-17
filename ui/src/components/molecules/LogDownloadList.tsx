import React, { useState } from 'react';

const logTypes = [
  { key: 'pss', label: 'PSS' },
  { key: 'obs', label: 'OBS' },
  { key: 'udp', label: 'UDP' },
];

type LogType = 'pss' | 'obs' | 'udp';

const dummyLogs: Record<LogType, { name: string; size: string }[]> = {
  pss: [
    { name: 'pss-log-2024-06-01.txt', size: '12 KB' },
    { name: 'pss-log-2024-06-02.txt', size: '8 KB' },
  ],
  obs: [
    { name: 'obs-log-2024-06-01.txt', size: '5 KB' },
  ],
  udp: [
    { name: 'udp-log-2024-06-01.txt', size: '20 KB' },
    { name: 'udp-log-2024-06-02.txt', size: '15 KB' },
  ],
};

const LogDownloadList: React.FC = () => {
  const [selectedType, setSelectedType] = useState<LogType>('pss');
  const logs = dummyLogs[selectedType];

  const handleDownload = (logName: string) => {
    alert(`Download ${logName}`);
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
          >
            {logTypes.map(type => (
              <option key={type.key} value={type.key}>{type.label}</option>
            ))}
          </select>
        </div>
        <div className="flex-1 overflow-x-auto">
          <table className="min-w-full text-left text-sm text-gray-200 border border-gray-700 rounded">
            <thead className="bg-[#101820]">
              <tr>
                <th className="px-3 py-2 font-semibold">File Name</th>
                <th className="px-3 py-2 font-semibold">Size</th>
              </tr>
            </thead>
            <tbody>
              {logs.length > 0 ? logs.map(log => (
                <tr
                  key={log.name}
                  className="hover:bg-blue-900 cursor-pointer transition-colors"
                  onDoubleClick={() => handleDownload(log.name)}
                  title="Double-click to download"
                >
                  <td className="px-3 py-2 whitespace-nowrap">{log.name}</td>
                  <td className="px-3 py-2 whitespace-nowrap">{log.size}</td>
                </tr>
              )) : (
                <tr>
                  <td colSpan={2} className="px-3 py-2 text-gray-400">No logs found.</td>
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