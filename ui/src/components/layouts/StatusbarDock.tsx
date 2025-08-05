import React from 'react';
import { StatusDot } from '../atoms/StatusDot';
import { usePssMatchStore } from '../../stores';
import { useObsStore } from '../../stores/obsStore';
import { obsCommands } from '../../utils/tauriCommands';

const StatusbarDock: React.FC = () => {
  // Get PSS data status
  const isPssDataLoaded = usePssMatchStore((state) => state.matchData.isLoaded);
  
  // Get OBS status from store
  const obsStatus = useObsStore((state) => state.obsStatus);
  
  // Debug function to test status listener
  const testStatusListener = async () => {
    console.log('🔧 Testing OBS status listener...');
    try {
      const result = await obsCommands.setupStatusListener();
      console.log('🔧 Status listener result:', result);
    } catch (error) {
      console.error('🔧 Error testing status listener:', error);
    }
  };
  
  return (
    <div className="w-full h-[4.5rem] bg-gradient-to-r from-gray-800/80 to-gray-900/90 backdrop-blur-sm flex items-center justify-center text-xs text-gray-300 px-8 border-t border-gray-600/30">
      <div className="flex items-center space-x-4">
        {/* Debug button */}
        <button 
          onClick={testStatusListener}
          className="px-2 py-1 bg-blue-500/20 rounded text-xs hover:bg-blue-500/30"
        >
          Test Status
        </button>
        
        {/* REC Status */}
        <div className={`flex items-center space-x-2 px-3 py-2 rounded-lg border backdrop-blur-sm ${
          obsStatus?.is_recording 
            ? 'bg-red-500/10 border-red-500/20' 
            : 'bg-gray-500/10 border-gray-500/20'
        }`}>
          <StatusDot 
            color={obsStatus?.is_recording ? "bg-red-500" : "bg-gray-500"} 
            size="w-3 h-3" 
            className={`shadow-lg ${
              obsStatus?.is_recording 
                ? 'shadow-red-500/50 animate-pulse' 
                : 'shadow-gray-500/50'
            }`} 
          />
          <span className="font-medium">REC</span>
        </div>
        
        {/* STR Status */}
        <div className={`flex items-center space-x-2 px-3 py-2 rounded-lg border backdrop-blur-sm ${
          obsStatus?.is_streaming 
            ? 'bg-orange-500/10 border-orange-500/20' 
            : 'bg-gray-500/10 border-gray-500/20'
        }`}>
          <StatusDot 
            color={obsStatus?.is_streaming ? "bg-orange-500" : "bg-gray-500"} 
            size="w-3 h-3" 
            className={`shadow-lg ${
              obsStatus?.is_streaming 
                ? 'shadow-orange-500/50 animate-pulse' 
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