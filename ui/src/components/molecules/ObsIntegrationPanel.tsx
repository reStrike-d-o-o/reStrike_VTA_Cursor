import React, { useState, useEffect } from 'react';
import Toggle from '../atoms/Toggle';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import StatusDot from '../atoms/StatusDot';
import { useObsStore } from '../../stores/obsStore';
import { obsObwsCommands } from '../../utils/tauriCommandsObws';

// OBS Integration Settings interface
interface ObsIntegrationSettings {
  autoConnectOnStartup: boolean;
  showStatusInOverlay: boolean;
  autoRecordOnClipPlay: boolean;
  saveReplayBufferOnClipCreation: boolean;
}

// Recording Configuration interface
interface RecordingConfig {
  connectionName: string;
  recordingPath: string;
  recordingFormat: string;
  filenamePattern: string;
  autoStartRecording: boolean;
  autoStartReplayBuffer: boolean;
  saveReplayBufferOnMatchEnd: boolean;
}

const ObsIntegrationPanel: React.FC = () => {
  // OBS Integration Settings state
  const [obsIntegrationSettings, setObsIntegrationSettings] = useState<ObsIntegrationSettings>({
    autoConnectOnStartup: true,
    showStatusInOverlay: true,
    autoRecordOnClipPlay: false,
    saveReplayBufferOnClipCreation: true,
  });

  // Recording Configuration state
  const [recordingConfig, setRecordingConfig] = useState<RecordingConfig>({
    connectionName: '',
    recordingPath: 'C:/Users/Damjan/Videos',
    recordingFormat: 'mp4',
    filenamePattern: '{matchNumber}_{player1}_{player2}_{date}_{time}',
    autoStartRecording: true,
    autoStartReplayBuffer: true,
    saveReplayBufferOnMatchEnd: true,
  });

  const [isLoadingSettings, setIsLoadingSettings] = useState(false);
  const [isLoadingConfig, setIsLoadingConfig] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [testResult, setTestResult] = useState<string>('');

  // Get OBS connections from store
  const { connections } = useObsStore();

  // Load OBS Integration settings
  const loadObsIntegrationSettings = async () => {
    try {
      setIsLoadingSettings(true);
      // TODO: Load from configuration system
      console.log('Loading OBS integration settings...');
    } catch (error) {
      console.error('Failed to load OBS integration settings:', error);
    } finally {
      setIsLoadingSettings(false);
    }
  };

  // Save OBS Integration settings
  const saveObsIntegrationSettings = async (newSettings: ObsIntegrationSettings) => {
    try {
      setIsLoadingSettings(true);
      // TODO: Save to configuration system
      console.log('Saving OBS integration settings:', newSettings);
      setObsIntegrationSettings(newSettings);
    } catch (error) {
      console.error('Failed to save OBS integration settings:', error);
    } finally {
      setIsLoadingSettings(false);
    }
  };

  // Handle OBS setting change
  const handleObsSettingChange = async (setting: keyof ObsIntegrationSettings, value: boolean) => {
    const newSettings = { ...obsIntegrationSettings, [setting]: value };
    await saveObsIntegrationSettings(newSettings);
  };

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
      setTestResult('Please select a connection first');
      return;
    }

    try {
      setIsSaving(true);
      const result = await obsObwsCommands.saveRecordingConfig({
        connection_name: recordingConfig.connectionName,
        recording_path: recordingConfig.recordingPath,
        recording_format: recordingConfig.recordingFormat,
        filename_pattern: recordingConfig.filenamePattern,
        auto_start_recording: recordingConfig.autoStartRecording,
        auto_start_replay_buffer: recordingConfig.autoStartReplayBuffer,
        save_replay_buffer_on_match_end: recordingConfig.saveReplayBufferOnMatchEnd,
      });

      if (result.success) {
        setTestResult('Recording configuration saved successfully!');
      } else {
        setTestResult(`Failed to save configuration: ${result.error}`);
      }
    } catch (error) {
      setTestResult(`Error saving configuration: ${error}`);
    } finally {
      setIsSaving(false);
    }
  };

  // Test recording functionality
  const testRecording = async () => {
    if (!recordingConfig.connectionName) {
      setTestResult('Please select a connection first');
      return;
    }

    try {
      setTestResult('Testing recording functionality...');
      
      // Test start recording
      const recordResult = await obsObwsCommands.startRecording(recordingConfig.connectionName);
      if (!recordResult.success) {
        setTestResult(`Recording test failed: ${recordResult.error}`);
        return;
      }

      // Test start replay buffer
      const replayResult = await obsObwsCommands.startReplayBuffer(recordingConfig.connectionName);
      if (!replayResult.success) {
        setTestResult(`Replay buffer test failed: ${replayResult.error}`);
        return;
      }

      setTestResult('Recording test successful! Recording and replay buffer started.');
    } catch (error) {
      setTestResult(`Test failed: ${error}`);
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
  const [pathTestResult, setPathTestResult] = useState<string>('');
  const [isTestingPath, setIsTestingPath] = useState(false);

  // Test path generation with sample data
  const testPathGeneration = async () => {
    try {
      setIsTestingPath(true);
      setPathTestResult('Testing path generation...');
      
      const result = await obsObwsCommands.testPathGeneration(pathTestData);
      
      if (result.success && result.data) {
        const data = result.data;
        setPathTestResult(`Path generation successful!
          
Full Path: ${data.full_path}
Directory: ${data.directory}
Filename: ${data.filename}
Tournament: ${data.tournament_name || 'None'}
Tournament Day: ${data.tournament_day || 'None'}
Match Number: ${data.match_number || 'None'}`);
      } else {
        setPathTestResult(`Path generation failed: ${result.error}`);
      }
    } catch (error) {
      setPathTestResult(`Test failed: ${error}`);
    } finally {
      setIsTestingPath(false);
    }
  };

  // Generate recording path from database
  const generateRecordingPathFromDb = async () => {
    try {
      setIsTestingPath(true);
      setPathTestResult('Generating recording path from database...');
      
      const result = await obsObwsCommands.generateRecordingPath(pathTestData.matchId);
      
      if (result.success && result.data) {
        const data = result.data;
        setPathTestResult(`Database-driven path generation successful!
          
Full Path: ${data.full_path}
Directory: ${data.directory}
Filename: ${data.filename}
Tournament: ${data.tournament_name || 'None'}
Tournament Day: ${data.tournament_day || 'None'}
Match Number: ${data.match_number || 'None'}
Player 1: ${data.player1_name || 'None'} (${data.player1_flag || 'None'})
Player 2: ${data.player2_name || 'None'} (${data.player2_flag || 'None'})`);
      } else {
        setPathTestResult(`Database-driven path generation failed: ${result.error}`);
      }
    } catch (error) {
      setPathTestResult(`Database generation failed: ${error}`);
    } finally {
      setIsTestingPath(false);
    }
  };

  // Get Windows Videos folder
  const getWindowsVideosFolder = async () => {
    try {
      const result = await obsObwsCommands.getWindowsVideosFolder();
      
      if (result.success && result.data) {
        const data = result.data;
        setPathTestResult(`Windows Videos folder detected:
          
Path: ${data.videos_path}
Exists: ${data.exists ? 'Yes' : 'No'}`);
      } else {
        setPathTestResult(`Failed to detect Videos folder: ${result.error}`);
      }
    } catch (error) {
      setPathTestResult(`Error: ${error}`);
    }
  };

  // Load settings on component mount
  useEffect(() => {
    loadObsIntegrationSettings();
  }, []);

  // Load recording config when connection changes
  useEffect(() => {
    if (recordingConfig.connectionName) {
      loadRecordingConfig();
    }
  }, [recordingConfig.connectionName]);

  return (
    <div className="space-y-6">
      {/* OBS Integration Settings */}
      <div className="p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
        <h3 className="text-lg font-semibold mb-4 text-gray-100">OBS Integration Settings</h3>
        {isLoadingSettings ? (
          <div className="text-sm text-gray-400">Loading settings...</div>
        ) : (
          <div className="space-y-4">
            <Toggle
              id="obs-auto-connect"
              checked={obsIntegrationSettings.autoConnectOnStartup}
              onChange={(e) => handleObsSettingChange('autoConnectOnStartup', e.target.checked)}
              label="Auto-connect to OBS on startup"
              labelPosition="right"
            />
            <Toggle
              id="obs-show-status"
              checked={obsIntegrationSettings.showStatusInOverlay}
              onChange={(e) => handleObsSettingChange('showStatusInOverlay', e.target.checked)}
              label="Show OBS status in overlay"
              labelPosition="right"
            />
            <Toggle
              id="obs-auto-record"
              checked={obsIntegrationSettings.autoRecordOnClipPlay}
              onChange={(e) => handleObsSettingChange('autoRecordOnClipPlay', e.target.checked)}
              label="Auto-record when playing clips"
              labelPosition="right"
            />
            <Toggle
              id="obs-save-replay"
              checked={obsIntegrationSettings.saveReplayBufferOnClipCreation}
              onChange={(e) => handleObsSettingChange('saveReplayBufferOnClipCreation', e.target.checked)}
              label="Save replay buffer on clip creation"
              labelPosition="right"
            />
          </div>
        )}
      </div>

      {/* Recording Configuration */}
      <div className="p-6 bg-gradient-to-br from-blue-900/20 to-blue-800/30 backdrop-blur-sm rounded-lg border border-blue-600/30 shadow-lg">
        <h3 className="text-lg font-semibold mb-4 text-gray-100">Recording Configuration</h3>
        
        {/* Connection Selection */}
        <div className="mb-6">
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

        {/* Recording Path Settings */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
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
        <div className="mb-6">
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

        {/* Recording Options */}
        <div className="space-y-4 mb-6">
          <Toggle
            id="auto-start-recording"
            checked={recordingConfig.autoStartRecording}
            onChange={(e) => setRecordingConfig(prev => ({ ...prev, autoStartRecording: e.target.checked }))}
            label="Auto-start recording on match begin"
            labelPosition="right"
          />
          <Toggle
            id="auto-start-replay-buffer"
            checked={recordingConfig.autoStartReplayBuffer}
            onChange={(e) => setRecordingConfig(prev => ({ ...prev, autoStartReplayBuffer: e.target.checked }))}
            label="Auto-start replay buffer on match begin"
            labelPosition="right"
          />
          <Toggle
            id="save-replay-buffer-on-match-end"
            checked={recordingConfig.saveReplayBufferOnMatchEnd}
            onChange={(e) => setRecordingConfig(prev => ({ ...prev, saveReplayBufferOnMatchEnd: e.target.checked }))}
            label="Save replay buffer on match end"
            labelPosition="right"
          />
        </div>

        {/* Action Buttons */}
        <div className="flex flex-wrap gap-3">
          <Button
            onClick={saveRecordingConfig}
            disabled={isSaving || !recordingConfig.connectionName}
            className="bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed"
          >
            {isSaving ? 'Saving...' : 'Save Configuration'}
          </Button>
          <Button
            onClick={testRecording}
            disabled={!recordingConfig.connectionName}
            className="bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed"
          >
            Test Recording
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
            {isTestingPath ? 'Testing...' : 'Test Path Generation'}
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
    </div>
  );
};

export default ObsIntegrationPanel;
