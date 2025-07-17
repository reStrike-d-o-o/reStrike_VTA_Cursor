import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useAppStore, VideoClip } from '../../stores';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import { Icon } from '../atoms/Icon';

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
        <Button
          onClick={() => setIsAddingClip(!isAddingClip)}
          variant="primary"
          size="md"
          className="px-4 py-2"
        >
          {isAddingClip ? 'Cancel' : '+ Add Clip'}
        </Button>
      </div>

      {/* Add New Clip Form */}
      {isAddingClip && (
        <motion.div
          initial={{ opacity: 0, height: 0 }}
          animate={{ opacity: 1, height: 'auto' }}
          exit={{ opacity: 0, height: 0 }}
          className="mb-6 p-4 bg-gray-800 rounded-lg"
        >
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <Label htmlFor="clipName">Clip Name *</Label>
                <Input
                  type="text"
                  value={newClip.name}
                  onChange={(e) => setNewClip(prev => ({ ...prev, name: e.target.value }))}
                  placeholder="e.g., Amazing Goal, Epic Save"
                />
              </div>

              <div>
                <Label htmlFor="filePath">File Path *</Label>
                <Input
                  type="text"
                  value={newClip.path}
                  onChange={(e) => setNewClip(prev => ({ ...prev, path: e.target.value }))}
                  placeholder="/path/to/video.mp4"
                />
              </div>

              <div>
                <Label htmlFor="duration">Duration (seconds)</Label>
                <Input
                  type="number"
                  value={newClip.duration}
                  onChange={(e) => setNewClip(prev => ({ ...prev, duration: parseInt(e.target.value) || 0 }))}
                  placeholder="120"
                />
              </div>

              <div>
                <Label htmlFor="tags">Tags (comma-separated)</Label>
                <Input
                  type="text"
                  value={newClip.tags.join(', ')}
                  onChange={(e) => setNewClip(prev => ({ 
                    ...prev, 
                    tags: e.target.value.split(',').map(tag => tag.trim()).filter(tag => tag)
                  }))}
                  placeholder="goal, save, highlight"
                />
              </div>
            </div>

            <div className="mt-4 flex gap-2">
              <Button
                onClick={handleAddClip}
                variant="success"
                size="md"
                className="px-4 py-2"
              >
                Add Clip
              </Button>
              <Button
                onClick={() => setIsAddingClip(false)}
                variant="secondary"
                size="md"
                className="px-4 py-2"
              >
                Cancel
              </Button>
            </div>
          </motion.div>
        )}

      {/* Search and Filter */}
      <div className="mb-6 space-y-4">
        <div>
          <Input
            type="text"
            placeholder="Search clips..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>

        {allTags.length > 0 && (
          <div>
            <Label htmlFor="filterTags">Filter by Tags</Label>
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
              <Button
                onClick={() => setIsAddingClip(true)}
                variant="primary"
                size="md"
                className="px-4 py-2"
              >
                Add Your First Clip
              </Button>
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
                <Icon name="🎥" size="text-4xl" />
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
                    <Button
                      onClick={(e) => {
                        e.stopPropagation();
                        handlePlayClip(clip);
                      }}
                      variant="primary"
                      size="sm"
                      className="p-1"
                      title="Play"
                    >
                      <Icon name="▶️" />
                    </Button>
                    <Button
                      onClick={(e) => {
                        e.stopPropagation();
                        removeVideoClip(clip.id);
                      }}
                      variant="danger"
                      size="sm"
                      className="p-1"
                      title="Delete"
                    >
                      <Icon name="🗑️" />
                    </Button>
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