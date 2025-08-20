import React from 'react';
import { useSimulationStore } from '../../../stores/simulationStore';
import Button from '../../atoms/Button';
import { useI18n } from '../../../i18n/index';

const StatusPanel: React.FC = () => {
  const { status, error, success, retry, installDependencies, loading } = useSimulationStore();
  const { t } = useI18n();

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
              <Button variant="outline" size="sm" onClick={retry} disabled={loading}>{t('common.retry', 'Retry')}</Button>
              {(error.includes('DependencyCheckFailed') || error.includes('PipInstallFailed')) && (
                <Button variant="outline" size="sm" onClick={installDependencies} disabled={loading}>
                  {loading ? t('common.installing', 'Installing...') : t('simulation.install_deps', 'Install Dependencies')}
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
        <h4 className="text-sm font-medium text-gray-300 mb-2">{t('simulation.status.title', 'Current Status')}</h4>
        <div className="space-y-1 text-xs text-gray-400">
          <div>{t('simulation.status.running', 'Running')}: {status.isRunning ? t('common.yes', 'Yes') : t('common.no', 'No')}</div>
          <div>{t('simulation.status.scenario', 'Scenario')}: {status.currentScenario}</div>
          <div>{t('simulation.status.mode', 'Mode')}: {status.currentMode}</div>
          <div>{t('simulation.status.events_sent', 'Events Sent')}: {status.eventsSent}</div>
          <div>{t('simulation.status.last_event', 'Last Event')}: {status.lastEvent}</div>
        </div>
      </div>
    </div>
  );
};

export default StatusPanel;


