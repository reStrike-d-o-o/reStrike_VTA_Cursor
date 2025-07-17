import React, { useState, useRef, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useAppStore } from '../../stores';

const Overlay: React.FC = () => {
  const {
    overlaySettings,
    currentClip,
    isPlaying,
    setPlaying,
    setCurrentClip,
    videoClips,
    obsConnections,
  } = useAppStore();

  const videoRef = useRef<HTMLVideoElement>(null);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [showControls, setShowControls] = useState(false);
  const [isFullscreen, setIsFullscreen] = useState(false);

  // Video event handlers
  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;

    const handleTimeUpdate = () => setCurrentTime(video.currentTime);
    const handleLoadedMetadata = () => setDuration(video.duration);
    const handleEnded = () => setPlaying(false);

    video.addEventListener('timeupdate', handleTimeUpdate);
    video.addEventListener('loadedmetadata', handleLoadedMetadata);
    video.addEventListener('ended', handleEnded);

    return () => {
      video.removeEventListener('timeupdate', handleTimeUpdate);
      video.removeEventListener('loadedmetadata', handleLoadedMetadata);
      video.removeEventListener('ended', handleEnded);
    };
  }, [setPlaying]);

  // Play/pause video
  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;

    if (isPlaying) {
      video.play().catch(console.error);
    } else {
      video.pause();
    }
  }, [isPlaying, currentClip]);

  // Format time
  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  // Handle seek
  const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
    const video = videoRef.current;
    if (!video) return;

    const newTime = parseFloat(e.target.value);
    video.currentTime = newTime;
    setCurrentTime(newTime);
  };

  // Handle fullscreen
  const toggleFullscreen = () => {
    if (!document.fullscreenElement) {
      videoRef.current?.requestFullscreen();
      setIsFullscreen(true);
    } else {
      document.exitFullscreen();
      setIsFullscreen(false);
    }
  };

  // Get position styles
  const getPositionStyles = () => {
    const baseStyles = {
      position: 'fixed' as const,
      zIndex: 9999,
      transform: `scale(${overlaySettings.scale})`,
      opacity: overlaySettings.opacity,
    };

    switch (overlaySettings.position) {
      case 'top-left':
        return { ...baseStyles, top: '20px', left: '20px' };
      case 'top-right':
        return { ...baseStyles, top: '20px', right: '20px' };
      case 'bottom-left':
        return { ...baseStyles, bottom: '20px', left: '20px' };
      case 'bottom-right':
        return { ...baseStyles, bottom: '20px', right: '20px' };
      case 'center':
        return { 
          ...baseStyles, 
          top: '50%', 
          left: '50%', 
          transform: `translate(-50%, -50%) scale(${overlaySettings.scale})` 
        };
      default:
        return { ...baseStyles, bottom: '20px', right: '20px' };
    }
  };

  // Get theme styles
  const getThemeStyles = () => {
    switch (overlaySettings.theme) {
      case 'light':
        return 'bg-white text-gray-900 border-gray-200';
      case 'transparent':
        return 'bg-transparent text-white';
      default:
        return 'bg-gray-900 text-white border-gray-700';
    }
  };

  if (!overlaySettings.visible) {
    return null;
  }

  return (
    <div style={getPositionStyles()} className="overlay-container">
      <motion.div
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        exit={{ opacity: 0, scale: 0.9 }}
        className={`rounded-lg shadow-2xl border-2 ${getThemeStyles()}`}
        onMouseEnter={() => setShowControls(true)}
        onMouseLeave={() => setShowControls(false)}
      >
        {/* Video Player */}
        <div className="relative">
          {currentClip ? (
            <video
              ref={videoRef}
              src={currentClip.path}
              className="w-full h-auto rounded-t-lg"
              onDoubleClick={toggleFullscreen}
            />
          ) : (
            <div className="w-96 h-64 bg-gray-800 rounded-t-lg flex items-center justify-center">
              <div className="text-center">
                <div className="text-4xl mb-2">🎬</div>
                <p className="text-gray-400">No video selected</p>
              </div>
            </div>
          )}

          {/* Video Controls Overlay */}
          {showControls && currentClip && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="absolute inset-0 bg-black bg-opacity-50 flex items-center justify-center"
            >
                <div className="bg-gray-900 bg-opacity-90 rounded-lg p-4 w-80">
                  <div className="space-y-4">
                    {/* Progress Bar */}
                    <div>
                      <input
                        type="range"
                        min="0"
                        max={duration || 0}
                        value={currentTime}
                        onChange={handleSeek}
                        className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer slider"
                      />
                      <div className="flex justify-between text-sm mt-1">
                        <span>{formatTime(currentTime)}</span>
                        <span>{formatTime(duration)}</span>
                      </div>
                    </div>

                    {/* Control Buttons */}
                    <div className="flex justify-center space-x-4">
                      <button
                        onClick={() => setPlaying(!isPlaying)}
                        className="bg-blue-600 hover:bg-blue-700 p-2 rounded-lg transition-colors"
                      >
                        {isPlaying ? '⏸️' : '▶️'}
                      </button>
                      <button
                        onClick={() => {
                          if (videoRef.current) {
                            videoRef.current.currentTime = 0;
                            setPlaying(false);
                          }
                        }}
                        className="bg-gray-600 hover:bg-gray-700 p-2 rounded-lg transition-colors"
                      >
                        ⏮️
                      </button>
                      <button
                        onClick={toggleFullscreen}
                        className="bg-gray-600 hover:bg-gray-700 p-2 rounded-lg transition-colors"
                      >
                        {isFullscreen ? '⏹️' : '⛶'}
                      </button>
                    </div>

                    {/* Clip Info */}
                    <div className="text-center">
                      <h3 className="font-semibold">{currentClip.name}</h3>
                      <p className="text-sm text-gray-400">
                        Duration: {formatTime(currentClip.duration)}
                      </p>
                    </div>
                  </div>
                </div>
              </motion.div>
            )}
        </div>

        {/* Status Bar */}
        <div className="px-4 py-2 border-t border-gray-700">
          <div className="flex items-center justify-between text-sm">
            <div className="flex items-center space-x-2">
              {/* OBS Status */}
              <div className="flex items-center space-x-1">
                <span className="w-2 h-2 rounded-full bg-green-500"></span>
                <span>OBS: {obsConnections.filter(c => c.status === 'Connected').length} connected</span>
              </div>
              
              {/* Video Status */}
              {currentClip && (
                <div className="flex items-center space-x-1">
                  <span className="w-2 h-2 rounded-full bg-blue-500"></span>
                  <span>Video: {currentClip.name}</span>
                </div>
              )}
            </div>

            {/* Playback Status */}
            {isPlaying && (
              <div className="flex items-center space-x-1">
                <span className="animate-pulse">▶️</span>
                <span>Playing</span>
              </div>
            )}
          </div>
        </div>
      </motion.div>

      {/* Quick Actions */}
      {showControls && (
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 10 }}
          className="mt-2 flex justify-center space-x-2"
        >
            <button
              onClick={() => useAppStore.getState().setCurrentView('clips')}
              className="bg-blue-600 hover:bg-blue-700 px-3 py-1 rounded text-sm transition-colors"
            >
              📁 Clips
            </button>
            <button
              onClick={() => useAppStore.getState().setCurrentView('obs-manager')}
              className="bg-green-600 hover:bg-green-700 px-3 py-1 rounded text-sm transition-colors"
            >
              🎥 OBS
            </button>
            <button
              onClick={() => useAppStore.getState().setCurrentView('settings')}
              className="bg-gray-600 hover:bg-gray-700 px-3 py-1 rounded text-sm transition-colors"
            >
              ⚙️ Settings
            </button>
          </motion.div>
        )}

      {/* Custom CSS for slider */}
      <style>{`
        .slider::-webkit-slider-thumb {
          appearance: none;
          height: 16px;
          width: 16px;
          border-radius: 50%;
          background: #3b82f6;
          cursor: pointer;
        }
        
        .slider::-moz-range-thumb {
          height: 16px;
          width: 16px;
          border-radius: 50%;
          background: #3b82f6;
          cursor: pointer;
          border: none;
        }
      `}</style>
    </div>
  );
};

export default Overlay;
