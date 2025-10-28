import React, { useEffect, useMemo } from 'react';
import Icon from '../atoms/Icon';
import SelfTestPanel from './SelfTestPanel';
import { useSimulationStore } from '../../stores/simulationStore';
import SimulationHeader from './simulation/SimulationHeader';
import SimulationModeTabs from './simulation/SimulationModeTabs';
import ManualPanel from './simulation/ManualPanel';
import AutomatedPanel from './simulation/AutomatedPanel';
import StatusPanel from './simulation/StatusPanel';
import ArcadeModePanel from './simulation/ArcadeModePanel';
import { useI18n } from '../../i18n/index';

interface SimulationPanelProps { className?: string; }

const SimulationPanelV2: React.FC<SimulationPanelProps> = ({ className = '' }) => {
  const {
    loadStatus,
    loadScenarios,
    status,
    showAutomated,
    showSelfTest,
    showArcade,
    loading,
  } = useSimulationStore();
  const shouldPoll = useMemo(() => status.isRunning || loading, [status.isRunning, loading]);
  const { t } = useI18n();

  useEffect(() => {
    loadStatus();
    loadScenarios();
  }, [loadStatus, loadScenarios]);

  useEffect(() => {
    if (!shouldPoll) {
      return;
    }
    const timer = setInterval(() => loadStatus(), 2000);
    return () => clearInterval(timer);
  }, [shouldPoll, loadStatus]);

  const renderBody = () => {
    if (showAutomated) return <AutomatedPanel />;
    if (showSelfTest) return <SelfTestPanel />;
    if (showArcade) return <ArcadeModePanel />;
    return <ManualPanel />;
  };

  return (
    <div className={`space-y-6 ${className}`}>
      <div className="flex items-center gap-3">
        <Icon name="🤖" className="w-6 h-6 text-blue-400" />
        <h3 className="text-lg font-semibold text-gray-200">{t('simulation.title', 'Simulation Control')}</h3>
      </div>
      <SimulationHeader />
      <StatusPanel />
      <SimulationModeTabs />
      {renderBody()}
    </div>
  );
};

export default SimulationPanelV2;


