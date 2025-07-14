import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useAppStore, VideoClip } from '../stores';

const VideoClips: React.FC = () => {
  const { videoClips, currentClip, setCurrentClip, addVideoClip, removeVideoClip, setPlaying } = useAppStore();
  const [isAddingClip, setIsAddingClip] = useState(false);
  const [newClip, setNewClip] = useState({
    name: '',
    path: '',
    duration: 0,
    tags: [] as string[],
  });
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedTags, setSelectedTags] = useState<string[]>([]);

  // Filter clips based on search and tags
  const filteredClips = videoClips.filter(clip => {
    const matchesSearch = clip.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         clip.path.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesTags = selectedTags.length === 0 || 
                       selectedTags.some(tag => clip.tags.includes(tag));
    return matchesSearch && matchesTags;
  });

  // Get all unique tags
  const allTags = Array.from(new Set(videoClips.flatMap(clip => clip.tags)));

  const handleAddClip = () => {
    if (!newClip.name.trim() || !newClip.path.trim()) {
      alert('Name and path are required');
      return;
    }

    addVideoClip({
      name: newClip.name,
      path: newClip.path,
      duration: newClip.duration,
      tags: newClip.tags,
    });

    // Reset form
    setNewClip({
      name: '',
      path: '',
      duration: 0,
      tags: [],
    });
    setIsAddingClip(false);
  };

  const handlePlayClip = (clip: VideoClip) => {
    setCurrentClip(clip);
    setPlaying(true);
    useAppStore.getState().setCurrentView('overlay');
  };

  const formatDuration = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const formatDate = (date: Date) => {
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    }).format(date);
  };

  return (
    <div className="p-6 bg-gray-900 text-white rounded-lg">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold">Video Clips</h2>
        <button
          onClick={() => setIsAddingClip(!isAddingClip)}
          className="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-lg transition-colors"
        >
          {isAddingClip ? 'Cancel' : '+ Add Clip'}
        </button>
      </div>

      {/* Add New Clip Form */}
      <AnimatePresence>
        {isAddingClip && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="mb-6 p-4 bg-gray-800 rounded-lg"
          >
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium mb-1">Clip Name *</label>
                <input
                  type="text"
                  value={newClip.name}
                  onChange={(e) => setNewClip(prev => ({ ...prev, name: e.target.value }))}
                  className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500"
                  placeholder="e.g., Amazing Goal, Epic Save"
                />
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">File Path *</label>
                <input
                  type="text"
                  value={newClip.path}
                  onChange={(e) => setNewClip(prev => ({ ...prev, path: e.target.value }))}
                  className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500"
                  placeholder="/path/to/video.mp4"
                />
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">Duration (seconds)</label>
                <input
                  type="number"
                  value={newClip.duration}
                  onChange={(e) => setNewClip(prev => ({ ...prev, duration: parseInt(e.target.value) || 0 }))}
                  className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500"
                  placeholder="120"
                />
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">Tags (comma-separated)</label>
                <input
                  type="text"
                  value={newClip.tags.join(', ')}
                  onChange={(e) => setNewClip(prev => ({ 
                    ...prev, 
                    tags: e.target.value.split(',').map(tag => tag.trim()).filter(tag => tag)
                  }))}
                  className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500"
                  placeholder="goal, save, highlight"
                />
              </div>
            </div>

            <div className="mt-4 flex gap-2">
              <button
                onClick={handleAddClip}
                className="bg-green-600 hover:bg-green-700 px-4 py-2 rounded-lg transition-colors"
              >
                Add Clip
              </button>
              <button
                onClick={() => setIsAddingClip(false)}
                className="bg-gray-600 hover:bg-gray-700 px-4 py-2 rounded-lg transition-colors"
              >
                Cancel
              </button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Search and Filter */}
      <div className="mb-6 space-y-4">
        <div>
          <input
            type="text"
            placeholder="Search clips..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
          />
        </div>

        {allTags.length > 0 && (
          <div>
            <label className="block text-sm font-medium mb-2">Filter by Tags</label>
            <div className="flex flex-wrap gap-2">
              {allTags.map(tag => (
                <button
                  key={tag}
                  onClick={() => setSelectedTags(prev => 
                    prev.includes(tag) 
                      ? prev.filter(t => t !== tag)
                      : [...prev, tag]
                  )}
                  className={`px-3 py-1 rounded-full text-sm transition-colors ${
                    selectedTags.includes(tag)
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                  }`}
                >
                  {tag}
                </button>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Clips Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {filteredClips.length === 0 ? (
          <div className="col-span-full text-center py-12">
            <div className="text-6xl mb-4">🎬</div>
            <p className="text-gray-400 mb-2">
              {videoClips.length === 0 ? 'No clips added yet' : 'No clips match your search'}
            </p>
            {videoClips.length === 0 && (
              <button
                onClick={() => setIsAddingClip(true)}
                className="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-lg transition-colors"
              >
                Add Your First Clip
              </button>
            )}
          </div>
        ) : (
          filteredClips.map((clip) => (
            <motion.div
              key={clip.id}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              className={`p-4 bg-gray-800 rounded-lg border-2 transition-all hover:border-blue-500 cursor-pointer ${
                currentClip?.id === clip.id ? 'border-blue-500' : 'border-gray-700'
              }`}
              onClick={() => handlePlayClip(clip)}
            >
              <div className="aspect-video bg-gray-700 rounded-lg mb-3 flex items-center justify-center">
                <div className="text-4xl">🎥</div>
              </div>

              <div className="space-y-2">
                <h3 className="font-semibold truncate">{clip.name}</h3>
                
                <div className="flex items-center justify-between text-sm text-gray-400">
                  <span>Duration: {formatDuration(clip.duration)}</span>
                  <span>{formatDate(clip.timestamp)}</span>
                </div>

                <div className="flex items-center justify-between">
                  <div className="flex flex-wrap gap-1">
                    {clip.tags.slice(0, 3).map(tag => (
                      <span
                        key={tag}
                        className="px-2 py-1 bg-gray-700 rounded text-xs"
                      >
                        {tag}
                      </span>
                    ))}
                    {clip.tags.length > 3 && (
                      <span className="px-2 py-1 bg-gray-700 rounded text-xs">
                        +{clip.tags.length - 3}
                      </span>
                    )}
                  </div>

                  <div className="flex space-x-2">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handlePlayClip(clip);
                      }}
                      className="bg-blue-600 hover:bg-blue-700 p-1 rounded transition-colors"
                      title="Play"
                    >
                      ▶️
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        removeVideoClip(clip.id);
                      }}
                      className="bg-red-600 hover:bg-red-700 p-1 rounded transition-colors"
                      title="Delete"
                    >
                      🗑️
                    </button>
                  </div>
                </div>
              </div>
            </motion.div>
          ))
        )}
      </div>

      {/* Statistics */}
      {videoClips.length > 0 && (
        <div className="mt-6 p-4 bg-gray-800 rounded-lg">
          <h3 className="text-lg font-semibold mb-3">Statistics</h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
            <div>
              <div className="text-gray-400">Total Clips</div>
              <div className="text-xl font-bold">{videoClips.length}</div>
            </div>
            <div>
              <div className="text-gray-400">Total Duration</div>
              <div className="text-xl font-bold">
                {formatDuration(videoClips.reduce((sum, clip) => sum + clip.duration, 0))}
              </div>
            </div>
            <div>
              <div className="text-gray-400">Unique Tags</div>
              <div className="text-xl font-bold">{allTags.length}</div>
            </div>
            <div>
              <div className="text-gray-400">Currently Playing</div>
              <div className="text-xl font-bold">
                {currentClip ? currentClip.name : 'None'}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default VideoClips; 