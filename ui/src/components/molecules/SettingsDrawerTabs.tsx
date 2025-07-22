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
    key: 'app-settings',
    label: 'App Settings',
    icon: (
      <svg className="w-5 h-5 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
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