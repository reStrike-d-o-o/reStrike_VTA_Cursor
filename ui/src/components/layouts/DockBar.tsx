import React, { useState } from 'react';

const DockBar: React.FC = () => {
  const [manualMode, setManualMode] = useState(false);

  return (
    <div className="flex flex-col items-center justify-between py-8 px-4 bg-[#181F26] w-40 shadow-lg h-full">
      <div className="flex flex-col items-center space-y-8">
        {/* Replay Button */}
        <button
          className="w-32 h-32 rounded-full bg-red-600 flex items-center justify-center text-xl font-bold text-white border-4 border-red-700 mb-4 shadow-2xl hover:bg-red-700 focus:outline-none transition-all duration-200"
          style={{ boxShadow: '0 0 20px rgba(220, 38, 38, 0.6), 0 0 0 4px #2B2B2B', animation: 'pulse 2s infinite' }}
          onClick={() => alert('Replay action (to be implemented)')}
        >
          REPLAY
        </button>
        {/* Manual Mode Toggle */}
        <div className="flex flex-col items-center space-y-3">
          <span className="text-sm text-gray-300 font-medium">Manual Mode</span>
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={manualMode}
              onChange={() => setManualMode((v) => !v)}
              className="sr-only peer"
              aria-label="Manual Mode Toggle"
            />
            <div className="w-12 h-6 bg-gray-700 rounded-full peer peer-focus:ring-2 peer-focus:ring-blue-500 transition-all duration-200 border border-gray-600" />
            <div
              className={`absolute left-0.5 top-0.5 w-5 h-5 rounded-full transition-transform duration-200 ${manualMode ? 'translate-x-6 bg-blue-500 shadow-lg' : 'bg-gray-400'}`}
            />
          </label>
        </div>
        {/* Advanced Button */}
        <button
          className="w-32 h-10 mt-6 bg-gray-800 hover:bg-gray-700 border border-gray-600 rounded-lg text-sm text-gray-200 font-medium transition-all duration-200 hover:border-gray-500"
          onClick={() => alert('Advanced action (to be implemented)')}
        >
          Advanced
        </button>
      </div>
      {/* StatusbarDock Placeholder */}
      <div className="w-full flex justify-between items-center text-xs text-gray-500 mt-8 px-2">
        <div className="flex items-center space-x-1">
          <span className="w-2 h-2 rounded-full bg-red-500" />
          <span>REC</span>
        </div>
        <div className="flex items-center space-x-1">
          <span className="w-2 h-2 rounded-full bg-red-500" />
          <span>STR</span>
        </div>
        <div className="flex items-center space-x-1">
          <span className="w-2 h-2 rounded-full bg-green-500" />
          <span>CPU 0%</span>
        </div>
      </div>
    </div>
  );
};

export default DockBar; 