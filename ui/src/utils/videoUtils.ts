// Video utility functions for reStrike VTA

import { VideoClip, VideoSettings } from '../types';

/**
 * Format time in seconds to MM:SS format
 */
export const formatTime = (seconds: number): string => {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
};

/**
 * Format time in seconds to HH:MM:SS format
 */
export const formatTimeLong = (seconds: number): string => {
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);
  
  if (hours > 0) {
    return `${hours}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }
  return `${mins}:${secs.toString().padStart(2, '0')}`;
};

/**
 * Get video file extension
 */
export const getVideoExtension = (filename: string): string => {
  return filename.split('.').pop()?.toLowerCase() || '';
};

/**
 * Check if file is a video file
 */
export const isVideoFile = (filename: string): boolean => {
  const videoExtensions = ['mp4', 'avi', 'mov', 'mkv', 'wmv', 'flv', 'webm'];
  const extension = getVideoExtension(filename);
  return videoExtensions.includes(extension);
};

/**
 * Generate video clip ID
 */
export const generateClipId = (): string => {
  return crypto.randomUUID();
};

/**
 * Create a new video clip
 */
export const createVideoClip = (
  name: string,
  path: string,
  duration: number,
  tags: string[] = []
): VideoClip => {
  return {
    id: generateClipId(),
    name,
    path,
    duration,
    timestamp: new Date(),
    tags,
    metadata: {},
  };
};

/**
 * Get default video settings
 */
export const getDefaultVideoSettings = (): VideoSettings => {
  return {
    volume: 1.0,
    playback_rate: 1.0,
    loop_enabled: false,
    hardware_acceleration: true,
  };
};

/**
 * Calculate video aspect ratio
 */
export const calculateAspectRatio = (width: number, height: number): number => {
  return width / height;
};

/**
 * Get video dimensions from aspect ratio
 */
export const getVideoDimensions = (
  aspectRatio: number,
  maxWidth: number,
  maxHeight: number
): { width: number; height: number } => {
  if (aspectRatio > 1) {
    // Landscape
    const width = Math.min(maxWidth, maxHeight * aspectRatio);
    const height = width / aspectRatio;
    return { width, height };
  } else {
    // Portrait
    const height = Math.min(maxHeight, maxWidth / aspectRatio);
    const width = height * aspectRatio;
    return { width, height };
  }
}; 