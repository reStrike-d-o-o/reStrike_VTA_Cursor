import React from 'react';

interface ReplayButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  className?: string;
}

const ReplayButton: React.FC<ReplayButtonProps> = ({ className = '', children, ...props }) => {
  return (
    <button
      type="button"
      className={`w-32 h-32 rounded-full bg-red-600 flex items-center justify-center text-xl font-bold text-white border-4 border-red-700 mb-4 shadow-2xl hover:bg-red-700 focus:outline-none transition-all duration-200 animate-pulse ${className}`}
      style={{ boxShadow: '0 0 20px rgba(220, 38, 38, 0.6), 0 0 0 4px #2B2B2B' }}
      {...props}
    >
      {children}
    </button>
  );
};

export default ReplayButton; 