import React from 'react';
import Toggle from '../../atoms/Toggle';
import Label from '../../atoms/Label';
import { useSimulationStore } from '../../../stores/simulationStore';

const SimulationModeTabs: React.FC = () => {
  const showAutomated = useSimulationStore((s) => s.showAutomated);
  const setShowAutomated = useSimulationStore((s) => s.setShowAutomated);
  const showSelfTest = useSimulationStore((s) => s.showSelfTest);
  const setShowSelfTest = useSimulationStore((s) => s.setShowSelfTest);
  const showArcade = useSimulationStore((s) => s.showArcade);
  const setShowArcade = useSimulationStore((s) => s.setShowArcade);
  const isRunning = useSimulationStore((s) => s.status.isRunning);
  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <Label>Automated Simulation</Label>
        <Toggle label="" checked={showAutomated} onChange={(e) => setShowAutomated(e.target.checked)} disabled={isRunning} />
      </div>
      <div className="flex items-center justify-between">
        <Label>System Self-Test</Label>
        <Toggle label="" checked={showSelfTest} onChange={(e) => setShowSelfTest(e.target.checked)} disabled={isRunning} />
      </div>
      <div className="flex items-center justify-between">
        <Label>Arcade Mode (Keyboard)</Label>
        <Toggle label="" checked={showArcade} onChange={(e) => setShowArcade(e.target.checked)} disabled={isRunning} />
      </div>
    </div>
  );
};

export default SimulationModeTabs;


