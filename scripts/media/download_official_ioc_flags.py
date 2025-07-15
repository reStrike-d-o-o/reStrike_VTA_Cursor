#!/usr/bin/env python3
"""
Official IOC Flag Downloader

Downloads flag images directly from the Wikipedia IOC codes page.
Scrapes all tables for IOC codes and flags, prioritizing Current NOCs.
"""

import os
import re
import json
import requests
from bs4 import BeautifulSoup
from typing import Dict, Optional, List, Tuple, Set
import time

class IOCFlagDownloader:
    def __init__(self, output_dir="ui/public/assets/flags"):
        self.output_dir = output_dir
        self.wikipedia_url = "https://en.wikipedia.org/wiki/List_of_IOC_country_codes"
        self.session = requests.Session()
        self.session.headers.update({
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36'
        })
        
        # Create output directory if it doesn't exist
        os.makedirs(output_dir, exist_ok=True)
        
        # Results tracking
        self.downloaded_flags = []
        self.failed_downloads = []
        self.missing_flag_urls = []
        self.downloaded_ioc_codes = set()  # Track which IOC codes we've already downloaded

    def scrape_ioc_codes_from_wikipedia(self) -> List[Dict]:
        """
        Scrape all IOC codes from the Wikipedia IOC codes page.
        Returns list of dictionaries with country info and IOC codes.
        """
        print(f"Scraping Wikipedia page: {self.wikipedia_url}")
        
        try:
            response = self.session.get(self.wikipedia_url)
            response.raise_for_status()
            soup = BeautifulSoup(response.content, 'html.parser')
            
            all_countries = []
            
            # Find all tables on the page
            tables = soup.find_all('table', {'class': 'wikitable'})
            
            for table_index, table in enumerate(tables):
                # Get table caption to identify the table type
                caption = table.find('caption')
                table_name = caption.get_text().strip() if caption else f"Table {table_index + 1}"
                print(f"\n📋 Processing table: {table_name}")
                
                # Get all rows except header
                rows = table.find_all('tr')[1:]  # Skip header row
                
                for row in rows:
                    cells = row.find_all(['td', 'th'])
                    if len(cells) >= 2:  # Need at least country name and IOC code
                        country_info = self.extract_country_info(cells, table_name)
                        if country_info:
                            all_countries.append(country_info)
            
            return all_countries
            
        except Exception as e:
            print(f"Error scraping Wikipedia: {e}")
            return []

    def extract_country_info(self, cells, table_name: str) -> Optional[Dict]:
        """Extract country information from table row"""
        try:
            # Different tables have different structures
            # Look for IOC code column (usually 3-letter code)
            ioc_code = None
            country_name = None
            flag_url = None
            
            for i, cell in enumerate(cells):
                cell_text = cell.get_text().strip()
                
                # Look for IOC code (3 uppercase letters)
                if re.match(r'^[A-Z]{3}$', cell_text):
                    ioc_code = cell_text
                
                # Look for country name (usually in first column or near IOC code)
                if not country_name and len(cell_text) > 2 and not re.match(r'^[A-Z]{3}$', cell_text):
                    # Check if this cell contains a country name
                    if not any(char.isdigit() for char in cell_text) and len(cell_text) < 100:
                        country_name = cell_text
            
            # If we found an IOC code, try to get country name from nearby cells
            if ioc_code and not country_name:
                for cell in cells:
                    cell_text = cell.get_text().strip()
                    if (len(cell_text) > 2 and 
                        not re.match(r'^[A-Z]{3}$', cell_text) and 
                        not any(char.isdigit() for char in cell_text) and
                        len(cell_text) < 100):
                        country_name = cell_text
                        break
            
            # Look for flag image in any cell
            for cell in cells:
                flag_img = cell.find('img')
                if flag_img:
                    flag_url = flag_img.get('src')
                    if flag_url:
                        # Convert to full URL if it's relative
                        if flag_url.startswith('//'):
                            flag_url = 'https:' + flag_url
                        elif flag_url.startswith('/'):
                            flag_url = 'https://en.wikipedia.org' + flag_url
                        break
            
            if ioc_code and country_name:
                return {
                    'country': country_name,
                    'ioc_code': ioc_code,
                    'flag_url': flag_url,
                    'table': table_name
                }
            
        except Exception as e:
            print(f"Error extracting country info: {e}")
        
        return None

    def download_flag_image(self, ioc_code: str, flag_url: str, country_name: str, table_name: str) -> bool:
        """Download flag image and save as IOC code"""
        try:
            print(f"Downloading {country_name} ({ioc_code}) from {table_name}")
            
            response = self.session.get(flag_url)
            response.raise_for_status()
            
            # Save as PNG with IOC code
            output_path = os.path.join(self.output_dir, f"{ioc_code}.png")
            
            with open(output_path, 'wb') as f:
                f.write(response.content)
            
            print(f"✅ Downloaded: {ioc_code}.png")
            return True
            
        except Exception as e:
            print(f"❌ Failed to download {ioc_code}: {e}")
            return False

    def download_all_ioc_flags(self):
        """Main method to download all IOC flags"""
        print("🚀 Starting IOC Flag Download from Wikipedia")
        print("=" * 60)
        
        # Scrape all IOC codes from Wikipedia
        all_countries = self.scrape_ioc_codes_from_wikipedia()
        
        if not all_countries:
            print("❌ No countries found on Wikipedia page")
            return
        
        print(f"\n📊 Found {len(all_countries)} countries with IOC codes")
        
        # Group by table type
        current_nocs = [c for c in all_countries if 'current' in c['table'].lower() and 'noc' in c['table'].lower()]
        other_countries = [c for c in all_countries if not ('current' in c['table'].lower() and 'noc' in c['table'].lower())]
        
        print(f"   Current NOCs: {len(current_nocs)}")
        print(f"   Other tables: {len(other_countries)}")
        
        # First, process Current NOCs (main table)
        print(f"\n🎯 Processing Current NOCs table...")
        for country in current_nocs:
            if country['flag_url']:
                success = self.download_flag_image(
                    country['ioc_code'], 
                    country['flag_url'], 
                    country['country'], 
                    country['table']
                )
                if success:
                    self.downloaded_flags.append(country)
                    self.downloaded_ioc_codes.add(country['ioc_code'])
                else:
                    self.failed_downloads.append({
                        **country,
                        'reason': 'Download failed'
                    })
            else:
                self.missing_flag_urls.append({
                    **country,
                    'reason': 'No flag URL found'
                })
        
        # Then process other tables, but only if IOC code not already downloaded
        print(f"\n🔍 Processing other tables (only new IOC codes)...")
        for country in other_countries:
            if country['ioc_code'] not in self.downloaded_ioc_codes:
                if country['flag_url']:
                    success = self.download_flag_image(
                        country['ioc_code'], 
                        country['flag_url'], 
                        country['country'], 
                        country['table']
                    )
                    if success:
                        self.downloaded_flags.append(country)
                        self.downloaded_ioc_codes.add(country['ioc_code'])
                    else:
                        self.failed_downloads.append({
                            **country,
                            'reason': 'Download failed'
                        })
                else:
                    self.missing_flag_urls.append({
                        **country,
                        'reason': 'No flag URL found'
                    })
            else:
                print(f"⏭️  Skipping {country['ioc_code']} - already downloaded from Current NOCs")
        
        # Generate comprehensive report
        self.generate_detailed_report()

    def generate_detailed_report(self):
        """Generate comprehensive download report"""
        print("\n" + "=" * 60)
        print("📋 IOC FLAG DOWNLOAD REPORT")
        print("=" * 60)
        
        # Summary statistics
        total_found = len(self.downloaded_flags) + len(self.failed_downloads) + len(self.missing_flag_urls)
        
        print(f"\n📊 SUMMARY:")
        print(f"   Total IOC codes found: {total_found}")
        print(f"   ✅ Successfully downloaded: {len(self.downloaded_flags)}")
        print(f"   ❌ Failed downloads: {len(self.failed_downloads)}")
        print(f"   🔍 Missing flag URLs: {len(self.missing_flag_urls)}")
        print(f"   🎯 Unique IOC codes downloaded: {len(self.downloaded_ioc_codes)}")
        
        # Successfully downloaded flags
        if self.downloaded_flags:
            print(f"\n✅ SUCCESSFULLY DOWNLOADED ({len(self.downloaded_flags)} flags):")
            for flag in self.downloaded_flags:
                print(f"   {flag['ioc_code']} - {flag['country']} ({flag['table']})")
        
        # Failed downloads
        if self.failed_downloads:
            print(f"\n❌ FAILED DOWNLOADS ({len(self.failed_downloads)} flags):")
            for flag in self.failed_downloads:
                print(f"   {flag['ioc_code']} - {flag['country']} ({flag['reason']})")
        
        # Missing flag URLs
        if self.missing_flag_urls:
            print(f"\n🔍 MISSING FLAG URLS ({len(self.missing_flag_urls)} flags):")
            print("   These countries have IOC codes but no flag URLs were found:")
            for flag in self.missing_flag_urls:
                print(f"   {flag['ioc_code']} - {flag['country']} ({flag['table']})")
        
        # Save detailed report to file
        report_data = {
            'summary': {
                'total_found': total_found,
                'successfully_downloaded': len(self.downloaded_flags),
                'failed_downloads': len(self.failed_downloads),
                'missing_flag_urls': len(self.missing_flag_urls),
                'unique_ioc_codes': len(self.downloaded_ioc_codes)
            },
            'downloaded_flags': self.downloaded_flags,
            'failed_downloads': self.failed_downloads,
            'missing_flag_urls': self.missing_flag_urls,
            'downloaded_ioc_codes': list(self.downloaded_ioc_codes)
        }
        
        report_path = os.path.join(self.output_dir, "ioc_flags_report.json")
        with open(report_path, 'w', encoding='utf-8') as f:
            json.dump(report_data, f, indent=2, ensure_ascii=False)
        
        # Generate markdown report
        markdown_report = self.generate_markdown_report(report_data)
        markdown_path = os.path.join(self.output_dir, "IOC_FLAGS_DOWNLOAD_REPORT.md")
        with open(markdown_path, 'w', encoding='utf-8') as f:
            f.write(markdown_report)
        
        print(f"\n📄 Reports saved:")
        print(f"   JSON: {report_path}")
        print(f"   Markdown: {markdown_path}")

    def generate_markdown_report(self, report_data):
        """Generate markdown format report"""
        md = """# IOC Flags Download Report

## Summary

- **Total IOC codes found**: {total_found}
- **Successfully downloaded**: {successfully_downloaded}
- **Failed downloads**: {failed_downloads}
- **Missing flag URLs**: {missing_flag_urls}
- **Unique IOC codes downloaded**: {unique_ioc_codes}

## Successfully Downloaded Flags

""".format(**report_data['summary'])
        
        if report_data['downloaded_flags']:
            md += "| IOC Code | Country | Table | Flag URL |\n"
            md += "|----------|---------|-------|----------|\n"
            for flag in report_data['downloaded_flags']:
                md += f"| {flag['ioc_code']} | {flag['country']} | {flag['table']} | {flag.get('flag_url', 'N/A')} |\n"
        else:
            md += "No flags were successfully downloaded.\n"
        
        md += "\n## Failed Downloads\n\n"
        if report_data['failed_downloads']:
            md += "| IOC Code | Country | Table | Reason |\n"
            md += "|----------|---------|-------|--------|\n"
            for flag in report_data['failed_downloads']:
                md += f"| {flag['ioc_code']} | {flag['country']} | {flag['table']} | {flag['reason']} |\n"
        else:
            md += "No failed downloads.\n"
        
        md += "\n## Missing Flag URLs\n\n"
        if report_data['missing_flag_urls']:
            md += "These countries have IOC codes but no flag URLs were found:\n\n"
            md += "| IOC Code | Country | Table |\n"
            md += "|----------|---------|-------|\n"
            for flag in report_data['missing_flag_urls']:
                md += f"| {flag['ioc_code']} | {flag['country']} | {flag['table']} |\n"
        else:
            md += "No missing flag URLs.\n"
        
        md += "\n## All Downloaded IOC Codes\n\n"
        if report_data['downloaded_ioc_codes']:
            md += "```\n"
            md += ", ".join(sorted(report_data['downloaded_ioc_codes']))
            md += "\n```\n"
        else:
            md += "No IOC codes were downloaded.\n"
        
        return md

def main():
    """Main execution function"""
    downloader = IOCFlagDownloader()
    downloader.download_all_ioc_flags()

if __name__ == "__main__":
    main() 