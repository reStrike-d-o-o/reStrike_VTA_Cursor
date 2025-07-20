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
  const activeTabContent = tabs.find(tab => tab.id === activeTab)?.content;

  return (
    <div className={`${className}`}>
      {/* Tab Navigation */}
      <div className="flex border-b border-gray-800 mb-6">
        {tabs.map((tab) => (
          <Tab
            key={tab.id}
            id={tab.id}
            label={tab.label}
            icon={tab.icon}
            isActive={activeTab === tab.id}
            onClick={() => onTabChange(tab.id)}
          />
        ))}
      </div>
      
      {/* Tab Content */}
      <div className="min-h-0">
        {activeTabContent}
      </div>
    </div>
  );
};

export default TabGroup; 