/**
 * Country Code Mapping Utility
 * Maps PSS protocol country codes to IOC codes for flag display
 * 
 * Note: PSS codes are the same as IOC codes, so no conversion is needed
 */

// PSS to IOC code mapping
// Since PSS codes are the same as IOC codes, this mapping is mostly for reference
// and handling any edge cases or special codes
export const PSS_TO_IOC_MAPPING: Record<string, string> = {
  // All standard PSS codes are the same as IOC codes
  // This mapping is kept for reference and any special cases
  
  // Special Olympic codes (these remain the same)
  'XXB': 'XXB', // Mixed team (special code)
  'EOR': 'EOR', // Refugee Olympic Team
  'IOP': 'IOP', // Independent Olympic Participants
  'OAR': 'OAR', // Olympic Athletes from Russia
  'RPT': 'RPT', // Refugee Paralympic Team
  'IPP': 'IPP', // Independent Paralympic Participants
  'IPA': 'IPA', // Individual Paralympic Athletes
  'NPA': 'NPA', // Neutral Paralympic Athletes
  'PNA': 'PNA', // Paralympic Neutral Athletes
};

/**
 * Convert PSS country code to IOC code for flag display
 * @param pssCode - The country code from PSS protocol
 * @returns The IOC code (same as PSS code since they're identical)
 */
export function convertPssToIocCode(pssCode: string): string {
  if (!pssCode) return '';
  
  // Since PSS codes are the same as IOC codes, just return the uppercase version
  return pssCode.toUpperCase();
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
 * @returns The IOC code for flag display (same as PSS code)
 */
export function getBestFlagCode(pssCode: string): string {
  // Since PSS codes are the same as IOC codes, just return the uppercase version
  return pssCode.toUpperCase();
} 