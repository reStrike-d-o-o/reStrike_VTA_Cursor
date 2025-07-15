#!/usr/bin/env python3
"""
Official IOC Flag Downloader

Downloads flag images for all countries listed in the Wikipedia List of Sovereign States
that have valid IOC codes (UN members and observers only). Flags are saved as {IOC}.png.
"""

import os
import re
import json
import requests
from bs4 import BeautifulSoup
from typing import Dict, Optional
import time

IOC_CODES = {
    # UN members and observers with IOC codes
    "afghanistan": "AFG",
    "albania": "ALB",
    "algeria": "ALG",
    "andorra": "AND",
    "angola": "ANG",
    "antigua and barbuda": "ANT",
    "argentina": "ARG",
    "armenia": "ARM",
    "australia": "AUS",
    "austria": "AUT",
    "azerbaijan": "AZE",
    "bahamas": "BAH",
    "bahrain": "BRN",
    "bangladesh": "BAN",
    "barbados": "BAR",
    "belarus": "BLR",
    "belgium": "BEL",
    "belize": "BIZ",
    "benin": "BEN",
    "bhutan": "BHU",
    "bolivia": "BOL",
    "bosnia and herzegovina": "BIH",
    "botswana": "BOT",
    "brazil": "BRA",
    "brunei": "BRU",
    "bulgaria": "BUL",
    "burkina faso": "BUR",
    "burundi": "BDI",
    "cabo verde": "CPV",
    "cambodia": "CAM",
    "cameroon": "CMR",
    "canada": "CAN",
    "central african republic": "CAF",
    "chad": "CHA",
    "chile": "CHI",
    "china": "CHN",
    "colombia": "COL",
    "comoros": "COM",
    "congo": "CGO",
    "costa rica": "CRC",
    "croatia": "CRO",
    "cuba": "CUB",
    "cyprus": "CYP",
    "czechia": "CZE",
    "democratic republic of the congo": "COD",
    "denmark": "DEN",
    "djibouti": "DJI",
    "dominica": "DMA",
    "dominican republic": "DOM",
    "ecuador": "ECU",
    "egypt": "EGY",
    "el salvador": "ESA",
    "equatorial guinea": "GEQ",
    "eritrea": "ERI",
    "estonia": "EST",
    "eswatini": "SWZ",
    "ethiopia": "ETH",
    "fiji": "FIJ",
    "finland": "FIN",
    "france": "FRA",
    "gabon": "GAB",
    "gambia": "GAM",
    "georgia": "GEO",
    "germany": "GER",
    "ghana": "GHA",
    "greece": "GRE",
    "grenada": "GRN",
    "guatemala": "GUA",
    "guinea": "GUI",
    "guinea-bissau": "GBS",
    "guyana": "GUY",
    "haiti": "HAI",
    "honduras": "HON",
    "hungary": "HUN",
    "iceland": "ISL",
    "india": "IND",
    "indonesia": "INA",
    "iran": "IRI",
    "iraq": "IRQ",
    "ireland": "IRL",
    "israel": "ISR",
    "italy": "ITA",
    "ivory coast": "CIV",
    "jamaica": "JAM",
    "japan": "JPN",
    "jordan": "JOR",
    "kazakhstan": "KAZ",
    "kenya": "KEN",
    "kiribati": "KIR",
    "kuwait": "KUW",
    "kyrgyzstan": "KGZ",
    "laos": "LAO",
    "latvia": "LAT",
    "lebanon": "LBN",
    "lesotho": "LES",
    "liberia": "LBR",
    "libya": "LBA",
    "liechtenstein": "LIE",
    "lithuania": "LTU",
    "luxembourg": "LUX",
    "madagascar": "MAD",
    "malawi": "MAW",
    "malaysia": "MAS",
    "maldives": "MDV",
    "mali": "MLI",
    "malta": "MLT",
    "marshall islands": "MHL",
    "mauritania": "MTN",
    "mauritius": "MRI",
    "mexico": "MEX",
    "micronesia": "FSM",
    "moldova": "MDA",
    "monaco": "MON",
    "mongolia": "MGL",
    "montenegro": "MNE",
    "morocco": "MAR",
    "mozambique": "MOZ",
    "myanmar": "MYA",
    "namibia": "NAM",
    "nauru": "NRU",
    "nepal": "NEP",
    "netherlands": "NED",
    "new zealand": "NZL",
    "nicaragua": "NCA",
    "niger": "NIG",
    "nigeria": "NGR",
    "north korea": "PRK",
    "north macedonia": "MKD",
    "norway": "NOR",
    "oman": "OMA",
    "pakistan": "PAK",
    "palau": "PLW",
    "palestine": "PLE",
    "panama": "PAN",
    "papua new guinea": "PNG",
    "paraguay": "PAR",
    "peru": "PER",
    "philippines": "PHI",
    "poland": "POL",
    "portugal": "POR",
    "qatar": "QAT",
    "romania": "ROU",
    "russia": "RUS",
    "rwanda": "RWA",
    "saint kitts and nevis": "SKN",
    "saint lucia": "LCA",
    "saint vincent and the grenadines": "VIN",
    "samoa": "SAM",
    "san marino": "SMR",
    "sao tome and principe": "STP",
    "saudi arabia": "KSA",
    "senegal": "SEN",
    "serbia": "SRB",
    "seychelles": "SEY",
    "sierra leone": "SLE",
    "singapore": "SGP",
    "slovakia": "SVK",
    "slovenia": "SLO",
    "solomon islands": "SOL",
    "somalia": "SOM",
    "south africa": "RSA",
    "south korea": "KOR",
    "south sudan": "SSD",
    "spain": "ESP",
    "sri lanka": "SRI",
    "sudan": "SUD",
    "suriname": "SUR",
    "sweden": "SWE",
    "switzerland": "SUI",
    "syria": "SYR",
    "tajikistan": "TJK",
    "tanzania": "TAN",
    "thailand": "THA",
    "timor-leste": "TLS",
    "togo": "TOG",
    "tonga": "TGA",
    "trinidad and tobago": "TTO",
    "tunisia": "TUN",
    "turkey": "TUR",
    "turkmenistan": "TKM",
    "tuvalu": "TUV",
    "uganda": "UGA",
    "ukraine": "UKR",
    "united arab emirates": "UAE",
    "united kingdom": "GBR",
    "united states": "USA",
    "uruguay": "URU",
    "uzbekistan": "UZB",
    "vanuatu": "VAN",
    "venezuela": "VEN",
    "vietnam": "VIE",
    "yemen": "YEM",
    "zambia": "ZAM",
    "zimbabwe": "ZIM"
}

