#!/usr/bin/env python3
"""
Download Asian Flags from Wikipedia
Downloads flags for Asian countries that have IOC codes from the Wikipedia page.
"""

import os
import json
import requests
from bs4 import BeautifulSoup
import re
from urllib.parse import urljoin, urlparse
import time
from typing import Dict, List, Optional, Tuple

class AsianFlagDownloader:
    def __init__(self, output_dir="ui/public/assets/flags"):
        self.output_dir = output_dir
        self.wikipedia_url = "https://en.wikipedia.org/wiki/Flags_of_Asia"
        self.asian_countries = {}
        self.downloaded_count = 0
        self.failed_count = 0
        
        # Create output directory if it doesn't exist
        os.makedirs(output_dir, exist_ok=True)
        
        # Load IOC database
        self.ioc_database = self.load_ioc_database()
        
    def load_ioc_database(self) -> Dict:
        """Load the IOC country codes database"""
        try:
            with open("scripts/media/ioc_flag_database.json", "r", encoding="utf-8") as f:
                return json.load(f)
        except (FileNotFoundError, json.JSONDecodeError):
            print("Warning: IOC database not found or corrupted, using basic Asian countries list")
            # Fallback to basic Asian countries with IOC codes
            return {
                "AFG": {"name": "Afghanistan", "category": "current"},
                "BHR": {"name": "Bahrain", "category": "current"},
                "BGD": {"name": "Bangladesh", "category": "current"},
                "BTN": {"name": "Bhutan", "category": "current"},
                "BRN": {"name": "Brunei", "category": "current"},
                "KHM": {"name": "Cambodia", "category": "current"},
                "CHN": {"name": "China", "category": "current"},
                "HKG": {"name": "Hong Kong", "category": "current"},
                "IND": {"name": "India", "category": "current"},
                "IDN": {"name": "Indonesia", "category": "current"},
                "IRN": {"name": "Iran", "category": "current"},
                "IRQ": {"name": "Iraq", "category": "current"},
                "ISR": {"name": "Israel", "category": "current"},
                "JPN": {"name": "Japan", "category": "current"},
                "JOR": {"name": "Jordan", "category": "current"},
                "KAZ": {"name": "Kazakhstan", "category": "current"},
                "KWT": {"name": "Kuwait", "category": "current"},
                "KGZ": {"name": "Kyrgyzstan", "category": "current"},
                "LAO": {"name": "Laos", "category": "current"},
                "LBN": {"name": "Lebanon", "category": "current"},
                "MAC": {"name": "Macau", "category": "current"},
                "MYS": {"name": "Malaysia", "category": "current"},
                "MDV": {"name": "Maldives", "category": "current"},
                "MNG": {"name": "Mongolia", "category": "current"},
                "MMR": {"name": "Myanmar", "category": "current"},
                "NPL": {"name": "Nepal", "category": "current"},
                "OMN": {"name": "Oman", "category": "current"},
                "PAK": {"name": "Pakistan", "category": "current"},
                "PSE": {"name": "Palestine", "category": "current"},
                "PHL": {"name": "Philippines", "category": "current"},
                "QAT": {"name": "Qatar", "category": "current"},
                "SAU": {"name": "Saudi Arabia", "category": "current"},
                "SGP": {"name": "Singapore", "category": "current"},
                "LKA": {"name": "Sri Lanka", "category": "current"},
                "SYR": {"name": "Syria", "category": "current"},
                "TWN": {"name": "Taiwan", "category": "current"},
                "TJK": {"name": "Tajikistan", "category": "current"},
                "THA": {"name": "Thailand", "category": "current"},
                "TLS": {"name": "Timor-Leste", "category": "current"},
                "TUR": {"name": "Turkey", "category": "current"},
                "TKM": {"name": "Turkmenistan", "category": "current"},
                "ARE": {"name": "United Arab Emirates", "category": "current"},
                "UZB": {"name": "Uzbekistan", "category": "current"},
                "VNM": {"name": "Vietnam", "category": "current"},
                "YEM": {"name": "Yemen", "category": "current"}
            }
    
    def scrape_wikipedia_flags(self) -> Dict[str, str]:
        """Scrape flag images from the Wikipedia page"""
        print(f"Scraping flags from: {self.wikipedia_url}")
        
        try:
            response = requests.get(self.wikipedia_url)
            response.raise_for_status()
        except requests.RequestException as e:
            print(f"Error fetching Wikipedia page: {e}")
            return {}
        
        soup = BeautifulSoup(response.content, 'html.parser')
        flag_urls = {}
        
        # Look for all images that might be flags
        all_images = soup.find_all('img')
        
        for img in all_images:
            src = img.get('src', '')
            alt = img.get('alt', '').lower()
            
            # Check if this looks like a flag image
            if ('flag' in alt or 'flag' in src) and ('svg' in src or 'png' in src):
                # Convert to full URL if needed
                if src.startswith('//'):
                    src = 'https:' + src
                elif src.startswith('/'):
                    src = 'https://en.wikipedia.org' + src
                
                # Extract country name from alt text
                country_name = self.extract_country_name_from_alt(alt)
                
                if country_name:
                    flag_urls[country_name] = src
                    print(f"Found flag for {country_name}: {src}")
        
        # Also look specifically in tables for sovereign states
        tables = soup.find_all('table', class_='wikitable')
        for table in tables:
            # Look for the "Flags of Asian sovereign states" table
            caption = table.find('caption')
            if caption and 'sovereign states' in caption.get_text().lower():
                rows = table.find_all('tr')
                for row in rows[1:]:  # Skip header row
                    cells = row.find_all(['td', 'th'])
                    if len(cells) >= 2:
                        # First cell usually contains the flag
                        flag_cell = cells[0]
                        flag_img = flag_cell.find('img')
                        
                        if flag_img:
                            src = flag_img.get('src', '')
                            alt = flag_img.get('alt', '')
                            
                            if src:
                                if src.startswith('//'):
                                    src = 'https:' + src
                                elif src.startswith('/'):
                                    src = 'https://en.wikipedia.org' + src
                                
                                # Try to get country name from the second cell
                                if len(cells) > 1:
                                    country_cell = cells[1]
                                    country_name = country_cell.get_text().strip()
                                    if country_name:
                                        flag_urls[country_name] = src
                                        print(f"Found flag for {country_name}: {src}")
        
        return flag_urls
    
    def extract_country_name_from_alt(self, alt_text: str) -> Optional[str]:
        """Extract country name from alt text"""
        if not alt_text:
            return None
        
        # Clean the alt text
        alt_text = alt_text.lower()
        alt_text = re.sub(r'flag\s+of\s+', '', alt_text)
        alt_text = re.sub(r'\s+flag', '', alt_text)
        alt_text = re.sub(r'\([^)]*\)', '', alt_text)
        alt_text = re.sub(r'\[[^\]]*\]', '', alt_text)
        alt_text = alt_text.strip()
        
        # Try to match with IOC database
        for ioc_code, data in self.ioc_database.items():
            country_name = data.get('name', '').lower()
            if country_name in alt_text:
                return data['name']
        
        # If no match found, try to clean and return the alt text
        if alt_text and len(alt_text) > 2:
            return alt_text.title()
        
        return None
    
    def clean_country_name(self, country_name: str) -> Optional[str]:
        """Clean and standardize country name"""
        if not country_name:
            return None
        
        # Remove common prefixes/suffixes
        name = country_name.strip()
        name = re.sub(r'^flag\s+of\s+', '', name, flags=re.IGNORECASE)
        name = re.sub(r'\s+flag$', '', name, flags=re.IGNORECASE)
        name = re.sub(r'\([^)]*\)', '', name)  # Remove parentheses
        name = name.strip()
        
        # Try to match with IOC database
        for ioc_code, data in self.ioc_database.items():
            if data.get('name', '').lower() == name.lower():
                return data['name']
        
        return name if name else None
    
    def download_flag_image(self, country_name: str, flag_url: str) -> bool:
        """Download a single flag image"""
        try:
            # Clean country name
            clean_name = self.clean_country_name(country_name)
            if not clean_name:
                print(f"Could not clean country name: {country_name}")
                return False
            
            # Find IOC code for this country
            ioc_code = None
            for code, data in self.ioc_database.items():
                if data.get('name', '').lower() == clean_name.lower():
                    ioc_code = code
                    break
            
            if not ioc_code:
                print(f"No IOC code found for: {clean_name}")
                return False
            
            # Check if file already exists
            output_path = os.path.join(self.output_dir, f"{ioc_code}.png")
            if os.path.exists(output_path):
                print(f"Flag already exists for {clean_name} ({ioc_code})")
                return True
            
            # Download the image
            print(f"Downloading flag for {clean_name} ({ioc_code}) from {flag_url}")
            
            response = requests.get(flag_url, timeout=30)
            response.raise_for_status()
            
            # Save the image
            with open(output_path, 'wb') as f:
                f.write(response.content)
            
            print(f"✅ Downloaded: {ioc_code}.png ({clean_name})")
            self.downloaded_count += 1
            return True
            
        except Exception as e:
            print(f"❌ Failed to download flag for {country_name}: {e}")
            self.failed_count += 1
            return False
    
    def download_all_asian_flags(self):
        """Download all Asian flags from Wikipedia"""
        print("🌏 Starting Asian flag download process...")
        
        # Scrape flag URLs from Wikipedia
        flag_urls = self.scrape_wikipedia_flags()
        
        if not flag_urls:
            print("❌ No flag URLs found on Wikipedia page")
            return
        
        print(f"Found {len(flag_urls)} potential flag URLs")
        
        # Download each flag
        for country_name, flag_url in flag_urls.items():
            self.download_flag_image(country_name, flag_url)
            time.sleep(0.5)  # Be respectful to Wikipedia servers
        
        # Generate report
        self.generate_report(flag_urls)
    
    def generate_report(self, flag_urls: Dict[str, str]):
        """Generate a summary report"""
        print("\n" + "="*60)
        print("🌏 ASIAN FLAGS DOWNLOAD SUMMARY")
        print("="*60)
        
        print(f"📊 Download Statistics:")
        print(f"   • Total flags found on Wikipedia: {len(flag_urls)}")
        print(f"   • Successfully downloaded: {self.downloaded_count}")
        print(f"   • Failed downloads: {self.failed_count}")
        print(f"   • Success rate: {(self.downloaded_count/len(flag_urls)*100):.1f}%")
        
        # List downloaded flags
        if self.downloaded_count > 0:
            print(f"\n✅ Successfully Downloaded Flags:")
            downloaded_flags = []
            for ioc_code, data in self.ioc_database.items():
                flag_path = os.path.join(self.output_dir, f"{ioc_code}.png")
                if os.path.exists(flag_path):
                    downloaded_flags.append((ioc_code, data.get('name', 'Unknown')))
            
            # Sort by IOC code
            downloaded_flags.sort(key=lambda x: x[0])
            
            for ioc_code, country_name in downloaded_flags:
                print(f"   • {ioc_code} - {country_name}")
        
        # List failed downloads
        if self.failed_count > 0:
            print(f"\n❌ Failed Downloads:")
            for country_name, flag_url in flag_urls.items():
                clean_name = self.clean_country_name(country_name)
                if clean_name:
                    ioc_code = None
                    for code, data in self.ioc_database.items():
                        if data.get('name', '').lower() == clean_name.lower():
                            ioc_code = code
                            break
                    
                    flag_path = os.path.join(self.output_dir, f"{ioc_code}.png") if ioc_code else None
                    if not flag_path or not os.path.exists(flag_path):
                        print(f"   • {country_name} ({ioc_code or 'No IOC code'})")
        
        # Save detailed report
        report_data = {
            "download_date": time.strftime("%Y-%m-%d %H:%M:%S"),
            "source_url": self.wikipedia_url,
            "statistics": {
                "total_found": len(flag_urls),
                "downloaded": self.downloaded_count,
                "failed": self.failed_count,
                "success_rate": round(self.downloaded_count/len(flag_urls)*100, 1) if flag_urls else 0
            },
            "downloaded_flags": [
                {"ioc_code": ioc_code, "country_name": data.get('name', 'Unknown')}
                for ioc_code, data in self.ioc_database.items()
                if os.path.exists(os.path.join(self.output_dir, f"{ioc_code}.png"))
            ],
            "failed_downloads": [
                {"country_name": country_name, "url": flag_url}
                for country_name, flag_url in flag_urls.items()
                if not self.download_flag_image(country_name, flag_url)
            ]
        }
        
        report_path = os.path.join(self.output_dir, "asian_flags_report.json")
        with open(report_path, 'w', encoding='utf-8') as f:
            json.dump(report_data, f, indent=2, ensure_ascii=False)
        
        print(f"\n📄 Detailed report saved to: {report_path}")
        print("="*60)

def main():
    """Main function"""
    downloader = AsianFlagDownloader()
    downloader.download_all_asian_flags()

if __name__ == "__main__":
    main() 