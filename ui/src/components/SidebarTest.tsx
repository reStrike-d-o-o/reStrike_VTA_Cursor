import React from 'react';
import SidebarSecondColumn from './SidebarSecondColumn';

const SidebarTest: React.FC = () => {
  return (
    <div className="min-h-screen bg-gray-950 flex">
      {/* First Column (Controls) - Narrow */}
      <div className="w-16 bg-gray-900 p-2 flex flex-col items-center space-y-4">
        {/* Big Red Replay Button */}
        <button className="w-12 h-12 bg-red-600 hover:bg-red-700 rounded-lg flex items-center justify-center text-white font-bold animate-pulse">
          ●
        </button>
        
        {/* Manual Mode Toggle */}
        <div className="flex flex-col items-center space-y-1">
          <span className="text-xs text-gray-400">Manual</span>
          <div className="w-8 h-4 bg-gray-700 rounded-full relative">
            <div className="w-4 h-4 bg-blue-500 rounded-full absolute left-0 transition-transform"></div>
          </div>
        </div>
        
        {/* Advanced Button */}
        <button className="w-12 h-8 bg-gray-700 hover:bg-gray-600 rounded text-xs text-gray-300">
          Advanced
        </button>
      </div>
      
      {/* Second Column (Info) - Wider */}
      <SidebarSecondColumn />
      
      {/* Main Content Area */}
      <div className="flex-1 p-8">
        <div className="max-w-4xl mx-auto">
          <h1 className="text-3xl font-bold text-white mb-6">Sidebar Second Column - Visual Review</h1>
          
          <div className="bg-gray-800 rounded-lg p-6 space-y-6">
            <div>
              <h2 className="text-xl font-semibold text-white mb-3">✅ What's Implemented:</h2>
              <ul className="text-gray-300 space-y-2">
                <li>• <strong>MatchInfoSection:</strong> Athletes with flags (🇺🇸 Benjamin Smith, 🇯🇵 Kei Tanaka)</li>
                <li>• <strong>Category & Stage:</strong> M-75kg and Semi-final stacked on the left</li>
                <li>• <strong>Match Number:</strong> Large "1254" right-aligned in red</li>
                <li>• <strong>EventTable:</strong> Header with subtle accent (RND | TIME | EVENT)</li>
                <li>• <strong>Event Rows:</strong> With colored dots (red, blue, yellow, green)</li>
                <li>• <strong>"Go to Top" Arrow:</strong> Positioned at bottom right of table</li>
                <li>• <strong>StatusBar:</strong> OBS Recording, CP 5%, PSS with colored dots</li>
              </ul>
            </div>
            
            <div>
              <h2 className="text-xl font-semibold text-white mb-3">🎨 Design Features:</h2>
              <ul className="text-gray-300 space-y-2">
                <li>• <strong>Two-Column Layout:</strong> Narrow controls (left) + Wide info (right)</li>
                <li>• <strong>Color Coding:</strong> Red/green dots for status indicators</li>
                <li>• <strong>Typography:</strong> Proper hierarchy with different text sizes</li>
                <li>• <strong>Spacing:</strong> Consistent padding and margins</li>
                <li>• <strong>Scrollable Event Table:</strong> For multiple events</li>
              </ul>
            </div>
            
            <div>
              <h2 className="text-xl font-semibold text-white mb-3">📋 Next Steps:</h2>
              <ul className="text-gray-300 space-y-2">
                <li>• <strong>Review:</strong> Please check the visual layout and styling</li>
                <li>• <strong>Approval:</strong> Once approved, we'll add real data and logic</li>
                <li>• <strong>Integration:</strong> Connect with backend and state management</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SidebarTest; 