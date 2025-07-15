#!/usr/bin/env python3
"""
Download African Flags from Wikipedia
Downloads flags for African countries that have IOC codes from the Wikipedia page.
"""

import os
import json
import requests
from bs4 import BeautifulSoup
import re
from urllib.parse import urljoin, urlparse
import time
from typing import Dict, List, Optional, Tuple

class AfricanFlagDownloader:
    def __init__(self, output_dir="ui/public/assets/flags"):
        self.output_dir = output_dir
        self.wikipedia_url = "https://en.wikipedia.org/wiki/Flags_of_Africa"
        self.african_countries = {}
        self.downloaded_count = 0
        self.failed_count = 0
        
        # Create output directory if it doesn't exist
        os.makedirs(output_dir, exist_ok=True)
        
        # Load IOC database
        self.ioc_database = self.load_ioc_database()
        
        # African countries with IOC codes (manually curated)
        self.african_ioc_codes = {
            'Algeria': 'ALG',
            'Angola': 'ANG', 
            'Benin': 'BEN',
            'Botswana': 'BOT',
            'Burkina Faso': 'BUR',
            'Burundi': 'BDI',
            'Cameroon': 'CMR',
            'Cape Verde': 'CPV',
            'Central African Republic': 'CAF',
            'Chad': 'CHA',
            'Comoros': 'COM',
            'Congo': 'CGO',
            'Democratic Republic of the Congo': 'COD',
            'Djibouti': 'DJI',
            'Egypt': 'EGY',
            'Equatorial Guinea': 'GNQ',
            'Eritrea': 'ERI',
            'Ethiopia': 'ETH',
            'Gabon': 'GAB',
            'Gambia': 'GAM',
            'Ghana': 'GHA',
            'Guinea': 'GIN',
            'Guinea-Bissau': 'GNB',
            'Ivory Coast': 'CIV',
            'Kenya': 'KEN',
            'Lesotho': 'LES',
            'Liberia': 'LBR',
            'Libya': 'LBY',
            'Madagascar': 'MDG',
            'Malawi': 'MWI',
            'Mali': 'MLI',
            'Mauritania': 'MRT',
            'Mauritius': 'MUS',
            'Morocco': 'MAR',
            'Mozambique': 'MOZ',
            'Namibia': 'NAM',
            'Niger': 'NER',
            'Nigeria': 'NGA',
            'Rwanda': 'RWA',
            'São Tomé and Príncipe': 'STP',
            'Senegal': 'SEN',
            'Seychelles': 'SYC',
            'Sierra Leone': 'SLE',
            'Somalia': 'SOM',
            'South Africa': 'ZAF',
            'South Sudan': 'SSD',
            'Sudan': 'SDN',
            'Swaziland': 'SWZ',
            'Tanzania': 'TAN',
            'Togo': 'TGO',
            'Tunisia': 'TUN',
            'Uganda': 'UGA',
            'Zambia': 'ZMB',
            'Zimbabwe': 'ZWE'
        }
    
    def load_ioc_database(self) -> Dict:
        """Load the IOC database"""
        try:
            with open('scripts/media/ioc_flag_database.json', 'r', encoding='utf-8') as f:
                data = json.load(f)
                return data.get('ioc_entries', {})
        except FileNotFoundError:
            print("❌ IOC database not found. Creating empty database.")
            return {}
        except json.JSONDecodeError as e:
            print(f"❌ Error parsing IOC database: {e}")
            return {}
    
    def scrape_wikipedia_flags(self) -> Dict[str, str]:
        """Scrape flag URLs from Wikipedia page"""
        print(f"🌍 Scraping African flags from: {self.wikipedia_url}")
        
        try:
            response = requests.get(self.wikipedia_url, timeout=30)
            response.raise_for_status()
            
            soup = BeautifulSoup(response.content, 'html.parser')
            flag_urls = {}
            
            # Find all images in the page
            all_images = soup.find_all('img')
            print(f"  🔍 Found {len(all_images)} total images on the page")
            
            for img in all_images:
                alt_text = img.get('alt', '')
                src = img.get('src', '')
                title = img.get('title', '')
                
                if not src:
                    continue
                
                # Look for flag images with various patterns
                is_flag = False
                country_name = None
                
                # Pattern 1: "Flag of [Country]"
                if alt_text.lower().startswith('flag of '):
                    country_name = alt_text[8:].strip()
                    is_flag = True
                elif alt_text.lower().startswith('flag of'):
                    country_name = alt_text[7:].strip()
                    is_flag = True
                
                # Pattern 2: "[Country] flag" or "[Country] Flag"
                elif alt_text.lower().endswith(' flag'):
                    country_name = alt_text[:-5].strip()
                    is_flag = True
                
                # Pattern 3: Check title attribute
                elif title.lower().startswith('flag of '):
                    country_name = title[8:].strip()
                    is_flag = True
                elif title.lower().endswith(' flag'):
                    country_name = title[:-5].strip()
                    is_flag = True
                
                # Pattern 4: Look for flag in filename
                elif 'flag' in src.lower() and ('svg' in src.lower() or 'png' in src.lower()):
                    # Try to extract country name from filename
                    filename = os.path.basename(src)
                    if 'flag' in filename.lower():
                        # Extract country name from filename like "Flag_of_Algeria.svg"
                        match = re.search(r'flag_of_([^.]+)', filename.lower())
                        if match:
                            country_name = match.group(1).replace('_', ' ').title()
                            is_flag = True
                
                if not is_flag or not country_name:
                    continue
                
                # Clean up country name
                country_name = self.clean_country_name(country_name)
                if not country_name:
                    continue
                
                # Check if this country has an IOC code
                if country_name in self.african_ioc_codes:
                    # Convert relative URL to absolute
                    if src.startswith('//'):
                        flag_url = 'https:' + src
                    elif src.startswith('/'):
                        flag_url = 'https://en.wikipedia.org' + src
                    else:
                        flag_url = src
                    
                    flag_urls[country_name] = flag_url
                    print(f"  ✅ Found flag for {country_name} ({self.african_ioc_codes[country_name]})")
                    print(f"     URL: {flag_url}")
            
            return flag_urls
            
        except requests.RequestException as e:
            print(f"❌ Error fetching Wikipedia page: {e}")
            return {}
    
    def clean_country_name(self, country_name: str) -> Optional[str]:
        """Clean and normalize country name"""
        if not country_name:
            return None
        
        # Remove common suffixes
        country_name = re.sub(r'\s*\(.*?\)\s*', '', country_name)  # Remove parentheses
        country_name = re.sub(r'\s*\[.*?\]\s*', '', country_name)  # Remove brackets
        country_name = country_name.strip()
        
        # Handle special cases
        if country_name.lower() in ['the african union', 'african union']:
            return None  # Skip supranational organizations
        
        # Normalize common variations
        name_mapping = {
            'Cote d\'Ivoire': 'Ivory Coast',
            'Côte d\'Ivoire': 'Ivory Coast',
            'Cote dIvoire': 'Ivory Coast',
            'Côte dIvoire': 'Ivory Coast',
            'Democratic Republic of Congo': 'Democratic Republic of the Congo',
            'DR Congo': 'Democratic Republic of the Congo',
            'DRC': 'Democratic Republic of the Congo',
            'Republic of Congo': 'Congo',
            'Republic of the Congo': 'Congo',
            'Sao Tome and Principe': 'São Tomé and Príncipe',
            'Sao Tome': 'São Tomé and Príncipe',
            'Eswatini': 'Swaziland',  # Eswatini is the new name for Swaziland
        }
        
        return name_mapping.get(country_name, country_name)
    
    def download_flag_image(self, country_name: str, flag_url: str) -> bool:
        """Download a single flag image"""
        ioc_code = self.african_ioc_codes.get(country_name)
        if not ioc_code:
            print(f"❌ No IOC code found for {country_name}")
            return False
        
        # Check if file already exists
        output_path = os.path.join(self.output_dir, f"{ioc_code}.png")
        if os.path.exists(output_path):
            print(f"  ⏭️  Flag for {country_name} ({ioc_code}) already exists")
            return True
        
        try:
            print(f"  📥 Downloading flag for {country_name} ({ioc_code})...")
            
            # Add user agent to avoid being blocked
            headers = {
                'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36'
            }
            
            response = requests.get(flag_url, headers=headers, timeout=30)
            response.raise_for_status()
            
            # Save the image
            with open(output_path, 'wb') as f:
                f.write(response.content)
            
            print(f"  ✅ Downloaded {country_name} ({ioc_code})")
            self.downloaded_count += 1
            return True
            
        except requests.RequestException as e:
            print(f"  ❌ Failed to download {country_name}: {e}")
            self.failed_count += 1
            return False
        except Exception as e:
            print(f"  ❌ Error saving {country_name}: {e}")
            self.failed_count += 1
            return False
    
    def download_all_african_flags(self):
        """Download all African flags"""
        print("🚀 Starting African flag download process...")
        
        # Scrape flag URLs from Wikipedia
        flag_urls = self.scrape_wikipedia_flags()
        
        if not flag_urls:
            print("❌ No flag URLs found on Wikipedia page")
            return
        
        print(f"\n📊 Found {len(flag_urls)} African flags to download")
        
        # Download each flag
        for country_name, flag_url in flag_urls.items():
            self.download_flag_image(country_name, flag_url)
            time.sleep(0.5)  # Be nice to Wikipedia servers
        
        # Generate report
        self.generate_report(flag_urls)
    
    def generate_report(self, flag_urls: Dict[str, str]):
        """Generate a download report"""
        print(f"\n📋 Download Report")
        print("=" * 50)
        print(f"✅ Successfully downloaded: {self.downloaded_count}")
        print(f"❌ Failed downloads: {self.failed_count}")
        print(f"📁 Total African countries with IOC codes: {len(self.african_ioc_codes)}")
        print(f"🌐 Flags found on Wikipedia: {len(flag_urls)}")
        
        # List downloaded flags
        if self.downloaded_count > 0:
            print(f"\n📥 Downloaded flags:")
            for country_name, ioc_code in self.african_ioc_codes.items():
                flag_path = os.path.join(self.output_dir, f"{ioc_code}.png")
                if os.path.exists(flag_path):
                    print(f"  ✅ {country_name} ({ioc_code})")
        
        # List missing flags
        missing_flags = []
        for country_name, ioc_code in self.african_ioc_codes.items():
            flag_path = os.path.join(self.output_dir, f"{ioc_code}.png")
            if not os.path.exists(flag_path):
                missing_flags.append((country_name, ioc_code))
        
        if missing_flags:
            print(f"\n❌ Missing flags:")
            for country_name, ioc_code in missing_flags:
                print(f"  ❌ {country_name} ({ioc_code})")
        
        # Save report to file
        report_data = {
            "download_date": time.strftime("%Y-%m-%d %H:%M:%S"),
            "total_african_countries": len(self.african_ioc_codes),
            "flags_found_on_wikipedia": len(flag_urls),
            "successfully_downloaded": self.downloaded_count,
            "failed_downloads": self.failed_count,
            "downloaded_flags": [
                {"country": country_name, "ioc_code": ioc_code}
                for country_name, ioc_code in self.african_ioc_codes.items()
                if os.path.exists(os.path.join(self.output_dir, f"{ioc_code}.png"))
            ],
            "missing_flags": [
                {"country": country_name, "ioc_code": ioc_code}
                for country_name, ioc_code in missing_flags
            ]
        }
        
        report_path = os.path.join(self.output_dir, "african_flags_report.json")
        with open(report_path, 'w', encoding='utf-8') as f:
            json.dump(report_data, f, indent=2, ensure_ascii=False)
        
        print(f"\n📄 Report saved to: {report_path}")

def main():
    """Main function"""
    print("🇿🇦 African Flag Downloader")
    print("=" * 40)
    
    downloader = AfricanFlagDownloader()
    downloader.download_all_african_flags()
    
    print(f"\n🎉 African flag download process completed!")

if __name__ == "__main__":
    main() 