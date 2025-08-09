import React from 'react';
import StatusDot from './StatusDot';

interface StatusRowProps {
  label: React.ReactNode;
  status?: 'ok' | 'warn' | 'error' | 'unknown';
  right?: React.ReactNode; // additional right-side content (e.g., numbers)
  className?: string;
}

const colorFor = (s: StatusRowProps['status']): string => {
  switch (s) {
    case 'ok':
      return 'bg-emerald-500';
    case 'warn':
      return 'bg-amber-500';
    case 'error':
      return 'bg-red-500';
    default:
      return 'bg-gray-500';
  }
};

const StatusRow: React.FC<StatusRowProps> = ({ label, status = 'unknown', right, className = '' }) => {
  return (
    <div className={`flex items-center justify-between p-3 rounded theme-surface-2 ${className}`}>
      <div className="flex items-center gap-2 min-w-0">
        <StatusDot color={colorFor(status)} />
        <div className="text-sm theme-text">{label}</div>
      </div>
      {right && (
        <div className="text-sm theme-text-muted">{right}</div>
      )}
    </div>
  );
};

export default StatusRow;


