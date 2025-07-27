#!/usr/bin/env python3
"""
Missing Flags Downloader - All Tables
Downloads missing flags using comprehensive data from all IOC tables
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

class AllTablesFlagDownloader:
    def __init__(self):
        self.flags_dir = project_root / "ui" / "public" / "assets" / "flags"
        self.svg_dir = self.flags_dir / "svg"
        self.all_tables_data_file = self.flags_dir / "all_ioc_tables_data.json"
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

    def load_all_tables_data(self) -> Dict:
        """Load the comprehensive IOC data from all tables"""
        if not self.all_tables_data_file.exists():
            raise FileNotFoundError(f"All tables data file not found: {self.all_tables_data_file}")
        
        with open(self.all_tables_data_file, 'r', encoding='utf-8') as f:
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

    def get_missing_codes(self) -> List[str]:
        """Get list of codes that don't have SVG files yet"""
        png_codes = self.get_png_country_codes()
        missing_codes = []
        
        for code in png_codes:
            svg_path = self.svg_dir / f"{code}.svg"
            if not svg_path.exists() or svg_path.stat().st_size < 100:
                missing_codes.append(code)
        
        return missing_codes

    def get_flag_urls_for_code(self, code: str, all_entries: List[Dict]) -> List[str]:
        """Generate multiple possible URLs for a country code using all table data"""
        urls = []
        
        # Find all entries for this code across all tables
        code_entries = [entry for entry in all_entries if entry.get('ioc_code') == code]
        
        for entry in code_entries:
            if entry.get('flag_url'):
                urls.append(self.convert_png_url_to_svg(entry['flag_url']))
        
        # Try direct Wikipedia patterns
        urls.extend([
            f"https://upload.wikimedia.org/wikipedia/commons/Flag_of_{code}.svg",
            f"https://upload.wikimedia.org/wikipedia/commons/thumb/0/0d/Flag_of_{code}.svg/1200px-Flag_of_{code}.svg",
            f"https://flagcdn.com/{code.lower()}.svg",
            f"https://flagpedia.net/data/flags/w2560/{code.lower()}.svg"
        ])
        
        # Try some common historical flag patterns based on country names
        for entry in code_entries:
            country_name = entry.get('country', '').replace(' ', '_').replace('-', '_')
            if country_name:
                urls.extend([
                    f"https://upload.wikimedia.org/wikipedia/commons/Flag_of_{country_name}.svg",
                    f"https://upload.wikimedia.org/wikipedia/commons/Flag_of_the_{country_name}.svg",
                    f"https://upload.wikimedia.org/wikipedia/commons/thumb/0/0d/Flag_of_{country_name}.svg/1200px-Flag_of_{country_name}.svg"
                ])
        
        return list(set(urls))  # Remove duplicates

    def download_flag(self, code: str, all_entries: List[Dict], max_retries: int = 3) -> bool:
        """Download flag for a code with retry logic"""
        svg_path = self.svg_dir / f"{code}.svg"
        
        # Skip if file already exists and has content
        if svg_path.exists() and svg_path.stat().st_size > 100:
            print(f"⏭️  Skipping {code}.svg (already exists)")
            return True
        
        urls = self.get_flag_urls_for_code(code, all_entries)
        
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

    def download_missing_flags(self):
        """Download missing flags using all table data"""
        # Load the comprehensive data
        try:
            all_data = self.load_all_tables_data()
            print(f"📋 Loaded comprehensive data with {len(all_data.get('all_entries', []))} total entries")
        except Exception as e:
            print(f"❌ Error loading all tables data: {e}")
            return
        
        all_entries = all_data.get('all_entries', [])
        
        # Show table breakdown
        print(f"\n📋 Available data from tables:")
        for table in all_data.get('table_info', []):
            print(f"   {table['title']}: {table['entries_count']} entries")
        
        # Get missing codes only
        missing_codes = self.get_missing_codes()
        print(f"\n📁 Found {len(missing_codes)} missing SVG flags to download")
        
        if not missing_codes:
            print("🎉 All flags already exist! Nothing to download.")
            return
        
        print(f"🚀 Starting SVG flag download for missing codes...")
        print(f"📁 Output directory: {self.svg_dir}")
        
        success_count = 0
        fail_count = 0
        failed_downloads = []
        
        for i, code in enumerate(missing_codes, 1):
            print(f"\n[{i}/{len(missing_codes)}] Processing {code}...")
            
            # Find entries for this code
            code_entries = [entry for entry in all_entries if entry.get('ioc_code') == code]
            if code_entries:
                print(f"   📋 Found {len(code_entries)} entries for {code}:")
                for entry in code_entries:
                    print(f"      - {entry.get('country', 'Unknown')} ({entry.get('table_source', 'Unknown table')})")
            else:
                print(f"   ⚠️  No entries found for {code} in any table")
            
            if self.download_flag(code, all_entries):
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
    downloader = AllTablesFlagDownloader()
    
    try:
        downloader.download_missing_flags()
    except KeyboardInterrupt:
        print("\n⏹️  Download interrupted by user")
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main() 