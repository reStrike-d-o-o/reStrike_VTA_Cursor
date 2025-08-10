import React from 'react';
import StatusRow from '../../atoms/StatusRow';
import { useSimulationStore } from '../../../stores/simulationStore';

const SimulationHeader: React.FC = () => {
  const status = useSimulationStore((s) => s.status);
  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
      <StatusRow label={<span>Connection</span>} status={status.isConnected ? 'ok' : 'error'} right={status.isConnected ? 'Connected' : 'Disconnected'} />
      <StatusRow label={<span>Mode</span>} status={status.isRunning ? 'ok' : 'warn'} right={status.currentMode || '—'} />
      <StatusRow label={<span>Scenario</span>} status={status.isRunning ? 'ok' : 'unknown'} right={status.currentScenario || '—'} />
    </div>
  );
};

export default SimulationHeader;


