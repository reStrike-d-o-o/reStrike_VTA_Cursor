/**
 * Button atom
 * - Variants: primary, secondary, danger, success, ghost, outline
 * - Sizes: sm, md, lg
 */
import React from 'react';

export type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'success' | 'ghost' | 'outline';
export type ButtonSize = 'sm' | 'md' | 'lg';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  className?: string;
}

const baseStyles = 'transition-all duration-300 font-medium focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-900 relative group square';
const variantStyles: Record<ButtonVariant, string> = {
  primary: 'bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-700 hover:to-blue-800 text-white shadow-none',
  secondary: 'bg-gradient-to-r from-gray-600 to-gray-700 hover:from-gray-700 hover:to-gray-800 text-white shadow-none',
  danger: 'bg-gradient-to-r from-red-600 to-red-700 hover:from-red-700 hover:to-red-800 text-white shadow-none',
  success: 'bg-gradient-to-r from-green-600 to-green-700 hover:from-green-700 hover:to-green-800 text-white shadow-none',
  ghost: 'bg-transparent hover:bg-gray-700/30 text-gray-300 hover:text-white border border-gray-600/30 hover:border-gray-500/40 shadow-none',
  outline: 'bg-transparent border border-gray-500/40 hover:bg-gray-700/20 text-gray-300 hover:text-white shadow-none',
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
      {/* Removed glow per sharp-corner theme */}
      <span className="relative z-10 flex items-center justify-center">{children}</span>
    </button>
  );
};

export default Button; 