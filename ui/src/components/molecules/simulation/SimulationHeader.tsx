import React from 'react';
import StatusRow from '../../atoms/StatusRow';
import { useSimulationStore } from '../../../stores/simulationStore';
import { useI18n } from '../../../i18n/index';

const SimulationHeader: React.FC = () => {
  const status = useSimulationStore((s) => s.status);
  const { t } = useI18n();
  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
      <StatusRow label={<span>{t('simulation.header.connection', 'Connection')}</span>} status={status.isConnected ? 'ok' : 'error'} right={status.isConnected ? t('common.connected', 'Connected') : t('common.disconnected', 'Disconnected')} />
      <StatusRow label={<span>{t('simulation.header.mode', 'Mode')}</span>} status={status.isRunning ? 'ok' : 'warn'} right={status.currentMode || '—'} />
      <StatusRow label={<span>{t('simulation.header.scenario', 'Scenario')}</span>} status={status.isRunning ? 'ok' : 'unknown'} right={status.currentScenario || '—'} />
    </div>
  );
};

export default SimulationHeader;


