import React from 'react';

export interface ToggleProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  labelPosition?: 'left' | 'right' | 'bottom';
}

const Toggle = React.forwardRef<HTMLInputElement, ToggleProps>(
  ({ label = '', labelPosition = 'right', className = '', onChange, ...props }, ref) => {
    const handleClick = (e: React.MouseEvent) => {
      console.log('Toggle handleClick called!', { label, labelPosition });
      // Prevent the click from bubbling up
      e.preventDefault();
      e.stopPropagation();
      
      // Find the input element and trigger a click on it
      const input = e.currentTarget.querySelector('input[type="checkbox"]') as HTMLInputElement;
      if (input) {
        console.log('Found input, triggering click');
        input.click();
      } else {
        console.log('Input not found!');
      }
    };

    const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      console.log('Toggle input onChange called!', { 
        checked: e.target.checked, 
        label, 
        labelPosition 
      });
      if (onChange) {
        onChange(e);
      }
    };

    if (labelPosition === 'bottom') {
      return (
        <div className="flex flex-col items-center">
          <label className="cursor-pointer" onClick={handleClick}>
            <div className="relative">
              <input
                ref={ref}
                type="checkbox"
                className={`sr-only peer ${className}`}
                {...props}
                onChange={handleInputChange}
              />
              <div className="w-11 h-6 bg-gray-600 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-800 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
            </div>
          </label>
          {label && (
            <span className="mt-2 text-sm font-medium text-gray-300">{label}</span>
          )}
        </div>
      );
    }

    return (
      <label className="flex items-center cursor-pointer" onClick={handleClick}>
        {label && labelPosition === 'left' && (
          <span className="mr-3 text-sm font-medium text-gray-300">{label}</span>
        )}
        
        <div className="relative">
          <input
            ref={ref}
            type="checkbox"
            className={`sr-only peer ${className}`}
            {...props}
            onChange={handleInputChange}
          />
          <div className="w-11 h-6 bg-gray-600 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-800 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
        </div>
        
        {label && labelPosition === 'right' && (
          <span className="ml-3 text-sm font-medium text-gray-300">{label}</span>
        )}
      </label>
    );
  }
);

Toggle.displayName = 'Toggle';

export default Toggle; 