import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useAppStore, VideoClip } from '../../stores';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import { Icon } from '../atoms/Icon';
import { useI18n } from '../../i18n/index';

const VideoClips: React.FC = () => {
  const { videoClips, currentClip, setCurrentClip, addVideoClip, removeVideoClip, setPlaying } = useAppStore();
  const { t } = useI18n();
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
      alert(t('clips.form.required', 'Name and path are required'));
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
        <h2 className="text-2xl font-bold">{t('clips.title', 'Video Clips')}</h2>
        <Button
          onClick={() => setIsAddingClip(!isAddingClip)}
          variant="primary"
          size="md"
          className="px-4 py-2"
        >
          {isAddingClip ? t('common.cancel', 'Cancel') : t('clips.add_toggle_add', '+ Add Clip')}
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
                <Label htmlFor="clipName">{t('clips.form.name_label', 'Clip Name *')}</Label>
                <Input
                  type="text"
                  value={newClip.name}
                  onChange={(e) => setNewClip(prev => ({ ...prev, name: e.target.value }))}
                  placeholder={t('clips.form.name_placeholder', 'e.g., Amazing Goal, Epic Save')}
                />
              </div>

              <div>
                <Label htmlFor="filePath">{t('clips.form.path_label', 'File Path *')}</Label>
                <Input
                  type="text"
                  value={newClip.path}
                  onChange={(e) => setNewClip(prev => ({ ...prev, path: e.target.value }))}
                  placeholder={t('clips.form.path_placeholder', '/path/to/video.mp4')}
                />
              </div>

              <div>
                <Label htmlFor="duration">{t('clips.form.duration_label', 'Duration (seconds)')}</Label>
                <Input
                  type="number"
                  value={newClip.duration}
                  onChange={(e) => setNewClip(prev => ({ ...prev, duration: parseInt(e.target.value) || 0 }))}
                  placeholder={t('clips.form.duration_placeholder', '120')}
                />
              </div>

              <div>
                <Label htmlFor="tags">{t('clips.form.tags_label', 'Tags (comma-separated)')}</Label>
                <Input
                  type="text"
                  value={newClip.tags.join(', ')}
                  onChange={(e) => setNewClip(prev => ({ 
                    ...prev, 
                    tags: e.target.value.split(',').map(tag => tag.trim()).filter(tag => tag)
                  }))}
                  placeholder={t('clips.form.tags_placeholder', 'goal, save, highlight')}
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
                {t('clips.add_button', 'Add Clip')}
              </Button>
              <Button
                onClick={() => setIsAddingClip(false)}
                variant="secondary"
                size="md"
                className="px-4 py-2"
              >
                {t('common.cancel', 'Cancel')}
              </Button>
            </div>
          </motion.div>
        )}

      {/* Search and Filter */}
      <div className="mb-6 space-y-4">
        <div>
          <Input
            type="text"
            placeholder={t('clips.search_placeholder', 'Search clips...')}
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>

        {allTags.length > 0 && (
          <div>
            <Label htmlFor="filterTags">{t('clips.filter.label', 'Filter by Tags')}</Label>
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
            <svg width="64" height="64" fill="none" viewBox="0 0 24 24" stroke="currentColor" className="mx-auto mb-4 text-gray-400">
              <rect x="3" y="5" width="18" height="14" rx="2" stroke="currentColor" strokeWidth="2"/>
              <polygon points="10,9 16,12 10,15" fill="currentColor"/>
            </svg>
            <p className="text-gray-400 mb-2">
              {videoClips.length === 0 ? t('clips.empty.no_clips', 'No clips added yet') : t('clips.empty.no_match', 'No clips match your search')}
            </p>
            {videoClips.length === 0 && (
              <Button
                onClick={() => setIsAddingClip(true)}
                variant="primary"
                size="md"
                className="px-4 py-2"
              >
                {t('clips.empty.add_first', 'Add Your First Clip')}
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
                <svg width="48" height="48" fill="none" viewBox="0 0 24 24" stroke="currentColor" className="text-gray-400">
                  <rect x="3" y="5" width="18" height="14" rx="2" stroke="currentColor" strokeWidth="2"/>
                  <polygon points="10,9 16,12 10,15" fill="currentColor"/>
                </svg>
              </div>

              <div className="space-y-2">
                <h3 className="font-semibold truncate">{clip.name}</h3>
                
                <div className="flex items-center justify-between text-sm text-gray-400">
                  <span>{t('clips.duration', 'Duration')}: {formatDuration(clip.duration)}</span>
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
                      title={t('common.play', 'Play')}
                    >
                      <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <polygon points="5,3 19,12 5,21" fill="currentColor"/>
                      </svg>
                    </Button>
                    <Button
                      onClick={(e) => {
                        e.stopPropagation();
                        removeVideoClip(clip.id);
                      }}
                      variant="danger"
                      size="sm"
                      className="p-1"
                      title={t('common.delete', 'Delete')}
                    >
                      <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                      </svg>
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