#!/usr/bin/env python3
"""
All PNG Codes Flag Downloader
Downloads SVG flags for ALL PNG files, including historical/obsolete codes
"""

import os
import sys
import requests
import time
import json
import re
from pathlib import Path
from typing import List, Dict, Optional

# Add project root to path for imports
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))

class AllCodesFlagDownloader:
    def __init__(self):
        self.flags_dir = project_root / "ui" / "public" / "assets" / "flags"
        self.svg_dir = self.flags_dir / "svg"
        self.scraped_data_file = self.flags_dir / "scraped_ioc_data.json"
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
        
        # Historical code mappings for obsolete countries
        self.historical_mappings = {
            'AHO': 'Netherlands_Antilles',
            'AIN': 'Independent_States_Team',
            'ANZ': 'Australasia',
            'BIR': 'Burma',  # Now Myanmar (MYA)
            'BOH': 'Bohemia',  # Now Czech Republic (CZE)
            'BWI': 'British_West_Indies',
            'CEY': 'Ceylon',  # Now Sri Lanka (SRI)
            'COR': 'Corsica',
            'DAH': 'Dahomey',  # Now Benin (BEN)
            'EOR': 'Refugee_Olympic_Team',
            'EUA': 'United_Team_of_Germany',
            'EUN': 'Unified_Team',
            'FRG': 'West_Germany',  # Now Germany (GER)
            'FRO': 'Faroe_Islands',
            'GDR': 'East_Germany',  # Now Germany (GER)
            'HBR': 'British_Honduras',  # Now Belize (BIZ)
            'IHO': 'Independent_Olympic_Participants',
            'IOA': 'Independent_Olympic_Athletes',
            'IOC': 'International_Olympic_Committee',
            'IOP': 'Independent_Olympic_Participants',
            'IPA': 'Independent_Paralympic_Athletes',
            'IPP': 'Independent_Paralympic_Participants',
            'KHM': 'Cambodia',  # Now CAM
            'MAC': 'Macau',
            'MAL': 'Malaya',  # Now Malaysia (MAS)
            'MIX': 'Mixed_Teams',
            'NBO': 'North_Borneo',
            'NFK': 'Norfolk_Island',
            'NIU': 'Niue',
            'NMI': 'Northern_Mariana_Islands',
            'NPA': 'North_Pacific_Alliance',
            'NRH': 'Northern_Rhodesia',  # Now Zambia (ZAM)
            'OAR': 'Olympic_Athletes_from_Russia',
            'PNA': 'Palestine_National_Authority',
            'PYF': 'French_Polynesia',
            'RAU': 'United_Arab_Republic',  # Now Egypt (EGY)
            'RHO': 'Rhodesia',  # Now Zimbabwe (ZIM)
            'ROC': 'Russian_Olympic_Committee',
            'RPC': 'Russian_Paralympic_Committee',
            'RPT': 'Republic_of_China_Taipei',  # Now Taiwan (TPE)
            'SAA': 'Saar',
            'SCG': 'Serbia_and_Montenegro',  # Now Serbia (SRB) and Montenegro (MNE)
            'TCH': 'Czechoslovakia',  # Now Czech Republic (CZE) and Slovakia (SVK)
            'TKL': 'Tokelau',
            'UAR': 'United_Arab_Republic',  # Now Egypt (EGY)
            'URS': 'Soviet_Union',  # Now Russia (RUS)
            'VAT': 'Vatican_City',
            'VNM': 'Vietnam',  # Now VIE
            'VOL': 'Upper_Volta',  # Now Burkina Faso (BUR)
            'WSM': 'Western_Samoa',  # Now Samoa (SAM)
            'XXB': 'Unknown_Team',
            'YAR': 'North_Yemen',  # Now Yemen (YEM)
            'YMD': 'South_Yemen',  # Now Yemen (YEM)
            'YUG': 'Yugoslavia',  # Now Serbia (SRB), Croatia (CRO), Slovenia (SLO), etc.
            'ZAI': 'Zaire',  # Now Democratic Republic of Congo (COD)
        }

    def load_scraped_data(self) -> Dict:
        """Load the scraped IOC data"""
        if not self.scraped_data_file.exists():
            raise FileNotFoundError(f"Scraped data file not found: {self.scraped_data_file}")
        
        with open(self.scraped_data_file, 'r', encoding='utf-8') as f:
            return json.load(f)

    def convert_png_url_to_svg(self, png_url: str) -> str:
        """Convert a PNG flag URL to SVG format"""
        svg_url = png_url.replace('/thumb/', '/')
        svg_url = re.sub(r'/\d+px-.*\.png$', '', svg_url)
        return svg_url

    def get_png_country_codes(self) -> List[str]:
        """Get list of country codes from PNG files"""
        png_files = list(self.flags_dir.glob("*.png"))
        return [f.stem for f in png_files if f.is_file()]

    def get_flag_urls_for_code(self, code: str, ioc_mapping: Dict) -> List[str]:
        """Generate multiple possible URLs for a country code"""
        urls = []
        
        # First try the IOC mapping
        if code in ioc_mapping:
            urls.append(self.convert_png_url_to_svg(ioc_mapping[code]))
        
        # Try historical mappings
        if code in self.historical_mappings:
            historical_name = self.historical_mappings[code]
            urls.extend([
                f"https://upload.wikimedia.org/wikipedia/commons/Flag_of_{historical_name}.svg",
                f"https://upload.wikimedia.org/wikipedia/commons/thumb/0/0d/Flag_of_{historical_name}.svg/1200px-Flag_of_{historical_name}.svg",
                f"https://flagcdn.com/{code.lower()}.svg",
                f"https://flagpedia.net/data/flags/w2560/{code.lower()}.svg"
            ])
        
        # Try direct Wikipedia patterns
        urls.extend([
            f"https://upload.wikimedia.org/wikipedia/commons/Flag_of_{code}.svg",
            f"https://upload.wikimedia.org/wikipedia/commons/thumb/0/0d/Flag_of_{code}.svg/1200px-Flag_of_{code}.svg",
            f"https://flagcdn.com/{code.lower()}.svg",
            f"https://flagpedia.net/data/flags/w2560/{code.lower()}.svg"
        ])
        
        # Try some common historical flag patterns
        if code in ['YUG', 'URS', 'TCH', 'GDR', 'FRG']:
            urls.extend([
                f"https://upload.wikimedia.org/wikipedia/commons/Flag_of_{self.historical_mappings[code]}.svg",
                f"https://upload.wikimedia.org/wikipedia/commons/Flag_of_the_{self.historical_mappings[code]}.svg",
                f"https://upload.wikimedia.org/wikipedia/commons/Flag_of_{self.historical_mappings[code].replace('_', '_the_')}.svg"
            ])
        
        return urls

    def download_flag(self, code: str, ioc_mapping: Dict, max_retries: int = 3) -> bool:
        """Download flag for a code with retry logic"""
        svg_path = self.svg_dir / f"{code}.svg"
        
        # Skip if file already exists and has content
        if svg_path.exists() and svg_path.stat().st_size > 100:
            print(f"⏭️  Skipping {code}.svg (already exists)")
            return True
        
        urls = self.get_flag_urls_for_code(code, ioc_mapping)
        
        for attempt in range(max_retries):
            for url in urls:
                try:
                    print(f"🔗 Trying {code}: {url}")
                    response = self.session.get(url, timeout=15)
                    
                    if response.status_code == 200 and response.content:
                        # Check if content looks like SVG
                        content = response.text
                        if '<svg' in content.lower() and len(content) > 100:
                            svg_path.write_text(content, encoding='utf-8')
                            print(f"✅ Downloaded: {code}.svg")
                            return True
                        else:
                            print(f"⚠️  Invalid SVG content for {code}")
                    else:
                        print(f"❌ HTTP {response.status_code} for {code}")
                        
                except requests.RequestException as e:
                    print(f"⚠️  Request failed for {code}: {e}")
                    continue
                except Exception as e:
                    print(f"❌ Error downloading {code}: {e}")
                    continue
            
            if attempt < max_retries - 1:
                print(f"🔄 Retrying {code} (attempt {attempt + 2}/{max_retries})")
                time.sleep(2 ** attempt)  # Exponential backoff
        
        print(f"❌ Failed to download {code}.svg after {max_retries} attempts")
        return False

    def download_all_flags(self):
        """Download all flags for all PNG codes"""
        # Load the scraped data
        try:
            scraped_data = self.load_scraped_data()
            print(f"📋 Loaded scraped data with {len(scraped_data.get('ioc_entries', []))} IOC entries")
        except Exception as e:
            print(f"❌ Error loading scraped data: {e}")
            return
        
        # Get the IOC entries and create mapping
        ioc_entries = scraped_data.get('ioc_entries', [])
        ioc_mapping = {}
        for entry in ioc_entries:
            ioc_code = entry.get('ioc_code')
            flag_url = entry.get('flag_url')
            if ioc_code and flag_url:
                ioc_mapping[ioc_code] = flag_url
        
        print(f"🗺️  Created IOC mapping for {len(ioc_mapping)} flags")
        
        # Get ALL PNG country codes
        png_codes = self.get_png_country_codes()
        print(f"📁 Found {len(png_codes)} PNG flag files to process")
        print(f"🚀 Starting SVG flag download for ALL codes...")
        print(f"📁 Output directory: {self.svg_dir}")
        
        success_count = 0
        fail_count = 0
        failed_downloads = []
        
        for i, code in enumerate(png_codes, 1):
            print(f"\n[{i}/{len(png_codes)}] Processing {code}...")
            
            if self.download_flag(code, ioc_mapping):
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
    downloader = AllCodesFlagDownloader()
    
    try:
        downloader.download_all_flags()
    except KeyboardInterrupt:
        print("\n⏹️  Download interrupted by user")
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main() 