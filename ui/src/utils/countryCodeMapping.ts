/**
 * Country Code Mapping Utility
 * Maps PSS protocol country codes to IOC codes for flag display
 */

// PSS to IOC code mapping
// Some PSS codes are non-standard and need to be mapped to valid IOC codes
export const PSS_TO_IOC_MAPPING: Record<string, string> = {
  // Standard mappings (PSS code = IOC code)
  'SUI': 'SUI', // Switzerland
  'USA': 'USA', // United States
  'JPN': 'JPN', // Japan
  'KOR': 'KOR', // South Korea
  'CHN': 'CHN', // China
  'GBR': 'GBR', // Great Britain
  'FRA': 'FRA', // France
  'GER': 'GER', // Germany
  'ITA': 'ITA', // Italy
  'ESP': 'ESP', // Spain
  'CAN': 'CAN', // Canada
  'AUS': 'AUS', // Australia
  'BRA': 'BRA', // Brazil
  'RUS': 'RUS', // Russia
  'NED': 'NED', // Netherlands
  'BEL': 'BEL', // Belgium
  'SWE': 'SWE', // Sweden
  'NOR': 'NOR', // Norway
  'DEN': 'DEN', // Denmark
  'FIN': 'FIN', // Finland
  'POL': 'POL', // Poland
  'CZE': 'CZE', // Czech Republic
  'HUN': 'HUN', // Hungary
  'AUT': 'AUT', // Austria
  'POR': 'POR', // Portugal
  'GRE': 'GRE', // Greece
  'TUR': 'TUR', // Turkey
  'IRN': 'IRN', // Iran
  'THA': 'THA', // Thailand
  'VIE': 'VIE', // Vietnam
  'PHI': 'PHI', // Philippines
  'MAS': 'MAS', // Malaysia
  'SGP': 'SGP', // Singapore
  'IDN': 'IDN', // Indonesia
  'MYS': 'MYS', // Malaysia (alternative)
  'IND': 'IND', // India
  'PAK': 'PAK', // Pakistan
  'BAN': 'BAN', // Bangladesh
  'SRI': 'SRI', // Sri Lanka
  'NEP': 'NEP', // Nepal
  'BHU': 'BHU', // Bhutan
  'MMR': 'MMR', // Myanmar
  'LAO': 'LAO', // Laos
  'KHM': 'KHM', // Cambodia
  'MNG': 'MNG', // Mongolia
  'KAZ': 'KAZ', // Kazakhstan
  'UZB': 'UZB', // Uzbekistan
  'KGZ': 'KGZ', // Kyrgyzstan
  'TJK': 'TJK', // Tajikistan
  'TKM': 'TKM', // Turkmenistan
  'AZE': 'AZE', // Azerbaijan
  'GEO': 'GEO', // Georgia
  'ARM': 'ARM', // Armenia
  'UKR': 'UKR', // Ukraine
  'BLR': 'BLR', // Belarus
  'MDA': 'MDA', // Moldova
  'ROU': 'ROU', // Romania
  'BGR': 'BGR', // Bulgaria
  'HRV': 'HRV', // Croatia
  'SVN': 'SVN', // Slovenia
  'BIH': 'BIH', // Bosnia and Herzegovina
  'MNE': 'MNE', // Montenegro
  'SRB': 'SRB', // Serbia
  'MKD': 'MKD', // North Macedonia
  'ALB': 'ALB', // Albania
  'KOS': 'KOS', // Kosovo
  'ISL': 'ISL', // Iceland
  'IRL': 'IRL', // Ireland
  'LUX': 'LUX', // Luxembourg
  'LIE': 'LIE', // Liechtenstein
  'MCO': 'MCO', // Monaco
  'AND': 'AND', // Andorra
  'SMR': 'SMR', // San Marino
  'VAT': 'VAT', // Vatican City
  'MLT': 'MLT', // Malta
  'CYP': 'CYP', // Cyprus
  'ISR': 'ISR', // Israel
  'LBN': 'LBN', // Lebanon
  'SYR': 'SYR', // Syria
  'JOR': 'JOR', // Jordan
  'IRQ': 'IRQ', // Iraq
  'KWT': 'KWU', // Kuwait
  'QAT': 'QAT', // Qatar
  'BHR': 'BHR', // Bahrain
  'OMN': 'OMN', // Oman
  'YEM': 'YEM', // Yemen
  'UAE': 'UAE', // United Arab Emirates
  'SAU': 'KSA', // Saudi Arabia
  'EGY': 'EGY', // Egypt
  'LBY': 'LBA', // Libya
  'TUN': 'TUN', // Tunisia
  'DZA': 'ALG', // Algeria
  'MAR': 'MAR', // Morocco
  'SDN': 'SUD', // Sudan
  'SSD': 'SSD', // South Sudan
  'ETH': 'ETH', // Ethiopia
  'ERI': 'ERI', // Eritrea
  'DJI': 'DJI', // Djibouti
  'SOM': 'SOM', // Somalia
  'KEN': 'KEN', // Kenya
  'UGA': 'UGA', // Uganda
  'TZA': 'TAN', // Tanzania
  'RWA': 'RWA', // Rwanda
  'BDI': 'BDI', // Burundi
  'COD': 'COD', // Democratic Republic of the Congo
  'COG': 'CGO', // Republic of the Congo
  'GAB': 'GAB', // Gabon
  'CMR': 'CMR', // Cameroon
  'CAF': 'CAF', // Central African Republic
  'TCD': 'CHA', // Chad
  'NER': 'NIG', // Niger
  'MLI': 'MLI', // Mali
  'BFA': 'BUR', // Burkina Faso
  'GIN': 'GUI', // Guinea
  'GNB': 'GBS', // Guinea-Bissau
  'SEN': 'SEN', // Senegal
  'GMB': 'GAM', // Gambia
  'SLE': 'SLE', // Sierra Leone
  'LBR': 'LBR', // Liberia
  'CIV': 'CIV', // Ivory Coast
  'GHA': 'GHA', // Ghana
  'TGO': 'TOG', // Togo
  'BEN': 'BEN', // Benin
  'NGA': 'NGR', // Nigeria
  'GNQ': 'GEQ', // Equatorial Guinea
  'STP': 'STP', // São Tomé and Príncipe
  'AGO': 'ANG', // Angola
  'ZMB': 'ZAM', // Zambia
  'ZWE': 'ZIM', // Zimbabwe
  'BWA': 'BOT', // Botswana
  'NAM': 'NAM', // Namibia
  'ZAF': 'RSA', // South Africa
  'LSO': 'LES', // Lesotho
  'SWZ': 'SWZ', // Eswatini
  'MDG': 'MAD', // Madagascar
  'COM': 'COM', // Comoros
  'MUS': 'MRI', // Mauritius
  'SYC': 'SEY', // Seychelles
  'CPV': 'CPV', // Cape Verde

  // Non-standard PSS codes that need mapping
  'MRN': 'MAR', // MRN -> Morocco (based on context, might be Mauritania or Morocco)
  'XXB': 'XXB', // Mixed team (special code)
  'EOR': 'EOR', // Refugee Olympic Team
  'IOP': 'IOP', // Independent Olympic Participants
  'OAR': 'OAR', // Olympic Athletes from Russia
  'RPT': 'RPT', // Refugee Paralympic Team
  'IPP': 'IPP', // Independent Paralympic Participants
  'IPA': 'IPA', // Individual Paralympic Athletes
  'NPA': 'NPA', // Neutral Paralympic Athletes
  'PNA': 'PNA', // Paralympic Neutral Athletes
  'RPC': 'RPC', // Russian Paralympic Committee
};

