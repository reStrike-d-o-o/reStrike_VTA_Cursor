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
            : 'bg-gray-500/10 border-gray-500/20'
        }`}>
          <StatusDot 
            color={
              recConnection && isConnectionActive(recConnection.status) ? "bg-red-500" :
              recConnection && isConnectionConnecting(recConnection.status) ? "bg-yellow-500" :
              recConnection && recConnection.status === 'Error' ? "bg-red-500" :
              "bg-gray-500"
            } 
            size="w-3 h-3" 
            className={`shadow-lg ${
              recConnection && isConnectionActive(recConnection.status)
                ? 'shadow-red-500/50 animate-pulse' 
                : recConnection && isConnectionConnecting(recConnection.status)
                ? 'shadow-yellow-500/50 animate-pulse'
                : recConnection && recConnection.status === 'Error'
                ? 'shadow-red-500/50'
                : 'shadow-gray-500/50'
            }`} 
          />
          <span className="font-medium">REC</span>
        </div>
        
        {/* STR Status - Shows OBS_STR connection status */}
        <div className={`flex items-center space-x-2 px-3 py-2 rounded-lg border backdrop-blur-sm ${
          strConnection && isConnectionActive(strConnection.status)
            ? 'bg-orange-500/10 border-orange-500/20' 
            : strConnection && isConnectionConnecting(strConnection.status)
            ? 'bg-yellow-500/10 border-yellow-500/20'
            : strConnection && strConnection.status === 'Error'
            ? 'bg-red-500/10 border-red-500/20'
            : 'bg-gray-500/10 border-gray-500/20'
        }`}>
          <StatusDot 
            color={
              strConnection && isConnectionActive(strConnection.status) ? "bg-orange-500" :
              strConnection && isConnectionConnecting(strConnection.status) ? "bg-yellow-500" :
              strConnection && strConnection.status === 'Error' ? "bg-red-500" :
              "bg-gray-500"
            } 
            size="w-3 h-3" 
            className={`shadow-lg ${
              strConnection && isConnectionActive(strConnection.status)
                ? 'shadow-orange-500/50 animate-pulse' 
                : strConnection && isConnectionConnecting(strConnection.status)
                ? 'shadow-yellow-500/50 animate-pulse'
                : strConnection && strConnection.status === 'Error'
                ? 'shadow-red-500/50'
                : 'shadow-gray-500/50'
            }`} 
          />
          <span className="font-medium">STR</span>
        </div>
        
        {/* PSS Data Status */}
        <div className={`flex items-center space-x-2 px-3 py-2 rounded-lg border backdrop-blur-sm ${
          isPssDataLoaded 
            ? 'bg-green-500/10 border-green-500/20' 
            : 'bg-gray-500/10 border-gray-500/20'
        }`}>
          <StatusDot 
            color={isPssDataLoaded ? "bg-green-500" : "bg-gray-500"} 
            size="w-3 h-3" 
            className={`shadow-lg ${
              isPssDataLoaded 
                ? 'shadow-green-500/50' 
                : 'shadow-gray-500/50'
            }`} 
          />
          <span className="font-medium">PSS</span>
        </div>
        
        {/* CPU Status */}
        <div className="flex items-center space-x-2 px-3 py-2 bg-green-500/10 rounded-lg border border-green-500/20 backdrop-blur-sm">
          <StatusDot color="bg-green-500" size="w-3 h-3" className="shadow-lg shadow-green-500/50" />
          <span className="font-medium">CPU {obsStatus?.cpu_usage?.toFixed(1) || '0.0'}%</span>
        </div>
      </div>
    </div>
  );
};

export default StatusbarDock; 