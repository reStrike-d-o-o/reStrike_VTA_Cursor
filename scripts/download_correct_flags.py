#!/usr/bin/env python3
"""
Download Correct Flags
Downloads the final 2 flags using the correct URLs found from Wikipedia
"""

import os
import sys
import requests
import time
from pathlib import Path

# Add project root to path for imports
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))

class CorrectFlagsDownloader:
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
        
        # Correct URLs found from Wikipedia search
        self.correct_urls = {
            'NFK': {
                'name': 'Norfolk Island',
                'url': 'https://upload.wikimedia.org/wikipedia/commons/4/48/Flag_of_Norfolk_Island.svg'
            },
            'VAT': {
                'name': 'Vatican City',
                'url': 'https://upload.wikimedia.org/wikipedia/commons/b/b3/Flag_of_Vatican_City_%282023%E2%80%93present%29.svg'
            }
        }

    def download_flag(self, code: str) -> bool:
        """Download flag for a specific code"""
        svg_path = self.svg_dir / f"{code}.svg"
        
        # Skip if file already exists and has content
        if svg_path.exists() and svg_path.stat().st_size > 100:
            print(f"⏭️  Skipping {code}.svg (already exists)")
            return True
        
        if code not in self.correct_urls:
            print(f"❌ No correct URL found for {code}")
            return False
        
        mapping = self.correct_urls[code]
        url = mapping['url']
        
        print(f"\n🎯 Downloading {code} ({mapping['name']})...")
        print(f"   🔗 URL: {url}")
        
        try:
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
        except Exception as e:
            print(f"   ❌ Error downloading {code}: {e}")
        
        print(f"   ❌ Failed to download {code}.svg")
        return False

    def download_correct_flags(self):
        """Download the final 2 flags with correct URLs"""
        final_codes = ['NFK', 'VAT']
        
        print(f"🚀 Starting download of final {len(final_codes)} flags with correct URLs...")
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
            
            # Small delay between requests
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
    downloader = CorrectFlagsDownloader()
    
    try:
        downloader.download_correct_flags()
    except KeyboardInterrupt:
        print("\n⏹️  Download interrupted by user")
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main() 