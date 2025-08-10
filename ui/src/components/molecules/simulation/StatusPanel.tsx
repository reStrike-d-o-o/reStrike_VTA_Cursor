import React from 'react';
import { useSimulationStore } from '../../../stores/simulationStore';
import Button from '../../atoms/Button';

const StatusPanel: React.FC = () => {
  const { status, error, success, retry, installDependencies, loading } = useSimulationStore();

  const isEnvErr = (msg: string) =>
    !!msg && (
      msg.includes('Simulation environment error') ||
      msg.includes('PythonNotFound') ||
      msg.includes('PythonVersionTooLow') ||
      msg.includes('PipInstallFailed') ||
      msg.includes('DependencyCheckFailed') ||
      msg.includes('SimulationPathNotFound')
    );

  return (
    <div className="space-y-3">
      {error && (
        <div className="bg-red-900/20 border border-red-500/50 p-3">
          <p className="text-red-400 text-sm mb-2">{error}</p>
          {isEnvErr(error) && (
            <div className="flex gap-2">
              <Button variant="outline" size="sm" onClick={retry} disabled={loading}>Retry</Button>
              {(error.includes('DependencyCheckFailed') || error.includes('PipInstallFailed')) && (
                <Button variant="outline" size="sm" onClick={installDependencies} disabled={loading}>
                  {loading ? 'Installing...' : 'Install Dependencies'}
                </Button>
              )}
            </div>
          )}
        </div>
      )}
      {success && (
        <div className="bg-green-900/20 border border-green-500/50 p-3 text-green-400 text-sm">{success}</div>
      )}

      <div className="theme-surface-2 p-3">
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

export default StatusPanel;


