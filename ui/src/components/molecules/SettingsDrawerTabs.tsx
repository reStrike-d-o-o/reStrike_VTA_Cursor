import React from 'react';
import IconDocument from '../atoms/IconDocument';

interface SettingsDrawerTabsProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
}

const tabs = [
  {
    key: 'diagnostics',
    label: 'Diagnostics & Logs Manager',
    icon: <IconDocument className="w-5 h-5 mr-2" />,
  },
  {
    key: 'database',
    label: 'Database',
    icon: (
      <svg className="w-5 h-5 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4" />
      </svg>
    ),
  },
  {
    key: 'backup-restore',
    label: 'Backup & Restore',
    icon: (
      <svg className="w-5 h-5 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
      </svg>
    ),
  },
  {
    key: 'app-settings',
    label: 'App Settings',
    icon: (
      <svg className="w-5 h-5 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
  },
  // Future tabs can be added here
];

const SettingsDrawerTabs: React.FC<SettingsDrawerTabsProps> = ({ activeTab, onTabChange }) => (
  <div className="flex border-b border-gray-800 mb-6">
    {tabs.map(tab => (
      <button
        key={tab.key}
        className={`flex items-center px-4 py-2 text-sm font-medium focus:outline-none transition-colors border-b-2 ${
          activeTab === tab.key
            ? 'border-blue-500 text-blue-200 bg-[#181F26]'
            : 'border-transparent text-gray-400 hover:text-blue-300'
        }`}
        onClick={() => onTabChange(tab.key)}
      >
        {tab.icon}
        {tab.label}
      </button>
    ))}
  </div>
);

export default SettingsDrawerTabs; 