import React, { useState } from 'react';

const SidebarTest: React.FC = () => {
  const [manualMode, setManualMode] = useState(false);

  return (
    <div className="min-h-screen flex bg-[#101820] text-white" style={{ fontFamily: 'Segoe UI, Roboto, sans-serif' }}>
      {/* Left Control Column */}
      <div className="flex flex-col items-center justify-between py-8 px-4 bg-[#181F26] w-40 shadow-lg">
        <div className="flex flex-col items-center space-y-8">
          {/* REPLAY Button */}
          <button
            className="w-32 h-32 rounded-full bg-red-600 shadow-2xl flex items-center justify-center text-xl font-bold text-white border-4 border-red-700 hover:bg-red-700 focus:outline-none mb-4 transition-all duration-200"
            style={{ 
              boxShadow: '0 0 20px rgba(220, 38, 38, 0.6), 0 0 0 4px #2B2B2B',
              animation: 'pulse 2s infinite'
            }}
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
              />
              <div className="w-12 h-6 bg-gray-700 rounded-full peer peer-focus:ring-2 peer-focus:ring-blue-500 transition-all duration-200 border border-gray-600" />
              <div
                className={`absolute left-0.5 top-0.5 w-5 h-5 rounded-full transition-transform duration-200 ${manualMode ? 'translate-x-6 bg-blue-500 shadow-lg' : 'bg-gray-400'}`}
              />
            </label>
          </div>

          {/* Advanced Button */}
          <button className="w-32 h-10 mt-6 bg-gray-800 hover:bg-gray-700 border border-gray-600 rounded-lg text-sm text-gray-200 font-medium transition-all duration-200 hover:border-gray-500">
            Advanced
          </button>
        </div>
        {/* Status Bar (bottom left) */}
        <div className="w-full flex justify-between items-center text-xs text-gray-500 mt-8 px-2">
          <span className="flex items-center">
            <div className="w-2 h-2 bg-red-500 rounded-full mr-2 animate-pulse"></div>
            OBS Recording
          </span>
          <span>CP 5%</span>
        </div>
      </div>

      {/* Right Info Column */}
      <div className="flex-1 flex flex-col justify-between bg-[#101820] px-8 py-6 min-w-[360px] max-w-[500px]">
        <div>
          {/* Athlete Info and Match Number */}
          <div className="flex items-center justify-between mb-4">
            <div className="flex flex-col space-y-1">
              <div className="flex items-center space-x-3 text-lg font-semibold">
                <span className="text-2xl">🇺🇸</span>
                <span className="text-white">Benjamin Smith</span>
              </div>
              <div className="flex items-center space-x-3 text-lg font-semibold">
                <span className="text-2xl">🇯🇵</span>
                <span className="text-white">Kei Tanaka</span>
              </div>
              <div className="flex flex-col text-gray-400 text-sm mt-1">
                <span className="font-medium">M-75kg</span>
                <span>Semi-final</span>
              </div>
            </div>
            <div className="text-6xl font-bold text-white tracking-tight">1254</div>
          </div>
          <hr className="border-gray-700 my-6" />
          {/* Event Table */}
          <div className="flex flex-col">
            <div className="flex items-center justify-between text-gray-400 text-xs font-medium mb-3 px-1">
              <span className="w-8">RND</span>
              <span className="w-20 text-center">TIME</span>
              <span className="flex-1">EVENT</span>
            </div>
            {/* Event Rows */}
            <div className="flex items-center justify-between py-2 px-1 hover:bg-gray-800 rounded transition-colors">
              <span className="font-bold text-white w-8">R1</span>
              <span className="text-gray-300 w-20 text-center text-sm">02.00.343</span>
              <span className="flex items-center space-x-3 flex-1">
                <span className="w-3 h-3 rounded-full bg-red-500 inline-block shadow-sm"></span>
                <span className="text-white text-sm">Punch</span>
              </span>
            </div>
            <div className="flex items-center justify-between py-2 px-1 hover:bg-gray-800 rounded transition-colors">
              <span className="font-bold text-white w-8">R2</span>
              <span className="text-gray-300 w-20 text-center text-sm">02.10.343</span>
              <span className="flex items-center space-x-3 flex-1">
                <span className="w-3 h-3 rounded-full bg-blue-400 inline-block shadow-sm"></span>
                <span className="text-white text-sm">Head Kick</span>
              </span>
            </div>
            <div className="flex items-center justify-between py-2 px-1 hover:bg-gray-800 rounded transition-colors">
              <span className="font-bold text-white w-8">R3</span>
              <span className="text-gray-300 w-20 text-center text-sm">02.20.343</span>
              <span className="flex items-center space-x-3 flex-1">
                <span className="w-3 h-3 rounded-full bg-yellow-400 inline-block shadow-sm"></span>
                <span className="text-white text-sm">Referee</span>
              </span>
            </div>
            {/* Up Arrow */}
            <div className="flex justify-end mt-4">
              <span className="text-gray-500 text-lg hover:text-gray-400 cursor-pointer transition-colors">↑</span>
            </div>
          </div>
        </div>
        {/* Status Bar (bottom right) - already handled in left column for this design */}
      </div>
    </div>
  );
};

export default SidebarTest; 