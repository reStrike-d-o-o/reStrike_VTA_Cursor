/**
 * LottieIcon atom
 * - Displays Lottie animations as icons (local/public assets supported)
 */
import React from 'react';
import Lottie from 'lottie-react';

interface LottieIconProps {
  animationData: any;
  size?: number; // px; will be mapped to nearest preset class
  loop?: boolean;
  autoplay?: boolean;
  className?: string;
  onComplete?: () => void;
}

const sizeToClass = (size: number): string => {
  if (size <= 16) return 'w-4 h-4';
  if (size <= 24) return 'w-6 h-6';
  if (size <= 32) return 'w-8 h-8';
  if (size <= 40) return 'w-10 h-10';
  if (size <= 48) return 'w-12 h-12';
  if (size <= 56) return 'w-14 h-14';
  if (size <= 64) return 'w-16 h-16';
  return 'w-20 h-20';
};

const LottieIcon: React.FC<LottieIconProps> = ({
  animationData,
  size = 48,
  loop = true,
  autoplay = true,
  className = '',
  onComplete,
}) => {
  const sizeClass = sizeToClass(size);
  return (
    <div className={`flex items-center justify-center ${sizeClass} ${className}`}>
      <Lottie
        animationData={animationData}
        loop={loop}
        autoplay={autoplay}
        className={`${sizeClass}`}
        onComplete={onComplete}
      />
    </div>
  );
};

export default LottieIcon; 