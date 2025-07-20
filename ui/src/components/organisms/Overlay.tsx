import React, { useState, useRef, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useAppStore } from '../../stores';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import { StatusDot } from '../atoms/StatusDot';
import { Icon } from '../atoms/Icon';

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
                <svg width="48" height="48" fill="none" viewBox="0 0 24 24" stroke="currentColor" className="mx-auto mb-2 text-gray-400">
                  <rect x="3" y="5" width="18" height="14" rx="2" stroke="currentColor" strokeWidth="2"/>
                  <polygon points="10,9 16,12 10,15" fill="currentColor"/>
                </svg>
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
                      <Input
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
                      <Button
                        onClick={() => setPlaying(!isPlaying)}
                        variant="primary"
                        size="sm"
                      >
                        {isPlaying ? (
                          <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <rect x="6" y="4" width="4" height="16" rx="1" stroke="currentColor" strokeWidth="2"/>
                            <rect x="14" y="4" width="4" height="16" rx="1" stroke="currentColor" strokeWidth="2"/>
                          </svg>
                        ) : (
                          <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <polygon points="5,3 19,12 5,21" fill="currentColor"/>
                          </svg>
                        )}
                      </Button>
                      <Button
                        onClick={() => {
                          if (videoRef.current) {
                            videoRef.current.currentTime = 0;
                            setPlaying(false);
                          }
                        }}
                        variant="secondary"
                        size="sm"
                      >
                        <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <polygon points="19,20 9,12 19,4" fill="currentColor"/>
                          <line x1="5" y1="19" x2="5" y2="5" stroke="currentColor" strokeWidth="2"/>
                        </svg>
                      </Button>
                      <Button
                        onClick={toggleFullscreen}
                        variant="secondary"
                        size="sm"
                      >
                        {isFullscreen ? (
                          <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                          </svg>
                        ) : (
                          <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
                          </svg>
                        )}
                      </Button>
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
                <StatusDot color="bg-green-500" />
                <span>OBS: {obsConnections.filter(c => c.status === 'Connected').length} connected</span>
              </div>
              
              {/* Video Status */}
              {currentClip && (
                <div className="flex items-center space-x-1">
                  <StatusDot color="bg-blue-500" />
                  <span>Video: {currentClip.name}</span>
                </div>
              )}
            </div>

            {/* Playback Status */}
            {isPlaying && (
                          <div className="flex items-center space-x-1">
              <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor" className="animate-pulse">
                <polygon points="5,3 19,12 5,21" fill="currentColor"/>
              </svg>
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
            <Button
              onClick={() => useAppStore.getState().setCurrentView('clips')}
              variant="primary"
              size="sm"
            >
              <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
            </Button>
            <Button
              onClick={() => useAppStore.getState().setCurrentView('obs-manager')}
              variant="success"
              size="sm"
            >
              <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <rect x="3" y="7" width="15" height="10" rx="2" stroke="currentColor" strokeWidth="2"/>
                <rect x="16" y="10" width="5" height="4" rx="1" stroke="currentColor" strokeWidth="2"/>
                <circle cx="10.5" cy="12" r="2.5" stroke="currentColor" strokeWidth="2"/>
              </svg>
            </Button>
            <Button
              onClick={() => useAppStore.getState().setCurrentView('settings')}
              variant="secondary"
              size="sm"
            >
              <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <circle cx="12" cy="12" r="3" stroke="currentColor" strokeWidth="2"/>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06-.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h.09A1.65 1.65 0 0 0 9 3.09V3a2 2 0 1 1 4 0v.09c0 .66.39 1.26 1 1.51h.09a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.09c.66 0 1.26.39 1.51 1H21a2 2 0 1 1 0 4h-.09c-.66 0-1.26.39-1.51 1z" stroke="currentColor" strokeWidth="2"/>
              </svg>
            </Button>
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
