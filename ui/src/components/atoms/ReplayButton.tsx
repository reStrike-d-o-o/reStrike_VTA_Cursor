import React from 'react';

interface ReplayButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  className?: string;
}

const ReplayButton: React.FC<ReplayButtonProps> = ({ className = '', children, ...props }) => {
  return (
    <button
      type="button"
      className={`w-32 h-32 rounded-full bg-gradient-to-br from-red-600 to-red-700 flex items-center justify-center text-xl font-bold text-white border-4 border-red-500/50 mb-4 shadow-2xl hover:shadow-red-500/40 hover:scale-105 focus:outline-none transition-all duration-300 animate-pulse relative group replay-button ${className}`}
      {...props}
    >
      {/* Inner glow effect */}
      <div className="absolute inset-2 rounded-full bg-red-500/20 blur-sm group-hover:bg-red-500/30 transition-all duration-300"></div>
      
      {/* Button content */}
      <span className="relative z-10 drop-shadow-lg">{children}</span>
      
      {/* Outer ring glow */}
      <div className="absolute inset-0 rounded-full border-2 border-red-400/30 group-hover:border-red-400/50 transition-all duration-300"></div>
    </button>
  );
};

export default ReplayButton; 