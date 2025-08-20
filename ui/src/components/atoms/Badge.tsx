/**
 * Badge atom
 * - Small status/label indicator
 */
import React from 'react';

interface BadgeProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'secondary' | 'outline';
}

export const Badge: React.FC<BadgeProps> = ({ className = '', children, variant = 'default', ...props }) => {
  const variantClass =
    variant === 'secondary'
      ? 'bg-gray-700 text-gray-200'
      : variant === 'outline'
      ? 'border border-gray-500 text-gray-200'
      : 'bg-gray-600 text-white';
  return (
    <div className={`inline-flex items-center px-2 py-0.5 rounded text-xs ${variantClass} ${className}`} {...props}>
      {children}
    </div>
  );
};

export default Badge; 