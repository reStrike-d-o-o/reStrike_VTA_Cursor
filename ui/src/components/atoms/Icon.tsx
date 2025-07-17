import React from 'react';

interface IconProps {
  name: string; // Emoji or icon name
  size?: string; // Tailwind text size class, e.g., 'text-xl'
  className?: string;
  ariaLabel?: string;
  onClick?: () => void;
}

/**
 * Icon atom for emoji or SVG icons (future-proofed for SVG)
 */
export const Icon: React.FC<IconProps> = ({
  name,
  size = 'text-xl',
  className = '',
  ariaLabel,
  onClick,
}) => (
  <span
    className={`inline-block align-middle ${size} ${className}`}
    aria-label={ariaLabel}
    tabIndex={onClick ? 0 : undefined}
    onClick={onClick}
    style={{ cursor: onClick ? 'pointer' : undefined }}
  >
    {name}
  </span>
);

export default Icon; 