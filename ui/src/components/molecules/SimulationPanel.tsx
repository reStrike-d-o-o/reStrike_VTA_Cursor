import React, { useState, useEffect } from 'react';
import Button from '../atoms/Button';
import Toggle from '../atoms/Toggle';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import StatusDot from '../atoms/StatusDot';
import Icon from '../atoms/Icon';
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

  // Load simulation status
  const loadStatus = async () => {
    try {
      const result = await invoke('simulation_get_status');
      if (result.success) {
        setStatus(result.data);
      }
    } catch (error) {
      console.error('Failed to load simulation status:', error);
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
      setError('');
      setSuccess('');

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

  // Load status on component mount
  useEffect(() => {
    loadStatus();
    const interval = setInterval(loadStatus, 2000); // Update every 2 seconds
    return () => clearInterval(interval);
  }, []);

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Status Section */}
      <div className="bg-gray-800/50 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-gray-100 mb-4">Simulation Status</h3>
        
        <div className="grid grid-cols-2 gap-4 mb-4">
          <div className="flex items-center space-x-2">
            <StatusDot 
              color={status.isRunning ? 'bg-green-500' : 'bg-red-500'} 
              size="w-3 h-3" 
            />
            <span className="text-sm text-gray-300">
              {status.isRunning ? 'Running' : 'Stopped'}
            </span>
          </div>
          
          <div className="flex items-center space-x-2">
            <StatusDot 
              color={status.isConnected ? 'bg-green-500' : 'bg-red-500'} 
              size="w-3 h-3" 
            />
            <span className="text-sm text-gray-300">
              {status.isConnected ? 'Connected' : 'Disconnected'}
            </span>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-4 text-sm">
          <div>
            <span className="text-gray-400">Scenario:</span>
            <span className="text-gray-200 ml-2">{status.currentScenario}</span>
          </div>
          <div>
            <span className="text-gray-400">Mode:</span>
            <span className="text-gray-200 ml-2">{status.currentMode}</span>
          </div>
          <div>
            <span className="text-gray-400">Events Sent:</span>
            <span className="text-gray-200 ml-2">{status.eventsSent}</span>
          </div>
          <div>
            <span className="text-gray-400">Last Event:</span>
            <span className="text-gray-200 ml-2">{status.lastEvent}</span>
          </div>
        </div>
      </div>

      {/* Control Section */}
      <div className="bg-gray-800/50 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-gray-100 mb-4">Simulation Control</h3>
        
        <div className="space-y-4">
          {/* Scenario Selection */}
          <div>
            <Label htmlFor="scenario" className="text-sm text-gray-300 mb-2 block">
              Scenario
            </Label>
            <select
              id="scenario"
              value={selectedScenario}
              onChange={(e) => setSelectedScenario(e.target.value)}
              className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
              aria-label="Select simulation scenario"
            >
              <option value="basic">Basic Match</option>
              <option value="championship">Championship Match</option>
              <option value="training">Training Match</option>
            </select>
          </div>

          {/* Mode Selection */}
          <div>
            <Label htmlFor="mode" className="text-sm text-gray-300 mb-2 block">
              Mode
            </Label>
            <select
              id="mode"
              value={selectedMode}
              onChange={(e) => setSelectedMode(e.target.value)}
              className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
              aria-label="Select simulation mode"
            >
              <option value="demo">Demo</option>
              <option value="random">Random Events</option>
              <option value="interactive">Interactive</option>
            </select>
          </div>

          {/* Duration (for demo/random modes) */}
          {selectedMode !== 'interactive' && (
            <div>
              <Label htmlFor="duration" className="text-sm text-gray-300 mb-2 block">
                Duration (seconds)
              </Label>
              <Input
                id="duration"
                type="number"
                value={duration}
                onChange={(e) => setDuration(parseInt(e.target.value) || 30)}
                min="10"
                max="600"
                className="w-full"
              />
            </div>
          )}

          {/* Control Buttons */}
          <div className="flex space-x-3 pt-2">
            <Button
              onClick={startSimulation}
              disabled={isLoading || status.isRunning}
              className="flex-1"
            >
              {isLoading ? 'Starting...' : 'Start Simulation'}
            </Button>
            
            <Button
              onClick={stopSimulation}
              disabled={isLoading || !status.isRunning}
              variant="secondary"
              className="flex-1"
            >
              {isLoading ? 'Stopping...' : 'Stop Simulation'}
            </Button>
          </div>
        </div>
      </div>

      {/* Manual Events Section */}
      <div className="bg-gray-800/50 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-gray-100 mb-4">Manual Events</h3>
        <p className="text-sm text-gray-300 mb-4">
          Send individual events manually (requires simulation to be running).
        </p>
        
        <div className="grid grid-cols-2 gap-3">
          <Button
            onClick={() => sendManualEvent('point', { athlete: 1, pointType: 1 })}
            disabled={!status.isRunning}
            size="sm"
            variant="outline"
          >
            Blue Punch
          </Button>
          
          <Button
            onClick={() => sendManualEvent('point', { athlete: 2, pointType: 3 })}
            disabled={!status.isRunning}
            size="sm"
            variant="outline"
          >
            Red Head Kick
          </Button>
          
          <Button
            onClick={() => sendManualEvent('warning', { athlete: 1 })}
            disabled={!status.isRunning}
            size="sm"
            variant="outline"
          >
            Blue Warning
          </Button>
          
          <Button
            onClick={() => sendManualEvent('warning', { athlete: 2 })}
            disabled={!status.isRunning}
            size="sm"
            variant="outline"
          >
            Red Warning
          </Button>
          
          <Button
            onClick={() => sendManualEvent('injury', { athlete: 1, duration: 30 })}
            disabled={!status.isRunning}
            size="sm"
            variant="outline"
          >
            Blue Injury
          </Button>
          
          <Button
            onClick={() => sendManualEvent('injury', { athlete: 2, duration: 30 })}
            disabled={!status.isRunning}
            size="sm"
            variant="outline"
          >
            Red Injury
          </Button>
        </div>
      </div>

      {/* Messages */}
      {error && (
        <div className="bg-red-900/20 border border-red-500/50 rounded-lg p-4">
          <div className="flex items-center space-x-2">
            <Icon name="alert-circle" className="text-red-400" />
            <span className="text-red-300">{error}</span>
          </div>
        </div>
      )}
      
      {success && (
        <div className="bg-green-900/20 border border-green-500/50 rounded-lg p-4">
          <div className="flex items-center space-x-2">
            <Icon name="check-circle" className="text-green-400" />
            <span className="text-green-300">{success}</span>
          </div>
        </div>
      )}

      {/* Information */}
      <div className="bg-blue-900/20 border border-blue-500/50 rounded-lg p-4">
        <h4 className="text-sm font-medium text-blue-200 mb-2">About Simulation</h4>
        <p className="text-xs text-blue-300">
          The tkStrike Hardware Simulator sends realistic PSS v2.3 protocol events to test reStrikeVTA functionality. 
          Events are sent to UDP port 8888 and will appear in the Event Table and Scoreboard Overlay.
        </p>
      </div>
    </div>
  );
};

export default SimulationPanel; 