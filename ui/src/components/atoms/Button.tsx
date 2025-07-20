import React from 'react';

export type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'success' | 'ghost';
export type ButtonSize = 'sm' | 'md' | 'lg';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  className?: string;
}

const baseStyles = 'rounded-lg transition-all duration-300 font-medium focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-900 relative group';
const variantStyles: Record<ButtonVariant, string> = {
  primary: 'bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-700 hover:to-blue-800 text-white shadow-lg shadow-blue-500/25 hover:shadow-xl hover:shadow-blue-500/30',
  secondary: 'bg-gradient-to-r from-gray-600 to-gray-700 hover:from-gray-700 hover:to-gray-800 text-white shadow-lg shadow-gray-500/25 hover:shadow-xl hover:shadow-gray-500/30',
  danger: 'bg-gradient-to-r from-red-600 to-red-700 hover:from-red-700 hover:to-red-800 text-white shadow-lg shadow-red-500/25 hover:shadow-xl hover:shadow-red-500/30',
  success: 'bg-gradient-to-r from-green-600 to-green-700 hover:from-green-700 hover:to-green-800 text-white shadow-lg shadow-green-500/25 hover:shadow-xl hover:shadow-green-500/30',
  ghost: 'bg-transparent hover:bg-gray-700/50 text-gray-300 hover:text-white border border-gray-600/30 hover:border-gray-500/50',
};
const sizeStyles: Record<ButtonSize, string> = {
  sm: 'px-3 py-1.5 text-sm',
  md: 'px-4 py-2 text-base',
  lg: 'px-6 py-3 text-lg',
};

const Button: React.FC<ButtonProps> = ({
  variant = 'primary',
  size = 'md',
  className = '',
  children,
  ...props
}) => {
  return (
    <button
      className={`${baseStyles} ${variantStyles[variant]} ${sizeStyles[size]} ${className}`}
      {...props}
    >
      {/* Subtle glow effect for primary buttons */}
      {variant === 'primary' && (
        <div className="absolute inset-0 rounded-lg bg-blue-500/20 blur-sm group-hover:bg-blue-500/30 transition-all duration-300"></div>
      )}
      <span className="relative z-10 flex items-center justify-center">{children}</span>
    </button>
  );
};

export default Button; 