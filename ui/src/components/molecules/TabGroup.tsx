import React from 'react';
import Tab from '../atoms/Tab';

interface TabItem {
  id: string;
  label: string;
  icon?: React.ReactNode;
  content: React.ReactNode;
}

interface TabGroupProps {
  tabs: TabItem[];
  activeTab: string;
  onTabChange: (tabId: string) => void;
  className?: string;
}

/**
 * TabGroup component for managing multiple tabs
 */
export const TabGroup: React.FC<TabGroupProps> = ({
  tabs,
  activeTab,
  onTabChange,
  className = '',
}) => {
  return (
    <div className={`${className}`}>
      {/* Tab Navigation */}
      <div className="flex border-b border-gray-800 mb-6 overflow-x-auto min-w-full">
        {tabs.map((tab) => (
          <Tab
            key={tab.id}
            id={tab.id}
            label={tab.label}
            icon={tab.icon}
            isActive={activeTab === tab.id}
            onClick={() => onTabChange(tab.id)}
            className="flex-shrink-0 min-w-0"
          />
        ))}
      </div>
      
      {/* Tab Content - keep all mounted to preserve state across tab switches */}
      <div className="min-h-0">
        {tabs.map(tab => (
          <div key={tab.id} className={activeTab === tab.id ? '' : 'hidden'}>
            {tab.content}
          </div>
        ))}
      </div>
    </div>
  );
};

export default TabGroup; 