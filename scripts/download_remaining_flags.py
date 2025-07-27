#!/usr/bin/env python3
"""
Remaining Flags Downloader
Downloads the last 7 missing flags using specialized approaches
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

class RemainingFlagsDownloader:
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
        
        # Specialized mappings for the remaining codes
        self.special_mappings = {
            'NFK': {
                'name': 'Norfolk Island',
                'urls': [
                    'https://upload.wikimedia.org/wikipedia/commons/4/4e/Flag_of_Norfolk_Island.svg',
                    'https://flagcdn.com/nfk.svg',
                    'https://flagpedia.net/data/flags/w2560/nfk.svg',
                    'https://upload.wikimedia.org/wikipedia/commons/thumb/4/4e/Flag_of_Norfolk_Island.svg/1200px-Flag_of_Norfolk_Island.svg'
                ]
            },
            'NIU': {
                'name': 'Niue',
                'urls': [
                    'https://upload.wikimedia.org/wikipedia/commons/0/01/Flag_of_Niue.svg',
                    'https://flagcdn.com/niu.svg',
                    'https://flagpedia.net/data/flags/w2560/niu.svg',
                    'https://upload.wikimedia.org/wikipedia/commons/thumb/0/01/Flag_of_Niue.svg/1200px-Flag_of_Niue.svg'
                ]
            },
            'NMI': {
                'name': 'Northern Mariana Islands',
                'urls': [
                    'https://upload.wikimedia.org/wikipedia/commons/e/e0/Flag_of_the_Northern_Mariana_Islands.svg',
                    'https://flagcdn.com/mnp.svg',  # ISO code for Northern Mariana Islands
                    'https://flagpedia.net/data/flags/w2560/mnp.svg',
                    'https://upload.wikimedia.org/wikipedia/commons/thumb/e/e0/Flag_of_the_Northern_Mariana_Islands.svg/1200px-Flag_of_the_Northern_Mariana_Islands.svg'
                ]
            },
            'PYF': {
                'name': 'French Polynesia',
                'urls': [
                    'https://upload.wikimedia.org/wikipedia/commons/d/db/Flag_of_French_Polynesia.svg',
                    'https://flagcdn.com/pyf.svg',
                    'https://flagpedia.net/data/flags/w2560/pyf.svg',
                    'https://upload.wikimedia.org/wikipedia/commons/thumb/d/db/Flag_of_French_Polynesia.svg/1200px-Flag_of_French_Polynesia.svg'
                ]
            },
            'RPC': {
                'name': 'Russian Paralympic Committee',
                'urls': [
                    'https://upload.wikimedia.org/wikipedia/commons/7/7f/Paralympic_flag_%282019%29.svg',
                    'https://upload.wikimedia.org/wikipedia/commons/9/9d/Paralympic_flag_%282010-2019%29.svg',
                    'https://upload.wikimedia.org/wikipedia/commons/f/f8/Paralympic_flag_%281988-1994%29.svg',
                    'https://upload.wikimedia.org/wikipedia/commons/a/a7/Olympic_flag.svg'
                ]
            },
            'TKL': {
                'name': 'Tokelau',
                'urls': [
                    'https://upload.wikimedia.org/wikipedia/commons/8/8e/Flag_of_Tokelau.svg',
                    'https://flagcdn.com/tkl.svg',
                    'https://flagpedia.net/data/flags/w2560/tkl.svg',
                    'https://upload.wikimedia.org/wikipedia/commons/thumb/8/8e/Flag_of_Tokelau.svg/1200px-Flag_of_Tokelau.svg'
                ]
            },
            'VAT': {
                'name': 'Vatican City',
                'urls': [
                    'https://upload.wikimedia.org/wikipedia/commons/0/00/Flag_of_the_Vatican_City.svg',
                    'https://flagcdn.com/vat.svg',
                    'https://flagpedia.net/data/flags/w2560/vat.svg',
                    'https://upload.wikimedia.org/wikipedia/commons/thumb/0/00/Flag_of_the_Vatican_City.svg/1200px-Flag_of_the_Vatican_City.svg'
                ]
            }
        }

    def download_flag(self, code: str, max_retries: int = 5) -> bool:
        """Download flag for a specific code with enhanced retry logic"""
        svg_path = self.svg_dir / f"{code}.svg"
        
        # Skip if file already exists and has content
        if svg_path.exists() and svg_path.stat().st_size > 100:
            print(f"⏭️  Skipping {code}.svg (already exists)")
            return True
        
        if code not in self.special_mappings:
            print(f"❌ No special mapping found for {code}")
            return False
        
        mapping = self.special_mappings[code]
        urls = mapping['urls']
        
        print(f"\n🎯 Downloading {code} ({mapping['name']})...")
        
        for attempt in range(max_retries):
            for i, url in enumerate(urls):
                try:
                    print(f"   🔗 Attempt {attempt + 1}/{max_retries}, URL {i + 1}/{len(urls)}: {url}")
                    response = self.session.get(url, timeout=20)
                    
                    if response.status_code == 200 and response.content:
                        # Check if content looks like SVG
                        content = response.text
                        if '<svg' in content.lower() and len(content) > 100:
                            svg_path.write_text(content, encoding='utf-8')
                            print(f"   ✅ Successfully downloaded: {code}.svg")
                            return True
                        else:
                            print(f"   ⚠️  Invalid SVG content for {code}")
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
                time.sleep(3 ** attempt)  # Longer exponential backoff
        
        print(f"   ❌ Failed to download {code}.svg after {max_retries} attempts")
        return False

    def download_remaining_flags(self):
        """Download the remaining 7 flags"""
        remaining_codes = ['NFK', 'NIU', 'NMI', 'PYF', 'RPC', 'TKL', 'VAT']
        
        print(f"🚀 Starting download of remaining {len(remaining_codes)} flags...")
        print(f"📁 Output directory: {self.svg_dir}")
        
        success_count = 0
        fail_count = 0
        failed_downloads = []
        
        for i, code in enumerate(remaining_codes, 1):
            print(f"\n[{i}/{len(remaining_codes)}] Processing {code}...")
            
            if self.download_flag(code):
                success_count += 1
            else:
                fail_count += 1
                failed_downloads.append(code)
            
            # Longer delay between requests to be respectful
            time.sleep(1)
        
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
    downloader = RemainingFlagsDownloader()
    
    try:
        downloader.download_remaining_flags()
    except KeyboardInterrupt:
        print("\n⏹️  Download interrupted by user")
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main() 