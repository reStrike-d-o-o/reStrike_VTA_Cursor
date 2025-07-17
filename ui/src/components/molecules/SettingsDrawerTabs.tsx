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