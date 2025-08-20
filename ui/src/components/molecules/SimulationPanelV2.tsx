/**
 * SimulationPanelV2
 * - Second iteration of simulation control UI
 */
import React, { useEffect, useState } from 'react';
import Icon from '../atoms/Icon';
import SelfTestPanel from './SelfTestPanel';
import { useSimulationStore } from '../../stores/simulationStore';
import SimulationHeader from './simulation/SimulationHeader';
import SimulationModeTabs from './simulation/SimulationModeTabs';
import ManualPanel from './simulation/ManualPanel';
import AutomatedPanel from './simulation/AutomatedPanel';
import StatusPanel from './simulation/StatusPanel';
import ArcadeModePanel from './simulation/ArcadeModePanel';

interface SimulationPanelProps { className?: string; }

const SimulationPanelV2: React.FC<SimulationPanelProps> = ({ className = '' }) => {
  const store = useSimulationStore();
  useEffect(() => { store.loadStatus(); store.loadScenarios(); }, []);
  useEffect(() => { const t = setInterval(() => store.loadStatus(), 2000); return () => clearInterval(t); }, [store]);

  const renderBody = () => {
    if (store.showAutomated) return <AutomatedPanel />;
    if (store.showSelfTest) return <SelfTestPanel />;
    if (store.showArcade) return <ArcadeModePanel />;
    return <ManualPanel />;
  };

  return (
    <div className={`space-y-6 ${className}`}>
      <div className="flex items-center gap-3">
        <Icon name="🤖" className="w-6 h-6 text-blue-400" />
        <h3 className="text-lg font-semibold text-gray-200">Simulation Control</h3>
      </div>
      <SimulationHeader />
      <StatusPanel />
      <SimulationModeTabs />
      {renderBody()}
    </div>
  );
};

export default SimulationPanelV2;


