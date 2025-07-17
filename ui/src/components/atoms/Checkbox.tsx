import React from 'react';

export interface CheckboxProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  labelPosition?: 'right' | 'left' | 'top' | 'bottom';
  className?: string;
}

const Checkbox = React.forwardRef<HTMLInputElement, CheckboxProps>(
  ({ label, labelPosition = 'right', className = '', ...props }, ref) => {
    // Layout classes for label position
    let containerClass = 'inline-flex items-center cursor-pointer select-none';
    let labelClass = 'text-sm text-gray-200';
    let toggle = (
      <span className="relative flex items-center">
        <input ref={ref} type="checkbox" className={`sr-only peer ${className}`} {...props} />
        <div className="w-11 h-6 bg-gray-700 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-500 rounded-full peer peer-checked:bg-blue-600 transition-colors duration-200"></div>
        <div className="absolute left-1 top-1 w-4 h-4 bg-white rounded-full shadow-md transition-transform duration-200 peer-checked:translate-x-5"></div>
      </span>
    );
    let content;
    switch (labelPosition) {
      case 'left':
        content = <><span className={labelClass}>{label}</span>{toggle}</>;
        containerClass += ' flex-row-reverse space-x-reverse space-x-3';
        break;
      case 'top':
        content = <><span className={labelClass}>{label}</span>{toggle}</>;
        containerClass += ' flex-col items-center space-y-1';
        break;
      case 'bottom':
        content = <>{toggle}<span className={labelClass}>{label}</span></>;
        containerClass += ' flex-col items-center space-y-1';
        break;
      case 'right':
      default:
        content = <>{toggle}{label && <span className={labelClass + ' ml-3'}>{label}</span>}</>;
        containerClass += ' space-x-3';
        break;
    }
    return <label className={containerClass}>{content}</label>;
  }
);

Checkbox.displayName = 'Checkbox';

export default Checkbox; 