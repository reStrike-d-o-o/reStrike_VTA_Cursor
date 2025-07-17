import React from 'react';

export type InputProps = React.InputHTMLAttributes<HTMLInputElement> & {
  className?: string;
};

const baseStyles = 'w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-blue-500 transition-colors';

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className = '', ...props }, ref) => (
    <input
      ref={ref}
      className={`${baseStyles} ${className}`}
      {...props}
    />
  )
);

Input.displayName = 'Input';

export default Input; 