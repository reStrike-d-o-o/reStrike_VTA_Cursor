import React, { useEffect } from 'react';
import Button from '../../atoms/Button';
import Label from '../../atoms/Label';
import { Progress } from '../../atoms/Progress';
import { useSimulationStore } from '../../../stores/simulationStore';

const AutomatedPanel: React.FC = () => {
  const {
    scenarios,
    selectedAutomatedScenario,
    setSelectedAutomatedScenario,
    startAutomated,
    stop,
    status,
    loading,
    progress,
  } = useSimulationStore();
  const showAutomated = useSimulationStore((s) => s.showAutomated);

  // Ensure scenarios are loaded when Automated mode is shown
  const loadScenarios = useSimulationStore((s) => s.loadScenarios);
  useEffect(() => {
    if (showAutomated && (!scenarios || scenarios.length === 0)) {
      loadScenarios();
    }
  }, [showAutomated]);

  // Ensure selection exists when scenarios arrive
  useEffect(() => {
    if ((!selectedAutomatedScenario || selectedAutomatedScenario === '') && scenarios && scenarios.length > 0) {
      setSelectedAutomatedScenario(scenarios[0].name);
    }
  }, [scenarios, selectedAutomatedScenario, setSelectedAutomatedScenario]);

  const selected = scenarios.find(s => s.name === selectedAutomatedScenario);

  return (
    <div className="space-y-4">
      <div>
        <Label>Automated Scenario</Label>
        <select
          aria-label={"Select automated simulation scenario"}
          value={selectedAutomatedScenario}
          onChange={(e) => setSelectedAutomatedScenario(e.target.value)}
          disabled={status.isRunning || loading}
          className="w-full bg-gray-800 border border-gray-600 square px-3 py-2 text-gray-200 focus:border-blue-500 focus:outline-none"
        >
          {(scenarios && scenarios.length > 0
            ? scenarios
            : [
                { name: 'basic', display_name: 'Basic Match', match_count: 1, estimated_duration: 90 },
                { name: 'quick_test', display_name: 'Quick Test', match_count: 1, estimated_duration: 45 },
              ] as any
          ).map((s: any) => (
            <option key={s.name} value={s.name}>
              {s.display_name} ({s.match_count} matches, ~{Math.round((s.estimated_duration || 60) / 60)}min)
            </option>
          ))}
        </select>
        {selected && (
          <p className="text-xs text-gray-400 mt-1">{selected.description}</p>
        )}
      </div>

      {/* Progress */}
      {status.isRunning && progress.total > 0 && (
        <div>
          <div className="flex justify-between text-xs text-gray-400 mb-1">
            <span>Progress</span>
            <span>{progress.current}/{progress.total} matches</span>
          </div>
          <Progress value={(progress.current / progress.total) * 100} />
        </div>
      )}

      {/* Controls */}
      <div className="flex gap-2">
        <Button variant="primary" size="sm" onClick={startAutomated} disabled={status.isRunning || loading || !selectedAutomatedScenario} className="flex-1">Start Automated</Button>
        <Button variant="secondary" size="sm" onClick={stop} disabled={!status.isRunning || loading}>Stop</Button>
      </div>
    </div>
  );
};

export default AutomatedPanel;


