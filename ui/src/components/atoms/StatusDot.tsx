/**
 * StatusDot atom
 * - Colored dot indicator for status (REC/STR/CPU etc.)
 * - Backward compatible: accepts semantic colors (red|green|yellow|gray) or full tailwind bg-* classes
 */
import React from 'react';

type SemanticColor = 'red' | 'green' | 'yellow' | 'gray';

interface StatusDotProps extends React.HTMLAttributes<HTMLSpanElement> {
  color?: SemanticColor | string;
  title?: string;
  size?: string; // e.g., 'w-3 h-3'
}

export const StatusDot: React.FC<StatusDotProps> = ({ color = 'gray', title, className = '', size, ...props }) => {
  const classFromColor = (c: string): string => {
    if (c.startsWith('bg-')) return c;
    switch (c as SemanticColor) {
      case 'red':
        return 'bg-red-500';
      case 'green':
        return 'bg-green-500';
      case 'yellow':
        return 'bg-yellow-500';
      default:
        return 'bg-gray-500';
    }
  };

  return (
    <span
      title={title}
      className={`inline-block ${size ? size : 'w-2 h-2'} rounded-full ${classFromColor(String(color))} ${className}`}
      {...props}
    />
  );
};

export default StatusDot;