import React, { useState, useEffect } from 'react';
import Button from '../atoms/Button';
import Toggle from '../atoms/Toggle';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import StatusDot from '../atoms/StatusDot';
import Icon from '../atoms/Icon';
import SelfTestPanel from './SelfTestPanel';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';

// Use the proper Tauri v2 invoke function with fallback
const invoke = async (command: string, args?: any) => {
  try {
    return await tauriInvoke(command, args);
  } catch (error) {
    if (typeof window !== 'undefined' && (window as any).__TAURI__ && (window as any).__TAURI__.core) {
      return await (window as any).__TAURI__.core.invoke(command, args);
    }
    throw new Error('Tauri v2 core module not available');
  }
};

interface SimulationStatus {
  isRunning: boolean;
  isConnected: boolean;
  currentScenario: string;
  currentMode: string;
  eventsSent: number;
  lastEvent: string;
  automatedScenarios?: AutomatedScenario[];
}

interface AutomatedScenario {
  name: string;
  display_name: string;
  description: string;
  match_count: number;
  estimated_duration: number;
}

interface SimulationPanelProps {
  className?: string;
}

const SimulationPanel: React.FC<SimulationPanelProps> = ({ className = '' }) => {
  const [status, setStatus] = useState<SimulationStatus>({
    isRunning: false,
    isConnected: false,
    currentScenario: 'None',
    currentMode: 'None',
    eventsSent: 0,
    lastEvent: 'None'
  });
  
  const [selectedScenario, setSelectedScenario] = useState('basic');
  const [selectedMode, setSelectedMode] = useState('demo');
  const [duration, setDuration] = useState(30);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string>('');
  const [success, setSuccess] = useState<string>('');
  
  // Automated simulation state
                const [showAutomated, setShowAutomated] = useState(false);
              const [showSelfTest, setShowSelfTest] = useState(false);
              const [selectedAutomatedScenario, setSelectedAutomatedScenario] = useState('');
              const [automatedScenarios, setAutomatedScenarios] = useState<AutomatedScenario[]>([]);
              const [progress, setProgress] = useState({ current: 0, total: 0 });

  // Load simulation status
  const loadStatus = async () => {
    try {
      const result = await invoke('simulation_get_detailed_status');
      if (result.success) {
        setStatus(result.data);
        if (result.data.automatedScenarios) {
          setAutomatedScenarios(result.data.automatedScenarios);
        }
      }
    } catch (error) {
      console.error('Failed to load simulation status:', error);
    }
  };

  // Load automated scenarios
  const loadAutomatedScenarios = async () => {
    try {
      const result = await invoke('simulation_get_scenarios');
      if (result.success) {
        setAutomatedScenarios(result.data);
        if (result.data.length > 0 && !selectedAutomatedScenario) {
          setSelectedAutomatedScenario(result.data[0].name);
        }
      }
    } catch (error) {
      console.error('Failed to load automated scenarios:', error);
    }
  };

  // Start simulation
  const startSimulation = async () => {
    try {
      setIsLoading(true);
      setError('');
      setSuccess('');

      const result = await invoke('simulation_start', {
        mode: selectedMode,
        scenario: selectedScenario,
        duration: duration
      });

      if (result.success) {
        setSuccess('Simulation started successfully!');
        await loadStatus();
      } else {
        setError(result.error || 'Failed to start simulation');
      }
    } catch (error) {
      setError(`Failed to start simulation: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  // Start automated simulation
  const startAutomatedSimulation = async () => {
    try {
      setIsLoading(true);
      setError('');
      setSuccess('');

      const result = await invoke('simulation_run_automated', {
        scenario_name: selectedAutomatedScenario
      });

      if (result.success) {
        setSuccess(`Automated ${selectedAutomatedScenario} simulation started successfully!`);
        await loadStatus();
      } else {
        setError(result.error || 'Failed to start automated simulation');
      }
    } catch (error) {
      setError(`Failed to start automated simulation: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  // Stop simulation
  const stopSimulation = async () => {
    try {
      setIsLoading(true);
      setError('');
      setSuccess('');

      const result = await invoke('simulation_stop');

      if (result.success) {
        setSuccess('Simulation stopped successfully!');
        await loadStatus();
      } else {
        setError(result.error || 'Failed to stop simulation');
      }
    } catch (error) {
      setError(`Failed to stop simulation: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  // Send manual event
  const sendManualEvent = async (eventType: string, params: any) => {
    try {
      const result = await invoke('simulation_send_event', {
        eventType,
        params
      });

      if (result.success) {
        setSuccess(`${eventType} event sent successfully!`);
        await loadStatus();
      } else {
        setError(result.error || `Failed to send ${eventType} event`);
      }
    } catch (error) {
      setError(`Failed to send ${eventType} event: ${error}`);
    }
  };

  // Load initial data
  useEffect(() => {
    loadStatus();
    loadAutomatedScenarios();
  }, []);

  // Auto-refresh status
  useEffect(() => {
    const interval = setInterval(() => {
      loadStatus();
    }, 2000);

    return () => clearInterval(interval);
  }, []);

  // Clear messages after 5 seconds
  useEffect(() => {
    if (success || error) {
      const timer = setTimeout(() => {
        setSuccess('');
        setError('');
      }, 5000);
      return () => clearTimeout(timer);
    }
  }, [success, error]);

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-3">
          <Icon name="🤖" className="w-6 h-6 text-blue-400" />
          <h3 className="text-lg font-semibold text-gray-200">Simulation Control</h3>
        </div>
        <div className="flex items-center space-x-2">
          <StatusDot
            color={status.isConnected ? 'bg-green-500' : 'bg-red-500'}
            size="w-3 h-3"
          />
          <span className="text-xs text-gray-400">
            {status.isConnected ? 'Connected' : 'Disconnected'}
          </span>
        </div>
      </div>

      {/* Status Messages */}
      {error && (
        <div className="bg-red-900/20 border border-red-500/50 rounded-lg p-3">
          <p className="text-red-400 text-sm">{error}</p>
        </div>
      )}
      {success && (
        <div className="bg-green-900/20 border border-green-500/50 rounded-lg p-3">
          <p className="text-green-400 text-sm">{success}</p>
        </div>
      )}

                        {/* Mode Toggles */}
                  <div className="space-y-3">
                    <div className="flex items-center justify-between">
                      <Label>Automated Simulation</Label>
                      <Toggle
                        checked={showAutomated}
                        onChange={(e) => setShowAutomated(e.target.checked)}
                        disabled={status.isRunning}
                      />
                    </div>
                    <div className="flex items-center justify-between">
                      <Label>System Self-Test</Label>
                      <Toggle
                        checked={showSelfTest}
                        onChange={(e) => setShowSelfTest(e.target.checked)}
                        disabled={status.isRunning}
                      />
                    </div>
                  </div>

      {showAutomated ? (
        /* Automated Simulation Panel */
        <div className="space-y-4">
          {/* Scenario Selection */}
          <div>
            <Label>Automated Scenario</Label>
            <select
              value={selectedAutomatedScenario}
              onChange={(e) => setSelectedAutomatedScenario(e.target.value)}
              disabled={status.isRunning || isLoading}
              className="w-full bg-gray-800 border border-gray-600 rounded-lg px-3 py-2 text-gray-200 focus:border-blue-500 focus:outline-none"
              aria-label="Select automated simulation scenario"
            >
              {automatedScenarios.map((scenario) => (
                <option key={scenario.name} value={scenario.name}>
                  {scenario.display_name} ({scenario.match_count} matches, ~{Math.round(scenario.estimated_duration / 60)}min)
                </option>
              ))}
            </select>
            {selectedAutomatedScenario && (
              <p className="text-xs text-gray-400 mt-1">
                {automatedScenarios.find(s => s.name === selectedAutomatedScenario)?.description}
              </p>
            )}
          </div>

          {/* Progress Bar */}
          {status.isRunning && progress.total > 0 && (
            <div>
              <div className="flex justify-between text-xs text-gray-400 mb-1">
                <span>Progress</span>
                <span>{progress.current}/{progress.total} matches</span>
              </div>
              <div className="w-full bg-gray-700 rounded-full h-2">
                <div
                  className="bg-blue-500 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${(progress.current / progress.total) * 100}%` }}
                />
              </div>
            </div>
          )}

          {/* Control Buttons */}
          <div className="flex space-x-2">
            <Button
              variant="primary"
              size="sm"
              onClick={startAutomatedSimulation}
              disabled={status.isRunning || isLoading || !selectedAutomatedScenario}
              className="flex-1"
            >
              {isLoading ? (
                <div className="flex items-center space-x-2">
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                  <span>Starting...</span>
                </div>
              ) : (
                <span>Start Automated</span>
              )}
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={stopSimulation}
              disabled={!status.isRunning || isLoading}
            >
              Stop
            </Button>
                                </div>
                    </div>
                  ) : showSelfTest ? (
                    /* Self-Test Panel */
                    <SelfTestPanel />
                  ) : (
                    /* Manual Simulation Panel */
                    <div className="space-y-4">
          {/* Mode Selection */}
          <div>
            <Label>Simulation Mode</Label>
            <select
              value={selectedMode}
              onChange={(e) => setSelectedMode(e.target.value)}
              disabled={status.isRunning || isLoading}
              className="w-full bg-gray-800 border border-gray-600 rounded-lg px-3 py-2 text-gray-200 focus:border-blue-500 focus:outline-none"
              aria-label="Select simulation mode"
            >
              <option value="demo">Demo</option>
              <option value="random">Random Events</option>
              <option value="interactive">Interactive</option>
            </select>
          </div>

          {/* Scenario Selection */}
          <div>
            <Label>Scenario</Label>
            <select
              value={selectedScenario}
              onChange={(e) => setSelectedScenario(e.target.value)}
              disabled={status.isRunning || isLoading}
              className="w-full bg-gray-800 border border-gray-600 rounded-lg px-3 py-2 text-gray-200 focus:border-blue-500 focus:outline-none"
              aria-label="Select simulation scenario"
            >
              <option value="basic">Basic Match</option>
              <option value="championship">Championship</option>
              <option value="training">Training</option>
            </select>
          </div>

          {/* Duration Input */}
          <div>
            <Label>Duration (seconds)</Label>
            <Input
              type="number"
              value={duration}
              onChange={(e) => setDuration(parseInt(e.target.value) || 30)}
              disabled={status.isRunning || isLoading}
              min={10}
              max={600}
            />
          </div>

          {/* Control Buttons */}
          <div className="flex space-x-2">
            <Button
              variant="primary"
              size="sm"
              onClick={startSimulation}
              disabled={status.isRunning || isLoading}
              className="flex-1"
            >
              {isLoading ? (
                <div className="flex items-center space-x-2">
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                  <span>Starting...</span>
                </div>
              ) : (
                <span>Start Simulation</span>
              )}
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={stopSimulation}
              disabled={!status.isRunning || isLoading}
            >
              Stop
            </Button>
          </div>

          {/* Manual Event Buttons */}
          <div>
            <Label>Manual Events</Label>
            <div className="grid grid-cols-2 gap-2 mt-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => sendManualEvent('point', { athlete: 1, point_type: 1 })}
                disabled={status.isRunning || isLoading}
              >
                Blue Punch
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => sendManualEvent('point', { athlete: 2, point_type: 3 })}
                disabled={status.isRunning || isLoading}
              >
                Red Head Kick
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => sendManualEvent('warning', { athlete: 1 })}
                disabled={status.isRunning || isLoading}
              >
                Blue Warning
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => sendManualEvent('injury', { athlete: 1, duration: 60 })}
                disabled={status.isRunning || isLoading}
              >
                Injury Time
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Current Status */}
      <div className="bg-gray-800/50 rounded-lg p-3">
        <h4 className="text-sm font-medium text-gray-300 mb-2">Current Status</h4>
        <div className="space-y-1 text-xs text-gray-400">
          <div>Running: {status.isRunning ? 'Yes' : 'No'}</div>
          <div>Scenario: {status.currentScenario}</div>
          <div>Mode: {status.currentMode}</div>
          <div>Events Sent: {status.eventsSent}</div>
          <div>Last Event: {status.lastEvent}</div>
        </div>
      </div>
    </div>
  );
};

export default SimulationPanel; 