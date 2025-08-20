import React from 'react';
import { useEnvironment } from '../../hooks/useEnvironment';
import { useI18n } from '../../i18n/index';

const TaskBar: React.FC = () => {
  const { environment, tauriAvailable } = useEnvironment();
  const { t, locale, setLocale } = useI18n();

  return (
    <div className="w-full bg-gray-800 border-b border-gray-700 py-4 px-8 flex items-center justify-between">
      <div className="flex items-center space-x-4">
        <h1 className="text-xl font-bold">{t('app.title', 'reStrike VTA - Windows Desktop')}</h1>
        <span className={`px-2 py-1 text-xs rounded ${
          environment === 'windows' && tauriAvailable 
            ? 'bg-green-600 text-white' 
            : 'bg-red-600 text-white'
        }`}>
          {environment === 'windows' && tauriAvailable ? t('env.windows_native', 'Windows Native') : t('env.web_mode', 'Web Mode')}
        </span>
      </div>
      <div className="flex items-center space-x-4">
        <span className="text-sm text-gray-400">{t('status.ready', 'Status: Ready')}</span>
        {/* Quick Action Buttons Placeholder */}
        <button className="bg-blue-600 hover:bg-blue-700 text-white px-3 py-1 rounded">{t('action.generic', 'Action')}</button>
      </div>
    </div>
  );
};

export default TaskBar; 