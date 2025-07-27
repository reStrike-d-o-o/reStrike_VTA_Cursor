#!/usr/bin/env python3
"""
IOC Wikipedia Scraper
Scrapes the Wikipedia IOC country codes page to understand the structure
"""

import requests
from bs4 import BeautifulSoup
import json
from pathlib import Path
import sys

# Add project root to path for imports
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))

def scrape_ioc_wikipedia():
    """Scrape the Wikipedia IOC country codes page"""
    url = "https://en.wikipedia.org/wiki/List_of_IOC_country_codes"
    
    headers = {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36',
        'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8',
        'Accept-Language': 'en-US,en;q=0.5',
        'Accept-Encoding': 'gzip, deflate',
        'Connection': 'keep-alive',
        'Upgrade-Insecure-Requests': '1',
    }
    
    try:
        print(f"🔗 Fetching: {url}")
        response = requests.get(url, headers=headers, timeout=15)
        response.raise_for_status()
        
        print("✅ Successfully fetched Wikipedia page")
        print(f"📄 Content length: {len(response.text)} characters")
        
        # Parse with BeautifulSoup
        soup = BeautifulSoup(response.text, 'html.parser')
        
        # Find the main content area
        content = soup.find('div', {'id': 'mw-content-text'})
        if not content:
            print("❌ Could not find main content area")
            return
        
        # Look for tables
        tables = content.find_all('table')
        print(f"📊 Found {len(tables)} tables")
        
        # Find the IOC codes table - use the first table which has "Code National Olympic Committee"
        ioc_table = tables[0]  # This is the main IOC codes table
        print(f"✅ Using table at index 0 (Code National Olympic Committee)")
        
        # Extract data from the table
        rows = ioc_table.find_all('tr')
        print(f"📋 Found {len(rows)} rows in IOC table")
        
        ioc_data = []
        for row in rows[1:]:  # Skip header row
            cells = row.find_all(['td', 'th'])
            if len(cells) >= 2:
                ioc_code = cells[0].get_text().strip()
                country = cells[1].get_text().strip()
                notes = cells[2].get_text().strip() if len(cells) > 2 else ""
                
                # Look for flag image in any cell
                flag_url = None
                for cell in cells:
                    flag_img = cell.find('img')
                    if flag_img:
                        src = flag_img.get('src')
                        if src:
                            if src.startswith('//'):
                                flag_url = 'https:' + src
                            elif src.startswith('/'):
                                flag_url = 'https://en.wikipedia.org' + src
                            else:
                                flag_url = src
                            break
                
                ioc_data.append({
                    'ioc_code': ioc_code,
                    'country': country,
                    'notes': notes,
                    'flag_url': flag_url
                })
        
        print(f"📝 Extracted {len(ioc_data)} IOC entries")
        
        # Save the scraped data
        output_file = project_root / "ui" / "public" / "assets" / "flags" / "scraped_ioc_data.json"
        with open(output_file, 'w', encoding='utf-8') as f:
            json.dump({
                'source_url': url,
                'scraped_at': str(Path(__file__).stat().st_mtime),
                'total_entries': len(ioc_data),
                'ioc_entries': ioc_data
            }, f, indent=2, ensure_ascii=False)
        
        print(f"💾 Saved scraped data to: {output_file}")
        
        # Show some examples
        print("\n📋 Sample IOC entries:")
        for entry in ioc_data[:10]:
            print(f"  {entry['ioc_code']}: {entry['country']} - Flag: {entry['flag_url']}")
        
        # Count flags found
        flags_found = sum(1 for entry in ioc_data if entry['flag_url'])
        print(f"\n🏁 Flags found: {flags_found}/{len(ioc_data)}")
        
        return ioc_data
        
    except Exception as e:
        print(f"❌ Error scraping Wikipedia: {e}")
        return None

def analyze_flag_urls(ioc_data):
    """Analyze the flag URLs to understand the pattern"""
    if not ioc_data:
        return
    
    print("\n🔍 Analyzing flag URL patterns:")
    
    flag_urls = [entry['flag_url'] for entry in ioc_data if entry['flag_url']]
    
    # Group by URL pattern
    patterns = {}
    for url in flag_urls:
        if 'upload.wikimedia.org' in url:
            if '/thumb/' in url:
                patterns['thumb'] = patterns.get('thumb', 0) + 1
            else:
                patterns['direct'] = patterns.get('direct', 0) + 1
        elif 'wikipedia.org' in url:
            patterns['wikipedia'] = patterns.get('wikipedia', 0) + 1
        else:
            patterns['other'] = patterns.get('other', 0) + 1
    
    for pattern, count in patterns.items():
        print(f"  {pattern}: {count} URLs")
    
    # Show some example URLs
    print("\n📋 Example flag URLs:")
    for url in flag_urls[:5]:
        print(f"  {url}")

if __name__ == "__main__":
    print("🌐 IOC Wikipedia Scraper")
    print("=" * 50)
    
    ioc_data = scrape_ioc_wikipedia()
    if ioc_data:
        analyze_flag_urls(ioc_data) 