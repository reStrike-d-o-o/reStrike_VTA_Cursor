import React from 'react';

// Flag utility functions for the sidebar component

export interface FlagConfig {
  countryCode: string;
  fallbackEmoji: string;
  altText: string;
}

// Comprehensive IOC codes from Wikipedia download (253 flags)
export const FLAG_CONFIGS: Record<string, FlagConfig> = {
  // Current NOCs (Table 1) - Main Olympic countries
  'AFG': { countryCode: 'AFG', fallbackEmoji: '🇦🇫', altText: 'Afghanistan Flag' },
  'ALB': { countryCode: 'ALB', fallbackEmoji: '🇦🇱', altText: 'Albania Flag' },
  'ALG': { countryCode: 'ALG', fallbackEmoji: '🇩🇿', altText: 'Algeria Flag' },
  'AND': { countryCode: 'AND', fallbackEmoji: '🇦🇩', altText: 'Andorra Flag' },
  'ANG': { countryCode: 'ANG', fallbackEmoji: '🇦🇴', altText: 'Angola Flag' },
  'ANT': { countryCode: 'ANT', fallbackEmoji: '🇦🇬', altText: 'Antigua and Barbuda Flag' },
  'ARG': { countryCode: 'ARG', fallbackEmoji: '🇦🇷', altText: 'Argentina Flag' },
  'ARM': { countryCode: 'ARM', fallbackEmoji: '🇦🇲', altText: 'Armenia Flag' },
  'ARU': { countryCode: 'ARU', fallbackEmoji: '🇦🇼', altText: 'Aruba Flag' },
  'ASA': { countryCode: 'ASA', fallbackEmoji: '🇦🇸', altText: 'American Samoa Flag' },
  'AUS': { countryCode: 'AUS', fallbackEmoji: '🇦🇺', altText: 'Australia Flag' },
  'AUT': { countryCode: 'AUT', fallbackEmoji: '🇦🇹', altText: 'Austria Flag' },
  'AZE': { countryCode: 'AZE', fallbackEmoji: '🇦🇿', altText: 'Azerbaijan Flag' },
  'BAH': { countryCode: 'BAH', fallbackEmoji: '🇧🇸', altText: 'Bahamas Flag' },
  'BAN': { countryCode: 'BAN', fallbackEmoji: '🇧🇩', altText: 'Bangladesh Flag' },
  'BAR': { countryCode: 'BAR', fallbackEmoji: '🇧🇧', altText: 'Barbados Flag' },
  'BDI': { countryCode: 'BDI', fallbackEmoji: '🇧🇮', altText: 'Burundi Flag' },
  'BEL': { countryCode: 'BEL', fallbackEmoji: '🇧🇪', altText: 'Belgium Flag' },
  'BEN': { countryCode: 'BEN', fallbackEmoji: '🇧🇯', altText: 'Benin Flag' },
  'BER': { countryCode: 'BER', fallbackEmoji: '🇧🇲', altText: 'Bermuda Flag' },
  'BHU': { countryCode: 'BHU', fallbackEmoji: '🇧🇹', altText: 'Bhutan Flag' },
  'BIH': { countryCode: 'BIH', fallbackEmoji: '🇧🇦', altText: 'Bosnia and Herzegovina Flag' },
  'BIZ': { countryCode: 'BIZ', fallbackEmoji: '🇧🇿', altText: 'Belize Flag' },
  'BLR': { countryCode: 'BLR', fallbackEmoji: '🇧🇾', altText: 'Belarus Flag' },
  'BOL': { countryCode: 'BOL', fallbackEmoji: '🇧🇴', altText: 'Bolivia Flag' },
  'BOT': { countryCode: 'BOT', fallbackEmoji: '🇧🇼', altText: 'Botswana Flag' },
  'BRA': { countryCode: 'BRA', fallbackEmoji: '🇧🇷', altText: 'Brazil Flag' },
  'BRN': { countryCode: 'BRN', fallbackEmoji: '🇧🇭', altText: 'Bahrain Flag' },
  'BRU': { countryCode: 'BRU', fallbackEmoji: '🇧🇳', altText: 'Brunei Flag' },
  'BUL': { countryCode: 'BUL', fallbackEmoji: '🇧🇬', altText: 'Bulgaria Flag' },
  'BUR': { countryCode: 'BUR', fallbackEmoji: '🇧🇫', altText: 'Burkina Faso Flag' },
  'CAF': { countryCode: 'CAF', fallbackEmoji: '🇨🇫', altText: 'Central African Republic Flag' },
  'CAM': { countryCode: 'CAM', fallbackEmoji: '🇰🇭', altText: 'Cambodia Flag' },
  'CAN': { countryCode: 'CAN', fallbackEmoji: '🇨🇦', altText: 'Canada Flag' },
  'CAY': { countryCode: 'CAY', fallbackEmoji: '🇰🇾', altText: 'Cayman Islands Flag' },
  'CGO': { countryCode: 'CGO', fallbackEmoji: '🇨🇬', altText: 'Republic of the Congo Flag' },
  'CHA': { countryCode: 'CHA', fallbackEmoji: '🇹🇩', altText: 'Chad Flag' },
  'CHI': { countryCode: 'CHI', fallbackEmoji: '🇨🇱', altText: 'Chile Flag' },
  'CIV': { countryCode: 'CIV', fallbackEmoji: '🇨🇮', altText: 'Ivory Coast Flag' },
  'CMR': { countryCode: 'CMR', fallbackEmoji: '🇨🇲', altText: 'Cameroon Flag' },
  'COD': { countryCode: 'COD', fallbackEmoji: '🇨🇩', altText: 'Democratic Republic of the Congo Flag' },
  'COK': { countryCode: 'COK', fallbackEmoji: '🇨🇰', altText: 'Cook Islands Flag' },
  'COL': { countryCode: 'COL', fallbackEmoji: '🇨🇴', altText: 'Colombia Flag' },
  'COM': { countryCode: 'COM', fallbackEmoji: '🇰🇲', altText: 'Comoros Flag' },
  'CPV': { countryCode: 'CPV', fallbackEmoji: '🇨🇻', altText: 'Cape Verde Flag' },
  'CRC': { countryCode: 'CRC', fallbackEmoji: '🇨🇷', altText: 'Costa Rica Flag' },
  'CRO': { countryCode: 'CRO', fallbackEmoji: '🇭🇷', altText: 'Croatia Flag' },
  'CUB': { countryCode: 'CUB', fallbackEmoji: '🇨🇺', altText: 'Cuba Flag' },
  'CYP': { countryCode: 'CYP', fallbackEmoji: '🇨🇾', altText: 'Cyprus Flag' },
  'CZE': { countryCode: 'CZE', fallbackEmoji: '🇨🇿', altText: 'Czechia Flag' },
  'DEN': { countryCode: 'DEN', fallbackEmoji: '🇩🇰', altText: 'Denmark Flag' },
  'DJI': { countryCode: 'DJI', fallbackEmoji: '🇩🇯', altText: 'Djibouti Flag' },
  'DMA': { countryCode: 'DMA', fallbackEmoji: '🇩🇲', altText: 'Dominica Flag' },
  'DOM': { countryCode: 'DOM', fallbackEmoji: '🇩🇴', altText: 'Dominican Republic Flag' },
  'ECU': { countryCode: 'ECU', fallbackEmoji: '🇪🇨', altText: 'Ecuador Flag' },
  'EGY': { countryCode: 'EGY', fallbackEmoji: '🇪🇬', altText: 'Egypt Flag' },
  'ERI': { countryCode: 'ERI', fallbackEmoji: '🇪🇷', altText: 'Eritrea Flag' },
  'ESA': { countryCode: 'ESA', fallbackEmoji: '🇸🇻', altText: 'El Salvador Flag' },
  'ESP': { countryCode: 'ESP', fallbackEmoji: '🇪🇸', altText: 'Spain Flag' },
  'EST': { countryCode: 'EST', fallbackEmoji: '🇪🇪', altText: 'Estonia Flag' },
  'ETH': { countryCode: 'ETH', fallbackEmoji: '🇪🇹', altText: 'Ethiopia Flag' },
  'FIJ': { countryCode: 'FIJ', fallbackEmoji: '🇫🇯', altText: 'Fiji Flag' },
  'FIN': { countryCode: 'FIN', fallbackEmoji: '🇫🇮', altText: 'Finland Flag' },
  'FRA': { countryCode: 'FRA', fallbackEmoji: '🇫🇷', altText: 'France Flag' },
  'FSM': { countryCode: 'FSM', fallbackEmoji: '🇫🇲', altText: 'Federated States of Micronesia Flag' },
  'GAB': { countryCode: 'GAB', fallbackEmoji: '🇬🇦', altText: 'Gabon Flag' },
  'GAM': { countryCode: 'GAM', fallbackEmoji: '🇬🇲', altText: 'The Gambia Flag' },
  'GBR': { countryCode: 'GBR', fallbackEmoji: '🇬🇧', altText: 'Great Britain Flag' },
  'GBS': { countryCode: 'GBS', fallbackEmoji: '🇬🇼', altText: 'Guinea-Bissau Flag' },
  'GEO': { countryCode: 'GEO', fallbackEmoji: '🇬🇪', altText: 'Georgia Flag' },
  'GEQ': { countryCode: 'GEQ', fallbackEmoji: '🇬🇶', altText: 'Equatorial Guinea Flag' },
  'GER': { countryCode: 'GER', fallbackEmoji: '🇩🇪', altText: 'Germany Flag' },
  'GHA': { countryCode: 'GHA', fallbackEmoji: '🇬🇭', altText: 'Ghana Flag' },
  'GRE': { countryCode: 'GRE', fallbackEmoji: '🇬🇷', altText: 'Greece Flag' },
  'GRN': { countryCode: 'GRN', fallbackEmoji: '🇬🇩', altText: 'Grenada Flag' },
  'GUA': { countryCode: 'GUA', fallbackEmoji: '🇬🇹', altText: 'Guatemala Flag' },
  'GUI': { countryCode: 'GUI', fallbackEmoji: '🇬🇳', altText: 'Guinea Flag' },
  'GUM': { countryCode: 'GUM', fallbackEmoji: '🇬🇺', altText: 'Guam Flag' },
  'GUY': { countryCode: 'GUY', fallbackEmoji: '🇬🇾', altText: 'Guyana Flag' },
  'HAI': { countryCode: 'HAI', fallbackEmoji: '🇭🇹', altText: 'Haiti Flag' },
  'HKG': { countryCode: 'HKG', fallbackEmoji: '🇭🇰', altText: 'Hong Kong Flag' },
  'HON': { countryCode: 'HON', fallbackEmoji: '🇭🇳', altText: 'Honduras Flag' },
  'HUN': { countryCode: 'HUN', fallbackEmoji: '🇭🇺', altText: 'Hungary Flag' },
  'INA': { countryCode: 'INA', fallbackEmoji: '🇮🇩', altText: 'Indonesia Flag' },
  'IND': { countryCode: 'IND', fallbackEmoji: '🇮🇳', altText: 'India Flag' },
  'IRI': { countryCode: 'IRI', fallbackEmoji: '🇮🇷', altText: 'Iran Flag' },
  'IRL': { countryCode: 'IRL', fallbackEmoji: '🇮🇪', altText: 'Ireland Flag' },
  'IRQ': { countryCode: 'IRQ', fallbackEmoji: '🇮🇶', altText: 'Iraq Flag' },
  'ISL': { countryCode: 'ISL', fallbackEmoji: '🇮🇸', altText: 'Iceland Flag' },
  'ISR': { countryCode: 'ISR', fallbackEmoji: '🇮🇱', altText: 'Israel Flag' },
  'ISV': { countryCode: 'ISV', fallbackEmoji: '🇻🇮', altText: 'Virgin Islands Flag' },
  'ITA': { countryCode: 'ITA', fallbackEmoji: '🇮🇹', altText: 'Italy Flag' },
  'IVB': { countryCode: 'IVB', fallbackEmoji: '🇻🇬', altText: 'British Virgin Islands Flag' },
  'JAM': { countryCode: 'JAM', fallbackEmoji: '🇯🇲', altText: 'Jamaica Flag' },
  'JOR': { countryCode: 'JOR', fallbackEmoji: '🇯🇴', altText: 'Jordan Flag' },
  'JPN': { countryCode: 'JPN', fallbackEmoji: '🇯🇵', altText: 'Japan Flag' },
  'KAZ': { countryCode: 'KAZ', fallbackEmoji: '🇰🇿', altText: 'Kazakhstan Flag' },
  'KEN': { countryCode: 'KEN', fallbackEmoji: '🇰🇪', altText: 'Kenya Flag' },
  'KGZ': { countryCode: 'KGZ', fallbackEmoji: '🇰🇬', altText: 'Kyrgyzstan Flag' },
  'KIR': { countryCode: 'KIR', fallbackEmoji: '🇰🇮', altText: 'Kiribati Flag' },
  'KOR': { countryCode: 'KOR', fallbackEmoji: '🇰🇷', altText: 'South Korea Flag' },
  'KOS': { countryCode: 'KOS', fallbackEmoji: '🇽🇰', altText: 'Kosovo Flag' },
  'KSA': { countryCode: 'KSA', fallbackEmoji: '🇸🇦', altText: 'Saudi Arabia Flag' },
  'KUW': { countryCode: 'KUW', fallbackEmoji: '🇰🇼', altText: 'Kuwait Flag' },
  'LAO': { countryCode: 'LAO', fallbackEmoji: '🇱🇦', altText: 'Laos Flag' },
  'LAT': { countryCode: 'LAT', fallbackEmoji: '🇱🇻', altText: 'Latvia Flag' },
  'LBA': { countryCode: 'LBA', fallbackEmoji: '🇱🇾', altText: 'Libya Flag' },
  'LBN': { countryCode: 'LBN', fallbackEmoji: '🇱🇧', altText: 'Lebanon Flag' },
  'LBR': { countryCode: 'LBR', fallbackEmoji: '🇱🇷', altText: 'Liberia Flag' },
  'LCA': { countryCode: 'LCA', fallbackEmoji: '🇱🇨', altText: 'Saint Lucia Flag' },
  'LES': { countryCode: 'LES', fallbackEmoji: '🇱🇸', altText: 'Lesotho Flag' },
  'LIE': { countryCode: 'LIE', fallbackEmoji: '🇱🇮', altText: 'Liechtenstein Flag' },
  'LTU': { countryCode: 'LTU', fallbackEmoji: '🇱🇹', altText: 'Lithuania Flag' },
  'LUX': { countryCode: 'LUX', fallbackEmoji: '🇱🇺', altText: 'Luxembourg Flag' },
  'MAD': { countryCode: 'MAD', fallbackEmoji: '🇲🇬', altText: 'Madagascar Flag' },
  'MAR': { countryCode: 'MAR', fallbackEmoji: '🇲🇦', altText: 'Morocco Flag' },
  'MAS': { countryCode: 'MAS', fallbackEmoji: '🇲🇾', altText: 'Malaysia Flag' },
  'MAW': { countryCode: 'MAW', fallbackEmoji: '🇲🇼', altText: 'Malawi Flag' },
  'MDA': { countryCode: 'MDA', fallbackEmoji: '🇲🇩', altText: 'Moldova Flag' },
  'MDV': { countryCode: 'MDV', fallbackEmoji: '🇲🇻', altText: 'Maldives Flag' },
  'MEX': { countryCode: 'MEX', fallbackEmoji: '🇲🇽', altText: 'Mexico Flag' },
  'MGL': { countryCode: 'MGL', fallbackEmoji: '🇲🇳', altText: 'Mongolia Flag' },
  'MHL': { countryCode: 'MHL', fallbackEmoji: '🇲🇭', altText: 'Marshall Islands Flag' },
  'MKD': { countryCode: 'MKD', fallbackEmoji: '🇲🇰', altText: 'North Macedonia Flag' },
  'MLI': { countryCode: 'MLI', fallbackEmoji: '🇲🇱', altText: 'Mali Flag' },
  'MLT': { countryCode: 'MLT', fallbackEmoji: '🇲🇹', altText: 'Malta Flag' },
  'MNE': { countryCode: 'MNE', fallbackEmoji: '🇲🇪', altText: 'Montenegro Flag' },
  'MON': { countryCode: 'MON', fallbackEmoji: '🇲🇨', altText: 'Monaco Flag' },
  'MOZ': { countryCode: 'MOZ', fallbackEmoji: '🇲🇿', altText: 'Mozambique Flag' },
  'MRI': { countryCode: 'MRI', fallbackEmoji: '🇲🇺', altText: 'Mauritius Flag' },
  'MTN': { countryCode: 'MTN', fallbackEmoji: '🇲🇷', altText: 'Mauritania Flag' },
  'MYA': { countryCode: 'MYA', fallbackEmoji: '🇲🇲', altText: 'Myanmar Flag' },
  'NAM': { countryCode: 'NAM', fallbackEmoji: '🇳🇦', altText: 'Namibia Flag' },
  'NCA': { countryCode: 'NCA', fallbackEmoji: '🇳🇮', altText: 'Nicaragua Flag' },
  'NED': { countryCode: 'NED', fallbackEmoji: '🇳🇱', altText: 'Netherlands Flag' },
  'NEP': { countryCode: 'NEP', fallbackEmoji: '🇳🇵', altText: 'Nepal Flag' },
  'NGR': { countryCode: 'NGR', fallbackEmoji: '🇳🇬', altText: 'Nigeria Flag' },
  'NIG': { countryCode: 'NIG', fallbackEmoji: '🇳🇪', altText: 'Niger Flag' },
  'NOR': { countryCode: 'NOR', fallbackEmoji: '🇳🇴', altText: 'Norway Flag' },
  'NRU': { countryCode: 'NRU', fallbackEmoji: '🇳🇷', altText: 'Nauru Flag' },
  'NZL': { countryCode: 'NZL', fallbackEmoji: '🇳🇿', altText: 'New Zealand Flag' },
  'OMA': { countryCode: 'OMA', fallbackEmoji: '🇴🇲', altText: 'Oman Flag' },
  'PAK': { countryCode: 'PAK', fallbackEmoji: '🇵🇰', altText: 'Pakistan Flag' },
  'PAN': { countryCode: 'PAN', fallbackEmoji: '🇵🇦', altText: 'Panama Flag' },
  'PAR': { countryCode: 'PAR', fallbackEmoji: '🇵🇾', altText: 'Paraguay Flag' },
  'PER': { countryCode: 'PER', fallbackEmoji: '🇵🇪', altText: 'Peru Flag' },
  'PHI': { countryCode: 'PHI', fallbackEmoji: '🇵🇭', altText: 'Philippines Flag' },
  'PLE': { countryCode: 'PLE', fallbackEmoji: '🇵🇸', altText: 'Palestine Flag' },
  'PLW': { countryCode: 'PLW', fallbackEmoji: '🇵🇼', altText: 'Palau Flag' },
  'PNG': { countryCode: 'PNG', fallbackEmoji: '🇵🇬', altText: 'Papua New Guinea Flag' },
  'POL': { countryCode: 'POL', fallbackEmoji: '🇵🇱', altText: 'Poland Flag' },
  'POR': { countryCode: 'POR', fallbackEmoji: '🇵🇹', altText: 'Portugal Flag' },
  'PRK': { countryCode: 'PRK', fallbackEmoji: '🇰🇵', altText: 'North Korea Flag' },
  'PUR': { countryCode: 'PUR', fallbackEmoji: '🇵🇷', altText: 'Puerto Rico Flag' },
  'QAT': { countryCode: 'QAT', fallbackEmoji: '🇶🇦', altText: 'Qatar Flag' },
  'ROU': { countryCode: 'ROU', fallbackEmoji: '🇷🇴', altText: 'Romania Flag' },
  'RSA': { countryCode: 'RSA', fallbackEmoji: '🇿🇦', altText: 'South Africa Flag' },
  'RUS': { countryCode: 'RUS', fallbackEmoji: '🇷🇺', altText: 'Russia Flag' },
  'RWA': { countryCode: 'RWA', fallbackEmoji: '🇷🇼', altText: 'Rwanda Flag' },
  'SAM': { countryCode: 'SAM', fallbackEmoji: '🇼🇸', altText: 'Samoa Flag' },
  'SEN': { countryCode: 'SEN', fallbackEmoji: '🇸🇳', altText: 'Senegal Flag' },
  'SEY': { countryCode: 'SEY', fallbackEmoji: '🇸🇨', altText: 'Seychelles Flag' },
  'SGP': { countryCode: 'SGP', fallbackEmoji: '🇸🇬', altText: 'Singapore Flag' },
  'SKN': { countryCode: 'SKN', fallbackEmoji: '🇰🇳', altText: 'Saint Kitts and Nevis Flag' },
  'SLE': { countryCode: 'SLE', fallbackEmoji: '🇸🇱', altText: 'Sierra Leone Flag' },
  'SLO': { countryCode: 'SLO', fallbackEmoji: '🇸🇮', altText: 'Slovenia Flag' },
  'SMR': { countryCode: 'SMR', fallbackEmoji: '🇸🇲', altText: 'San Marino Flag' },
  'SOL': { countryCode: 'SOL', fallbackEmoji: '🇸🇧', altText: 'Solomon Islands Flag' },
  'SOM': { countryCode: 'SOM', fallbackEmoji: '🇸🇴', altText: 'Somalia Flag' },
  'SRB': { countryCode: 'SRB', fallbackEmoji: '🇷🇸', altText: 'Serbia Flag' },
  'SRI': { countryCode: 'SRI', fallbackEmoji: '🇱🇰', altText: 'Sri Lanka Flag' },
  'SSD': { countryCode: 'SSD', fallbackEmoji: '🇸🇸', altText: 'South Sudan Flag' },
  'STP': { countryCode: 'STP', fallbackEmoji: '🇸🇹', altText: 'São Tomé and Príncipe Flag' },
  'SUD': { countryCode: 'SUD', fallbackEmoji: '🇸🇩', altText: 'Sudan Flag' },
  'SUI': { countryCode: 'SUI', fallbackEmoji: '🇨🇭', altText: 'Switzerland Flag' },
  'SUR': { countryCode: 'SUR', fallbackEmoji: '🇸🇷', altText: 'Suriname Flag' },
  'SVK': { countryCode: 'SVK', fallbackEmoji: '🇸🇰', altText: 'Slovakia Flag' },
  'SWE': { countryCode: 'SWE', fallbackEmoji: '🇸🇪', altText: 'Sweden Flag' },
  'SWZ': { countryCode: 'SWZ', fallbackEmoji: '🇸🇿', altText: 'Eswatini Flag' },
  'SYR': { countryCode: 'SYR', fallbackEmoji: '🇸🇾', altText: 'Syria Flag' },
  'TAN': { countryCode: 'TAN', fallbackEmoji: '🇹🇿', altText: 'Tanzania Flag' },
  'TGA': { countryCode: 'TGA', fallbackEmoji: '🇹🇴', altText: 'Tonga Flag' },
  'THA': { countryCode: 'THA', fallbackEmoji: '🇹🇭', altText: 'Thailand Flag' },
  'TJK': { countryCode: 'TJK', fallbackEmoji: '🇹🇯', altText: 'Tajikistan Flag' },
  'TKM': { countryCode: 'TKM', fallbackEmoji: '🇹🇲', altText: 'Turkmenistan Flag' },
  'TLS': { countryCode: 'TLS', fallbackEmoji: '🇹🇱', altText: 'Timor-Leste Flag' },
  'TOG': { countryCode: 'TOG', fallbackEmoji: '🇹🇬', altText: 'Togo Flag' },
  'TTO': { countryCode: 'TTO', fallbackEmoji: '🇹🇹', altText: 'Trinidad and Tobago Flag' },
  'TUN': { countryCode: 'TUN', fallbackEmoji: '🇹🇳', altText: 'Tunisia Flag' },
  'TUR': { countryCode: 'TUR', fallbackEmoji: '🇹🇷', altText: 'Turkey Flag' },
  'TUV': { countryCode: 'TUV', fallbackEmoji: '🇹🇻', altText: 'Tuvalu Flag' },
  'UAE': { countryCode: 'UAE', fallbackEmoji: '🇦🇪', altText: 'United Arab Emirates Flag' },
  'UGA': { countryCode: 'UGA', fallbackEmoji: '🇺🇬', altText: 'Uganda Flag' },
  'UKR': { countryCode: 'UKR', fallbackEmoji: '🇺🇦', altText: 'Ukraine Flag' },
  'URU': { countryCode: 'URU', fallbackEmoji: '🇺🇾', altText: 'Uruguay Flag' },
  'USA': { countryCode: 'USA', fallbackEmoji: '🇺🇸', altText: 'United States Flag' },
  'UZB': { countryCode: 'UZB', fallbackEmoji: '🇺🇿', altText: 'Uzbekistan Flag' },
  'VAN': { countryCode: 'VAN', fallbackEmoji: '🇻🇺', altText: 'Vanuatu Flag' },
  'VEN': { countryCode: 'VEN', fallbackEmoji: '🇻🇪', altText: 'Venezuela Flag' },
  'VIE': { countryCode: 'VIE', fallbackEmoji: '🇻🇳', altText: 'Vietnam Flag' },
  'VIN': { countryCode: 'VIN', fallbackEmoji: '🇻🇨', altText: 'Saint Vincent and the Grenadines Flag' },
  'YEM': { countryCode: 'YEM', fallbackEmoji: '🇾🇪', altText: 'Yemen Flag' },
  'ZAM': { countryCode: 'ZAM', fallbackEmoji: '🇿🇲', altText: 'Zambia Flag' },
  'ZIM': { countryCode: 'ZIM', fallbackEmoji: '🇿🇼', altText: 'Zimbabwe Flag' },

  // Additional territories (Table 2)
  'FRO': { countryCode: 'FRO', fallbackEmoji: '🇫🇴', altText: 'Faroe Islands Flag' },
  'MAC': { countryCode: 'MAC', fallbackEmoji: '🇲🇴', altText: 'Macau Flag' },

  // Historic NOCs (Table 3)
  'AHO': { countryCode: 'AHO', fallbackEmoji: '🇧🇶', altText: 'Netherlands Antilles Flag' },
  'ANZ': { countryCode: 'ANZ', fallbackEmoji: '🏳️', altText: 'Australasia Flag' },
  'BOH': { countryCode: 'BOH', fallbackEmoji: '🇨🇿', altText: 'Bohemia Flag' },
  'BWI': { countryCode: 'BWI', fallbackEmoji: '🏳️', altText: 'British West Indies Flag' },
  'EUA': { countryCode: 'EUA', fallbackEmoji: '🇩🇪', altText: 'United Team of Germany Flag' },
  'EUN': { countryCode: 'EUN', fallbackEmoji: '🏳️', altText: 'Unified Team Flag' },
  'FRG': { countryCode: 'FRG', fallbackEmoji: '🇩🇪', altText: 'West Germany Flag' },
  'GDR': { countryCode: 'GDR', fallbackEmoji: '🇩🇪', altText: 'East Germany Flag' },
  'SCG': { countryCode: 'SCG', fallbackEmoji: '🇷🇸', altText: 'Serbia and Montenegro Flag' },
  'TCH': { countryCode: 'TCH', fallbackEmoji: '🇨🇿', altText: 'Czechoslovakia Flag' },
  'URS': { countryCode: 'URS', fallbackEmoji: '🇷🇺', altText: 'Soviet Union Flag' },
  'VNM': { countryCode: 'VNM', fallbackEmoji: '🇻🇳', altText: 'South Vietnam Flag' },
  'YUG': { countryCode: 'YUG', fallbackEmoji: '🇷🇸', altText: 'Yugoslavia Flag' },

  // Historic country names (Table 4)
  'BIR': { countryCode: 'BIR', fallbackEmoji: '🇲🇲', altText: 'Burma Flag' },
  'CEY': { countryCode: 'CEY', fallbackEmoji: '🇱🇰', altText: 'Ceylon Flag' },
  'DAH': { countryCode: 'DAH', fallbackEmoji: '🇧🇯', altText: 'Dahomey Flag' },
  'HBR': { countryCode: 'HBR', fallbackEmoji: '🇧🇿', altText: 'British Honduras Flag' },
  'IHO': { countryCode: 'IHO', fallbackEmoji: '🇮🇩', altText: 'Dutch East Indies Flag' },
  'KHM': { countryCode: 'KHM', fallbackEmoji: '🇰🇭', altText: 'Khmer Republic Flag' },
  'MAL': { countryCode: 'MAL', fallbackEmoji: '🇲🇾', altText: 'Malaya Flag' },
  'NBO': { countryCode: 'NBO', fallbackEmoji: '🇲🇾', altText: 'North Borneo Flag' },
  'NRH': { countryCode: 'NRH', fallbackEmoji: '🇿🇲', altText: 'Northern Rhodesia Flag' },
  'RAU': { countryCode: 'RAU', fallbackEmoji: '🇪🇬', altText: 'United Arab Republic Flag' },
  'RHO': { countryCode: 'RHO', fallbackEmoji: '🇿🇼', altText: 'Rhodesia Flag' },
  'ROC': { countryCode: 'ROC', fallbackEmoji: '🇹🇼', altText: 'Republic of China Flag' },
  'SAA': { countryCode: 'SAA', fallbackEmoji: '🇩🇪', altText: 'Saar Flag' },
  'UAR': { countryCode: 'UAR', fallbackEmoji: '🇪🇬', altText: 'United Arab Republic Flag' },
  'VOL': { countryCode: 'VOL', fallbackEmoji: '🇧🇫', altText: 'Upper Volta Flag' },
  'WSM': { countryCode: 'WSM', fallbackEmoji: '🇼🇸', altText: 'Western Samoa Flag' },
  'YAR': { countryCode: 'YAR', fallbackEmoji: '🇾🇪', altText: 'North Yemen Flag' },
  'YMD': { countryCode: 'YMD', fallbackEmoji: '🇾🇪', altText: 'South Yemen Flag' },
  'ZAI': { countryCode: 'ZAI', fallbackEmoji: '🇨🇩', altText: 'Zaire Flag' },

  // Special Olympic codes (Table 5)
  'AIN': { countryCode: 'AIN', fallbackEmoji: '🏳️', altText: 'Individual Neutral Athletes Flag' },
  'COR': { countryCode: 'COR', fallbackEmoji: '🇰🇷', altText: 'Korea Flag' },
  'EOR': { countryCode: 'EOR', fallbackEmoji: '🏳️', altText: 'Refugee Olympic Team Flag' },
  'IOP': { countryCode: 'IOP', fallbackEmoji: '🏳️', altText: 'Independent Olympic Participants Flag' },
  'IOA': { countryCode: 'IOA', fallbackEmoji: '🏳️', altText: 'Independent Olympic Athletes Flag' },
  'IOC': { countryCode: 'IOC', fallbackEmoji: '🏳️', altText: 'Athletes from Kuwait Flag' },
  'MIX': { countryCode: 'MIX', fallbackEmoji: '🏳️', altText: 'Mixed-NOCs Flag' },
  'OAR': { countryCode: 'OAR', fallbackEmoji: '🏳️', altText: 'Olympic Athletes from Russia Flag' },
  'XXB': { countryCode: 'XXB', fallbackEmoji: '🏳️', altText: 'Mixed team Flag' },

  // Special Paralympic codes (Table 6)
  'IPP': { countryCode: 'IPP', fallbackEmoji: '🏳️', altText: 'Independent Paralympic Participants Flag' },
  'IPA': { countryCode: 'IPA', fallbackEmoji: '🏳️', altText: 'Individual Paralympic Athletes Flag' },
  'NPA': { countryCode: 'NPA', fallbackEmoji: '🏳️', altText: 'Neutral Paralympic Athletes Flag' },
  'PNA': { countryCode: 'PNA', fallbackEmoji: '🏳️', altText: 'Paralympic Neutral Athletes Flag' },
  'RPC': { countryCode: 'RPC', fallbackEmoji: '🏳️', altText: 'Russian Paralympic Committee Flag' },
  'RPT': { countryCode: 'RPT', fallbackEmoji: '🏳️', altText: 'Refugee Paralympic Team Flag' }
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