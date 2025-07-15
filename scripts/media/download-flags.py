#!/usr/bin/env python3
"""
Flag Download and Rename Script for reStrike VTA

This script helps download flag images and rename them to IOC country codes.
"""

import os
import sys
import requests
from pathlib import Path

# Common country codes for taekwondo competitions
COUNTRY_CODES = {
    'USA': 'United States',
    'KOR': 'South Korea', 
    'CHN': 'China',
    'JPN': 'Japan',
    'GBR': 'Great Britain',
    'FRA': 'France',
    'GER': 'Germany',
    'ITA': 'Italy',
    'ESP': 'Spain',
    'CAN': 'Canada',
    'AUS': 'Australia',
    'BRA': 'Brazil',
    'RUS': 'Russia',
    'TUR': 'Turkey',
    'IRN': 'Iran',
    'THA': 'Thailand',
    'VIE': 'Vietnam',
    'PHI': 'Philippines',
    'MAS': 'Malaysia',
    'SGP': 'Singapore'
}

def download_flag(country_code: str, country_name: str) -> bool:
    """Download flag image for a country"""
    try:
        # Using a free flag API
        url = f"https://flagcdn.com/w80/{country_code.lower()}.png"
        response = requests.get(url, timeout=10)
        
        if response.status_code == 200:
            # Save to flags directory
            flags_dir = Path("ui/public/assets/flags")
            flags_dir.mkdir(parents=True, exist_ok=True)
            
            file_path = flags_dir / f"{country_code}.png"
            with open(file_path, 'wb') as f:
                f.write(response.content)
            
            print(f"✅ Downloaded {country_code}.png ({country_name})")
            return True
        else:
            print(f"❌ Failed to download {country_code} ({country_name})")
            return False
            
    except Exception as e:
        print(f"❌ Error downloading {country_code}: {e}")
        return False

def main():
    print("🏁 Flag Download Script for reStrike VTA")
    print("=" * 50)
    
    success_count = 0
    total_count = len(COUNTRY_CODES)
    
    for code, name in COUNTRY_CODES.items():
        if download_flag(code, name):
            success_count += 1
    
    print("=" * 50)
    print(f"📊 Download Summary: {success_count}/{total_count} flags downloaded")
    
    if success_count > 0:
        print("\n📁 Flags saved to: ui/public/assets/flags/")
        print("🔗 Usage in React: <FlagImage countryCode=\"USA\" />")
        print("📚 Reference: IOC Country Codes on Wikipedia")

if __name__ == "__main__":
    main() 