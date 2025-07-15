#!/usr/bin/env python3
"""
Oceanian Flag Downloader

Downloads flag images from the Wikipedia page for Oceanian flags.
Matches flags with IOC country codes and downloads them to the flags directory.
"""

import os
import re
import json
import requests
from bs4 import BeautifulSoup
from urllib.parse import urljoin, urlparse
from typing import Dict, Optional, List
import time

class OceanianFlagDownloader:
    def __init__(self, output_dir="ui/public/assets/flags"):
        self.output_dir = output_dir
        self.wikipedia_url = "https://en.wikipedia.org/wiki/Flags_of_Oceania"
        self.session = requests.Session()
        self.session.headers.update({
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36'
        })
        
        # Ensure output directory exists
        os.makedirs(output_dir, exist_ok=True)
        
        # Load IOC database for country code matching
        self.ioc_database = self.load_ioc_database()

    def load_ioc_database(self) -> Dict:
        """Load IOC country codes database"""
        try:
            with open("scripts/media/ioc_flag_database.json", "r", encoding="utf-8") as f:
                data = json.load(f)
                return {entry["name"].lower(): entry["code"] for entry in data.get("countries", [])}
        except (FileNotFoundError, json.JSONDecodeError):
            print("Warning: IOC database not found or corrupted, using fallback Oceanian countries list")
            # Fallback list of Oceanian countries with IOC codes
            return {
                "australia": "AUS",
                "fiji": "FIJ", 
                "kiribati": "KIR",
                "marshall islands": "MHL",
                "federated states of micronesia": "FSM",
                "nauru": "NRU",
                "new zealand": "NZL",
                "palau": "PLW",
                "papua new guinea": "PNG",
                "samoa": "SAM",
                "solomon islands": "SOL",
                "tonga": "TGA",
                "tuvalu": "TUV",
                "vanuatu": "VAN",
                "cook islands": "COK",
                "niue": "NIU",
                "tokelau": "TKL",
                "american samoa": "ASA",
                "french polynesia": "PYF",
                "guam": "GUM",
                "new caledonia": "NCL",
                "northern mariana islands": "MNP",
                "pitcairn islands": "PCN",
                "wallis and futuna": "WLF",
                "norfolk island": "NFK",
                "christmas island": "CXR",
                "cocos (keeling) islands": "CCK",
                "easter island": "RAP",
                "wake island": "WAK"
            }

    def scrape_wikipedia_flags(self) -> Dict[str, str]:
        """Scrape flag URLs from the Wikipedia page"""
        print(f"Scraping flags from: {self.wikipedia_url}")
        
        try:
            response = self.session.get(self.wikipedia_url)
            response.raise_for_status()
        except requests.RequestException as e:
            print(f"Error fetching Wikipedia page: {e}")
            return {}

        soup = BeautifulSoup(response.content, 'html.parser')
        flag_urls = {}

        # Look for the main flags table - "Flags of Oceanian sovereign states"
        tables = soup.find_all('table', {'class': 'wikitable'})
        
        for table in tables:
            # Check if this table contains Oceanian sovereign states
            # Look for the heading before the table
            prev_element = table.find_previous_sibling()
            if prev_element and prev_element.name == 'div' and 'mw-heading' in prev_element.get('class', []):
                heading = prev_element.find('h2')
                if heading and 'sovereign states' in heading.get_text().lower():
                    print(f"Found sovereign states table: {heading.get_text().strip()}")
                    
                    rows = table.find_all('tr')[1:]  # Skip header row
                    
                    for row in rows:
                        cells = row.find_all(['td', 'th'])
                        if len(cells) >= 3:  # Flag, Date, Use columns
                            # Look for flag image in the first cell
                            flag_img = cells[0].find('img')
                            if flag_img:
                                flag_url = flag_img.get('src')
                                if flag_url:
                                    # Convert to full URL
                                    if flag_url.startswith('//'):
                                        flag_url = 'https:' + flag_url
                                    elif flag_url.startswith('/'):
                                        flag_url = 'https://en.wikipedia.org' + flag_url
                                    
                                    # Extract country name from the third cell (Use column)
                                    country_cell = cells[2]
                                    country_name = self.extract_country_name(country_cell)
                                    
                                    if country_name:
                                        flag_urls[country_name] = flag_url
                                        print(f"Found flag for {country_name}: {flag_url}")

        # Also check for dependencies and territories
        for table in tables:
            # Look for the heading before the table
            prev_element = table.find_previous_sibling()
            if prev_element and prev_element.name == 'div' and 'mw-heading' in prev_element.get('class', []):
                heading = prev_element.find('h2')
                if heading and ('dependencies' in heading.get_text().lower() or 'territories' in heading.get_text().lower()):
                    print(f"Found dependencies/territories table: {heading.get_text().strip()}")
                    
                    rows = table.find_all('tr')[1:]  # Skip header row
                    
                    for row in rows:
                        cells = row.find_all(['td', 'th'])
                        if len(cells) >= 3:  # Flag, Date, Use columns
                            flag_img = cells[0].find('img')
                            if flag_img:
                                flag_url = flag_img.get('src')
                                if flag_url:
                                    if flag_url.startswith('//'):
                                        flag_url = 'https:' + flag_url
                                    elif flag_url.startswith('/'):
                                        flag_url = 'https://en.wikipedia.org' + flag_url
                                    
                                    country_cell = cells[2]  # Use column
                                    country_name = self.extract_country_name(country_cell)
                                    
                                    if country_name:
                                        flag_urls[country_name] = flag_url
                                        print(f"Found flag for {country_name}: {flag_url}")

        return flag_urls

    def extract_country_name(self, cell) -> Optional[str]:
        """Extract country name from table cell"""
        # Remove any links and get clean text
        text = cell.get_text().strip()
        
        # Clean up common Wikipedia formatting
        text = re.sub(r'\[.*?\]', '', text)  # Remove citations
        text = re.sub(r'\(.*?\)', '', text)  # Remove parentheses
        text = re.sub(r'see also:.*', '', text)  # Remove "see also" text
        text = re.sub(r'flag of\s+', '', text, flags=re.IGNORECASE)  # Remove "flag of" prefix
        text = re.sub(r'flags of\s+', '', text, flags=re.IGNORECASE)  # Remove "flags of" prefix
        text = text.strip()
        
        # Handle special cases
        if text.lower() == "federated states of micronesia":
            return "federated states of micronesia"
        elif text.lower() == "marshall islands":
            return "marshall islands"
        elif text.lower() == "solomon islands":
            return "solomon islands"
        elif text.lower() == "cook islands":
            return "cook islands"
        elif text.lower() == "northern mariana islands":
            return "northern mariana islands"
        elif text.lower() == "pitcairn islands":
            return "pitcairn islands"
        elif text.lower() == "wallis and futuna":
            return "wallis and futuna"
        elif text.lower() == "new caledonia":
            return "new caledonia"
        elif text.lower() == "norfolk island":
            return "norfolk island"
        elif text.lower() == "christmas island":
            return "christmas island"
        elif text.lower() == "cocos (keeling) islands":
            return "cocos (keeling) islands"
        elif text.lower() == "easter island":
            return "easter island"
        elif text.lower() == "wake island":
            return "wake island"
        
        return text.lower() if text else None

    def clean_country_name(self, country_name: str) -> Optional[str]:
        """Clean and standardize country name for IOC matching"""
        if not country_name:
            return None
            
        # Remove common prefixes/suffixes
        name = country_name.lower()
        name = re.sub(r'^flag of\s+', '', name)
        name = re.sub(r'\s+flag$', '', name)
        name = name.strip()
        
        # Handle special cases
        name_mapping = {
            "federated states of micronesia": "federated states of micronesia",
            "marshall islands": "marshall islands", 
            "solomon islands": "solomon islands",
            "cook islands": "cook islands",
            "northern mariana islands": "northern mariana islands",
            "pitcairn islands": "pitcairn islands",
            "wallis and futuna": "wallis and futuna",
            "american samoa": "american samoa",
            "french polynesia": "french polynesia",
            "new caledonia": "new caledonia"
        }
        
        return name_mapping.get(name, name)

    def download_flag_image(self, country_name: str, flag_url: str) -> bool:
        """Download flag image and save with IOC code"""
        try:
            # Clean country name
            clean_name = self.clean_country_name(country_name)
            if not clean_name:
                print(f"Could not clean country name: {country_name}")
                return False

            # Find IOC code
            ioc_code = self.ioc_database.get(clean_name)
            if not ioc_code:
                print(f"No IOC code found for: {clean_name}")
                return False

            # Check if file already exists
            output_path = os.path.join(self.output_dir, f"{ioc_code}.png")
            if os.path.exists(output_path):
                print(f"Flag already exists for {clean_name} ({ioc_code})")
                return True

            # Download image
            print(f"Downloading flag for {clean_name} ({ioc_code}) from {flag_url}")
            response = self.session.get(flag_url, timeout=30)
            response.raise_for_status()

            # Save image
            with open(output_path, 'wb') as f:
                f.write(response.content)

            print(f"✅ Downloaded: {ioc_code}.png ({clean_name})")
            return True

        except Exception as e:
            print(f"❌ Error downloading flag for {country_name}: {e}")
            return False

    def download_all_oceanian_flags(self):
        """Download all Oceanian flags"""
        print("🌏 Starting Oceanian flag download...")
        
        # Scrape flag URLs
        flag_urls = self.scrape_wikipedia_flags()
        
        if not flag_urls:
            print("No flags found on the Wikipedia page")
            return

        print(f"\nFound {len(flag_urls)} flags to download:")
        for country, url in flag_urls.items():
            print(f"  - {country}")

        # Download flags
        successful_downloads = 0
        for country_name, flag_url in flag_urls.items():
            if self.download_flag_image(country_name, flag_url):
                successful_downloads += 1
            time.sleep(0.5)  # Be respectful to Wikipedia servers

        print(f"\n🎉 Download complete!")
        print(f"Successfully downloaded: {successful_downloads}/{len(flag_urls)} flags")

        # Generate report
        self.generate_report(flag_urls)

    def generate_report(self, flag_urls: Dict[str, str]):
        """Generate a report of the download process"""
        report = {
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "source_url": self.wikipedia_url,
            "total_flags_found": len(flag_urls),
            "successful_downloads": 0,
            "failed_downloads": 0,
            "flags": []
        }

        for country_name, flag_url in flag_urls.items():
            clean_name = self.clean_country_name(country_name)
            ioc_code = self.ioc_database.get(clean_name) if clean_name else None
            
            flag_info = {
                "country_name": country_name,
                "clean_name": clean_name,
                "ioc_code": ioc_code,
                "flag_url": flag_url,
                "downloaded": False,
                "file_path": None
            }

            if ioc_code:
                file_path = os.path.join(self.output_dir, f"{ioc_code}.png")
                if os.path.exists(file_path):
                    flag_info["downloaded"] = True
                    flag_info["file_path"] = f"{ioc_code}.png"
                    report["successful_downloads"] += 1
                else:
                    report["failed_downloads"] += 1
            else:
                report["failed_downloads"] += 1

            report["flags"].append(flag_info)

        # Save report
        report_path = os.path.join(self.output_dir, "oceanian_flags_report.json")
        with open(report_path, 'w', encoding='utf-8') as f:
            json.dump(report, f, indent=2, ensure_ascii=False)

        # Generate summary
        summary_path = os.path.join(self.output_dir, "OCEANIAN_FLAGS_DOWNLOAD_SUMMARY.md")
        with open(summary_path, 'w', encoding='utf-8') as f:
            f.write("# Oceanian Flags Download Summary\n\n")
            f.write(f"**Download Date**: {report['timestamp']}\n")
            f.write(f"**Source**: {report['source_url']}\n\n")
            
            f.write(f"## Statistics\n")
            f.write(f"- **Total Flags Found**: {report['total_flags_found']}\n")
            f.write(f"- **Successfully Downloaded**: {report['successful_downloads']}\n")
            f.write(f"- **Failed Downloads**: {report['failed_downloads']}\n\n")
            
            f.write("## Successfully Downloaded Flags\n")
            for flag in report["flags"]:
                if flag["downloaded"]:
                    f.write(f"- **{flag['ioc_code']}** - {flag['country_name']}\n")
            
            if report["failed_downloads"] > 0:
                f.write("\n## Failed Downloads\n")
                for flag in report["flags"]:
                    if not flag["downloaded"]:
                        f.write(f"- **{flag['country_name']}** - {flag.get('ioc_code', 'No IOC code')}\n")

        print(f"\n📊 Reports generated:")
        print(f"  - JSON: {report_path}")
        print(f"  - Summary: {summary_path}")

def main():
    """Main function"""
    downloader = OceanianFlagDownloader()
    downloader.download_all_oceanian_flags()

if __name__ == "__main__":
    main() 