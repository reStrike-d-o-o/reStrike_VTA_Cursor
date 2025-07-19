import React, { useState, useEffect } from 'react';
import Button from '../atoms/Button';
import StatusDot from '../atoms/StatusDot';
import { Icon } from '../atoms/Icon';

// Use the proper Tauri v2 invoke function
const invoke = async (command: string, args?: any) => {
  try {
    // Use the global window.__TAURI__.core.invoke for Tauri v2
    if (window.__TAURI__?.core?.invoke) {
      return await window.__TAURI__.core.invoke(command, args);
    }
    throw new Error('Tauri not available');
  } catch (error) {
    console.error('Tauri invoke failed:', error);
    throw error;
  }
};

interface CpuProcessData {
  process_name: string;
  cpu_percent: number;
  memory_mb: number;
  last_update: string;
}

interface SystemCpuData {
  total_cpu_percent: number;
  cores: number[];
  last_update: string;
}

interface CpuMonitoringSectionProps {
  className?: string;
}

export const CpuMonitoringSection: React.FC<CpuMonitoringSectionProps> = ({ className = '' }) => {
  const [processData, setProcessData] = useState<CpuProcessData[]>([]);
  const [systemData, setSystemData] = useState<SystemCpuData | null>(null);
  const [isMonitoring, setIsMonitoring] = useState<boolean>(true);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());

  // Fetch CPU data
  const fetchCpuData = async () => {
    try {
      // Get process data
      const processResult = await invoke('cpu_get_process_data');
      console.log('Process result:', processResult);
      if (processResult && typeof processResult === 'object' && 'success' in processResult && processResult.success) {
        if ('processes' in processResult && Array.isArray(processResult.processes)) {
          setProcessData(processResult.processes as CpuProcessData[]);
        }
      }

      // Get system data
      const systemResult = await invoke('cpu_get_system_data');
      console.log('System result:', systemResult);
      if (systemResult && typeof systemResult === 'object' && 'success' in systemResult && systemResult.success) {
        if ('system' in systemResult && systemResult.system) {
          setSystemData(systemResult.system as SystemCpuData);
        }
      }

      setLastUpdate(new Date());
    } catch (error) {
      console.error('Failed to fetch CPU data:', error);
    }
  };

  // Toggle monitoring
  const toggleMonitoring = async () => {
    setIsMonitoring(!isMonitoring);
    // TODO: Implement config update to enable/disable monitoring
  };

  // Update data every 2 seconds when monitoring is active
  useEffect(() => {
    if (!isMonitoring) return;

    fetchCpuData(); // Initial fetch
    const interval = setInterval(fetchCpuData, 2000);

    return () => clearInterval(interval);
  }, [isMonitoring]);

  // Get CPU usage color based on percentage
  const getCpuColor = (percentage: number): string => {
    if (percentage < 30) return 'green';
    if (percentage < 70) return 'yellow';
    return 'red';
  };

  // Format timestamp
  const formatTimestamp = (timestamp: string): string => {
    try {
      return new Date(timestamp).toLocaleTimeString();
    } catch {
      return 'Unknown';
    }
  };

  return (
    <div className={`bg-gray-800 rounded-lg p-4 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center space-x-2">
          <Icon name="🖥️" className="w-5 h-5 text-blue-400" />
          <h3 className="text-lg font-semibold text-white">CPU Monitoring</h3>
          <StatusDot 
            color={isMonitoring ? 'bg-green-400' : 'bg-gray-400'} 
            className="ml-2" 
          />
        </div>
        <div className="flex items-center space-x-2">
          <span className="text-sm text-gray-400">
            Last update: {lastUpdate.toLocaleTimeString()}
          </span>
          <Button
            variant={isMonitoring ? 'secondary' : 'primary'}
            size="sm"
            onClick={toggleMonitoring}
          >
            {isMonitoring ? 'Stop' : 'Start'} Monitoring
          </Button>
        </div>
      </div>

      {/* System CPU Overview */}
      {systemData && (
        <div className="mb-6 p-3 bg-gray-700 rounded-lg">
          <h4 className="text-md font-medium text-white mb-2">System CPU</h4>
          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-2">
              <span className="text-sm text-gray-300">Total:</span>
              <span 
                className={`text-lg font-bold ${getCpuColor(systemData.total_cpu_percent || 0) === 'red' ? 'text-red-400' : 
                  getCpuColor(systemData.total_cpu_percent || 0) === 'yellow' ? 'text-yellow-400' : 'text-green-400'}`}
              >
                {(systemData.total_cpu_percent || 0).toFixed(1)}%
              </span>
            </div>
            <div className="flex items-center space-x-2">
              <span className="text-sm text-gray-300">Cores:</span>
              <span className="text-sm text-gray-300">
                {systemData.cores?.length || 0} detected
              </span>
            </div>
            <div className="flex items-center space-x-2">
              <span className="text-sm text-gray-300">Updated:</span>
              <span className="text-sm text-gray-400">
                {formatTimestamp(systemData.last_update)}
              </span>
            </div>
          </div>
        </div>
      )}



      {/* Process List */}
      <div className="space-y-2">
        <h4 className="text-md font-medium text-white mb-2">All Running Processes ({processData.length})</h4>
        {processData.length > 0 ? (
          <div className="space-y-2 max-h-64 overflow-y-auto">
            {processData
              .sort((a, b) => (b.cpu_percent || 0) - (a.cpu_percent || 0)) // Sort by CPU usage (highest first)
              .slice(0, 20) // Show top 20 processes
              .map((process, index) => (
              <div key={index} className="flex items-center justify-between p-2 bg-gray-700 rounded">
                <div className="flex items-center space-x-3">
                  <span className="text-sm font-medium text-white min-w-[120px] truncate">
                    {process.process_name}
                  </span>
                  <div className="flex items-center space-x-4">
                    <div className="flex items-center space-x-1">
                      <span className="text-xs text-gray-400">CPU:</span>
                      <span 
                        className={`text-sm font-medium ${getCpuColor(process.cpu_percent || 0) === 'red' ? 'text-red-400' : 
                          getCpuColor(process.cpu_percent || 0) === 'yellow' ? 'text-yellow-400' : 'text-green-400'}`}
                      >
                        {(process.cpu_percent || 0).toFixed(1)}%
                      </span>
                    </div>
                    <div className="flex items-center space-x-1">
                      <span className="text-xs text-gray-400">Memory:</span>
                      <span className="text-sm text-gray-300">
                        {(process.memory_mb || 0).toFixed(1)} MB
                      </span>
                    </div>
                  </div>
                </div>
                <div className="flex items-center space-x-2">
                  <StatusDot 
                    color={(process.cpu_percent || 0) > 70 ? 'bg-red-400' : (process.cpu_percent || 0) > 30 ? 'bg-yellow-400' : 'bg-green-400'} 
                  />
                  <span className="text-xs text-gray-400">
                    {formatTimestamp(process.last_update)}
                  </span>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-center py-4 text-gray-400">
            {isMonitoring ? 'No processes detected' : 'Monitoring is disabled'}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="mt-4 pt-3 border-t border-gray-600">
        <div className="flex items-center justify-between text-xs text-gray-400">
          <span>CPU monitoring powered by system commands (wmic/ps)</span>
          <span>Update interval: 2 seconds</span>
        </div>
      </div>
    </div>
  );
}; 