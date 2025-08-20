/**
 * StatusBar (legacy)
 * - Simple REC/STR/CPU status row using legacy ObsStatusInfo fields
 * - Production status is derived from obws status via DockBar/StatusbarDock
 */
import React, { useEffect, useState } from 'react';
import { useAppStore, ObsStatusInfo } from '../../stores';
import Button from '../atoms/Button';

const StatusBar: React.FC = () => {
  const { obsStatus, obsConnections, updateObsStatus } = useAppStore();
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());

  useEffect(() => {
    const interval = setInterval(() => {
      setLastUpdate(new Date());
    }, 3000);
    return () => clearInterval(interval);
  }, []);



  const getCpuColor = (cpuUsage: number) => {
    if (cpuUsage <= 30) return 'bg-green-500';
    if (cpuUsage <= 70) return 'bg-yellow-500';
    return 'bg-red-500';
  };
  const getRecordingColor = (isRecording: boolean) => isRecording ? 'bg-red-500' : 'bg-green-500';
  const getStreamingColor = (isStreaming: boolean) => isStreaming ? 'bg-red-500' : 'bg-green-500';

  const isRecording = obsStatus?.is_recording ?? false;
  const isStreaming = obsStatus?.is_streaming ?? false;
  const cpuUsage = obsStatus?.cpu_usage ?? 0;
  const hasObsConnections = obsConnections.length > 0;

  return (
    <div className="space-y-2">
      {/* Status Bar */}
      <div className="flex items-center justify-between text-xs text-gray-400 border-t border-gray-700 pt-2">
        <div className="flex items-center space-x-1">
          <span className={`w-2 h-2 rounded-full ${getRecordingColor(isRecording)} ${!hasObsConnections ? 'opacity-50' : ''}`} title={hasObsConnections ? `Recording: ${isRecording ? 'ON' : 'OFF'}` : 'No OBS connection'}></span>
          <span className={!hasObsConnections ? 'opacity-50' : ''}>REC</span>
        </div>
        <div className="flex items-center space-x-1">
          <span className={`w-2 h-2 rounded-full ${getStreamingColor(isStreaming)} ${!hasObsConnections ? 'opacity-50' : ''}`} title={hasObsConnections ? `Streaming: ${isStreaming ? 'ON' : 'OFF'}` : 'No OBS connection'}></span>
          <span className={!hasObsConnections ? 'opacity-50' : ''}>STR</span>
        </div>
        <div className="flex items-center space-x-1">
          <span className={`w-2 h-2 rounded-full ${getCpuColor(cpuUsage)} ${!hasObsConnections ? 'opacity-50' : ''}`} title={hasObsConnections ? `CPU: ${cpuUsage.toFixed(1)}%` : 'No OBS connection'}></span>
          <span className={!hasObsConnections ? 'opacity-50' : ''}>CPU {hasObsConnections ? `${cpuUsage.toFixed(0)}%` : '--'}</span>
        </div>
      </div>
    </div>
  );
};

export default StatusBar; 