import React, { useEffect, useState } from 'react';
import { useAppStore, ObsStatusInfo } from '../stores';

// Static dummy data
const dummyAthletes = [
  { flag: '🇺🇸', name: 'Benjamin Smith' },
  { flag: '🇯🇵', name: 'Kei Tanaka' }
];

const dummyEvents = [
  { round: 'R1', time: '02.00.343', event: 'Punch', color: 'red' },
  { round: 'R2', time: '02.10.343', event: 'Head Kick', color: 'blue' },
  { round: 'R3', time: '02.20.343', event: 'Referee', color: 'yellow' },
  { round: 'R1', time: '02.30.343', event: 'Kick', color: 'green' },
  { round: 'R2', time: '02.40.343', event: 'Punch', color: 'red' }
];

const MatchInfoSection: React.FC = () => {
  return (
    <div className="mb-4">
      {/* Athletes */}
      <div className="space-y-1 mb-2">
        {dummyAthletes.map((athlete, index) => (
          <div key={index} className="flex items-center space-x-2">
            <span className="text-lg">{athlete.flag}</span>
            <span className="text-sm text-gray-300">{athlete.name}</span>
          </div>
        ))}
      </div>
      
      {/* Category and Stage */}
      <div className="space-y-1 mb-3">
        <div className="text-sm text-gray-400">M-75kg</div>
        <div className="text-sm text-gray-400">Semi-final</div>
      </div>
      
      {/* Match Number */}
      <div className="text-right">
        <span className="text-3xl font-bold text-red-500">1254</span>
      </div>
    </div>
  );
};

const EventTable: React.FC = () => {
  return (
    <div className="mb-4 relative">
      {/* Header */}
      <div className="grid grid-cols-12 gap-2 text-xs text-gray-400 mb-2 border-b border-gray-700 pb-1">
        <div className="col-span-2">RND</div>
        <div className="col-span-4">TIME</div>
        <div className="col-span-6">EVENT</div>
      </div>
      
      {/* Event Rows */}
      <div className="space-y-1 max-h-32 overflow-y-auto">
        {dummyEvents.map((event, index) => (
          <div key={index} className="grid grid-cols-12 gap-2 text-xs">
            <div className="col-span-2 text-gray-300">{event.round}</div>
            <div className="col-span-4 text-gray-300">{event.time}</div>
            <div className="col-span-6 flex items-center space-x-1">
              <span 
                className={`w-2 h-2 rounded-full ${
                  event.color === 'red' ? 'bg-red-500' :
                  event.color === 'blue' ? 'bg-blue-500' :
                  event.color === 'yellow' ? 'bg-yellow-500' :
                  'bg-green-500'
                }`}
              ></span>
              <span className="text-gray-300">{event.event}</span>
            </div>
          </div>
        ))}
      </div>
      
      {/* Go to Top Arrow */}
      <div className="absolute bottom-0 right-0">
        <button className="text-gray-400 hover:text-white text-xs">
          ↑
        </button>
      </div>
    </div>
  );
};

