import React from 'react';

interface TabProps {
  id: string;
  label: string;
  icon?: React.ReactNode;
  isActive: boolean;
  onClick: () => void;
  className?: string;
}

/**
 * Tab component for drawer navigation
 */
export const Tab: React.FC<TabProps> = ({
  id,
  label,
  icon,
  isActive,
  onClick,
  className = '',
}) => {
  return (
    <button
      id={id}
      onClick={onClick}
      className={`
        flex items-center px-4 py-2 text-sm font-medium focus:outline-none transition-colors border-b-2
        ${isActive 
          ? 'border-blue-500 text-blue-200 bg-[#181F26]' 
          : 'border-transparent text-gray-400 hover:text-blue-300'
        }
        ${className}
      `}
    >
      {icon && <span className="w-4 h-4 mr-2">{icon}</span>}
      {label}
    </button>
  );
};

export default Tab; 