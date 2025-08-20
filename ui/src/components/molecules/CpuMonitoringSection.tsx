import React, { useState, useEffect } from 'react';
import Button from '../atoms/Button';
import StatusDot from '../atoms/StatusDot';
import { Icon } from '../atoms/Icon';
import { useI18n } from '../../i18n/index';

// Use the same safeInvoke function as the environment hook
const safeInvoke = async (command: string, args?: any) => {
  try {
    // Check if the global Tauri object is available
    if (typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.core) {
    
      // In Tauri v2, invoke is available through the core module
      return await window.__TAURI__.core.invoke(command, args);
    }
    
    throw new Error('Tauri v2 core module not available - ensure app is running in desktop mode');
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
  const { t } = useI18n();
  const [processData, setProcessData] = useState<CpuProcessData[]>([]);
  const [systemData, setSystemData] = useState<SystemCpuData | null>(null);
  const [isMonitoring, setIsMonitoring] = useState<boolean>(false); // Start disabled
  const [refreshMs, setRefreshMs] = useState<number>(2000);
  const [topN, setTopN] = useState<number>(20);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());

  // Check monitoring status on mount
  useEffect(() => {
    const checkMonitoringStatus = async () => {
      try {
        const status = await safeInvoke('cpu_get_monitoring_status');
    
        setIsMonitoring(status);
      } catch (error) {
        console.error('❌ [CPU_MONITOR] Failed to get monitoring status:', error);
        setIsMonitoring(false);
      }
    };
    
    checkMonitoringStatus();
  }, []);

  // Fetch CPU data
  const fetchCpuData = async () => {
    try {
      // Get process data
      const processResult = await safeInvoke('cpu_get_process_data');
      if (processResult && typeof processResult === 'object' && 'success' in processResult && processResult.success) {
        if ('processes' in processResult && Array.isArray(processResult.processes)) {
          setProcessData(processResult.processes as CpuProcessData[]);
        }
      }

      // Get system data
      const systemResult = await safeInvoke('cpu_get_system_data');
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
    try {
      if (isMonitoring) {
        // Disable monitoring
        await safeInvoke('cpu_disable_monitoring');
        setIsMonitoring(false);
        // Clear frontend data when stopping
        setProcessData([]);
        setSystemData(null);
        setLastUpdate(new Date());
      } else {
        // Enable monitoring
        await safeInvoke('cpu_enable_monitoring');
        setIsMonitoring(true);
      }
    } catch (error) {
      console.error('Failed to toggle monitoring:', error);
    }
  };

  // Update data when monitoring is active, with configurable interval
  useEffect(() => {
    if (!isMonitoring) {
      return;
    }

    const updateInterval = refreshMs; // configurable
    fetchCpuData(); // Initial fetch
    const interval = setInterval(fetchCpuData, updateInterval);

    return () => {
      clearInterval(interval);
    };
  }, [isMonitoring, refreshMs]);

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
    <div className={`bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg p-4 border border-gray-600/30 shadow-lg ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center space-x-2">
          <svg width="20" height="20" fill="none" viewBox="0 0 24 24" stroke="currentColor" className="w-5 h-5 text-blue-400">
            <rect x="2" y="3" width="20" height="14" rx="2" ry="2" stroke="currentColor" strokeWidth="2"/>
            <line x1="8" y1="21" x2="16" y2="21" stroke="currentColor" strokeWidth="2"/>
            <line x1="12" y1="17" x2="12" y2="21" stroke="currentColor" strokeWidth="2"/>
          </svg>
          <h3 className="text-lg font-semibold text-white">{t('cpu.title', 'CPU Monitoring')}</h3>
          <StatusDot 
            color={isMonitoring ? 'bg-green-400' : 'bg-gray-400'} 
            className="ml-2" 
          />
        </div>
        <div className="flex items-center space-x-2">
          <span className="text-sm text-gray-400">
            {t('cpu.last_update', 'Last update: {time}', { time: lastUpdate.toLocaleTimeString() })}
          </span>
          <Button
            variant={isMonitoring ? 'secondary' : 'primary'}
            size="sm"
            onClick={toggleMonitoring}
          >
            {isMonitoring ? t('cpu.stop_monitoring', 'Stop Monitoring') : t('cpu.start_monitoring', 'Start Monitoring')}
          </Button>
        </div>
      </div>

      {/* Controls */}
      <div className="flex items-center gap-3 mb-4">
        <span className="text-sm text-gray-300">{t('cpu.refresh', 'Refresh')}</span>
        <select className="theme-surface-2 px-2 py-1" value={refreshMs} onChange={(e)=>setRefreshMs(parseInt(e.target.value)||2000)}
          aria-label={t('cpu.refresh', 'Refresh')} title={t('cpu.refresh', 'Refresh')}>
          <option value={2000}>2s</option>
          <option value={5000}>5s</option>
          <option value={10000}>10s</option>
        </select>
        <span className="text-sm text-gray-300 ml-4">{t('cpu.top', 'Top')}</span>
        <select className="theme-surface-2 px-2 py-1" value={topN} onChange={(e)=>setTopN(parseInt(e.target.value)||20)}
          aria-label={t('cpu.top', 'Top')} title={t('cpu.top', 'Top')}>
          <option value={5}>5</option>
          <option value={10}>10</option>
          <option value={20}>20</option>
          <option value={50}>50</option>
        </select>
      </div>

      {/* System CPU Overview */}
      {systemData && (
        <div className="mb-6 p-3 bg-gray-700 rounded-lg">
          <h4 className="text-md font-medium text-white mb-2">{t('cpu.system_cpu', 'System CPU')}</h4>
          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-2">
              <span className="text-sm text-gray-300">{t('cpu.total', 'Total:')}</span>
              <span 
                className={`text-lg font-bold ${getCpuColor(systemData.total_cpu_percent || 0) === 'red' ? 'text-red-400' : 
                  getCpuColor(systemData.total_cpu_percent || 0) === 'yellow' ? 'text-yellow-400' : 'text-green-400'}`}
              >
                {(systemData.total_cpu_percent || 0).toFixed(1)}%
              </span>
            </div>
            <div className="flex items-center space-x-2">
              <span className="text-sm text-gray-300">{t('cpu.cores', 'Cores:')}</span>
              <span className="text-sm text-gray-300">
                {t('cpu.cores_detected', '{n} detected', { n: systemData.cores?.length || 0 })}
              </span>
            </div>
            <div className="flex items-center space-x-2">
              <span className="text-sm text-gray-300">{t('cpu.updated', 'Updated:')}</span>
              <span className="text-sm text-gray-400">
                {formatTimestamp(systemData.last_update)}
              </span>
            </div>
          </div>
        </div>
      )}



      {/* Process List */}
      <div className="space-y-2">
        <h4 className="text-md font-medium text-white mb-2">{t('cpu.all_processes', 'All Running Processes ({n})', { n: processData.length })}</h4>
        {processData.length > 0 ? (
          <div className="space-y-2 max-h-64 overflow-y-auto">
            {processData
              .sort((a, b) => (b.cpu_percent || 0) - (a.cpu_percent || 0)) // Sort by CPU usage (highest first)
              .slice(0, topN)
              .map((process, index) => (
              <div key={index} className="flex items-center justify-between p-2 bg-gray-700 rounded">
                <div className="flex items-center space-x-3">
                  <span className="text-sm font-medium text-white min-w-[120px] truncate">
                    {process.process_name}
                  </span>
                  <div className="flex items-center space-x-4">
                    <div className="flex items-center space-x-1">
                      <span className="text-xs text-gray-400">{t('cpu.cpu', 'CPU:')}</span>
                      <span 
                        className={`text-sm font-medium ${getCpuColor(process.cpu_percent || 0) === 'red' ? 'text-red-400' : 
                          getCpuColor(process.cpu_percent || 0) === 'yellow' ? 'text-yellow-400' : 'text-green-400'}`}
                      >
                        {(process.cpu_percent || 0).toFixed(1)}%
                      </span>
                    </div>
                    <div className="flex items-center space-x-1">
                      <span className="text-xs text-gray-400">{t('cpu.memory', 'Memory:')}</span>
                      <span className="text-sm text-gray-300">
                        {(process.memory_mb || 0).toFixed(1)} {t('cpu.mb', 'MB')}
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
            {isMonitoring ? t('cpu.no_processes', 'No processes detected') : t('cpu.disabled', 'Monitoring is disabled')}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="mt-4 pt-3 border-t border-gray-600">
        <div className="flex items-center justify-between text-xs text-gray-400">
          <span>{t('cpu.powered_by', 'CPU monitoring powered by system commands (wmic/ps)')}</span>
          <span>{t('cpu.update_interval', 'Update interval: {s}', { s: process.env.NODE_ENV === 'production' ? '2 seconds' : '1 second' })}</span>
        </div>
      </div>
    </div>
  );
}; 