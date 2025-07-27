#!/usr/bin/env python3
"""
Wikipedia Flag Search
Searches Wikipedia directly for the correct flag URLs
"""

import requests
from bs4 import BeautifulSoup
import json
from pathlib import Path
import sys
import re

project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))

def search_wikipedia_flags():
    headers = {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36',
        'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8',
        'Accept-Language': 'en-US,en;q=0.5',
        'Accept-Encoding': 'gzip, deflate',
        'Connection': 'keep-alive',
        'Upgrade-Insecure-Requests': '1',
    }
    
    # Search for Norfolk Island flag
    print("🔍 Searching for Norfolk Island flag...")
    nfk_url = "https://en.wikipedia.org/wiki/Norfolk_Island"
    try:
        response = requests.get(nfk_url, headers=headers, timeout=15)
        response.raise_for_status()
        soup = BeautifulSoup(response.text, 'html.parser')
        
        # Look for flag images
        flag_images = soup.find_all('img', alt=re.compile(r'flag|Flag', re.I))
        print(f"Found {len(flag_images)} potential flag images for Norfolk Island:")
        
        for img in flag_images:
            src = img.get('src')
            alt = img.get('alt', '')
            if src:
                if src.startswith('//'):
                    full_url = 'https:' + src
                elif src.startswith('/'):
                    full_url = 'https://en.wikipedia.org' + src
                else:
                    full_url = src
                print(f"  - {alt}: {full_url}")
                
                # Convert to SVG if it's a thumbnail
                if '/thumb/' in full_url:
                    svg_url = full_url.replace('/thumb/', '/')
                    svg_url = re.sub(r'/\d+px-.*\.png$', '', svg_url)
                    print(f"    SVG URL: {svg_url}")
        
    except Exception as e:
        print(f"❌ Error searching Norfolk Island: {e}")
    
    print("\n" + "="*50 + "\n")
    
    # Search for Vatican City flag
    print("🔍 Searching for Vatican City flag...")
    vat_url = "https://en.wikipedia.org/wiki/Vatican_City"
    try:
        response = requests.get(vat_url, headers=headers, timeout=15)
        response.raise_for_status()
        soup = BeautifulSoup(response.text, 'html.parser')
        
        # Look for flag images
        flag_images = soup.find_all('img', alt=re.compile(r'flag|Flag', re.I))
        print(f"Found {len(flag_images)} potential flag images for Vatican City:")
        
        for img in flag_images:
            src = img.get('src')
            alt = img.get('alt', '')
            if src:
                if src.startswith('//'):
                    full_url = 'https:' + src
                elif src.startswith('/'):
                    full_url = 'https://en.wikipedia.org' + src
                else:
                    full_url = src
                print(f"  - {alt}: {full_url}")
                
                # Convert to SVG if it's a thumbnail
                if '/thumb/' in full_url:
                    svg_url = full_url.replace('/thumb/', '/')
                    svg_url = re.sub(r'/\d+px-.*\.png$', '', svg_url)
                    print(f"    SVG URL: {svg_url}")
        
    except Exception as e:
        print(f"❌ Error searching Vatican City: {e}")

if __name__ == "__main__":
    search_wikipedia_flags() 