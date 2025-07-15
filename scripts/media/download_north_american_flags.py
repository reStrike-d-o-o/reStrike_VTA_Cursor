#!/usr/bin/env python3
"""
Download North American Flags from Wikipedia
Downloads flags for North American countries that have IOC codes from the Wikipedia page.
"""

import os
import json
import requests
from bs4 import BeautifulSoup
import re
from urllib.parse import urljoin, urlparse
from typing import Dict

class NorthAmericanFlagDownloader:
    def __init__(self, output_dir="ui/public/assets/flags"):
        self.output_dir = output_dir
        self.wikipedia_url = "https://en.wikipedia.org/wiki/Flags_of_North_America"
        self.north_american_countries = {}
        self.downloaded_count = 0
        self.failed_count = 0
        os.makedirs(output_dir, exist_ok=True)
        self.ioc_db = self.load_ioc_database()

    def load_ioc_database(self) -> Dict:
        try:
            with open("scripts/media/ioc_flag_database.json", "r", encoding="utf-8") as f:
                return json.load(f)
        except (FileNotFoundError, json.JSONDecodeError):
            print("Warning: IOC database not found or corrupted, using basic North American countries list")
            # Fallback to basic North American countries with IOC codes
            return {
                "ANT": {"name": "Netherlands Antilles", "category": "current"},
                "ARU": {"name": "Aruba", "category": "current"},
                "BAH": {"name": "Bahamas", "category": "current"},
                "BAR": {"name": "Barbados", "category": "current"},
                "BER": {"name": "Bermuda", "category": "current"},
                "CAN": {"name": "Canada", "category": "current"},
                "CAY": {"name": "Cayman Islands", "category": "current"},
                "CRC": {"name": "Costa Rica", "category": "current"},
                "CUB": {"name": "Cuba", "category": "current"},
                "DMA": {"name": "Dominica", "category": "current"},
                "DOM": {"name": "Dominican Republic", "category": "current"},
                "ESA": {"name": "El Salvador", "category": "current"},
                "GRN": {"name": "Grenada", "category": "current"},
                "GUA": {"name": "Guatemala", "category": "current"},
                "HAI": {"name": "Haiti", "category": "current"},
                "HON": {"name": "Honduras", "category": "current"},
                "ISV": {"name": "Virgin Islands", "category": "current"},
                "JAM": {"name": "Jamaica", "category": "current"},
                "LCA": {"name": "Saint Lucia", "category": "current"},
                "MEX": {"name": "Mexico", "category": "current"},
                "NCA": {"name": "Nicaragua", "category": "current"},
                "PAN": {"name": "Panama", "category": "current"},
                "PUR": {"name": "Puerto Rico", "category": "current"},
                "SKN": {"name": "Saint Kitts and Nevis", "category": "current"},
                "TRI": {"name": "Trinidad and Tobago", "category": "current"},
                "USA": {"name": "United States", "category": "current"},
                "VIN": {"name": "Saint Vincent and the Grenadines", "category": "current"},
            }

    def scrape_wikipedia_flags(self) -> Dict[str, str]:
        print(f"Scraping flags from: {self.wikipedia_url}")
        try:
            response = requests.get(self.wikipedia_url)
            response.raise_for_status()
        except requests.RequestException as e:
            print(f"Error fetching Wikipedia page: {e}")
            return {}
        soup = BeautifulSoup(response.content, 'html.parser')
        flag_urls = {}
        # Find all images in the sovereign states table
        tables = soup.find_all('table', {'class': 'wikitable'})
        for table in tables:
            rows = table.find_all('tr')
            for row in rows:
                cells = row.find_all('td')
                if len(cells) >= 2:
                    img = cells[0].find('img')
                    country_name = cells[1].get_text(strip=True)
                    if img and country_name:
                        src = img.get('src')
                        if src and not src.startswith('data:'):
                            # Clean up country name for matching
                            country_name_clean = re.sub(r'\[.*?\]', '', country_name).strip()
                            flag_urls[country_name_clean] = urljoin('https:', src)
        return flag_urls

    def download_sovereign_state_flags(self):
        try:
            response = requests.get(self.wikipedia_url)
            response.raise_for_status()
        except requests.RequestException as e:
            print(f"Error fetching Wikipedia page: {e}")
            return
        
        soup = BeautifulSoup(response.content, 'html.parser')
        # Find the section header for sovereign states
        header = soup.find('h2', id='Flags_of_North_American_sovereign_states')
        if not header:
            print("Could not find the sovereign states section header!")
            return
        # Find the next table after the header
        table = header.find_next('table', {'class': 'wikitable'})
        if not table:
            print("No sovereign states table found!")
            return
        flag_map = {}
        for row in table.find_all('tr')[1:]:  # skip header
            cells = row.find_all('td')
            if len(cells) >= 3:
                img = cells[0].find('img')
                country_tag = cells[2].find('b')
                country_name = None
                if country_tag:
                    a_tag = country_tag.find('a')
                    if a_tag:
                        country_name = a_tag.get_text(strip=True)
                if img and country_name:
                    src = img.get('src')
                    if src and not src.startswith('data:'):
                        flag_map[country_name] = urljoin('https:', src)
        # Download flags for IOC countries
        for ioc, info in self.ioc_db.items():
            country = info['name']
            url = flag_map.get(country)
            if not url:
                # Try alternative match (case-insensitive, partial)
                for k, v in flag_map.items():
                    if country.lower() in k.lower() or k.lower() in country.lower():
                        url = v
                        break
            if url:
                ext = os.path.splitext(urlparse(url).path)[1]
                filename = f"{ioc}{ext if ext else '.png'}"
                out_path = os.path.join(self.output_dir, filename)
                try:
                    img_data = requests.get(url).content
                    with open(out_path, 'wb') as f:
                        f.write(img_data)
                    print(f"Downloaded: {country} ({ioc}) -> {filename}")
                    self.downloaded_count += 1
                except Exception as e:
                    print(f"Failed to download {country} ({ioc}): {e}")
                    self.failed_count += 1
            else:
                print(f"No flag found for {country} ({ioc})")
                self.failed_count += 1
        print(f"\nDownload complete: {self.downloaded_count} flags downloaded, {self.failed_count} failed.")

    def run(self):
        self.download_sovereign_state_flags()

if __name__ == "__main__":
    downloader = NorthAmericanFlagDownloader()
    downloader.run() 