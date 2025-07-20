import React from 'react';

interface StatusDotProps {
  color?: string; // Tailwind class or custom
  size?: string; // Tailwind size class, e.g., 'w-2 h-2'
  className?: string;
  ariaLabel?: string;
}

/**
 * StatusDot atom for colored status indicators (e.g., REC, STR, CPU)
 */
export const StatusDot: React.FC<StatusDotProps> = ({
  color = 'bg-gray-400',
  size = 'w-2 h-2',
  className = '',
  ariaLabel,
}) => (
  <span
    className={`inline-block rounded-full ${size} ${color} shadow-lg transition-all duration-300 flex-shrink-0 ${className}`}
    aria-label={ariaLabel}
  />
);

export default StatusDot; 