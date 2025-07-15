import React from 'react';

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
  // Newly recognized countries from enhanced recognition
  'PAK': { countryCode: 'PAK', fallbackEmoji: '🇵🇰', altText: 'Pakistan Flag' },
  'KAZ': { countryCode: 'KAZ', fallbackEmoji: '🇰🇿', altText: 'Kazakhstan Flag' },
  'ISR': { countryCode: 'ISR', fallbackEmoji: '🇮🇱', altText: 'Israel Flag' },
  'IND': { countryCode: 'IND', fallbackEmoji: '🇮🇳', altText: 'India Flag' },
  'KSA': { countryCode: 'KSA', fallbackEmoji: '🇸🇦', altText: 'Saudi Arabia Flag' },
  // African countries downloaded from Wikipedia
  'ALG': { countryCode: 'ALG', fallbackEmoji: '🇩🇿', altText: 'Algeria Flag' },
  'ANG': { countryCode: 'ANG', fallbackEmoji: '🇦🇴', altText: 'Angola Flag' },
  'BEN': { countryCode: 'BEN', fallbackEmoji: '🇧🇯', altText: 'Benin Flag' },
  'BOT': { countryCode: 'BOT', fallbackEmoji: '🇧🇼', altText: 'Botswana Flag' },
  'BUR': { countryCode: 'BUR', fallbackEmoji: '🇧🇫', altText: 'Burkina Faso Flag' },
  'BDI': { countryCode: 'BDI', fallbackEmoji: '🇧🇮', altText: 'Burundi Flag' },
  'CMR': { countryCode: 'CMR', fallbackEmoji: '🇨🇲', altText: 'Cameroon Flag' },
  'CPV': { countryCode: 'CPV', fallbackEmoji: '🇨🇻', altText: 'Cape Verde Flag' },
  'CHA': { countryCode: 'CHA', fallbackEmoji: '🇹🇩', altText: 'Chad Flag' },
  'DJI': { countryCode: 'DJI', fallbackEmoji: '🇩🇯', altText: 'Djibouti Flag' },
  'EGY': { countryCode: 'EGY', fallbackEmoji: '🇪🇬', altText: 'Egypt Flag' },
  'GNQ': { countryCode: 'GNQ', fallbackEmoji: '🇬🇶', altText: 'Equatorial Guinea Flag' },
  'ERI': { countryCode: 'ERI', fallbackEmoji: '🇪🇷', altText: 'Eritrea Flag' },
  'ETH': { countryCode: 'ETH', fallbackEmoji: '🇪🇹', altText: 'Ethiopia Flag' },
  'GAB': { countryCode: 'GAB', fallbackEmoji: '🇬🇦', altText: 'Gabon Flag' },
  'GHA': { countryCode: 'GHA', fallbackEmoji: '🇬🇭', altText: 'Ghana Flag' },
  'GIN': { countryCode: 'GIN', fallbackEmoji: '🇬🇳', altText: 'Guinea Flag' },
  'GNB': { countryCode: 'GNB', fallbackEmoji: '🇬🇼', altText: 'Guinea-Bissau Flag' },
  'KEN': { countryCode: 'KEN', fallbackEmoji: '🇰🇪', altText: 'Kenya Flag' },
  'LES': { countryCode: 'LES', fallbackEmoji: '🇱🇸', altText: 'Lesotho Flag' },
  'LBR': { countryCode: 'LBR', fallbackEmoji: '🇱🇷', altText: 'Liberia Flag' },
  'LBY': { countryCode: 'LBY', fallbackEmoji: '🇱🇾', altText: 'Libya Flag' },
  'MDG': { countryCode: 'MDG', fallbackEmoji: '🇲🇬', altText: 'Madagascar Flag' },
  'MWI': { countryCode: 'MWI', fallbackEmoji: '🇲🇼', altText: 'Malawi Flag' },
  'MLI': { countryCode: 'MLI', fallbackEmoji: '🇲🇱', altText: 'Mali Flag' },
  'MRT': { countryCode: 'MRT', fallbackEmoji: '🇲🇷', altText: 'Mauritania Flag' },
  'MUS': { countryCode: 'MUS', fallbackEmoji: '🇲🇺', altText: 'Mauritius Flag' },
  'MAR': { countryCode: 'MAR', fallbackEmoji: '🇲🇦', altText: 'Morocco Flag' },
  'MOZ': { countryCode: 'MOZ', fallbackEmoji: '🇲🇿', altText: 'Mozambique Flag' },
  'NAM': { countryCode: 'NAM', fallbackEmoji: '🇳🇦', altText: 'Namibia Flag' },
  'NER': { countryCode: 'NER', fallbackEmoji: '🇳🇪', altText: 'Niger Flag' },
  'NGA': { countryCode: 'NGA', fallbackEmoji: '🇳🇬', altText: 'Nigeria Flag' },
  'RWA': { countryCode: 'RWA', fallbackEmoji: '🇷🇼', altText: 'Rwanda Flag' },
  'STP': { countryCode: 'STP', fallbackEmoji: '🇸🇹', altText: 'São Tomé and Príncipe Flag' },
  'SEN': { countryCode: 'SEN', fallbackEmoji: '🇸🇳', altText: 'Senegal Flag' },
  'SYC': { countryCode: 'SYC', fallbackEmoji: '🇸🇨', altText: 'Seychelles Flag' },
  'SLE': { countryCode: 'SLE', fallbackEmoji: '🇸🇱', altText: 'Sierra Leone Flag' },
  'SOM': { countryCode: 'SOM', fallbackEmoji: '🇸🇴', altText: 'Somalia Flag' },
  'ZAF': { countryCode: 'ZAF', fallbackEmoji: '🇿🇦', altText: 'South Africa Flag' },
  'SSD': { countryCode: 'SSD', fallbackEmoji: '🇸🇸', altText: 'South Sudan Flag' },
  'SDN': { countryCode: 'SDN', fallbackEmoji: '🇸🇩', altText: 'Sudan Flag' },
  'SWZ': { countryCode: 'SWZ', fallbackEmoji: '🇸🇿', altText: 'Swaziland Flag' },
  'TAN': { countryCode: 'TAN', fallbackEmoji: '🇹🇿', altText: 'Tanzania Flag' },
  'TGO': { countryCode: 'TGO', fallbackEmoji: '🇹🇬', altText: 'Togo Flag' },
  'TUN': { countryCode: 'TUN', fallbackEmoji: '🇹🇳', altText: 'Tunisia Flag' },
  'UGA': { countryCode: 'UGA', fallbackEmoji: '🇺🇬', altText: 'Uganda Flag' },
  'ZMB': { countryCode: 'ZMB', fallbackEmoji: '🇿🇲', altText: 'Zambia Flag' },
  'ZWE': { countryCode: 'ZWE', fallbackEmoji: '🇿🇼', altText: 'Zimbabwe Flag' }
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