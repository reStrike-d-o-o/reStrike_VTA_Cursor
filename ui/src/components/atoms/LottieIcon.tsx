import React from 'react';
import Lottie from 'lottie-react';

interface LottieIconProps {
  animationData: any;
  size?: number;
  loop?: boolean;
  autoplay?: boolean;
  className?: string;
  style?: React.CSSProperties;
  onComplete?: () => void;
  color?: string;
  filter?: string;
}

/**
 * LottieIcon Component
 * 
 * Displays Lottie animations as icons. Can import animations from:
 * - src/assets/icons/json/ (local assets)
 * - public/assets/icons/json/ (public assets - thanks to craco webpack config)
 * 
 * Usage:
 * import diagramAnimation from '../../../public/assets/icons/json/diagram.json';
 * <LottieIcon animationData={diagramAnimation} size={24} />
 */
const LottieIcon: React.FC<LottieIconProps> = ({
  animationData,
  size = 48,
  loop = true,
  autoplay = true,
  className = '',
  style = {},
  onComplete,
  color,
  filter
}) => {
  return (
    <div 
      className={`flex items-center justify-center ${className}`}
      style={{ width: size, height: size, ...style }}
    >
      <Lottie
        animationData={animationData}
        loop={loop}
        autoplay={autoplay}
        style={{ 
          width: size, 
          height: size,
          filter: filter || (color ? `brightness(0) saturate(100%) invert(1) sepia(1) saturate(10000%) hue-rotate(${color === 'blue' ? '200deg' : color === 'red' ? '0deg' : color === 'green' ? '120deg' : '0deg'})` : undefined)
        }}
        onComplete={onComplete}
      />
    </div>
  );
};

export default LottieIcon; 