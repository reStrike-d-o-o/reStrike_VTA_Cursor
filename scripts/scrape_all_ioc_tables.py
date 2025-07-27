#!/usr/bin/env python3
"""
Comprehensive IOC Wikipedia Scraper
Scrapes ALL IOC tables to find missing flags
"""

import requests
from bs4 import BeautifulSoup
import json
from pathlib import Path
import sys

project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))

def scrape_all_ioc_tables():
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
        response = requests.get(url, headers=headers, timeout=15)
        response.raise_for_status()
        soup = BeautifulSoup(response.text, 'html.parser')
        content = soup.find('div', {'id': 'mw-content-text'})
        tables = content.find_all('table')
        
        print(f"📋 Found {len(tables)} tables on the page")
        
        all_ioc_data = []
        table_info = []
        
        for i, table in enumerate(tables):
            # Try to find table caption or nearby heading
            caption = table.find('caption')
            if caption:
                table_title = caption.get_text().strip()
            else:
                # Look for nearby heading
                prev_heading = table.find_previous(['h2', 'h3', 'h4'])
                if prev_heading:
                    table_title = prev_heading.get_text().strip()
                else:
                    table_title = f"Table {i+1}"
            
            print(f"\n🔍 Processing table {i+1}: {table_title}")
            
            rows = table.find_all('tr')
            if len(rows) < 2:  # Skip tables with no data rows
                print(f"   ⏭️  Skipping - no data rows")
                continue
            
            # Check if this table has IOC-like data
            headers = rows[0].find_all(['th', 'td'])
            header_text = ' '.join([h.get_text().strip() for h in headers]).lower()
            
            if any(keyword in header_text for keyword in ['code', 'ioc', 'country', 'nation']):
                print(f"   ✅ IOC-like table detected")
                
                table_entries = []
                for row in rows[1:]:  # Skip header row
                    cells = row.find_all(['td', 'th'])
                    if len(cells) >= 2:
                        ioc_code = cells[0].get_text().strip()
                        country = cells[1].get_text().strip()
                        notes = cells[2].get_text().strip() if len(cells) > 2 else ""
                        
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
                        
                        entry = {
                            'ioc_code': ioc_code,
                            'country': country,
                            'notes': notes,
                            'flag_url': flag_url,
                            'table_source': table_title
                        }
                        table_entries.append(entry)
                        all_ioc_data.append(entry)
                
                table_info.append({
                    'table_index': i,
                    'title': table_title,
                    'entries_count': len(table_entries),
                    'entries': table_entries
                })
                print(f"   📊 Found {len(table_entries)} entries")
            else:
                print(f"   ⏭️  Skipping - not IOC-related")
        
        # Save comprehensive data
        output_file = project_root / "ui" / "public" / "assets" / "flags" / "all_ioc_tables_data.json"
        with open(output_file, 'w', encoding='utf-8') as f:
            json.dump({
                'source_url': url,
                'scraped_at': str(Path(__file__).stat().st_mtime),
                'total_tables': len(tables),
                'ioc_tables': len(table_info),
                'total_entries': len(all_ioc_data),
                'table_info': table_info,
                'all_entries': all_ioc_data
            }, f, indent=2, ensure_ascii=False)
        
        print(f"\n📊 Summary:")
        print(f"   Total tables found: {len(tables)}")
        print(f"   IOC-related tables: {len(table_info)}")
        print(f"   Total IOC entries: {len(all_ioc_data)}")
        print(f"   Data saved to: {output_file}")
        
        # Show table breakdown
        print(f"\n📋 Table Breakdown:")
        for table in table_info:
            print(f"   {table['title']}: {table['entries_count']} entries")
        
        return all_ioc_data
        
    except Exception as e:
        print(f"❌ Error scraping Wikipedia: {e}")
        return None

if __name__ == "__main__":
    ioc_data = scrape_all_ioc_tables()
    if ioc_data:
        print(f"\n✅ Successfully scraped {len(ioc_data)} IOC entries from all tables")
    else:
        print("❌ Failed to scrape IOC data") 