#!/usr/bin/env python3
"""
SVG Flag Downloader Script - Using Existing IOC Report
Downloads SVG flags using the existing IOC flags report data
"""

import os
import sys
import requests
import time
import json
from pathlib import Path
from typing import List, Dict, Optional
import re

# Add project root to path for imports
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))

class FlagDownloaderFromReport:
    def __init__(self):
        self.flags_dir = project_root / "ui" / "public" / "assets" / "flags"
        self.svg_dir = self.flags_dir / "svg"
        self.report_file = self.flags_dir / "official_ioc_flags_report.json"
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

    def load_ioc_report(self) -> Dict:
        """Load the IOC flags report"""
        if not self.report_file.exists():
            raise FileNotFoundError(f"IOC report file not found: {self.report_file}")
        
        with open(self.report_file, 'r', encoding='utf-8') as f:
            return json.load(f)

    def convert_png_url_to_svg(self, png_url: str) -> str:
        """Convert a PNG flag URL to SVG format"""
        # Example: https://upload.wikimedia.org/wikipedia/commons/thumb/5/5c/Flag_of_the_Taliban.svg/40px-Flag_of_the_Taliban.svg.png
        # To: https://upload.wikimedia.org/wikipedia/commons/5/5c/Flag_of_the_Taliban.svg
        
        # Remove the /thumb/ and /40px- parts and .png extension
        svg_url = png_url.replace('/thumb/', '/')
        svg_url = re.sub(r'/\d+px-.*\.png$', '', svg_url)
        
        return svg_url

    def get_png_country_codes(self) -> List[str]:
        """Get list of country codes from PNG files"""
        png_files = list(self.flags_dir.glob("*.png"))
        return [f.stem for f in png_files if f.is_file()]

    def download_flag(self, country_code: str, flag_url: str, max_retries: int = 3) -> bool:
        """Download flag for a country code with retry logic"""
        svg_path = self.svg_dir / f"{country_code}.svg"
        
        # Skip if file already exists and has content
        if svg_path.exists() and svg_path.stat().st_size > 100:
            print(f"⏭️  Skipping {country_code}.svg (already exists)")
            return True
        
        # Convert PNG URL to SVG URL
        svg_url = self.convert_png_url_to_svg(flag_url)
        
        for attempt in range(max_retries):
            try:
                print(f"🔗 Trying {country_code}: {svg_url}")
                response = self.session.get(svg_url, timeout=15)
                
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
            except Exception as e:
                print(f"❌ Error downloading {country_code}: {e}")
            
            if attempt < max_retries - 1:
                print(f"🔄 Retrying {country_code} (attempt {attempt + 2}/{max_retries})")
                time.sleep(2 ** attempt)  # Exponential backoff
        
        print(f"❌ Failed to download {country_code}.svg after {max_retries} attempts")
        return False

    def download_all_flags(self, test_mode: bool = False):
        """Download all flags using the IOC report"""
        # Load the IOC report
        try:
            report_data = self.load_ioc_report()
            print(f"📋 Loaded IOC report with {len(report_data.get('downloaded_flags', []))} flag entries")
        except Exception as e:
            print(f"❌ Error loading IOC report: {e}")
            return
        
        # Get the flag data
        flag_data = report_data.get('downloaded_flags', [])
        
        # Create a mapping of IOC codes to flag URLs
        flag_mapping = {}
        for flag_info in flag_data:
            ioc_code = flag_info.get('ioc_code')
            flag_url = flag_info.get('flag_url')
            if ioc_code and flag_url:
                flag_mapping[ioc_code] = flag_url
        
        print(f"🗺️  Created mapping for {len(flag_mapping)} flags")
        
        # Get PNG country codes to process
        png_codes = self.get_png_country_codes()
        
        if test_mode:
            # Test with a small subset
            test_codes = ['USA', 'GBR', 'FRA', 'GER', 'ITA', 'ESP', 'RUS', 'CHN', 'JPN', 'BRA']
            png_codes = [code for code in png_codes if code in test_codes]
        
        print(f"📁 Found {len(png_codes)} PNG flag files to process")
        print(f"🚀 Starting SVG flag download...")
        print(f"📁 Output directory: {self.svg_dir}")
        
        success_count = 0
        fail_count = 0
        failed_downloads = []
        
        for i, code in enumerate(png_codes, 1):
            print(f"\n[{i}/{len(png_codes)}] Processing {code}...")
            
            if code in flag_mapping:
                flag_url = flag_mapping[code]
                if self.download_flag(code, flag_url):
                    success_count += 1
                else:
                    fail_count += 1
                    failed_downloads.append(code)
            else:
                print(f"⚠️  No flag URL found for {code} in IOC report")
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
    downloader = FlagDownloaderFromReport()
    
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