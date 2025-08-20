/**
 * Progress atom
 * - Simple progress bar with percentage width
 */
import React from 'react';

interface ProgressProps {
  value: number;
  max?: number;
  className?: string;
}

const Progress: React.FC<ProgressProps> = ({ 
  value, 
  max = 100, 
  className = '' 
}) => {
  const percentage = Math.min(Math.max((value / max) * 100, 0), 100);
  const widthClass = (() => {
    if (percentage >= 100) return 'w-full';
    if (percentage >= 90) return 'w-[90%]';
    if (percentage >= 80) return 'w-[80%]';
    if (percentage >= 70) return 'w-[70%]';
    if (percentage >= 60) return 'w-[60%]';
    if (percentage >= 50) return 'w-[50%]';
    if (percentage >= 40) return 'w-[40%]';
    if (percentage >= 30) return 'w-[30%]';
    if (percentage >= 20) return 'w-[20%]';
    if (percentage >= 10) return 'w-[10%]';
    return 'w-[5%]';
  })();

  return (
    <div className={`w-full bg-gray-700 rounded-full h-2 ${className}`}>
      <div 
        className={`bg-blue-600 h-2 rounded-full transition-all duration-300 ${widthClass}`}
      />
    </div>
  );
};

export { Progress }; 