#!/usr/bin/env python3
"""
SVG Flag Downloader Script
Downloads SVG flags for IOC country codes from multiple sources
"""

import os
import sys
import requests
import time
from pathlib import Path
from typing import List, Dict, Optional
import json

# Add project root to path for imports
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))

class FlagDownloader:
    def __init__(self):
        self.flags_dir = project_root / "ui" / "public" / "assets" / "flags"
        self.svg_dir = self.flags_dir / "svg"
        self.session = requests.Session()
        
        # Set up session headers to avoid blocking
        self.session.headers.update({
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36',
            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8',
            'Accept-Language': 'en-US,en;q=0.5',
            'Accept-Encoding': 'gzip, deflate',
            'Connection': 'keep-alive',
            'Upgrade-Insecure-Requests': '1',
        })
        
        # Create SVG directory if it doesn't exist
        self.svg_dir.mkdir(parents=True, exist_ok=True)
        
        # Country code to name mapping for Wikipedia URLs
        self.country_mapping = {
            'USA': 'United_States',
            'GBR': 'United_Kingdom', 
            'FRA': 'France',
            'GER': 'Germany',
            'ITA': 'Italy',
            'ESP': 'Spain',
            'RUS': 'Russia',
            'CHN': 'China',
            'JPN': 'Japan',
            'BRA': 'Brazil',
            'CAN': 'Canada',
            'AUS': 'Australia',
            'NED': 'Netherlands',
            'SWE': 'Sweden',
            'NOR': 'Norway',
            'DEN': 'Denmark',
            'FIN': 'Finland',
            'SUI': 'Switzerland',
            'AUT': 'Austria',
            'BEL': 'Belgium',
            'POL': 'Poland',
            'CZE': 'Czech_Republic',
            'SVK': 'Slovakia',
            'HUN': 'Hungary',
            'ROU': 'Romania',
            'BUL': 'Bulgaria',
            'CRO': 'Croatia',
            'SLO': 'Slovenia',
            'SRB': 'Serbia',
            'BIH': 'Bosnia_and_Herzegovina',
            'MNE': 'Montenegro',
            'MKD': 'North_Macedonia',
            'ALB': 'Albania',
            'GRE': 'Greece',
            'TUR': 'Turkey',
            'UKR': 'Ukraine',
            'BLR': 'Belarus',
            'LVA': 'Latvia',
            'LTU': 'Lithuania',
            'EST': 'Estonia',
            'MDA': 'Moldova',
            'GEO': 'Georgia',
            'ARM': 'Armenia',
            'AZE': 'Azerbaijan',
            'KAZ': 'Kazakhstan',
            'UZB': 'Uzbekistan',
            'TJK': 'Tajikistan',
            'TKM': 'Turkmenistan',
            'KGZ': 'Kyrgyzstan',
            'MNG': 'Mongolia',
            'KOR': 'South_Korea',
            'PRK': 'North_Korea',
            'JPN': 'Japan',
            'CHN': 'China',
            'TWN': 'Taiwan',
            'HKG': 'Hong_Kong',
            'MAC': 'Macau',
            'MNG': 'Mongolia',
            'VNM': 'Vietnam',
            'THA': 'Thailand',
            'LAO': 'Laos',
            'KHM': 'Cambodia',
            'MMR': 'Myanmar',
            'MYS': 'Malaysia',
            'SGP': 'Singapore',
            'IDN': 'Indonesia',
            'PHL': 'Philippines',
            'BRN': 'Brunei',
            'TLS': 'East_Timor',
            'IND': 'India',
            'PAK': 'Pakistan',
            'BGD': 'Bangladesh',
            'LKA': 'Sri_Lanka',
            'NPL': 'Nepal',
            'BTN': 'Bhutan',
            'MDV': 'Maldives',
            'AFG': 'Afghanistan',
            'IRN': 'Iran',
            'IRQ': 'Iraq',
            'SYR': 'Syria',
            'LBN': 'Lebanon',
            'JOR': 'Jordan',
            'ISR': 'Israel',
            'PSE': 'Palestine',
            'SAU': 'Saudi_Arabia',
            'YEM': 'Yemen',
            'OMN': 'Oman',
            'ARE': 'United_Arab_Emirates',
            'QAT': 'Qatar',
            'BHR': 'Bahrain',
            'KWT': 'Kuwait',
            'EGY': 'Egypt',
            'LBY': 'Libya',
            'TUN': 'Tunisia',
            'DZA': 'Algeria',
            'MAR': 'Morocco',
            'ESH': 'Western_Sahara',
            'MRT': 'Mauritania',
            'MLI': 'Mali',
            'BFA': 'Burkina_Faso',
            'NER': 'Niger',
            'TCD': 'Chad',
            'SDN': 'Sudan',
            'SSD': 'South_Sudan',
            'ETH': 'Ethiopia',
            'ERI': 'Eritrea',
            'DJI': 'Djibouti',
            'SOM': 'Somalia',
            'KEN': 'Kenya',
            'UGA': 'Uganda',
            'RWA': 'Rwanda',
            'BDI': 'Burundi',
            'TZA': 'Tanzania',
            'MWI': 'Malawi',
            'ZMB': 'Zambia',
            'ZWE': 'Zimbabwe',
            'BWA': 'Botswana',
            'NAM': 'Namibia',
            'ZAF': 'South_Africa',
            'LSO': 'Lesotho',
            'SWZ': 'Eswatini',
            'MOZ': 'Mozambique',
            'MDG': 'Madagascar',
            'COM': 'Comoros',
            'MUS': 'Mauritius',
            'SYC': 'Seychelles',
            'GAB': 'Gabon',
            'CMR': 'Cameroon',
            'CAF': 'Central_African_Republic',
            'COG': 'Republic_of_the_Congo',
            'COD': 'Democratic_Republic_of_the_Congo',
            'GNQ': 'Equatorial_Guinea',
            'STP': 'São_Tomé_and_Príncipe',
            'AGO': 'Angola',
            'ZAR': 'Zaire',  # Historical
            'NGA': 'Nigeria',
            'GHA': 'Ghana',
            'CIV': 'Ivory_Coast',
            'GIN': 'Guinea',
            'GNB': 'Guinea-Bissau',
            'SLE': 'Sierra_Leone',
            'LBR': 'Liberia',
            'SEN': 'Senegal',
            'GMB': 'Gambia',
            'CPV': 'Cape_Verde',
            'TGO': 'Togo',
            'BEN': 'Benin',
            'NIC': 'Nicaragua',
            'HND': 'Honduras',
            'GTM': 'Guatemala',
            'BLZ': 'Belize',
            'SLV': 'El_Salvador',
            'CRI': 'Costa_Rica',
            'PAN': 'Panama',
            'COL': 'Colombia',
            'VEN': 'Venezuela',
            'GUY': 'Guyana',
            'SUR': 'Suriname',
            'BRA': 'Brazil',
            'ECU': 'Ecuador',
            'PER': 'Peru',
            'BOL': 'Bolivia',
            'PRY': 'Paraguay',
            'URY': 'Uruguay',
            'ARG': 'Argentina',
            'CHL': 'Chile',
            'CUB': 'Cuba',
            'JAM': 'Jamaica',
            'HTI': 'Haiti',
            'DOM': 'Dominican_Republic',
            'PRI': 'Puerto_Rico',
            'TTO': 'Trinidad_and_Tobago',
            'BRB': 'Barbados',
            'GRD': 'Grenada',
            'LCA': 'Saint_Lucia',
            'VCT': 'Saint_Vincent_and_the_Grenadines',
            'ATG': 'Antigua_and_Barbuda',
            'DMA': 'Dominica',
            'KNA': 'Saint_Kitts_and_Nevis',
            'BHS': 'Bahamas',
            'ISL': 'Iceland',
            'IRL': 'Ireland',
            'PRT': 'Portugal',
            'AND': 'Andorra',
            'MCO': 'Monaco',
            'LIE': 'Liechtenstein',
            'SMR': 'San_Marino',
            'VAT': 'Vatican_City',
            'MLT': 'Malta',
            'CYP': 'Cyprus',
            'LUX': 'Luxembourg',
            'MEX': 'Mexico',
            'NZL': 'New_Zealand',
            'FJI': 'Fiji',
            'PNG': 'Papua_New_Guinea',
            'VUT': 'Vanuatu',
            'NCL': 'New_Caledonia',
            'WSM': 'Samoa',
            'TON': 'Tonga',
            'KIR': 'Kiribati',
            'TUV': 'Tuvalu',
            'NRU': 'Nauru',
            'PLW': 'Palau',
            'FSM': 'Micronesia',
            'MHL': 'Marshall_Islands',
            'GUM': 'Guam',
            'MNP': 'Northern_Mariana_Islands',
            'ASM': 'American_Samoa',
            'COK': 'Cook_Islands',
            'NIU': 'Niue',
            'TKL': 'Tokelau',
            'WLF': 'Wallis_and_Futuna',
            'PYF': 'French_Polynesia',
            'PCN': 'Pitcairn_Islands',
            'ATA': 'Antarctica',
            'BVT': 'Bouvet_Island',
            'IOT': 'British_Indian_Ocean_Territory',
            'SGS': 'South_Georgia_and_the_South_Sandwich_Islands',
            'HMD': 'Heard_Island_and_McDonald_Islands',
            'ATF': 'French_Southern_and_Antarctic_Lands',
            'SHN': 'Saint_Helena',
            'TCA': 'Turks_and_Caicos_Islands',
            'BMU': 'Bermuda',
            'CYM': 'Cayman_Islands',
            'VGB': 'British_Virgin_Islands',
            'AIA': 'Anguilla',
            'MSR': 'Montserrat',
            'GIB': 'Gibraltar',
            'IMN': 'Isle_of_Man',
            'JEY': 'Jersey',
            'GGY': 'Guernsey',
            'FRO': 'Faroe_Islands',
            'GRL': 'Greenland',
            'SPM': 'Saint_Pierre_and_Miquelon',
            'ABW': 'Aruba',
            'CUW': 'Curaçao',
            'SXM': 'Sint_Maarten',
            'BES': 'Caribbean_Netherlands',
            'GUF': 'French_Guiana',
            'MTQ': 'Martinique',
            'GLP': 'Guadeloupe',
            'REU': 'Réunion',
            'MYT': 'Mayotte',
            'CPT': 'Clipperton_Island',
            'MAC': 'Macau',
            'HKG': 'Hong_Kong',
            'TWN': 'Taiwan',
            'PSE': 'Palestine',
            'KOS': 'Kosovo',
            'TCA': 'Turks_and_Caicos_Islands',
            'CXR': 'Christmas_Island',
            'CCK': 'Cocos_Islands',
            'NFK': 'Norfolk_Island',
            'AUS': 'Australia',
            'CAN': 'Canada',
            'USA': 'United_States',
            'MEX': 'Mexico',
            'BRA': 'Brazil',
            'ARG': 'Argentina',
            'CHL': 'Chile',
            'PER': 'Peru',
            'COL': 'Colombia',
            'VEN': 'Venezuela',
            'ECU': 'Ecuador',
            'BOL': 'Bolivia',
            'PRY': 'Paraguay',
            'URY': 'Uruguay',
            'GUY': 'Guyana',
            'SUR': 'Suriname',
            'FRA': 'France',
            'DEU': 'Germany',
            'ITA': 'Italy',
            'ESP': 'Spain',
            'GBR': 'United_Kingdom',
            'NLD': 'Netherlands',
            'BEL': 'Belgium',
            'CHE': 'Switzerland',
            'AUT': 'Austria',
            'LUX': 'Luxembourg',
            'LIE': 'Liechtenstein',
            'MCO': 'Monaco',
            'AND': 'Andorra',
            'SMR': 'San_Marino',
            'VAT': 'Vatican_City',
            'MLT': 'Malta',
            'CYP': 'Cyprus',
            'GRC': 'Greece',
            'ALB': 'Albania',
            'MKD': 'North_Macedonia',
            'SRB': 'Serbia',
            'MNE': 'Montenegro',
            'BIH': 'Bosnia_and_Herzegovina',
            'HRV': 'Croatia',
            'SVN': 'Slovenia',
            'HUN': 'Hungary',
            'SVK': 'Slovakia',
            'CZE': 'Czech_Republic',
            'POL': 'Poland',
            'LTU': 'Lithuania',
            'LVA': 'Latvia',
            'EST': 'Estonia',
            'FIN': 'Finland',
            'SWE': 'Sweden',
            'NOR': 'Norway',
            'DNK': 'Denmark',
            'ISL': 'Iceland',
            'IRL': 'Ireland',
            'PRT': 'Portugal',
            'RUS': 'Russia',
            'BLR': 'Belarus',
            'UKR': 'Ukraine',
            'MDA': 'Moldova',
            'ROU': 'Romania',
            'BGR': 'Bulgaria',
            'TUR': 'Turkey',
            'GEO': 'Georgia',
            'ARM': 'Armenia',
            'AZE': 'Azerbaijan',
            'KAZ': 'Kazakhstan',
            'UZB': 'Uzbekistan',
            'TJK': 'Tajikistan',
            'TKM': 'Turkmenistan',
            'KGZ': 'Kyrgyzstan',
            'MNG': 'Mongolia',
            'CHN': 'China',
            'TWN': 'Taiwan',
            'HKG': 'Hong_Kong',
            'MAC': 'Macau',
            'JPN': 'Japan',
            'KOR': 'South_Korea',
            'PRK': 'North_Korea',
            'MNG': 'Mongolia',
            'VNM': 'Vietnam',
            'LAO': 'Laos',
            'KHM': 'Cambodia',
            'THA': 'Thailand',
            'MMR': 'Myanmar',
            'MYS': 'Malaysia',
            'SGP': 'Singapore',
            'IDN': 'Indonesia',
            'PHL': 'Philippines',
            'BRN': 'Brunei',
            'TLS': 'East_Timor',
            'IND': 'India',
            'PAK': 'Pakistan',
            'BGD': 'Bangladesh',
            'LKA': 'Sri_Lanka',
            'NPL': 'Nepal',
            'BTN': 'Bhutan',
            'MDV': 'Maldives',
            'AFG': 'Afghanistan',
            'IRN': 'Iran',
            'IRQ': 'Iraq',
            'SYR': 'Syria',
            'LBN': 'Lebanon',
            'JOR': 'Jordan',
            'ISR': 'Israel',
            'PSE': 'Palestine',
            'SAU': 'Saudi_Arabia',
            'YEM': 'Yemen',
            'OMN': 'Oman',
            'ARE': 'United_Arab_Emirates',
            'QAT': 'Qatar',
            'BHR': 'Bahrain',
            'KWT': 'Kuwait',
            'EGY': 'Egypt',
            'LBY': 'Libya',
            'TUN': 'Tunisia',
            'DZA': 'Algeria',
            'MAR': 'Morocco',
            'ESH': 'Western_Sahara',
            'MRT': 'Mauritania',
            'MLI': 'Mali',
            'BFA': 'Burkina_Faso',
            'NER': 'Niger',
            'TCD': 'Chad',
            'SDN': 'Sudan',
            'SSD': 'South_Sudan',
            'ETH': 'Ethiopia',
            'ERI': 'Eritrea',
            'DJI': 'Djibouti',
            'SOM': 'Somalia',
            'KEN': 'Kenya',
            'UGA': 'Uganda',
            'RWA': 'Rwanda',
            'BDI': 'Burundi',
            'TZA': 'Tanzania',
            'MWI': 'Malawi',
            'ZMB': 'Zambia',
            'ZWE': 'Zimbabwe',
            'BWA': 'Botswana',
            'NAM': 'Namibia',
            'ZAF': 'South_Africa',
            'LSO': 'Lesotho',
            'SWZ': 'Eswatini',
            'MOZ': 'Mozambique',
            'MDG': 'Madagascar',
            'COM': 'Comoros',
            'MUS': 'Mauritius',
            'SYC': 'Seychelles',
            'GAB': 'Gabon',
            'CMR': 'Cameroon',
            'CAF': 'Central_African_Republic',
            'COG': 'Republic_of_the_Congo',
            'COD': 'Democratic_Republic_of_the_Congo',
            'GNQ': 'Equatorial_Guinea',
            'STP': 'São_Tomé_and_Príncipe',
            'AGO': 'Angola',
            'NGA': 'Nigeria',
            'GHA': 'Ghana',
            'CIV': 'Ivory_Coast',
            'GIN': 'Guinea',
            'GNB': 'Guinea-Bissau',
            'SLE': 'Sierra_Leone',
            'LBR': 'Liberia',
            'SEN': 'Senegal',
            'GMB': 'Gambia',
            'CPV': 'Cape_Verde',
            'TGO': 'Togo',
            'BEN': 'Benin',
            'NIC': 'Nicaragua',
            'HND': 'Honduras',
            'GTM': 'Guatemala',
            'BLZ': 'Belize',
            'SLV': 'El_Salvador',
            'CRI': 'Costa_Rica',
            'PAN': 'Panama',
        }

    def get_png_country_codes(self) -> List[str]:
        """Get list of country codes from PNG files"""
        png_files = list(self.flags_dir.glob("*.png"))
        return [f.stem for f in png_files if f.is_file()]

    def get_flag_urls(self, country_code: str) -> List[str]:
        """Generate multiple possible URLs for a country flag"""
        urls = []
        
        # Source 1: FlagCDN API
        urls.append(f"https://flagcdn.com/{country_code.lower()}.svg")
        
        # Source 2: Wikipedia Commons (if we have country name mapping)
        if country_code in self.country_mapping:
            country_name = self.country_mapping[country_code]
            urls.append(f"https://upload.wikimedia.org/wikipedia/commons/thumb/0/0d/Flag_of_{country_name}.svg/1200px-Flag_of_{country_name}.svg")
            urls.append(f"https://upload.wikimedia.org/wikipedia/commons/Flag_of_{country_name}.svg")
        
        # Source 3: Alternative Wikipedia patterns
        urls.append(f"https://upload.wikimedia.org/wikipedia/commons/thumb/0/0d/Flag_of_{country_code}.svg/1200px-Flag_of_{country_code}.svg")
        urls.append(f"https://upload.wikimedia.org/wikipedia/commons/Flag_of_{country_code}.svg")
        
        # Source 4: Flagpedia
        urls.append(f"https://flagpedia.net/data/flags/w2560/{country_code.lower()}.svg")
        
        return urls

    def download_flag(self, country_code: str, max_retries: int = 3) -> bool:
        """Download flag for a country code with retry logic"""
        svg_path = self.svg_dir / f"{country_code}.svg"
        
        # Skip if file already exists and has content
        if svg_path.exists() and svg_path.stat().st_size > 100:
            print(f"⏭️  Skipping {country_code}.svg (already exists)")
            return True
        
        urls = self.get_flag_urls(country_code)
        
        for attempt in range(max_retries):
            for url in urls:
                try:
                    print(f"🔗 Trying {country_code}: {url}")
                    response = self.session.get(url, timeout=10)
                    
                    if response.status_code == 200 and response.content:
                        # Check if content looks like SVG
                        content = response.text
                        if '<svg' in content.lower() and len(content) > 100:
                            svg_path.write_text(content, encoding='utf-8')
                            print(f"✅ Downloaded: {country_code}.svg")
                            return True
                        else:
                            print(f"⚠️  Invalid SVG content for {country_code}")
                    else:
                        print(f"❌ HTTP {response.status_code} for {country_code}")
                        
                except requests.RequestException as e:
                    print(f"⚠️  Request failed for {country_code}: {e}")
                    continue
                except Exception as e:
                    print(f"❌ Error downloading {country_code}: {e}")
                    continue
            
            if attempt < max_retries - 1:
                print(f"🔄 Retrying {country_code} (attempt {attempt + 2}/{max_retries})")
                time.sleep(2 ** attempt)  # Exponential backoff
        
        print(f"❌ Failed to download {country_code}.svg after {max_retries} attempts")
        return False

    def download_all_flags(self, test_mode: bool = False):
        """Download all flags"""
        country_codes = self.get_png_country_codes()
        
        if test_mode:
            # Test with a small subset
            test_codes = ['USA', 'GBR', 'FRA', 'GER', 'ITA', 'ESP', 'RUS', 'CHN', 'JPN', 'BRA']
            country_codes = [code for code in country_codes if code in test_codes]
        
        print(f"📁 Found {len(country_codes)} PNG flag files")
        print(f"🚀 Starting SVG flag download...")
        print(f"📁 Output directory: {self.svg_dir}")
        
        success_count = 0
        fail_count = 0
        failed_downloads = []
        
        for i, code in enumerate(country_codes, 1):
            print(f"\n[{i}/{len(country_codes)}] Processing {code}...")
            
            if self.download_flag(code):
                success_count += 1
            else:
                fail_count += 1
                failed_downloads.append(code)
            
            # Small delay between requests
            time.sleep(0.5)
        
        # Print summary
        print(f"\n📊 Download Summary:")
        print(f"✅ Successfully downloaded: {success_count} flags")
        print(f"❌ Failed downloads: {fail_count} flags")
        
        if failed_downloads:
            print(f"\n❌ Failed downloads:")
            for code in failed_downloads:
                print(f"   {code}")
        
        print(f"\n🎉 SVG flags saved to: {self.svg_dir}")

def main():
    """Main function"""
    downloader = FlagDownloader()
    
    # Check if test mode is requested
    test_mode = '--test' in sys.argv
    
    if test_mode:
        print("🧪 Running in test mode with limited flags...")
    
    try:
        downloader.download_all_flags(test_mode=test_mode)
    except KeyboardInterrupt:
        print("\n⏹️  Download interrupted by user")
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main() 