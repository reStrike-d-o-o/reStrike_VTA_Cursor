// OBS utility functions for reStrike VTA

import { ObsConnection, ObsConnectionStatus, ObsStatusInfo } from '../types';

/**
 * Create a default OBS connection
 */
export const createDefaultObsConnection = (
  name: string,
  host: string = 'localhost',
  port: number = 4455,
  password?: string
): ObsConnection => {
  return {
    name,
    host,
    port,
    password,
    protocol_version: 'v5',
    enabled: true,
    status: 'Disconnected',
  };
};

/**
 * Get status color for OBS connection
 */
export const getObsStatusColor = (status: ObsConnectionStatus): string => {
  switch (status) {
    case 'Connected':
    case 'Authenticated':
      return 'bg-green-500';
    case 'Connecting':
    case 'Authenticating':
      return 'bg-yellow-500';
    case 'Error':
      return 'bg-red-500';
    default:
      return 'bg-gray-500';
  }
};

/**
 * Get status text for OBS connection
 */
export const getObsStatusText = (status: ObsConnectionStatus): string => {
  switch (status) {
    case 'Connected':
      return 'Connected';
    case 'Authenticated':
      return 'Authenticated';
    case 'Connecting':
      return 'Connecting...';
    case 'Authenticating':
      return 'Authenticating...';
    case 'Error':
      return 'Error';
    default:
      return 'Disconnected';
  }
};

/**
 * Check if OBS connection is active
 */
export const isObsConnectionActive = (status: ObsConnectionStatus): boolean => {
  return status === 'Connected' || status === 'Authenticated';
};

/**
 * Get OBS WebSocket URL
 */
export const getObsWebSocketUrl = (connection: ObsConnection): string => {
  const protocol = connection.protocol_version === 'v5' ? 'ws' : 'ws';
  return `${protocol}://${connection.host}:${connection.port}`;
};

/**
 * Validate OBS connection settings
 */
export const validateObsConnection = (connection: ObsConnection): string[] => {
  const errors: string[] = [];

  if (!connection.name.trim()) {
    errors.push('Connection name is required');
  }

  if (!connection.host.trim()) {
    errors.push('Host is required');
  }

  if (connection.port < 1 || connection.port > 65535) {
    errors.push('Port must be between 1 and 65535');
  }

  return errors;
};

/**
 * Get default OBS status info
 */
export const getDefaultObsStatusInfo = (): ObsStatusInfo => {
  return {
    is_recording: false,
    is_streaming: false,
    cpu_usage: 0,
    recording_connection: undefined,
    streaming_connection: undefined,
  };
};

/**
 * Format CPU usage for display
 */
export const formatCpuUsage = (cpuUsage: number): string => {
  return `${cpuUsage.toFixed(1)}%`;
};

/**
 * Get recording status text
 */
export const getRecordingStatusText = (isRecording: boolean): string => {
  return isRecording ? 'Recording' : 'Not Recording';
};

/**
 * Get streaming status text
 */
export const getStreamingStatusText = (isStreaming: boolean): string => {
  return isStreaming ? 'Streaming' : 'Not Streaming';
}; 