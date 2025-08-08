import React, { useState, useEffect } from 'react';
import Toggle from '../atoms/Toggle';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import StatusDot from '../atoms/StatusDot';
import { useObsStore } from '../../stores/obsStore';
import { obsObwsCommands } from '../../utils/tauriCommandsObws';

// Recording Configuration interface
interface RecordingConfig {
  connectionName: string;
  recordingPath: string;
  recordingFormat: string;
  filenamePattern: string;
  folderPattern?: string;
  autoStartRecording: boolean;
  autoStartReplayBuffer: boolean;
  saveReplayBufferOnMatchEnd: boolean;
}

const ObsIntegrationPanel: React.FC = () => {
  // Recording Configuration state
  const [recordingConfig, setRecordingConfig] = useState<RecordingConfig>({
    connectionName: '',
    recordingPath: 'C:/Users/Damjan/Videos',
    recordingFormat: 'mp4',
    filenamePattern: '{matchNumber}_{player1}_{player2}_{date}_{time}',
    folderPattern: '{tournament}/{tournamentDay}',
    autoStartRecording: true,
    autoStartReplayBuffer: true,
    saveReplayBufferOnMatchEnd: true,
  });
  // Folder pattern for future customization of directory layout
  const [folderPattern, setFolderPattern] = useState<string>('{tournament}/{tournamentDay}');

  const [isLoadingConfig, setIsLoadingConfig] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [testResult, setTestResult] = useState<string>('');

  // Get OBS connections from store
  const { connections, setConnections } = useObsStore();

  // Load OBS connections on component mount
  useEffect(() => {
      const loadConnections = async () => {
    try {
      // Get all configured OBS connections (like WebSocketManager does)
      const result = await obsObwsCommands.getConnections();
      if (result.success && result.data) {
        // The backend returns { "connections": [...] }, so we need to access result.data.connections
        const connectionsArray = result.data.connections || [];
        const connectionsWithStatus = await Promise.all(
          connectionsArray.map(async (conn: any) => {
            // Get status for each connection
            const statusResult = await obsObwsCommands.getConnectionStatus(conn.name);
            return {
              name: conn.name,
              host: conn.host || 'localhost',
              port: conn.port || 4455,
              password: conn.password,
              enabled: conn.enabled || true,
              status: statusResult.success ? statusResult.data?.status || 'disconnected' : 'disconnected',
              error: statusResult.success ? statusResult.data?.error : null
            };
          })
        );
        setConnections(connectionsWithStatus);
      }
    } catch (error) {
      console.error('Failed to load OBS connections:', error);
    }
  };

    loadConnections();
  }, [setConnections]);

  // Load recording configuration
  const loadRecordingConfig = async () => {
    if (!recordingConfig.connectionName) return;
    
    try {
      setIsLoadingConfig(true);
      const result = await obsObwsCommands.getRecordingConfig(recordingConfig.connectionName);
      
      if (result.success && result.data?.config) {
        const config = result.data.config;
        setRecordingConfig(prev => ({
          ...prev,
          recordingPath: config.recording_path || prev.recordingPath,
          recordingFormat: config.recording_format || prev.recordingFormat,
          filenamePattern: config.filename_pattern || prev.filenamePattern,
          folderPattern: config.folder_pattern || prev.folderPattern,
          autoStartRecording: config.auto_start_recording ?? prev.autoStartRecording,
          autoStartReplayBuffer: config.auto_start_replay_buffer ?? prev.autoStartReplayBuffer,
          saveReplayBufferOnMatchEnd: config.save_replay_buffer_on_match_end ?? prev.saveReplayBufferOnMatchEnd,
        }));
      }
    } catch (error) {
      console.error('Failed to load recording config:', error);
    } finally {
      setIsLoadingConfig(false);
    }
  };

  // Save recording configuration
  const saveRecordingConfig = async () => {
    if (!recordingConfig.connectionName) {
      console.error('No connection selected');
      return;
    }
    
    try {
      setIsSaving(true);
      const result = await obsObwsCommands.saveRecordingConfig({
        obs_connection_name: recordingConfig.connectionName,
        recording_root_path: recordingConfig.recordingPath,
        recording_format: recordingConfig.recordingFormat,
        recording_quality: 'high', // Default quality
        recording_bitrate: null,
        recording_resolution: null,
        replay_buffer_enabled: recordingConfig.autoStartReplayBuffer,
        replay_buffer_duration: 30, // Default 30 seconds
        auto_start_recording: recordingConfig.autoStartRecording,
        auto_start_replay_buffer: recordingConfig.autoStartReplayBuffer,
        filename_template: recordingConfig.filenamePattern,
        is_active: true,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      });
      
      if (result.success) {
        console.log('Recording config saved successfully');
        setTestResult('Configuration saved successfully!');
      } else {
        console.error('Failed to save recording config:', result.error);
        setTestResult(`Failed to save configuration: ${result.error}`);
      }
    } catch (error) {
      console.error('Failed to save recording config:', error);
      setTestResult(`Failed to save configuration: ${error}`);
    } finally {
      setIsSaving(false);
    }
  };

  // Test recording
  const testRecording = async () => {
    if (!recordingConfig.connectionName) {
      setTestResult('Please select a connection first');
      return;
    }
    try {
      const result = await obsObwsCommands.startRecording(recordingConfig.connectionName);
      if (result.success) {
        setTestResult('Start recording command sent successfully.');
      } else {
        setTestResult(`Failed to send start recording: ${result.error}`);
      }
    } catch (error) {
      setTestResult(`Failed to send start recording: ${error}`);
    }
  };

  // Path generation test state
  const [pathTestData, setPathTestData] = useState({
    matchId: '101',
    tournamentName: 'Test Tournament',
    tournamentDay: 'Day 1',
    matchNumber: '101',
    player1Name: 'N. DESMOND',
    player1Flag: 'MRN',
    player2Name: 'M. THIBAULT',
    player2Flag: 'SUI',
  });
  const [isTestingPath, setIsTestingPath] = useState(false);
  const [pathTestResult, setPathTestResult] = useState<string>('');

  // Test path generation
  const testPathGeneration = async () => {
    try {
      setIsTestingPath(true);
      const result = await obsObwsCommands.createTestFolders(pathTestData);
      
      if (result.success && result.data) {
        setPathTestResult(`✅ Folders created successfully!\n\n📁 Directory: ${result.data.directory}\n📄 Filename: ${result.data.filename}\n📍 Full Path: ${result.data.full_path}`);
      } else {
        setPathTestResult(`❌ Failed to create folders: ${result.error}`);
      }
    } catch (error) {
      setPathTestResult(`❌ Failed to create folders: ${error}`);
    } finally {
      setIsTestingPath(false);
    }
  };

  // Send configuration to OBS
  const sendConfigToObs = async () => {
    if (!recordingConfig.connectionName) {
      setTestResult('Please select a connection first');
      return;
    }
    
    try {
      setIsSaving(true);
      const result = await obsObwsCommands.sendConfigToObs(
        recordingConfig.connectionName,
        recordingConfig.recordingPath,
        recordingConfig.filenamePattern
      );
      
      if (result.success) {
        setTestResult(`✅ Configuration sent to OBS successfully!\n\n📁 Recording Path: ${result.data?.recording_path}\n📄 Filename Template: ${result.data?.filename_template}`);
      } else {
        setTestResult(`❌ Failed to send configuration to OBS: ${result.error}`);
      }
    } catch (error) {
      setTestResult(`❌ Failed to send configuration to OBS: ${error}`);
    } finally {
      setIsSaving(false);
    }
  };

  // Generate recording path from database
  const generateRecordingPathFromDb = async () => {
    try {
      setIsTestingPath(true);
      const result = await obsObwsCommands.generateRecordingPath(pathTestData.matchId);
      
      if (result.success && result.data) {
        setPathTestResult(result.data.full_path || 'Path generated from database successfully');
      } else {
        setPathTestResult(`Database path generation failed: ${result.error}`);
      }
    } catch (error) {
      setPathTestResult(`Database path generation failed: ${error}`);
    } finally {
      setIsTestingPath(false);
    }
  };

  // Get Windows videos folder
  const getWindowsVideosFolder = async () => {
    try {
      const result = await obsObwsCommands.getWindowsVideosFolder();
      
      if (result.success && result.data?.videos_path) {
        setRecordingConfig(prev => ({ ...prev, recordingPath: result.data.videos_path }));
        setPathTestResult(`Detected videos folder: ${result.data.videos_path}`);
      } else {
        setPathTestResult(`Failed to detect videos folder: ${result.error}`);
      }
    } catch (error) {
      setPathTestResult(`Failed to detect videos folder: ${error}`);
    }
  };

  // Automatic recording configuration state
  const [autoRecordingConfig, setAutoRecordingConfig] = useState({
    enabled: false,
    autoStopOnMatchEnd: true,
    autoStopOnWinner: true,
    stopDelaySeconds: 30,
    includeReplayBuffer: true,
  });
  const [currentSession, setCurrentSession] = useState<any>(null);

  // Load automatic recording configuration
  const loadAutoRecordingConfig = async () => {
    try {
      setIsLoadingConfig(true);
      const result = await obsObwsCommands.getAutomaticRecordingConfig();

      if (result.success && result.data) {
        setAutoRecordingConfig({
          enabled: result.data.enabled || false,
          autoStopOnMatchEnd: result.data.auto_stop_on_match_end !== false,
          autoStopOnWinner: result.data.auto_stop_on_winner !== false,
          stopDelaySeconds: result.data.stop_delay_seconds || 30,
          includeReplayBuffer: result.data.include_replay_buffer !== false,
        });
      }
    } catch (error) {
      console.error('Failed to load auto recording config:', error);
    } finally {
      setIsLoadingConfig(false);
    }
  };

  // Save automatic recording configuration
  const saveAutoRecordingConfig = async () => {
    try {
      setIsLoadingConfig(true);
      const result = await obsObwsCommands.updateAutomaticRecordingConfig({
        enabled: autoRecordingConfig.enabled,
        obs_connection_name: recordingConfig.connectionName || undefined,
        autoStopOnMatchEnd: autoRecordingConfig.autoStopOnMatchEnd,
        autoStopOnWinner: autoRecordingConfig.autoStopOnWinner,
        stopDelaySeconds: autoRecordingConfig.stopDelaySeconds,
        includeReplayBuffer: autoRecordingConfig.includeReplayBuffer,
      });

      if (result.success) {
        console.log('Auto recording config saved successfully');
      } else {
        console.error('Failed to save auto recording config:', result.error);
      }
    } catch (error) {
      console.error('Failed to save auto recording config:', error);
    } finally {
      setIsLoadingConfig(false);
    }
  };

  // Load current recording session
  const loadCurrentSession = async () => {
    try {
      const result = await obsObwsCommands.getCurrentRecordingSession();

      if (result.success) {
        setCurrentSession(result.data);
      }
    } catch (error) {
      console.error('Failed to load current session:', error);
    }
  };

  // Clear recording session
  const clearSession = async () => {
    try {
      const result = await obsObwsCommands.clearRecordingSession();

      if (result.success) {
        setCurrentSession(null);
        console.log('Recording session cleared');
      } else {
        console.error('Failed to clear session:', result.error);
      }
    } catch (error) {
      console.error('Failed to clear session:', error);
    }
  };

  // Manual recording controls
  const [manualMatchId, setManualMatchId] = useState('101');
  const [manualConnectionName, setManualConnectionName] = useState('');

  // Keep manualConnectionName in sync with selected connection to avoid mismatch
  useEffect(() => {
    if (recordingConfig.connectionName) {
      setManualConnectionName(recordingConfig.connectionName);
    }
  }, [recordingConfig.connectionName]);

  const startManualRecording = async () => {
    try {
      const result = await obsObwsCommands.startRecording(manualConnectionName);
      if (result.success) {
        console.log('Manual recording started');
        loadCurrentSession();
      } else {
        console.error('Failed to start manual recording:', result.error);
      }
    } catch (error) {
      console.error('Failed to start manual recording:', error);
    }
  };

  const stopManualRecording = async () => {
    try {
      const result = await obsObwsCommands.stopRecording(manualConnectionName);
      if (result.success) {
        console.log('Manual recording stopped');
        loadCurrentSession();
      } else {
        console.error('Failed to stop manual recording:', result.error);
      }
    } catch (error) {
      console.error('Failed to stop manual recording:', error);
    }
  };

  // Consolidated save function that saves both configurations
  const saveAllConfigurations = async () => {
    try {
      setIsSaving(true);
      
      // Save recording configuration first
      if (recordingConfig.connectionName) {
        const recordingResult = await obsObwsCommands.saveRecordingConfig({
          obs_connection_name: recordingConfig.connectionName,
          recording_root_path: recordingConfig.recordingPath,
          recording_format: recordingConfig.recordingFormat,
          recording_quality: 'high', // Default quality
          recording_bitrate: null,
          recording_resolution: null,
          replay_buffer_enabled: recordingConfig.autoStartReplayBuffer,
          replay_buffer_duration: 30, // Default 30 seconds
          auto_start_recording: recordingConfig.autoStartRecording,
          auto_start_replay_buffer: recordingConfig.autoStartReplayBuffer,
          filename_template: recordingConfig.filenamePattern,
          folder_pattern: recordingConfig.folderPattern || '{tournament}/{tournamentDay}',
          is_active: true,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        });
        
        if (!recordingResult.success) {
          setTestResult(`Failed to save recording configuration: ${recordingResult.error}`);
          return;
        }
      }
      
      // Save automatic recording configuration
      const autoResult = await obsObwsCommands.updateAutomaticRecordingConfig({
        enabled: autoRecordingConfig.enabled,
        obs_connection_name: recordingConfig.connectionName || undefined,
        autoStopOnMatchEnd: autoRecordingConfig.autoStopOnMatchEnd,
        autoStopOnWinner: autoRecordingConfig.autoStopOnWinner,
        stopDelaySeconds: autoRecordingConfig.stopDelaySeconds,
        includeReplayBuffer: autoRecordingConfig.includeReplayBuffer,
      });

      if (autoResult.success) {
        setTestResult('All configurations saved successfully!');
      } else {
        setTestResult(`Failed to save automatic recording configuration: ${autoResult.error}`);
      }
    } catch (error) {
      setTestResult(`Failed to save configurations: ${error}`);
    } finally {
      setIsSaving(false);
    }
  };

  // Load config and session on component mount
  useEffect(() => {
    loadAutoRecordingConfig();
    loadCurrentSession();
  }, []);

  // Load recording config when connection changes
  useEffect(() => {
    if (recordingConfig.connectionName) {
      loadRecordingConfig();
    }
  }, [recordingConfig.connectionName]);

  return (
    <div className="space-y-4">
      {/* OBS Recording Automatisation Section */}
      <div className="p-6 bg-gradient-to-br from-gray-900/20 to-gray-800/30 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
        <h3 className="text-lg font-semibold mb-4 text-gray-100">🎬 OBS Recording Automatisation</h3>
        
        {/* Connection Selection, Recording Path, and Recording Format in 3 columns */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
          {/* Connection Selection */}
          <div>
            <Label htmlFor="connection-select" className="block text-sm font-medium text-gray-300 mb-2">
              OBS WebSocket Connection
            </Label>
            <select
              id="connection-select"
              value={recordingConfig.connectionName}
              onChange={(e) => setRecordingConfig(prev => ({ ...prev, connectionName: e.target.value }))}
              className="w-full px-3 py-2 bg-gray-800/50 border border-gray-600/30 rounded-md text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              aria-label="Select OBS WebSocket connection"
            >
              <option value="">Select a connection...</option>
              {connections.map((conn) => (
                <option key={conn.name} value={conn.name}>
                  {conn.name} {conn.status === 'connected' ? ' (Connected)' : ''}
                </option>
              ))}
            </select>
          </div>

          {/* Recording Path */}
          <div>
            <Label htmlFor="recording-path" className="block text-sm font-medium text-gray-300 mb-2">
              Recording Path
            </Label>
            <Input
              id="recording-path"
              type="text"
              value={recordingConfig.recordingPath}
              onChange={(e) => setRecordingConfig(prev => ({ ...prev, recordingPath: e.target.value }))}
              placeholder="C:/Users/Damjan/Videos"
              className="w-full"
            />
          </div>

          {/* Recording Format */}
          <div>
            <Label htmlFor="recording-format" className="block text-sm font-medium text-gray-300 mb-2">
              Recording Format
            </Label>
            <select
              id="recording-format"
              value={recordingConfig.recordingFormat}
              onChange={(e) => setRecordingConfig(prev => ({ ...prev, recordingFormat: e.target.value }))}
              className="w-full px-3 py-2 bg-gray-800/50 border border-gray-600/30 rounded-md text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              aria-label="Select recording format"
            >
              <option value="mp4">MP4</option>
              <option value="mkv">MKV</option>
              <option value="mov">MOV</option>
              <option value="avi">AVI</option>
            </select>
          </div>
        </div>

        {/* Filename Pattern */}
        <div className="mb-4">
          <Label htmlFor="filename-pattern" className="block text-sm font-medium text-gray-300 mb-2">
            Filename Pattern
          </Label>
          <Input
            id="filename-pattern"
            type="text"
            value={recordingConfig.filenamePattern}
            onChange={(e) => setRecordingConfig(prev => ({ ...prev, filenamePattern: e.target.value }))}
            placeholder="{matchNumber}_{player1}_{player2}_{date}_{time}"
            className="w-full"
          />
          <p className="text-xs text-gray-400 mt-1">
            Available variables: {'{matchNumber}'}, {'{player1}'}, {'{player2}'}, {'{date}'}, {'{time}'}, {'{tournament}'}, {'{tournamentDay}'}
          </p>
        </div>

        {/* Folder Pattern (directory layout) */}
        <div className="mb-4">
          <Label htmlFor="folder-pattern" className="block text-sm font-medium text-gray-300 mb-2">
            Folder Pattern
          </Label>
          <Input
            id="folder-pattern"
            type="text"
            value={recordingConfig.folderPattern || ''}
            onChange={(e) => setRecordingConfig(prev => ({ ...prev, folderPattern: e.target.value }))}
            placeholder="{tournament}/{tournamentDay}"
            className="w-full"
          />
          <p className="text-xs text-gray-400 mt-1">
            The app currently sends the Recording Path directly. This pattern will be used by path generation to build the directory path.
          </p>
        </div>

        {/* Automatic Recording Settings */}
        <div className="border-t border-gray-600/30 pt-4 mb-4">
          <div className="flex items-center justify-between mb-3">
            <h4 className="text-md font-semibold text-gray-100">Automatic Recording Settings</h4>
          </div>
          
          {/* Toggles in 3 columns */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3 mb-3">
            <Toggle
              label="Enable Automatic Recording"
              checked={autoRecordingConfig.enabled}
              onChange={(e) => setAutoRecordingConfig({ ...autoRecordingConfig, enabled: e.target.checked })}
            />
            
            <Toggle
              label="Auto-start recording on match begin"
              checked={recordingConfig.autoStartRecording}
              onChange={(e) => setRecordingConfig(prev => ({ ...prev, autoStartRecording: e.target.checked }))}
            />
            
            <Toggle
              label="Auto-start replay buffer on match begin"
              checked={recordingConfig.autoStartReplayBuffer}
              onChange={(e) => setRecordingConfig(prev => ({ ...prev, autoStartReplayBuffer: e.target.checked }))}
            />
            
            <Toggle
              label="Auto Stop on Match End"
              checked={autoRecordingConfig.autoStopOnMatchEnd}
              onChange={(e) => setAutoRecordingConfig({ ...autoRecordingConfig, autoStopOnMatchEnd: e.target.checked })}
            />
            
            <Toggle
              label="Auto Stop on Winner"
              checked={autoRecordingConfig.autoStopOnWinner}
              onChange={(e) => setAutoRecordingConfig({ ...autoRecordingConfig, autoStopOnWinner: e.target.checked })}
            />
            
            <Toggle
              label="Save replay buffer on match end"
              checked={recordingConfig.saveReplayBufferOnMatchEnd}
              onChange={(e) => setRecordingConfig(prev => ({ ...prev, saveReplayBufferOnMatchEnd: e.target.checked }))}
            />
            
            <Toggle
              label="Include Replay Buffer"
              checked={autoRecordingConfig.includeReplayBuffer}
              onChange={(e) => setAutoRecordingConfig({ ...autoRecordingConfig, includeReplayBuffer: e.target.checked })}
            />
          </div>

          {/* Stop Delay */}
          <div className="flex items-center gap-3">
            <label className="text-sm font-medium text-gray-300">Stop Delay:</label>
            <Input
              type="number"
              value={autoRecordingConfig.stopDelaySeconds}
              onChange={(e) => setAutoRecordingConfig({ ...autoRecordingConfig, stopDelaySeconds: parseInt(e.target.value) || 30 })}
              placeholder="30"
              className="w-20"
            />
            <span className="text-sm text-gray-400">seconds</span>
          </div>
        </div>

        {/* Manual Recording Controls Section (Red Color) */}
        <div className="border-t border-red-600/30 pt-4 mb-4">
          <h4 className="text-md font-semibold text-red-300 mb-3">🎬 Manual Recording Controls</h4>
          
          <div className="grid grid-cols-2 gap-4 mb-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">Match ID</label>
              <Input
                value={manualMatchId}
                onChange={(e) => setManualMatchId(e.target.value)}
                placeholder="101"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">OBS Connection Name</label>
              <Input
                value={manualConnectionName}
                onChange={(e) => setManualConnectionName(e.target.value)}
                placeholder="OBS_REC"
              />
            </div>
          </div>

          <div className="flex gap-3 mb-4">
            <Button
              onClick={startManualRecording}
              disabled={!manualMatchId || !manualConnectionName}
              className="bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed"
            >
              Start Manual Recording
            </Button>
            <Button
              onClick={stopManualRecording}
              disabled={!manualConnectionName}
              className="bg-red-600 hover:bg-red-700 disabled:bg-gray-600 disabled:cursor-not-allowed"
            >
              Stop Manual Recording
            </Button>
          </div>
        </div>

        {/* Consolidated Action Buttons */}
        <div className="flex flex-wrap gap-3">
          <Button
            onClick={saveAllConfigurations}
            disabled={isSaving}
            className="bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed"
          >
            {isSaving ? 'Saving...' : 'Save Configuration'}
          </Button>
          <Button
            onClick={loadAutoRecordingConfig}
            disabled={isLoadingConfig}
            className="bg-gray-600 hover:bg-gray-700 disabled:bg-gray-600 disabled:cursor-not-allowed"
          >
            {isLoadingConfig ? 'Loading...' : 'Load Configuration'}
          </Button>
          <Button
            onClick={testRecording}
            disabled={!recordingConfig.connectionName}
            className="bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed"
          >
            Test Recording
          </Button>
          <Button
            onClick={sendConfigToObs}
            disabled={!recordingConfig.connectionName || isSaving}
            className="bg-purple-600 hover:bg-purple-700 disabled:bg-gray-600 disabled:cursor-not-allowed"
          >
            {isSaving ? 'Sending...' : 'Send Config to OBS'}
          </Button>
        </div>

        {/* Test Result */}
        {testResult && (
          <div className="mt-4 p-3 bg-gray-800/50 border border-gray-600/30 rounded-md">
            <p className="text-sm text-gray-300">{testResult}</p>
          </div>
        )}
      </div>

      {/* Path Generation Test */}
      <div className="p-6 bg-gradient-to-br from-green-900/20 to-green-800/30 backdrop-blur-sm rounded-lg border border-green-600/30 shadow-lg">
        <h3 className="text-lg font-semibold mb-4 text-gray-100">Path Generation Test</h3>
        
        {/* Test Data Input */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
          <div>
            <Label htmlFor="test-match-id" className="block text-sm font-medium text-gray-300 mb-2">
              Match ID
            </Label>
            <Input
              id="test-match-id"
              type="text"
              value={pathTestData.matchId}
              onChange={(e) => setPathTestData(prev => ({ ...prev, matchId: e.target.value }))}
              placeholder="101"
              className="w-full"
            />
          </div>
          <div>
            <Label htmlFor="test-match-number" className="block text-sm font-medium text-gray-300 mb-2">
              Match Number
            </Label>
            <Input
              id="test-match-number"
              type="text"
              value={pathTestData.matchNumber}
              onChange={(e) => setPathTestData(prev => ({ ...prev, matchNumber: e.target.value }))}
              placeholder="101"
              className="w-full"
            />
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
          <div>
            <Label htmlFor="test-tournament-name" className="block text-sm font-medium text-gray-300 mb-2">
              Tournament Name
            </Label>
            <Input
              id="test-tournament-name"
              type="text"
              value={pathTestData.tournamentName}
              onChange={(e) => setPathTestData(prev => ({ ...prev, tournamentName: e.target.value }))}
              placeholder="Test Tournament"
              className="w-full"
            />
          </div>
          <div>
            <Label htmlFor="test-tournament-day" className="block text-sm font-medium text-gray-300 mb-2">
              Tournament Day
            </Label>
            <Input
              id="test-tournament-day"
              type="text"
              value={pathTestData.tournamentDay}
              onChange={(e) => setPathTestData(prev => ({ ...prev, tournamentDay: e.target.value }))}
              placeholder="Day 1"
              className="w-full"
            />
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
          <div>
            <Label htmlFor="test-player1-name" className="block text-sm font-medium text-gray-300 mb-2">
              Player 1 Name
            </Label>
            <Input
              id="test-player1-name"
              type="text"
              value={pathTestData.player1Name}
              onChange={(e) => setPathTestData(prev => ({ ...prev, player1Name: e.target.value }))}
              placeholder="N. DESMOND"
              className="w-full"
            />
          </div>
          <div>
            <Label htmlFor="test-player1-flag" className="block text-sm font-medium text-gray-300 mb-2">
              Player 1 Flag
            </Label>
            <Input
              id="test-player1-flag"
              type="text"
              value={pathTestData.player1Flag}
              onChange={(e) => setPathTestData(prev => ({ ...prev, player1Flag: e.target.value }))}
              placeholder="MRN"
              className="w-full"
            />
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
          <div>
            <Label htmlFor="test-player2-name" className="block text-sm font-medium text-gray-300 mb-2">
              Player 2 Name
            </Label>
            <Input
              id="test-player2-name"
              type="text"
              value={pathTestData.player2Name}
              onChange={(e) => setPathTestData(prev => ({ ...prev, player2Name: e.target.value }))}
              placeholder="M. THIBAULT"
              className="w-full"
            />
          </div>
          <div>
            <Label htmlFor="test-player2-flag" className="block text-sm font-medium text-gray-300 mb-2">
              Player 2 Flag
            </Label>
            <Input
              id="test-player2-flag"
              type="text"
              value={pathTestData.player2Flag}
              onChange={(e) => setPathTestData(prev => ({ ...prev, player2Flag: e.target.value }))}
              placeholder="SUI"
              className="w-full"
            />
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex flex-wrap gap-3 mb-4">
          <Button
            onClick={testPathGeneration}
            disabled={isTestingPath}
            className="bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed"
          >
            {isTestingPath ? 'Creating...' : 'Create Test Folders'}
          </Button>
          <Button
            onClick={generateRecordingPathFromDb}
            disabled={isTestingPath}
            className="bg-purple-600 hover:bg-purple-700 disabled:bg-gray-600 disabled:cursor-not-allowed"
          >
            {isTestingPath ? 'Generating...' : 'Generate from Database'}
          </Button>
          <Button
            onClick={getWindowsVideosFolder}
            className="bg-blue-600 hover:bg-blue-700"
          >
            Detect Videos Folder
          </Button>
        </div>

        {/* Path Test Result */}
        {pathTestResult && (
          <div className="p-3 bg-gray-800/50 border border-gray-600/30 rounded-md">
            <p className="text-sm text-gray-300 whitespace-pre-line">{pathTestResult}</p>
          </div>
        )}
      </div>

      {/* Current Recording Session Section */}
      <div className="bg-gray-800 rounded-lg p-4 mb-4">
        <h3 className="text-lg font-semibold text-white mb-4">🎬 Current Recording Session</h3>
        
        {currentSession ? (
          <div className="bg-gray-900 rounded p-3 mb-4">
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span className="text-gray-400">Match ID:</span>
                <span className="text-white ml-2">{currentSession.match_id}</span>
              </div>
              <div>
                <span className="text-gray-400">State:</span>
                <span className="text-white ml-2">{currentSession.state}</span>
              </div>
              <div>
                <span className="text-gray-400">Tournament:</span>
                <span className="text-white ml-2">{currentSession.tournament_name || 'None'}</span>
              </div>
              <div>
                <span className="text-gray-400">Tournament Day:</span>
                <span className="text-white ml-2">{currentSession.tournament_day || 'None'}</span>
              </div>
              <div>
                <span className="text-gray-400">Match Number:</span>
                <span className="text-white ml-2">{currentSession.match_number || 'None'}</span>
              </div>
              <div>
                <span className="text-gray-400">OBS Connection:</span>
                <span className="text-white ml-2">{currentSession.obs_connection_name || 'None'}</span>
              </div>
              <div>
                <span className="text-gray-400">Player 1:</span>
                <span className="text-white ml-2">{currentSession.player1_name || 'None'} ({currentSession.player1_flag || 'None'})</span>
              </div>
              <div>
                <span className="text-gray-400">Player 2:</span>
                <span className="text-white ml-2">{currentSession.player2_name || 'None'} ({currentSession.player2_flag || 'None'})</span>
              </div>
              <div className="col-span-2">
                <span className="text-gray-400">Recording Path:</span>
                <span className="text-white ml-2">{currentSession.recording_path || 'None'}</span>
              </div>
              <div className="col-span-2">
                <span className="text-gray-400">Filename:</span>
                <span className="text-white ml-2">{currentSession.recording_filename || 'None'}</span>
              </div>
            </div>
          </div>
        ) : (
          <div className="bg-gray-900 rounded p-3 mb-4 text-gray-400">
            No active recording session
          </div>
        )}

        <div className="flex gap-3">
          <Button
            onClick={loadCurrentSession}
            className="bg-blue-600 hover:bg-blue-700"
          >
            Refresh Session
          </Button>
          <Button
            onClick={clearSession}
            className="bg-red-600 hover:bg-red-700"
          >
            Clear Session
          </Button>
        </div>
      </div>
    </div>
  );
};

export default ObsIntegrationPanel;
