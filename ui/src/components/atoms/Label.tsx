/**
 * Label atom
 * - Accessible label for form controls
 */
import React from 'react';

const Label: React.FC<React.LabelHTMLAttributes<HTMLLabelElement>> = ({ className = '', children, ...props }) => (
  <label className={`block text-sm font-medium text-gray-300 mb-1 ${className}`} {...props}>
    {children}
  </label>
);

export default Label; 