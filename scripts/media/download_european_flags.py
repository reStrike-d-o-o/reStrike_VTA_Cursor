#!/usr/bin/env python3
"""
Download European Flags from Wikipedia
Downloads flags for European countries that have IOC codes from the Wikipedia page.
"""

import os
import json
import requests
from bs4 import BeautifulSoup
import re
from urllib.parse import urljoin, urlparse
from typing import Dict

class EuropeanFlagDownloader:
    def __init__(self, output_dir="ui/public/assets/flags"):
        self.output_dir = output_dir
        self.wikipedia_url = "https://en.wikipedia.org/wiki/Flags_of_Europe"
        self.european_countries = {}
        self.downloaded_count = 0
        self.failed_count = 0
        os.makedirs(output_dir, exist_ok=True)
        self.ioc_db = self.load_ioc_database()

    def load_ioc_database(self) -> Dict:
        try:
            with open("scripts/media/ioc_flag_database.json", "r", encoding="utf-8") as f:
                return json.load(f)
        except (FileNotFoundError, json.JSONDecodeError):
            print("Warning: IOC database not found or corrupted, using basic European countries list")
            # Fallback to basic European countries with IOC codes
            return {
                "ALB": {"name": "Albania", "category": "current"},
                "AND": {"name": "Andorra", "category": "current"},
                "ARM": {"name": "Armenia", "category": "current"},
                "AUT": {"name": "Austria", "category": "current"},
                "AZE": {"name": "Azerbaijan", "category": "current"},
                "BEL": {"name": "Belgium", "category": "current"},
                "BIH": {"name": "Bosnia and Herzegovina", "category": "current"},
                "BLR": {"name": "Belarus", "category": "current"},
                "BUL": {"name": "Bulgaria", "category": "current"},
                "CRO": {"name": "Croatia", "category": "current"},
                "CYP": {"name": "Cyprus", "category": "current"},
                "CZE": {"name": "Czech Republic", "category": "current"},
                "DEN": {"name": "Denmark", "category": "current"},
                "ESP": {"name": "Spain", "category": "current"},
                "EST": {"name": "Estonia", "category": "current"},
                "FIN": {"name": "Finland", "category": "current"},
                "FRA": {"name": "France", "category": "current"},
                "GBR": {"name": "Great Britain", "category": "current"},
                "GEO": {"name": "Georgia", "category": "current"},
                "GER": {"name": "Germany", "category": "current"},
                "GRE": {"name": "Greece", "category": "current"},
                "HUN": {"name": "Hungary", "category": "current"},
                "IRL": {"name": "Ireland", "category": "current"},
                "ISL": {"name": "Iceland", "category": "current"},
                "ISR": {"name": "Israel", "category": "current"},
                "ITA": {"name": "Italy", "category": "current"},
                "KOS": {"name": "Kosovo", "category": "current"},
                "LAT": {"name": "Latvia", "category": "current"},
                "LIE": {"name": "Liechtenstein", "category": "current"},
                "LTU": {"name": "Lithuania", "category": "current"},
                "LUX": {"name": "Luxembourg", "category": "current"},
                "MLT": {"name": "Malta", "category": "current"},
                "MDA": {"name": "Moldova", "category": "current"},
                "MON": {"name": "Monaco", "category": "current"},
                "MNE": {"name": "Montenegro", "category": "current"},
                "NED": {"name": "Netherlands", "category": "current"},
                "MKD": {"name": "North Macedonia", "category": "current"},
                "NOR": {"name": "Norway", "category": "current"},
                "POL": {"name": "Poland", "category": "current"},
                "POR": {"name": "Portugal", "category": "current"},
                "ROU": {"name": "Romania", "category": "current"},
                "RUS": {"name": "Russia", "category": "current"},
                "SMR": {"name": "San Marino", "category": "current"},
                "SRB": {"name": "Serbia", "category": "current"},
                "SVK": {"name": "Slovakia", "category": "current"},
                "SLO": {"name": "Slovenia", "category": "current"},
                "SUI": {"name": "Switzerland", "category": "current"},
                "SWE": {"name": "Sweden", "category": "current"},
                "TUR": {"name": "Turkey", "category": "current"},
                "UKR": {"name": "Ukraine", "category": "current"},
                "VAT": {"name": "Vatican City", "category": "current"},
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

    def match_and_download_flags(self):
        flag_urls = self.scrape_wikipedia_flags()
        for ioc, info in self.ioc_db.items():
            country = info['name']
            # Try direct match
            url = flag_urls.get(country)
            # Try alternative match (case-insensitive, partial)
            if not url:
                for k, v in flag_urls.items():
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

    def run(self):
        self.match_and_download_flags()
        print(f"\nDownload complete: {self.downloaded_count} flags downloaded, {self.failed_count} failed.")

    def debug_html_structure(self):
        with open("scripts/media/europe_html_sample.html", "r", encoding="utf-8") as f:
            html = f.read()
        soup = BeautifulSoup(html, 'html.parser')
        for img in soup.find_all('img'):
            parent = img.find_parent()
            print("IMG:", img)
            print("PARENT:", parent)
            print("NEIGHBORS:", parent.find_next_siblings()[:2])
            print("---")

    def debug_flag_table(self):
        with open("scripts/media/europe_html_sample.html", "r", encoding="utf-8") as f:
            html = f.read()
        soup = BeautifulSoup(html, 'html.parser')
        table = soup.find('table', {'class': 'wikitable sortable'})
        if not table:
            print("No wikitable sortable found!")
            return
        for row in table.find_all('tr')[1:]:  # skip header
            cells = row.find_all('td')
            if len(cells) >= 3:
                img = cells[0].find('img')
                use = cells[2].get_text(strip=True)
                if img and use:
                    print(f"Flag src: {img.get('src')}, Use: {use}")

    def download_sovereign_state_flags(self):
        with open("scripts/media/europe_html_sample.html", "r", encoding="utf-8") as f:
            html = f.read()
        soup = BeautifulSoup(html, 'html.parser')
        # Find the section header for sovereign states
        header = soup.find('h2', id='Flags_of_European_sovereign_states')
        if not header:
            header = soup.find('span', string='Flags of European sovereign states')
            if header:
                header = header.find_parent('h2')
        if not header:
            print("Could not find the sovereign states section header!")
            return
        # Find the next table after the header
        table = header.find_next('table', {'class': 'wikitable sortable'})
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

if __name__ == "__main__":
    downloader = EuropeanFlagDownloader()
    downloader.download_sovereign_state_flags() 