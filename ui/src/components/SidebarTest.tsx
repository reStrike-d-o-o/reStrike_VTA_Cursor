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
            className="w-28 h-28 rounded-full bg-red-600 shadow-xl flex items-center justify-center text-2xl font-bold text-white border-4 border-[#2B2B2B] animate-pulse focus:outline-none mb-2"
            style={{ boxShadow: '0 0 0 6px #2B2B2B' }}
          >
            REPLAY
          </button>

          {/* Manual Mode Toggle */}
          <div className="flex flex-col items-center space-y-2">
            <span className="text-base text-gray-200">Manual Mode</span>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={manualMode}
                onChange={() => setManualMode((v) => !v)}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-gray-700 rounded-full peer peer-focus:ring-2 peer-focus:ring-blue-500 transition-all duration-200" />
              <div
                className={`absolute left-0 top-0 w-6 h-6 rounded-full transition-transform duration-200 ${manualMode ? 'translate-x-5 bg-blue-500' : 'bg-gray-400'}`}
              />
            </label>
          </div>

          {/* Advanced Button */}
          <button className="w-32 h-10 mt-4 bg-gray-800 hover:bg-gray-700 border border-gray-600 rounded-lg text-base text-gray-200 font-medium transition-colors">
            Advanced
          </button>
        </div>
        {/* Status Bar (bottom left) */}
        <div className="w-full flex justify-between items-center text-xs text-gray-400 mt-8">
          <span>OBS Recording</span>
          <span>CP 5%</span>
        </div>
      </div>

      {/* Right Info Column */}
      <div className="flex-1 flex flex-col justify-between bg-[#101820] px-10 py-8 min-w-[340px] max-w-[480px]">
        <div>
          {/* Athlete Info */}
          <div className="flex flex-col space-y-1 mb-2">
            <div className="flex items-center space-x-2 text-lg font-semibold">
              <span className="text-2xl">🇺🇸</span>
              <span>Benjamin Smith</span>
            </div>
            <div className="flex items-center space-x-2 text-lg font-semibold">
              <span className="text-2xl">🇯🇵</span>
              <span>Kei Tanaka</span>
            </div>
          </div>
          {/* Match Metadata */}
          <div className="flex items-center justify-between mb-2">
            <div className="flex flex-col text-gray-400 text-base">
              <span>M-75kg</span>
              <span>Semi-final</span>
            </div>
            <div className="text-5xl font-bold text-right text-white tracking-tight">1254</div>
          </div>
          <hr className="border-gray-700 my-4" />
          {/* Event Table */}
          <div className="flex flex-col">
            <div className="flex items-center justify-between text-gray-400 text-sm mb-2">
              <span className="">RND</span>
              <span className="">TIME</span>
              <span className="">EVENT</span>
            </div>
            {/* Event Rows */}
            <div className="flex items-center justify-between py-1">
              <span className="font-bold text-white">R1</span>
              <span className="text-gray-200">02.00.343</span>
              <span className="flex items-center space-x-2">
                <span className="w-3 h-3 rounded-full bg-red-500 inline-block"></span>
                <span className="text-white">Punch</span>
              </span>
            </div>
            <div className="flex items-center justify-between py-1">
              <span className="font-bold text-white">R2</span>
              <span className="text-gray-200">02.10.343</span>
              <span className="flex items-center space-x-2">
                <span className="w-3 h-3 rounded-full bg-blue-400 inline-block"></span>
                <span className="text-white">Head Kick</span>
              </span>
            </div>
            <div className="flex items-center justify-between py-1">
              <span className="font-bold text-white">R3</span>
              <span className="text-gray-200">02.20.343</span>
              <span className="flex items-center space-x-2">
                <span className="w-3 h-3 rounded-full bg-yellow-400 inline-block"></span>
                <span className="text-white">Referee</span>
              </span>
            </div>
            {/* Up Arrow */}
            <div className="flex justify-end mt-2">
              <span className="text-gray-500 text-xl">↑</span>
            </div>
          </div>
        </div>
        {/* Status Bar (bottom right) - already handled in left column for this design */}
      </div>
    </div>
  );
};

export default SidebarTest; 