import React from 'react';
import SidebarSecondColumn from './SidebarSecondColumn';

const SidebarTest: React.FC = () => {
  return (
    <div className="min-h-screen bg-gray-950 flex">
      {/* Mock First Column (Controls) */}
      <div className="w-16 bg-gray-900 p-2 flex flex-col items-center space-y-4">
        {/* Big Red Replay Button */}
        <button className="w-12 h-12 bg-red-600 hover:bg-red-700 rounded-lg flex items-center justify-center text-white font-bold animate-pulse">
          ●
        </button>
        
        {/* Manual Mode Toggle */}
        <div className="flex flex-col items-center space-y-1">
          <span className="text-xs text-gray-400">Manual</span>
          <div className="w-8 h-4 bg-gray-700 rounded-full relative">
            <div className="w-3 h-3 bg-blue-500 rounded-full absolute top-0.5 left-0.5"></div>
          </div>
        </div>
        
        {/* Advanced Button */}
        <button className="w-12 h-8 bg-gray-700 hover:bg-gray-600 rounded text-xs text-white">
          Advanced
        </button>
      </div>
      
      {/* Second Column (Info) */}
      <div className="w-80 bg-gray-800">
        <SidebarSecondColumn />
      </div>
      
      {/* Main Content Area */}
      <div className="flex-1 bg-gray-950 p-8">
        <div className="text-white">
          <h1 className="text-2xl font-bold mb-4">Sidebar Second Column - Test View</h1>
          <p className="text-gray-400">
            This is a test view of the sidebar's second column. The left column shows the controls,
            and the right column shows the match info, event table, and status bar as per your specifications.
          </p>
          <div className="mt-4 p-4 bg-gray-800 rounded">
            <h2 className="text-lg font-semibold mb-2">Layout Features:</h2>
            <ul className="text-sm text-gray-300 space-y-1">
              <li>• Athletes with flags (left-aligned)</li>
              <li>• Category and stage (left, stacked)</li>
              <li>• Large match number (right-aligned)</li>
              <li>• Event table with colored dots</li>
              <li>• "Go to Top" arrow (bottom right of table)</li>
              <li>• Status bar with colored indicators</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SidebarTest; 