const StatusBar: React.FC = () => {
  const { obsStatus, obsConnections, updateObsStatus } = useAppStore();
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());

  // Simulate OBS status updates (in real implementation, this would come from backend)
  useEffect(() => {
    const interval = setInterval(() => {
      setLastUpdate(new Date());
    }, 3000); // Update every 3 seconds

    return () => clearInterval(interval);
  }, []);

  // Test function to simulate different OBS statuses
  const testObsStatus = (scenario: 'idle' | 'recording' | 'streaming' | 'both' | 'high-cpu') => {
    let testStatus: ObsStatusInfo = {
      is_recording: false,
      is_streaming: false,
      cpu_usage: 0,
    };

    switch (scenario) {
      case 'idle':
        testStatus = {
          is_recording: false,
          is_streaming: false,
          cpu_usage: 15,
        };
        break;
      case 'recording':
        testStatus = {
          is_recording: true,
          is_streaming: false,
          cpu_usage: 45,
          recording_connection: 'OBS_SINGLE',
        };
        break;
      case 'streaming':
        testStatus = {
          is_recording: false,
          is_streaming: true,
          cpu_usage: 35,
          streaming_connection: 'OBS_SINGLE',
        };
        break;
      case 'both':
        testStatus = {
          is_recording: true,
          is_streaming: true,
          cpu_usage: 65,
          recording_connection: 'OBS_REC',
          streaming_connection: 'OBS_STR',
        };
        break;
      case 'high-cpu':
        testStatus = {
          is_recording: true,
          is_streaming: false,
          cpu_usage: 85,
          recording_connection: 'OBS_SINGLE',
        };
        break;
    }

    updateObsStatus(testStatus);
  };

  // Get CPU color based on usage
  const getCpuColor = (cpuUsage: number) => {
    if (cpuUsage <= 30) return 'bg-green-500';
    if (cpuUsage <= 70) return 'bg-yellow-500';
    return 'bg-red-500';
  };

  // Get recording status color
  const getRecordingColor = (isRecording: boolean) => {
    return isRecording ? 'bg-red-500' : 'bg-green-500';
  };

  // Get streaming status color
  const getStreamingColor = (isStreaming: boolean) => {
    return isStreaming ? 'bg-red-500' : 'bg-green-500';
  };

  // Default values if no OBS status available
  const isRecording = obsStatus?.is_recording ?? false;
  const isStreaming = obsStatus?.is_streaming ?? false;
  const cpuUsage = obsStatus?.cpu_usage ?? 0;
  const hasObsConnections = obsConnections.length > 0;

  return (
    <div className="space-y-2">
      {/* Test Controls (remove in production) */}
      <div className="flex flex-wrap gap-1 text-xs">
        <button 
          onClick={() => testObsStatus('idle')}
          className="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded"
        >
          Idle
        </button>
        <button 
          onClick={() => testObsStatus('recording')}
          className="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded"
        >
          REC
        </button>
        <button 
          onClick={() => testObsStatus('streaming')}
          className="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded"
        >
          STR
        </button>
        <button 
          onClick={() => testObsStatus('both')}
          className="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded"
        >
          Both
        </button>
        <button 
          onClick={() => testObsStatus('high-cpu')}
          className="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded"
        >
          High CPU
        </button>
      </div>

      {/* Status Bar */}
      <div className="flex items-center justify-between text-xs text-gray-400 border-t border-gray-700 pt-2">
        {/* OBS Recording Status */}
        <div className="flex items-center space-x-1">
          <span 
            className={`w-2 h-2 rounded-full ${getRecordingColor(isRecording)} ${
              !hasObsConnections ? 'opacity-50' : ''
            }`}
            title={hasObsConnections ? `Recording: ${isRecording ? 'ON' : 'OFF'}` : 'No OBS connection'}
          ></span>
          <span className={!hasObsConnections ? 'opacity-50' : ''}>REC</span>
        </div>
        
        {/* OBS Streaming Status */}
        <div className="flex items-center space-x-1">
          <span 
            className={`w-2 h-2 rounded-full ${getStreamingColor(isStreaming)} ${
              !hasObsConnections ? 'opacity-50' : ''
            }`}
            title={hasObsConnections ? `Streaming: ${isStreaming ? 'ON' : 'OFF'}` : 'No OBS connection'}
          ></span>
          <span className={!hasObsConnections ? 'opacity-50' : ''}>STR</span>
        </div>
        
        {/* OBS CPU Usage */}
        <div className="flex items-center space-x-1">
          <span 
            className={`w-2 h-2 rounded-full ${getCpuColor(cpuUsage)} ${
              !hasObsConnections ? 'opacity-50' : ''
            }`}
            title={hasObsConnections ? `CPU: ${cpuUsage.toFixed(1)}%` : 'No OBS connection'}
          ></span>
          <span className={!hasObsConnections ? 'opacity-50' : ''}>
            CPU {hasObsConnections ? `${cpuUsage.toFixed(0)}%` : '--'}
          </span>
        </div>
      </div>
    </div>
  );
};

const SidebarSecondColumn: React.FC = () => {
  return (
    <div className="w-64 bg-gray-900 p-4 text-white">
      <MatchInfoSection />
      <EventTable />
      <StatusBar />
    </div>
  );
};

export default SidebarSecondColumn; 