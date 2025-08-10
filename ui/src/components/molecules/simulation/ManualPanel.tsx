import React from 'react';
import Button from '../../atoms/Button';
import Input from '../../atoms/Input';
import Label from '../../atoms/Label';
import { useSimulationStore } from '../../../stores/simulationStore';

const ManualPanel: React.FC = () => {
  const {
    selectedMode,
    selectedScenario,
    duration,
    setSelectedMode,
    setSelectedScenario,
    setDuration,
    startManual,
    stop,
    sendManualEvent,
    status,
    loading,
  } = useSimulationStore();

  const disabled = status.isRunning || loading;

  return (
    <div className="space-y-4">
      {/* Mode */}
      <div>
        <Label>Simulation Mode</Label>
        <select
          aria-label="Select simulation mode"
          className="w-full bg-gray-800 border border-gray-600 square px-3 py-2 text-gray-200 focus:border-blue-500 focus:outline-none"
          value={selectedMode}
          onChange={(e) => setSelectedMode(e.target.value)}
          disabled={disabled}
        >
          <option value="demo">Demo</option>
          <option value="random">Random Events</option>
          <option value="interactive">Interactive</option>
        </select>
      </div>

      {/* Scenario */}
      <div>
        <Label>Scenario</Label>
        <select
          aria-label="Select simulation scenario"
          className="w-full bg-gray-800 border border-gray-600 square px-3 py-2 text-gray-200 focus:border-blue-500 focus:outline-none"
          value={selectedScenario}
          onChange={(e) => setSelectedScenario(e.target.value)}
          disabled={disabled}
        >
          <option value="basic">Basic Match</option>
          <option value="championship">Championship</option>
          <option value="training">Training</option>
        </select>
      </div>

      {/* Duration */}
      <div>
        <Label>Duration (seconds)</Label>
        <Input type="number" value={duration} onChange={(e) => setDuration(parseInt(e.target.value) || 30)} disabled={disabled} min={10} max={600} />
      </div>

      {/* Controls */}
      <div className="flex gap-2">
        <Button variant="primary" size="sm" onClick={startManual} disabled={disabled} className="flex-1">Start Simulation</Button>
        <Button variant="secondary" size="sm" onClick={stop} disabled={!status.isRunning || loading}>Stop</Button>
      </div>

      {/* Event Palette */}
      <div className="grid md:grid-cols-2 gap-4">
        {/* Points Blue */}
        <div className="bg-blue-900/10 border border-blue-800/30 p-2">
          <h5 className="text-xs font-medium text-blue-300 mb-2">Points (Blue Athlete)</h5>
          <div className="grid grid-cols-2 gap-1.5">
            <Button className="px-2 text-xs border-blue-600/40 text-blue-200 hover:bg-blue-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('point', { athlete: 1, point_type: 1 })} disabled={disabled}>Punch (1pt)</Button>
            <Button className="px-2 text-xs border-blue-600/40 text-blue-200 hover:bg-blue-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('point', { athlete: 1, point_type: 2 })} disabled={disabled}>Body Kick (2pt)</Button>
            <Button className="px-2 text-xs border-blue-600/40 text-blue-200 hover:bg-blue-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('point', { athlete: 1, point_type: 3 })} disabled={disabled}>Head Kick (3pt)</Button>
            <Button className="px-2 text-xs border-blue-600/40 text-blue-200 hover:bg-blue-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('point', { athlete: 1, point_type: 4 })} disabled={disabled}>Tech Body (4pt)</Button>
            <Button className="px-2 text-xs border-blue-600/40 text-blue-200 hover:bg-blue-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('point', { athlete: 1, point_type: 5 })} disabled={disabled}>Tech Head (5pt)</Button>
          </div>
        </div>

        {/* Points Red */}
        <div className="bg-red-900/10 border border-red-800/30 p-2">
          <h5 className="text-xs font-medium text-red-300 mb-2">Points (Red Athlete)</h5>
          <div className="grid grid-cols-2 gap-1.5">
            <Button className="px-2 text-xs border-red-600/40 text-red-200 hover:bg-red-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('point', { athlete: 2, point_type: 1 })} disabled={disabled}>Punch (1pt)</Button>
            <Button className="px-2 text-xs border-red-600/40 text-red-200 hover:bg-red-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('point', { athlete: 2, point_type: 2 })} disabled={disabled}>Body Kick (2pt)</Button>
            <Button className="px-2 text-xs border-red-600/40 text-red-200 hover:bg-red-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('point', { athlete: 2, point_type: 3 })} disabled={disabled}>Head Kick (3pt)</Button>
            <Button className="px-2 text-xs border-red-600/40 text-red-200 hover:bg-red-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('point', { athlete: 2, point_type: 4 })} disabled={disabled}>Tech Body (4pt)</Button>
            <Button className="px-2 text-xs border-red-600/40 text-red-200 hover:bg-red-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('point', { athlete: 2, point_type: 5 })} disabled={disabled}>Tech Head (5pt)</Button>
          </div>
        </div>

        {/* Hit Levels */}
        <div>
          <h5 className="text-xs font-medium text-gray-300 mb-2">Hit Levels</h5>
          <div className="grid grid-cols-2 gap-4">
            <div className="bg-blue-900/10 border border-blue-800/30 p-2">
              <div className="grid grid-cols-2 gap-1.5">
                <Button className="px-2 text-xs border-blue-600/40 text-blue-200 hover:bg-blue-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('hit_level', { athlete: 1, level: 25 })} disabled={disabled}>Blue Low (25)</Button>
                <Button className="px-2 text-xs border-blue-600/40 text-blue-200 hover:bg-blue-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('hit_level', { athlete: 1, level: 75 })} disabled={disabled}>Blue High (75)</Button>
              </div>
            </div>
            <div className="bg-red-900/10 border border-red-800/30 p-2">
              <div className="grid grid-cols-2 gap-1.5">
                <Button className="px-2 text-xs border-red-600/40 text-red-200 hover:bg-red-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('hit_level', { athlete: 2, level: 25 })} disabled={disabled}>Red Low (25)</Button>
                <Button className="px-2 text-xs border-red-600/40 text-red-200 hover:bg-red-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('hit_level', { athlete: 2, level: 75 })} disabled={disabled}>Red High (75)</Button>
              </div>
            </div>
          </div>
        </div>

        {/* Warnings */}
        <div className="bg-blue-900/10 border border-blue-800/30 p-2">
          <h5 className="text-xs font-medium text-gray-300 mb-2">Warnings</h5>
          <div className="grid grid-cols-2 gap-1.5">
            <Button className="px-2 text-xs border-blue-600/40 text-blue-200 hover:bg-blue-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('warning', { athlete: 1 })} disabled={disabled}>Blue Warning</Button>
            <Button className="px-2 text-xs border-red-600/40 text-red-200 hover:bg-red-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('warning', { athlete: 2 })} disabled={disabled}>Red Warning</Button>
          </div>
        </div>

        {/* Injury */}
        <div className="bg-blue-900/10 border border-blue-800/30 p-2">
          <h5 className="text-xs font-medium text-gray-300 mb-2">Injury Time</h5>
          <div className="grid grid-cols-2 gap-1.5">
            <Button className="px-2 text-xs border-blue-600/40 text-blue-200 hover:bg-blue-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('injury', { athlete: 1, duration: 60 })} disabled={disabled}>Blue Injury</Button>
            <Button className="px-2 text-xs border-red-600/40 text-red-200 hover:bg-red-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('injury', { athlete: 2, duration: 60 })} disabled={disabled}>Red Injury</Button>
          </div>
        </div>

        {/* Challenges */}
        <div className="bg-blue-900/10 border border-blue-800/30 p-2">
          <h5 className="text-xs font-medium text-gray-300 mb-2">Challenges</h5>
          <div className="grid grid-cols-2 gap-1.5">
            <Button className="px-2 text-xs border-blue-600/40 text-blue-200 hover:bg-blue-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('challenge', { source: 1, accepted: true, won: true })} disabled={disabled}>Blue Win</Button>
            <Button className="px-2 text-xs border-red-600/40 text-red-200 hover:bg-red-900/20 w-auto" variant="outline" size="sm" onClick={() => sendManualEvent('challenge', { source: 2, accepted: true, won: false })} disabled={disabled}>Red Lose</Button>
          </div>
        </div>

        {/* Break */}
        <div>
          <h5 className="text-xs font-medium text-gray-300 mb-2">Break Time</h5>
          <div className="grid grid-cols-2 gap-2">
            <Button variant="outline" size="sm" onClick={() => sendManualEvent('break', { duration: 60 })} disabled={disabled}>Start Break</Button>
            <Button variant="outline" size="sm" onClick={() => sendManualEvent('break_end', {})} disabled={disabled}>End Break</Button>
          </div>
        </div>

        {/* Clock */}
        <div>
          <h5 className="text-xs font-medium text-gray-300 mb-2">Clock Control</h5>
          <div className="grid grid-cols-2 gap-2">
            <Button variant="outline" size="sm" onClick={() => sendManualEvent('clock_start', {})} disabled={disabled}>Start Clock</Button>
            <Button variant="outline" size="sm" onClick={() => sendManualEvent('clock_stop', {})} disabled={disabled}>Stop Clock</Button>
          </div>
        </div>

        {/* Rounds */}
        <div>
          <h5 className="text-xs font-medium text-gray-300 mb-2">Round Control</h5>
          <div className="grid grid-cols-3 gap-2">
            <Button variant="outline" size="sm" onClick={() => sendManualEvent('round', { round: 1 })} disabled={disabled}>Round 1</Button>
            <Button variant="outline" size="sm" onClick={() => sendManualEvent('round', { round: 2 })} disabled={disabled}>Round 2</Button>
            <Button variant="outline" size="sm" onClick={() => sendManualEvent('round', { round: 3 })} disabled={disabled}>Round 3</Button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ManualPanel;


