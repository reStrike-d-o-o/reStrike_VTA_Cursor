import React from 'react';
import IconDocument from '../atoms/IconDocument';
import LottieIcon from '../atoms/LottieIcon';
import { bugAnimation, databaseAnimation, usbAnimation, brainAnimation } from '../../assets/icons/json';
import { useI18n } from '../../i18n';

interface SettingsDrawerTabsProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
}

const SettingsTabs: React.FC<{ active: string; onChange: (k: string) => void }> = ({ active, onChange }) => {
  const { t } = useI18n();
  const tabs = [
  {
    key: 'diagnostics',
    label: t('settings.tabs.diagnostics', 'Diagnostics & Logs Manager'),
    icon: <LottieIcon animationData={bugAnimation} size={32} />,
  },
  {
    key: 'database',
    label: t('settings.tabs.database', 'Database'),
    icon: <LottieIcon animationData={databaseAnimation} size={32} />,
  },
  {
    key: 'backup-restore',
    label: t('settings.tabs.backup', 'Backup & Restore'),
    icon: <LottieIcon animationData={usbAnimation} size={32} />,
  },
  {
    key: 'app-settings',
    label: t('settings.tabs.app', 'App Settings'),
    icon: <LottieIcon animationData={brainAnimation} size={32} />,
  },
  // Future tabs can be added here
  ];
  return (
    <div className="flex border-b border-gray-800 mb-6">
      {tabs.map(tab => (
        <button
          key={tab.key}
          className={`flex items-center px-4 py-2 text-sm font-medium focus:outline-none transition-colors border-b-2 ${
            active === tab.key
              ? 'border-blue-500 text-blue-200 bg-[#181F26]'
              : 'border-transparent text-gray-400 hover:text-blue-300'
          }`}
          onClick={() => onChange(tab.key)}
        >
          {tab.icon}
          {tab.label}
        </button>
      ))}
    </div>
  );
};

const SettingsDrawerTabs: React.FC<SettingsDrawerTabsProps> = ({ activeTab, onTabChange }) => (
  <SettingsTabs active={activeTab} onChange={onTabChange} />
);

export default SettingsDrawerTabs; 