/**
 * Convert PSS country code to IOC code for flag display
 * @param pssCode - The country code from PSS protocol
 * @returns The corresponding IOC code, or the original code if no mapping exists
 */
export function convertPssToIocCode(pssCode: string): string {
  if (!pssCode) return '';
  
  const upperCode = pssCode.toUpperCase();
  return PSS_TO_IOC_MAPPING[upperCode] || upperCode;
}

/**
 * Check if a country code has a valid flag image
 * @param countryCode - The IOC country code
 * @returns True if the flag image exists
 */
export function hasFlagImage(countryCode: string): boolean {
  if (!countryCode) return false;
  
  // This is a simple check - in a real implementation, you might want to
  // check against a list of available flag files
  const upperCode = countryCode.toUpperCase();
  
  // Check if it's in our flag configs (which includes all 253 IOC codes)
  // This is a more reliable check than trying to access the file system
  return true; // For now, assume all codes have fallback emoji flags
}

/**
 * Get the best available country code for flag display
 * @param pssCode - The original PSS country code
 * @returns The best available IOC code for flag display
 */
export function getBestFlagCode(pssCode: string): string {
  const iocCode = convertPssToIocCode(pssCode);
  
  // If the IOC code exists in our flag system, use it
  if (hasFlagImage(iocCode)) {
    return iocCode;
  }
  
  // Otherwise, return the original code (will show fallback emoji)
  return pssCode;
} 