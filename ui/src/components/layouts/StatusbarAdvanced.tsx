/**
 * StatusbarAdvanced
 * - Minimal footer bar for Advanced view; can host diagnostics/actions
 */
import React from 'react';

const StatusbarAdvanced: React.FC = () => {
  return (
    <div className="w-full h-[4.5rem] flex justify-end items-center text-xs text-gray-500 px-8 bg-[#101820] border-t border-gray-800">
      <span>Statusbar Advanced</span>
    </div>
  );
};

export default StatusbarAdvanced; 