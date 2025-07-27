#!/usr/bin/env python3
"""
Final Flags Downloader
Downloads the last 2 missing flags (NFK and VAT) using alternative approaches
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

class FinalFlagsDownloader:
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
        
        # Alternative URLs for the final 2 flags
        self.final_mappings = {
            'NFK': {
                'name': 'Norfolk Island',
                'urls': [
                    'https://upload.wikimedia.org/wikipedia/commons/4/4e/Flag_of_Norfolk_Island.svg',
                    'https://upload.wikimedia.org/wikipedia/commons/thumb/4/4e/Flag_of_Norfolk_Island.svg/1200px-Flag_of_Norfolk_Island.svg',
                    'https://upload.wikimedia.org/wikipedia/commons/4/4e/Flag_of_Norfolk_Island.svg?download=1',
                    'https://flagcdn.com/nfk.svg',
                    'https://flagpedia.net/data/flags/w2560/nfk.svg',
                    'https://upload.wikimedia.org/wikipedia/commons/4/4e/Flag_of_Norfolk_Island.svg?uselang=en',
                    # Try with different file extensions
                    'https://upload.wikimedia.org/wikipedia/commons/4/4e/Flag_of_Norfolk_Island.png',
                    'https://flagcdn.com/nfk.png',
                    'https://flagpedia.net/data/flags/w2560/nfk.png'
                ]
            },
            'VAT': {
                'name': 'Vatican City',
                'urls': [
                    'https://upload.wikimedia.org/wikipedia/commons/0/00/Flag_of_the_Vatican_City.svg',
                    'https://upload.wikimedia.org/wikipedia/commons/thumb/0/00/Flag_of_the_Vatican_City.svg/1200px-Flag_of_the_Vatican_City.svg',
                    'https://upload.wikimedia.org/wikipedia/commons/0/00/Flag_of_the_Vatican_City.svg?download=1',
                    'https://flagcdn.com/vat.svg',
                    'https://flagpedia.net/data/flags/w2560/vat.svg',
                    'https://upload.wikimedia.org/wikipedia/commons/0/00/Flag_of_the_Vatican_City.svg?uselang=en',
                    # Try alternative Vatican flag URLs
                    'https://upload.wikimedia.org/wikipedia/commons/0/00/Flag_of_Vatican_City.svg',
                    'https://upload.wikimedia.org/wikipedia/commons/0/00/Flag_of_the_Holy_See.svg',
                    'https://upload.wikimedia.org/wikipedia/commons/0/00/Flag_of_the_Papal_States.svg',
                    # Try with different file extensions
                    'https://upload.wikimedia.org/wikipedia/commons/0/00/Flag_of_the_Vatican_City.png',
                    'https://flagcdn.com/vat.png',
                    'https://flagpedia.net/data/flags/w2560/vat.png'
                ]
            }
        }

    def download_flag(self, code: str, max_retries: int = 3) -> bool:
        """Download flag for a specific code with enhanced retry logic"""
        svg_path = self.svg_dir / f"{code}.svg"
        
        # Skip if file already exists and has content
        if svg_path.exists() and svg_path.stat().st_size > 100:
            print(f"⏭️  Skipping {code}.svg (already exists)")
            return True
        
        if code not in self.final_mappings:
            print(f"❌ No mapping found for {code}")
            return False
        
        mapping = self.final_mappings[code]
        urls = mapping['urls']
        
        print(f"\n🎯 Downloading {code} ({mapping['name']})...")
        
        for attempt in range(max_retries):
            for i, url in enumerate(urls):
                try:
                    print(f"   🔗 Attempt {attempt + 1}/{max_retries}, URL {i + 1}/{len(urls)}: {url}")
                    response = self.session.get(url, timeout=25)
                    
                    if response.status_code == 200 and response.content:
                        # Check if content looks like SVG or PNG
                        content_type = response.headers.get('content-type', '')
                        content = response.content
                        
                        if 'svg' in content_type.lower() or (b'<svg' in content and len(content) > 100):
                            # It's an SVG
                            svg_path.write_text(content.decode('utf-8'), encoding='utf-8')
                            print(f"   ✅ Successfully downloaded: {code}.svg")
                            return True
                        elif 'png' in content_type.lower() or content.startswith(b'\x89PNG'):
                            # It's a PNG, we'll need to convert or use as is
                            print(f"   ⚠️  Got PNG for {code}, saving as SVG placeholder")
                            # Create a simple SVG wrapper for the PNG
                            svg_content = f'''<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="800" viewBox="0 0 1200 800">
  <image href="{url}" width="1200" height="800"/>
</svg>'''
                            svg_path.write_text(svg_content, encoding='utf-8')
                            print(f"   ✅ Created SVG wrapper for {code}.svg")
                            return True
                        else:
                            print(f"   ⚠️  Unknown content type: {content_type}")
                    else:
                        print(f"   ❌ HTTP {response.status_code} for {code}")
                        
                except requests.RequestException as e:
                    print(f"   ⚠️  Request failed for {code}: {e}")
                    continue
                except Exception as e:
                    print(f"   ❌ Error downloading {code}: {e}")
                    continue
            
            if attempt < max_retries - 1:
                print(f"   🔄 Retrying {code} (attempt {attempt + 2}/{max_retries})")
                time.sleep(5 ** attempt)  # Longer exponential backoff
        
        print(f"   ❌ Failed to download {code}.svg after {max_retries} attempts")
        return False

    def download_final_flags(self):
        """Download the final 2 flags"""
        final_codes = ['NFK', 'VAT']
        
        print(f"🚀 Starting download of final {len(final_codes)} flags...")
        print(f"📁 Output directory: {self.svg_dir}")
        
        success_count = 0
        fail_count = 0
        failed_downloads = []
        
        for i, code in enumerate(final_codes, 1):
            print(f"\n[{i}/{len(final_codes)}] Processing {code}...")
            
            if self.download_flag(code):
                success_count += 1
            else:
                fail_count += 1
                failed_downloads.append(code)
            
            # Longer delay between requests
            time.sleep(2)
        
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
    downloader = FinalFlagsDownloader()
    
    try:
        downloader.download_final_flags()
    except KeyboardInterrupt:
        print("\n⏹️  Download interrupted by user")
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main() 