// Flag utility functions for the sidebar component

export interface FlagConfig {
  countryCode: string;
  fallbackEmoji: string;
  altText: string;
}

// Common country codes and their fallback emojis
export const FLAG_CONFIGS: Record<string, FlagConfig> = {
  'USA': { countryCode: 'USA', fallbackEmoji: '🇺🇸', altText: 'USA Flag' },
  'JPN': { countryCode: 'JPN', fallbackEmoji: '🇯🇵', altText: 'Japan Flag' },
  'KOR': { countryCode: 'KOR', fallbackEmoji: '🇰🇷', altText: 'South Korea Flag' },
  'CHN': { countryCode: 'CHN', fallbackEmoji: '🇨🇳', altText: 'China Flag' },
  'GBR': { countryCode: 'GBR', fallbackEmoji: '🇬🇧', altText: 'Great Britain Flag' },
  'FRA': { countryCode: 'FRA', fallbackEmoji: '🇫🇷', altText: 'France Flag' },
  'GER': { countryCode: 'GER', fallbackEmoji: '🇩🇪', altText: 'Germany Flag' },
  'ITA': { countryCode: 'ITA', fallbackEmoji: '🇮🇹', altText: 'Italy Flag' },
  'ESP': { countryCode: 'ESP', fallbackEmoji: '🇪🇸', altText: 'Spain Flag' },
  'CAN': { countryCode: 'CAN', fallbackEmoji: '🇨🇦', altText: 'Canada Flag' },
  'AUS': { countryCode: 'AUS', fallbackEmoji: '🇦🇺', altText: 'Australia Flag' },
  'BRA': { countryCode: 'BRA', fallbackEmoji: '🇧🇷', altText: 'Brazil Flag' },
  'RUS': { countryCode: 'RUS', fallbackEmoji: '🇷🇺', altText: 'Russia Flag' },
  'TUR': { countryCode: 'TUR', fallbackEmoji: '🇹🇷', altText: 'Turkey Flag' },
  'IRN': { countryCode: 'IRN', fallbackEmoji: '🇮🇷', altText: 'Iran Flag' },
  'THA': { countryCode: 'THA', fallbackEmoji: '🇹🇭', altText: 'Thailand Flag' },
  'VIE': { countryCode: 'VIE', fallbackEmoji: '🇻🇳', altText: 'Vietnam Flag' },
  'PHI': { countryCode: 'PHI', fallbackEmoji: '🇵🇭', altText: 'Philippines Flag' },
  'MAS': { countryCode: 'MAS', fallbackEmoji: '🇲🇾', altText: 'Malaysia Flag' },
  'SGP': { countryCode: 'SGP', fallbackEmoji: '🇸🇬', altText: 'Singapore Flag' },
};

/**
 * Get flag configuration for a country code
 */
export function getFlagConfig(countryCode: string): FlagConfig {
  const upperCode = countryCode.toUpperCase();
  return FLAG_CONFIGS[upperCode] || {
    countryCode: upperCode,
    fallbackEmoji: '🏳️',
    altText: `${upperCode} Flag`
  };
}

/**
 * Get flag image URL for a country code
 */
export function getFlagUrl(countryCode: string): string {
  const config = getFlagConfig(countryCode);
  return `/assets/flags/${config.countryCode}.png`;
}

/**
 * Handle flag image error with fallback to emoji
 */
export function handleFlagError(event: React.SyntheticEvent<HTMLImageElement, Event>, countryCode: string): void {
  const target = event.target as HTMLImageElement;
  const config = getFlagConfig(countryCode);
  
  // Hide the failed image
  target.style.display = 'none';
  
  // Create emoji fallback
  const emoji = document.createElement('span');
  emoji.textContent = config.fallbackEmoji;
  emoji.className = 'text-2xl';
  
  // Insert emoji before the image
  target.parentNode?.insertBefore(emoji, target);
}

/**
 * Flag component with automatic fallback
 */
export function FlagImage({ countryCode, className = "w-8 h-6 object-cover rounded-sm shadow-sm" }: {
  countryCode: string;
  className?: string;
}): JSX.Element {
  const config = getFlagConfig(countryCode);
  
  return (
    <img 
      src={getFlagUrl(countryCode)}
      alt={config.altText}
      className={className}
      onError={(e) => handleFlagError(e, countryCode)}
    />
  );
} 