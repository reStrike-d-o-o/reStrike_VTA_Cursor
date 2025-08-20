/**
 * StatusbarDock
 * - Shows REC/STR/CPU indicators tied to obws connections (OBS_REC/OBS_STR)
 * - Reflects Connected/Connecting/Error states and CPU usage coloring
 */
import React from 'react';
import { StatusDot } from '../atoms/StatusDot';
import { usePssMatchStore } from '../../stores';
import { useAppStore } from '../../stores';

const StatusbarDock: React.FC = () => {
  // Get PSS data status
  const isPssDataLoaded = usePssMatchStore((state) => state.matchData.isLoaded);
  
  // Get OBS status from app store (for CPU usage)
  const obsStatus = useAppStore((state) => state.obsStatus);
  
  // Get connections directly from the app store (same store as WebSocketManager)
  const obsConnections = useAppStore((state) => state.obsConnections);
  
  // Find OBS_REC and OBS_STR connections
  const recConnection = obsConnections.find(c => c.name === 'OBS_REC');
  const strConnection = obsConnections.find(c => c.name === 'OBS_STR');
  
  // Helper function to determine if connection is active
  const isConnectionActive = (status: string) => {
    return status === 'Connected' || status === 'Authenticated';
  };
  
  // Helper function to determine if connection is connecting
  const isConnectionConnecting = (status: string) => {
    return status === 'Connecting' || status === 'Authenticating';
  };
  
  return (
    <div className="w-full h-[4.5rem] bg-gradient-to-r from-gray-800/80 to-gray-900/90 backdrop-blur-sm flex items-center justify-center text-xs text-gray-300 px-8 border-t border-gray-600/30">
      <div className="flex items-center space-x-4">
        {/* REC Status - Shows OBS_REC connection status */}
        <div className={`flex items-center space-x-2 px-3 py-2 rounded-lg border backdrop-blur-sm ${
          recConnection && isConnectionActive(recConnection.status)
            ? 'bg-red-500/10 border-red-500/20' 
            : recConnection && isConnectionConnecting(recConnection.status)
            ? 'bg-yellow-500/10 border-yellow-500/20'
            : recConnection && recConnection.status === 'Error'
            ? 'bg-red-500/10 border-red-500/20'
            : 'bg-gray-700/10 border-gray-600/20'
        }`}>
          <div className={`w-2 h-2 rounded-full ${
            recConnection && isConnectionActive(recConnection.status)
              ? 'bg-red-500' 
              : recConnection && isConnectionConnecting(recConnection.status)
              ? 'bg-yellow-500'
              : 'bg-gray-500'
          }`} />
          <span>REC</span>
        </div>

        {/* STR Status - Shows OBS_STR connection status */}
        <div className={`flex items-center space-x-2 px-3 py-2 rounded-lg border backdrop-blur-sm ${
          strConnection && isConnectionActive(strConnection.status)
            ? 'bg-green-500/10 border-green-500/20' 
            : strConnection && isConnectionConnecting(strConnection.status)
            ? 'bg-yellow-500/10 border-yellow-500/20'
            : strConnection && strConnection.status === 'Error'
            ? 'bg-red-500/10 border-red-500/20'
            : 'bg-gray-700/10 border-gray-600/20'
        }`}>
          <div className={`w-2 h-2 rounded-full ${
            strConnection && isConnectionActive(strConnection.status)
              ? 'bg-green-500' 
              : strConnection && isConnectionConnecting(strConnection.status)
              ? 'bg-yellow-500'
              : 'bg-gray-500'
          }`} />
          <span>STR</span>
        </div>

        {/* CPU Status - Reflects OBS load from app store */}
        <div className="flex items-center space-x-2 px-3 py-2 rounded-lg border backdrop-blur-sm bg-gray-700/10 border-gray-600/20">
          <div className={`w-2 h-2 rounded-full ${
            (obsStatus?.cpu_usage ?? 0) < 50
              ? 'bg-green-500'
              : (obsStatus?.cpu_usage ?? 0) < 75
              ? 'bg-yellow-500'
              : 'bg-red-500'
          }`} />
          <span>CPU {obsStatus?.cpu_usage?.toFixed?.(0) ?? '--'}%</span>
        </div>
      </div>
    </div>
  );
};

export default StatusbarDock; 