class OfficialIOCFlagDownloader:
    def __init__(self, output_dir="ui/public/assets/flags"):
        self.output_dir = output_dir
        self.wikipedia_url = "https://en.wikipedia.org/wiki/List_of_sovereign_states"
        self.session = requests.Session()
        self.session.headers.update({
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36'
        })
        os.makedirs(output_dir, exist_ok=True)

    def scrape_sovereign_states(self):
        print(f"Scraping sovereign states from: {self.wikipedia_url}")
        try:
            response = self.session.get(self.wikipedia_url)
            response.raise_for_status()
        except requests.RequestException as e:
            print(f"Error fetching Wikipedia page: {e}")
            return []
        soup = BeautifulSoup(response.content, 'html.parser')
        countries = []
        # The main table is the first wikitable sortable
        table = soup.find('table', {'class': 'wikitable'})
        if not table:
            print("No main table found on the Wikipedia page.")
            return []
        rows = table.find_all('tr')[1:]  # Skip header
        for row in rows:
            cells = row.find_all('td')
            if len(cells) < 2:
                continue
            # The first cell contains the country name and flag
            country_cell = cells[0]
            # Extract country name
            country_name = country_cell.get_text().strip().lower()
            # Remove footnotes and parentheticals
            country_name = re.sub(r'\[.*?\]', '', country_name)
            country_name = re.sub(r'\(.*?\)', '', country_name)
            country_name = country_name.strip()
            # Only include if in IOC_CODES
            if country_name in IOC_CODES:
                countries.append((country_name, IOC_CODES[country_name], country_cell))
        return countries

    def extract_flag_url(self, country_cell) -> Optional[str]:
        img = country_cell.find('img')
        if img:
            flag_url = img.get('src')
            if flag_url:
                if flag_url.startswith('//'):
                    flag_url = 'https:' + flag_url
                elif flag_url.startswith('/'):
                    flag_url = 'https://en.wikipedia.org' + flag_url
                return flag_url
        return None

    def download_flag_image(self, ioc_code: str, flag_url: str, country_name: str) -> bool:
        try:
            output_path = os.path.join(self.output_dir, f"{ioc_code}.png")
            if os.path.exists(output_path):
                print(f"Flag already exists for {country_name} ({ioc_code})")
                return True
            print(f"Downloading flag for {country_name} ({ioc_code}) from {flag_url}")
            response = self.session.get(flag_url, timeout=30)
            response.raise_for_status()
            with open(output_path, 'wb') as f:
                f.write(response.content)
            print(f"✅ Downloaded: {ioc_code}.png ({country_name})")
            return True
        except Exception as e:
            print(f"❌ Error downloading flag for {country_name}: {e}")
            return False

    def download_all_official_ioc_flags(self):
        print("🌐 Starting official IOC flag download...")
        countries = self.scrape_sovereign_states()
        if not countries:
            print("No countries found on the Wikipedia page")
            return
        print(f"\nFound {len(countries)} IOC countries to download:")
        for name, ioc, _ in countries:
            print(f"  - {name} ({ioc})")
        successful_downloads = 0
        for country_name, ioc_code, country_cell in countries:
            flag_url = self.extract_flag_url(country_cell)
            if flag_url and self.download_flag_image(ioc_code, flag_url, country_name):
                successful_downloads += 1
            time.sleep(0.5)
        print(f"\n🎉 Download complete!")
        print(f"Successfully downloaded: {successful_downloads}/{len(countries)} flags")

def main():
    downloader = OfficialIOCFlagDownloader()
    downloader.download_all_official_ioc_flags()

if __name__ == "__main__":
    main() 