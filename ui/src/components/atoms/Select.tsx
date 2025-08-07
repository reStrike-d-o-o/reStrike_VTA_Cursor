import React, { useState, useRef, useEffect } from 'react';

interface SelectProps {
  disabled?: boolean;
  children: React.ReactNode;
  value?: string;
  onValueChange?: (value: string) => void;
  className?: string;
}

interface SelectTriggerProps {
  children: React.ReactNode;
  className?: string;
  onClick?: () => void;
  isOpen?: boolean;
  selectedValue?: string;
}

interface SelectValueProps {
  placeholder?: string;
  className?: string;
  value?: string;
}

interface SelectContentProps {
  children: React.ReactNode;
  className?: string;
  onSelect?: (value: string) => void;
}

interface SelectItemProps {
  value: string;
  children: React.ReactNode;
  className?: string;
  onClick?: () => void;
}

const Select: React.FC<SelectProps> = ({
  children,
  value,
  onValueChange,
  className = '',
  disabled = false
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [selectedValue, setSelectedValue] = useState(value || '');
  const selectRef = useRef<HTMLDivElement>(null);

  // Update selectedValue when value prop changes
  useEffect(() => {
    setSelectedValue(value || '');
  }, [value]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (selectRef.current && !selectRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleSelect = (newValue: string) => {
    setSelectedValue(newValue);
    onValueChange?.(newValue);
    setIsOpen(false);
  };

  return (
    <div ref={selectRef} className={`relative ${disabled ? 'opacity-50 pointer-events-none' : ''} ${className}`}>
      {React.Children.map(children, (child) => {
        if (React.isValidElement(child)) {
          if (child.type === SelectTrigger) {
            return React.cloneElement(child as React.ReactElement<SelectTriggerProps>, {
              onClick: () => setIsOpen(!isOpen),
              isOpen,
              selectedValue,
            });
          }
          if (child.type === SelectValue) {
            return React.cloneElement(child as React.ReactElement<SelectValueProps>, {
              value: selectedValue,
            });
          }
          if (child.type === SelectContent && isOpen) {
            return React.cloneElement(child as React.ReactElement<SelectContentProps>, {
              onSelect: handleSelect,
            });
          }
        }
        return child;
      })}
    </div>
  );
};

const SelectTrigger: React.FC<SelectTriggerProps> = ({
  children,
  className = '',
  onClick,
  isOpen,
  selectedValue
}) => {
  return (
    <button
      onClick={onClick}
      className={`
        flex items-center justify-between w-full px-3 py-2 text-sm bg-gray-700 border border-gray-600 rounded-md
        hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-blue-500
        ${className}
      `}
    >
      {children}
      <svg
        className={`w-4 h-4 transition-transform ${isOpen ? 'rotate-180' : ''}`}
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
      </svg>
    </button>
  );
};

const SelectValue: React.FC<SelectValueProps> = ({ placeholder, className = '', value }) => {
  const displayText = value || placeholder;
  return (
    <span className={`text-gray-300 ${className}`}>
      {displayText}
    </span>
  );
};

const SelectContent: React.FC<SelectContentProps> = ({
  children,
  className = '',
  onSelect
}) => {
  return (
    <div className={`
      absolute top-full left-0 right-0 z-50 mt-1 bg-gray-700 border border-gray-600 rounded-md shadow-lg
      max-h-60 overflow-auto
      ${className}
    `}>
      {React.Children.map(children, (child) => {
        if (React.isValidElement(child) && child.type === SelectItem) {
          return React.cloneElement(child as React.ReactElement<SelectItemProps>, {
            onClick: () => onSelect?.(child.props.value),
          });
        }
        return child;
      })}
    </div>
  );
};

const SelectItem: React.FC<SelectItemProps> = ({
  children,
  className = '',
  onClick
}) => {
  return (
    <button
      onClick={onClick}
      className={`
        w-full px-3 py-2 text-sm text-left text-gray-300 hover:bg-gray-600 focus:bg-gray-600 focus:outline-none
        ${className}
      `}
    >
      {children}
    </button>
  );
};

export { Select, SelectTrigger, SelectValue, SelectContent, SelectItem }; 