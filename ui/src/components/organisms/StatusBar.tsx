import React, { useEffect, useState } from 'react';
import { useAppStore, ObsStatusInfo } from '../../stores';

const StatusBar: React.FC = () => {
  const { obsStatus, obsConnections, updateObsStatus } = useAppStore();
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());

  useEffect(() => {
    const interval = setInterval(() => {
      setLastUpdate(new Date());
    }, 3000);
    return () => clearInterval(interval);
  }, []);

  const testObsStatus = (scenario: 'idle' | 'recording' | 'streaming' | 'both' | 'high-cpu') => {
    let testStatus: ObsStatusInfo = {
      is_recording: false,
      is_streaming: false,
      cpu_usage: 0,
    };
    switch (scenario) {
      case 'idle':
        testStatus = { is_recording: false, is_streaming: false, cpu_usage: 15 };
        break;
      case 'recording':
        testStatus = { is_recording: true, is_streaming: false, cpu_usage: 45, recording_connection: 'OBS_SINGLE' };
        break;
      case 'streaming':
        testStatus = { is_recording: false, is_streaming: true, cpu_usage: 35, streaming_connection: 'OBS_SINGLE' };
        break;
      case 'both':
        testStatus = { is_recording: true, is_streaming: true, cpu_usage: 65, recording_connection: 'OBS_REC', streaming_connection: 'OBS_STR' };
        break;
      case 'high-cpu':
        testStatus = { is_recording: true, is_streaming: false, cpu_usage: 85, recording_connection: 'OBS_SINGLE' };
        break;
    }
    updateObsStatus(testStatus);
  };

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
      {/* Test Controls (remove in production) */}
      <div className="flex flex-wrap gap-1 text-xs">
        <button onClick={() => testObsStatus('idle')} className="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded">Idle</button>
        <button onClick={() => testObsStatus('recording')} className="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded">REC</button>
        <button onClick={() => testObsStatus('streaming')} className="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded">STR</button>
        <button onClick={() => testObsStatus('both')} className="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded">Both</button>
        <button onClick={() => testObsStatus('high-cpu')} className="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded">High CPU</button>
      </div>
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