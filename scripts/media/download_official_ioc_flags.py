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
from typing import Dict, Optional, List, Tuple
import time
from urllib.parse import urljoin

class OfficialIOCFlagDownloader:
    def __init__(self, output_dir="ui/public/assets/flags"):
        self.output_dir = output_dir
        self.wikipedia_url = "https://en.wikipedia.org/wiki/List_of_sovereign_states"
        self.session = requests.Session()
        self.session.headers.update({
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36'
        })
        
        # Comprehensive IOC codes mapping
        self.ioc_codes = {
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
            "vatican city": "VAT",
            "venezuela": "VEN",
            "vietnam": "VIE",
            "yemen": "YEM",
            "zambia": "ZAM",
            "zimbabwe": "ZIM",
            
            # Current NPCs (National Paralympic Committees)
            "afghanistan": "AFG",  # Already included above
            "albania": "ALB",      # Already included above
            "algeria": "ALG",      # Already included above
            "andorra": "AND",      # Already included above
            "angola": "ANG",       # Already included above
            "antigua and barbuda": "ANT",  # Already included above
            "argentina": "ARG",    # Already included above
            "armenia": "ARM",      # Already included above
            "australia": "AUS",    # Already included above
            "austria": "AUT",      # Already included above
            "azerbaijan": "AZE",   # Already included above
            "bahamas": "BAH",      # Already included above
            "bahrain": "BRN",      # Already included above
            "bangladesh": "BAN",   # Already included above
            "barbados": "BAR",     # Already included above
            "belarus": "BLR",      # Already included above
            "belgium": "BEL",      # Already included above
            "belize": "BIZ",       # Already included above
            "benin": "BEN",        # Already included above
            "bhutan": "BHU",       # Already included above
            "bolivia": "BOL",      # Already included above
            "bosnia and herzegovina": "BIH",  # Already included above
            "botswana": "BOT",     # Already included above
            "brazil": "BRA",       # Already included above
            "brunei": "BRU",       # Already included above
            "bulgaria": "BUL",     # Already included above
            "burkina faso": "BUR", # Already included above
            "burundi": "BDI",      # Already included above
            "cabo verde": "CPV",   # Already included above
            "cambodia": "CAM",     # Already included above
            "cameroon": "CMR",     # Already included above
            "canada": "CAN",       # Already included above
            "central african republic": "CAF",  # Already included above
            "chad": "CHA",         # Already included above
            "chile": "CHI",        # Already included above
            "china": "CHN",        # Already included above
            "colombia": "COL",     # Already included above
            "comoros": "COM",      # Already included above
            "congo, democratic republic of the": "COD",  # Already included above
            "congo, republic of the": "CGO",    # Already included above
            "costa rica": "CRC",   # Already included above
            "croatia": "CRO",      # Already included above
            "cuba": "CUB",         # Already included above
            "cyprus": "CYP",       # Already included above
            "czechia": "CZE",      # Already included above
            "denmark": "DEN",      # Already included above
            "djibouti": "DJI",     # Already included above
            "dominica": "DMA",     # Already included above
            "dominican republic": "DOM",  # Already included above
            "ecuador": "ECU",      # Already included above
            "egypt": "EGY",        # Already included above
            "el salvador": "ESA",  # Already included above
            "equatorial guinea": "GEQ",  # Already included above
            "eritrea": "ERI",      # Already included above
            "estonia": "EST",      # Already included above
            "eswatini": "SWZ",     # Already included above
            "ethiopia": "ETH",     # Already included above
            "fiji": "FIJ",         # Already included above
            "finland": "FIN",      # Already included above
            "france": "FRA",       # Already included above
            "gabon": "GAB",        # Already included above
            "gambia": "GAM",       # Already included above
            "georgia": "GEO",      # Already included above
            "germany": "GER",      # Already included above
            "ghana": "GHA",        # Already included above
            "greece": "GRE",       # Already included above
            "grenada": "GRN",      # Already included above
            "guatemala": "GUA",    # Already included above
            "guinea": "GUI",       # Already included above
            "guinea-bissau": "GBS",  # Already included above
            "guyana": "GUY",       # Already included above
            "haiti": "HAI",        # Already included above
            "honduras": "HON",     # Already included above
            "hungary": "HUN",      # Already included above
            "iceland": "ISL",      # Already included above
            "india": "IND",        # Already included above
            "indonesia": "INA",    # Already included above
            "iran": "IRI",         # Already included above
            "iraq": "IRQ",         # Already included above
            "ireland": "IRL",      # Already included above
            "israel": "ISR",       # Already included above
            "italy": "ITA",        # Already included above
            "ivory coast": "CIV",  # Already included above
            "jamaica": "JAM",      # Already included above
            "japan": "JPN",        # Already included above
            "jordan": "JOR",       # Already included above
            "kazakhstan": "KAZ",   # Already included above
            "kenya": "KEN",        # Already included above
            "kiribati": "KIR",     # Already included above
            "kuwait": "KUW",       # Already included above
            "kyrgyzstan": "KGZ",   # Already included above
            "laos": "LAO",         # Already included above
            "latvia": "LAT",       # Already included above
            "lebanon": "LBN",      # Already included above
            "lesotho": "LES",      # Already included above
            "liberia": "LBR",      # Already included above
            "libya": "LBA",        # Already included above
            "liechtenstein": "LIE",  # Already included above
            "lithuania": "LTU",    # Already included above
            "luxembourg": "LUX",   # Already included above
            "madagascar": "MAD",   # Already included above
            "malawi": "MAW",       # Already included above
            "malaysia": "MAS",     # Already included above
            "maldives": "MDV",     # Already included above
            "mali": "MLI",         # Already included above
            "malta": "MLT",        # Already included above
            "marshall islands": "MHL",  # Already included above
            "mauritania": "MTN",   # Already included above
            "mauritius": "MRI",    # Already included above
            "mexico": "MEX",       # Already included above
            "micronesia": "FSM",   # Already included above
            "moldova": "MDA",      # Already included above
            "monaco": "MON",       # Already included above
            "mongolia": "MGL",     # Already included above
            "montenegro": "MNE",   # Already included above
            "morocco": "MAR",      # Already included above
            "mozambique": "MOZ",   # Already included above
            "myanmar": "MYA",      # Already included above
            "namibia": "NAM",      # Already included above
            "nauru": "NRU",        # Already included above
            "nepal": "NEP",        # Already included above
            "netherlands": "NED",  # Already included above
            "new zealand": "NZL",  # Already included above
            "nicaragua": "NCA",    # Already included above
            "niger": "NIG",        # Already included above
            "nigeria": "NGR",      # Already included above
            "north korea": "PRK",  # Already included above
            "north macedonia": "MKD",  # Already included above
            "norway": "NOR",       # Already included above
            "oman": "OMA",         # Already included above
            "pakistan": "PAK",     # Already included above
            "palau": "PLW",        # Already included above
            "palestine": "PLE",    # Already included above
            "panama": "PAN",       # Already included above
            "papua new guinea": "PNG",  # Already included above
            "paraguay": "PAR",     # Already included above
            "peru": "PER",         # Already included above
            "philippines": "PHI",  # Already included above
            "poland": "POL",       # Already included above
            "portugal": "POR",     # Already included above
            "qatar": "QAT",        # Already included above
            "romania": "ROU",      # Already included above
            "russia": "RUS",       # Already included above
            "rwanda": "RWA",       # Already included above
            "saint kitts and nevis": "SKN",  # Already included above
            "saint lucia": "LCA",  # Already included above
            "saint vincent and the grenadines": "VIN",  # Already included above
            "samoa": "SAM",        # Already included above
            "san marino": "SMR",   # Already included above
            "sao tome and principe": "STP",  # Already included above
            "saudi arabia": "KSA", # Already included above
            "senegal": "SEN",      # Already included above
            "serbia": "SRB",       # Already included above
            "seychelles": "SEY",   # Already included above
            "sierra leone": "SLE", # Already included above
            "singapore": "SGP",    # Already included above
            "slovakia": "SVK",     # Already included above
            "slovenia": "SLO",     # Already included above
            "solomon islands": "SOL",  # Already included above
            "somalia": "SOM",      # Already included above
            "south africa": "RSA", # Already included above
            "south korea": "KOR",  # Already included above
            "south sudan": "SSD",  # Already included above
            "spain": "ESP",        # Already included above
            "sri lanka": "SRI",    # Already included above
            "sudan": "SUD",        # Already included above
            "suriname": "SUR",     # Already included above
            "sweden": "SWE",       # Already included above
            "switzerland": "SUI",  # Already included above
            "syria": "SYR",        # Already included above
            "tajikistan": "TJK",   # Already included above
            "tanzania": "TAN",     # Already included above
            "thailand": "THA",     # Already included above
            "timor-leste": "TLS",  # Already included above
            "togo": "TOG",         # Already included above
            "tonga": "TGA",        # Already included above
            "trinidad and tobago": "TTO",  # Already included above
            "tunisia": "TUN",      # Already included above
            "turkey": "TUR",       # Already included above
            "turkmenistan": "TKM", # Already included above
            "tuvalu": "TUV",       # Already included above
            "uganda": "UGA",       # Already included above
            "ukraine": "UKR",      # Already included above
            "united arab emirates": "UAE",  # Already included above
            "united kingdom": "GBR",  # Already included above
            "united states": "USA",  # Already included above
            "uruguay": "URU",      # Already included above
            "uzbekistan": "UZB",   # Already included above
            "vanuatu": "VAN",      # Already included above
            "vatican city": "VAT", # Already included above
            "venezuela": "VEN",    # Already included above
            "vietnam": "VIE",      # Already included above
            "yemen": "YEM",        # Already included above
            "zambia": "ZAM",       # Already included above
            "zimbabwe": "ZIM",     # Already included above
            
            # Historic NOCs and teams
            "british west indies": "BWI",
            "czechoslovakia": "TCH",
            "east germany": "GDR",
            "soviet union": "URS",
            "unified team": "EUN",
            "west germany": "FRG",
            "yugoslavia": "YUG",
            "netherlands antilles": "AHO",
            "serbia and montenegro": "SCG",
            "virgin islands": "ISV",
            "british virgin islands": "IVB",
            "cayman islands": "CAY",
            "bermuda": "BER",
            "puerto rico": "PUR",
            "american samoa": "ASA",
            "cook islands": "COK",
            "fiji": "FIJ",  # Already included above
            "french polynesia": "PYF",
            "guam": "GUM",
            "kiribati": "KIR",  # Already included above
            "marshall islands": "MHL",  # Already included above
            "micronesia": "FSM",  # Already included above
            "nauru": "NRU",  # Already included above
            "new caledonia": "NCL",
            "new zealand": "NZL",  # Already included above
            "niue": "NIU",
            "norfolk island": "NFK",
            "northern mariana islands": "NMI",
            "palau": "PLW",  # Already included above
            "papua new guinea": "PNG",  # Already included above
            "samoa": "SAM",  # Already included above
            "solomon islands": "SOL",  # Already included above
            "tokelau": "TKL",
            "tonga": "TGA",  # Already included above
            "tuvalu": "TUV",  # Already included above
            "vanuatu": "VAN",  # Already included above
            "wallis and futuna": "WLF",
            
            # Special codes for Olympics
            "independent olympic participants": "IOP",
            "independent states team": "IST",
            "mixed teams": "ZZX",
            "olympic athletes from russia": "OAR",
            "refugee olympic team": "ROT",
            "russian olympic committee": "ROC",
            "unified team": "EUN",  # Already included above
            
            # Special codes for Paralympics
            "independent paralympic participants": "IPP",
            "mixed paralympic team": "MPT",
            "paralympic athletes from russia": "RPC",
            "refugee paralympic team": "RPT",
            
            # Special codes for World Games
            "world games": "WGA",
            "international world games association": "IWGA"
        }
        
        # Create output directory if it doesn't exist
        os.makedirs(output_dir, exist_ok=True)
        
        # Results tracking
        self.downloaded_flags = []
        self.missing_ioc_codes = []
        self.failed_downloads = []
        self.countries_without_ioc = []

    def scrape_sovereign_states(self) -> List[Tuple[str, str, Optional[str]]]:
        """
        Scrape the Wikipedia List of Sovereign States and extract country names and flag URLs.
        Returns list of (country_name, flag_url, ioc_code) tuples.
        """
        print(f"Scraping Wikipedia page: {self.wikipedia_url}")
        
        try:
            response = self.session.get(self.wikipedia_url)
            response.raise_for_status()
            soup = BeautifulSoup(response.content, 'html.parser')
            
            countries = []
            
            # Find the main table with sovereign states (sortable wikitable sticky-header)
            table = soup.find('table', {'class': 'sortable wikitable sticky-header'})
            
            if not table:
                print("❌ Main sovereign states table not found")
                return []
            
            print("✅ Found main sovereign states table")
            
            # Get all rows except header
            rows = table.find_all('tr')[1:]  # Skip header row
            
            for row in rows:
                cells = row.find_all(['td', 'th'])
                if len(cells) >= 1:
                    # Extract country name from the first cell
                    country_cell = cells[0]
                    country_name = self.extract_country_name(country_cell)
                    
                    if country_name:
                        # Extract flag URL
                        flag_url = self.extract_flag_url(country_cell)
                        
                        # Check if country has IOC code
                        ioc_code = self.ioc_codes.get(country_name.lower())
                        
                        countries.append((country_name, flag_url, ioc_code))
                        
                        if ioc_code:
                            print(f"Found: {country_name} -> {ioc_code}")
                        else:
                            print(f"No IOC code for: {country_name}")
            
            return countries
            
        except Exception as e:
            print(f"Error scraping Wikipedia: {e}")
            return []

    def extract_country_name(self, cell) -> Optional[str]:
        """Extract clean country name from table cell"""
        # Look for the country name in bold tags first
        bold_tag = cell.find('b')
        if bold_tag:
            # Get the text from the bold tag (country name)
            text = bold_tag.get_text().strip()
        else:
            # Fallback to full cell text
            text = cell.get_text().strip()
        
        # Clean up common Wikipedia formatting
        text = re.sub(r'\[.*?\]', '', text)  # Remove citations
        text = re.sub(r'\(.*?\)', '', text)  # Remove parentheses
        text = text.strip()
        
        # Remove "–" and everything after it (formal names)
        if '–' in text:
            text = text.split('–')[0].strip()
        
        # Handle special cases and variations
        name_mappings = {
            "cabo verde": "cabo verde",
            "czech republic": "czechia",
            "east timor": "timor-leste",
            "federated states of micronesia": "micronesia",
            "holy see": "vatican city",
            "myanmar": "myanmar",
            "north macedonia": "north macedonia",
            "republic of the congo": "congo",
            "republic of the sudan": "sudan",
            "russian federation": "russia",
            "saint vincent and the grenadines": "saint vincent and the grenadines",
            "sao tome and principe": "sao tome and principe",
            "swaziland": "eswatini",
            "the bahamas": "bahamas",
            "the gambia": "gambia",
            "the netherlands": "netherlands",
            "the philippines": "philippines",
            "united kingdom of great britain and northern ireland": "united kingdom",
            "united states of america": "united states",
            "viet nam": "vietnam",
            # Fix the 7 missing flags
            "bahamas, the": "bahamas",
            "cape verde": "cabo verde",
            "congo, democratic republic of the": "congo, democratic republic of the",
            "congo, republic of the": "congo, republic of the",
            "gambia, the": "gambia",
            "micronesia, federated states of": "micronesia",
            "são tomé and príncipe": "sao tome and principe"
        }
        
        # Apply name mappings
        for wiki_name, standard_name in name_mappings.items():
            if text.lower() == wiki_name:
                return standard_name
        
        return text if text else None

    def extract_flag_url(self, country_cell) -> Optional[str]:
        """Extract flag image URL from country cell"""
        # Look for flag image in the cell
        flag_img = country_cell.find('img')
        if flag_img:
            flag_url = flag_img.get('src')
            if flag_url:
                # Convert to full URL if it's relative
                if flag_url.startswith('//'):
                    flag_url = 'https:' + flag_url
                elif flag_url.startswith('/'):
                    flag_url = 'https://en.wikipedia.org' + flag_url
                return flag_url
        return None

    def download_flag_image(self, ioc_code: str, flag_url: str, country_name: str) -> bool:
        """Download flag image and save as IOC code"""
        try:
            print(f"Downloading {country_name} ({ioc_code}) from {flag_url}")
            
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

    def download_additional_ioc_flags(self):
        """Download flags for additional IOC codes not on sovereign states list"""
        print("\n🔍 Downloading additional IOC flags...")
        
        # Additional IOC codes that need flags (not on sovereign states list)
        additional_codes = {
            # Historic NOCs and teams
            "british west indies": "BWI",
            "czechoslovakia": "TCH", 
            "east germany": "GDR",
            "soviet union": "URS",
            "unified team": "EUN",
            "west germany": "FRG",
            "yugoslavia": "YUG",
            "netherlands antilles": "AHO",
            "serbia and montenegro": "SCG",
            "virgin islands": "ISV",
            "british virgin islands": "IVB",
            "cayman islands": "CAY",
            "bermuda": "BER",
            "puerto rico": "PUR",
            "american samoa": "ASA",
            "cook islands": "COK",
            "french polynesia": "PYF",
            "guam": "GUM",
            "new caledonia": "NCL",
            "niue": "NIU",
            "norfolk island": "NFK",
            "northern mariana islands": "NMI",
            "tokelau": "TKL",
            "wallis and futuna": "WLF",
            
            # Special codes for Olympics
            "independent olympic participants": "IOP",
            "independent states team": "IST",
            "mixed teams": "ZZX",
            "olympic athletes from russia": "OAR",
            "refugee olympic team": "ROT",
            "russian olympic committee": "ROC",
            
            # Special codes for Paralympics
            "independent paralympic participants": "IPP",
            "mixed paralympic team": "MPT",
            "paralympic athletes from russia": "RPC",
            "refugee paralympic team": "RPT",
            
            # Special codes for World Games
            "world games": "WGA",
            "international world games association": "IWGA"
        }
        
        # Flag URLs for additional codes (from reliable sources)
        additional_flag_urls = {
            "BWI": "https://upload.wikimedia.org/wikipedia/commons/thumb/8/8a/Flag_of_the_West_Indies_Federation.svg/40px-Flag_of_the_West_Indies_Federation.svg.png",
            "TCH": "https://upload.wikimedia.org/wikipedia/commons/thumb/c/cb/Flag_of_Czechoslovakia.svg/40px-Flag_of_Czechoslovakia.svg.png",
            "GDR": "https://upload.wikimedia.org/wikipedia/commons/thumb/c/c3/Flag_of_East_Germany.svg/40px-Flag_of_East_Germany.svg.png",
            "URS": "https://upload.wikimedia.org/wikipedia/commons/thumb/a/a9/Flag_of_the_Soviet_Union.svg/40px-Flag_of_the_Soviet_Union.svg.png",
            "EUN": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/0e/Flag_of_the_Olympic_Movement.svg/40px-Flag_of_the_Olympic_Movement.svg.png",
            "FRG": "https://upload.wikimedia.org/wikipedia/commons/thumb/b/ba/Flag_of_Germany.svg/40px-Flag_of_Germany.svg.png",
            "YUG": "https://upload.wikimedia.org/wikipedia/commons/thumb/6/60/Flag_of_Yugoslavia_%281946-1992%29.svg/40px-Flag_of_Yugoslavia_%281946-1992%29.svg.png",
            "AHO": "https://upload.wikimedia.org/wikipedia/commons/thumb/2/20/Flag_of_the_Netherlands_Antilles.svg/40px-Flag_of_the_Netherlands_Antilles.svg.png",
            "SCG": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/00/Flag_of_Serbia_and_Montenegro.svg/40px-Flag_of_Serbia_and_Montenegro.svg.png",
            "ISV": "https://upload.wikimedia.org/wikipedia/commons/thumb/f/f8/Flag_of_the_United_States_Virgin_Islands.svg/40px-Flag_of_the_United_States_Virgin_Islands.svg.png",
            "IVB": "https://upload.wikimedia.org/wikipedia/commons/thumb/4/42/Flag_of_the_British_Virgin_Islands.svg/40px-Flag_of_the_British_Virgin_Islands.svg.png",
            "CAY": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/0f/Flag_of_the_Cayman_Islands.svg/40px-Flag_of_the_Cayman_Islands.svg.png",
            "BER": "https://upload.wikimedia.org/wikipedia/commons/thumb/b/bf/Flag_of_Bermuda.svg/40px-Flag_of_Bermuda.svg.png",
            "PUR": "https://upload.wikimedia.org/wikipedia/commons/thumb/2/28/Flag_of_Puerto_Rico.svg/40px-Flag_of_Puerto_Rico.svg.png",
            "ASA": "https://upload.wikimedia.org/wikipedia/commons/thumb/8/87/Flag_of_American_Samoa.svg/40px-Flag_of_American_Samoa.svg.png",
            "COK": "https://upload.wikimedia.org/wikipedia/commons/thumb/3/35/Flag_of_the_Cook_Islands.svg/40px-Flag_of_the_Cook_Islands.svg.png",
            "PYF": "https://upload.wikimedia.org/wikipedia/commons/thumb/d/db/Flag_of_French_Polynesia.svg/40px-Flag_of_French_Polynesia.svg.png",
            "GUM": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/07/Flag_of_Guam.svg/40px-Flag_of_Guam.svg.png",
            "NCL": "https://upload.wikimedia.org/wikipedia/commons/thumb/3/30/Flag_of_New_Caledonia.svg/40px-Flag_of_New_Caledonia.svg.png",
            "NIU": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/01/Flag_of_Niue.svg/40px-Flag_of_Niue.svg.png",
            "NFK": "https://upload.wikimedia.org/wikipedia/commons/thumb/4/48/Flag_of_Norfolk_Island.svg/40px-Flag_of_Norfolk_Island.svg.png",
            "NMI": "https://upload.wikimedia.org/wikipedia/commons/thumb/e/e0/Flag_of_the_Northern_Mariana_Islands.svg/40px-Flag_of_the_Northern_Mariana_Islands.svg.png",
            "TKL": "https://upload.wikimedia.org/wikipedia/commons/thumb/8/8e/Flag_of_Tokelau.svg/40px-Flag_of_Tokelau.svg.png",
            "WLF": "https://upload.wikimedia.org/wikipedia/commons/thumb/4/4a/Flag_of_Wallis_and_Futuna.svg/40px-Flag_of_Wallis_and_Futuna.svg.png",
            
            # Special Olympic codes
            "IOP": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/0e/Flag_of_the_Olympic_Movement.svg/40px-Flag_of_the_Olympic_Movement.svg.png",
            "IST": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/0e/Flag_of_the_Olympic_Movement.svg/40px-Flag_of_the_Olympic_Movement.svg.png",
            "ZZX": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/0e/Flag_of_the_Olympic_Movement.svg/40px-Flag_of_the_Olympic_Movement.svg.png",
            "OAR": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/0e/Flag_of_the_Olympic_Movement.svg/40px-Flag_of_the_Olympic_Movement.svg.png",
            "ROT": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/0e/Flag_of_the_Olympic_Movement.svg/40px-Flag_of_the_Olympic_Movement.svg.png",
            "ROC": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/0e/Flag_of_the_Olympic_Movement.svg/40px-Flag_of_the_Olympic_Movement.svg.png",
            
            # Special Paralympic codes
            "IPP": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/0e/Flag_of_the_Olympic_Movement.svg/40px-Flag_of_the_Olympic_Movement.svg.png",
            "MPT": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/0e/Flag_of_the_Olympic_Movement.svg/40px-Flag_of_the_Olympic_Movement.svg.png",
            "RPC": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/0e/Flag_of_the_Olympic_Movement.svg/40px-Flag_of_the_Olympic_Movement.svg.png",
            "RPT": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/0e/Flag_of_the_Olympic_Movement.svg/40px-Flag_of_the_Olympic_Movement.svg.png",
            
            # Special World Games codes
            "WGA": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/0e/Flag_of_the_Olympic_Movement.svg/40px-Flag_of_the_Olympic_Movement.svg.png",
            "IWGA": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/0e/Flag_of_the_Olympic_Movement.svg/40px-Flag_of_the_Olympic_Movement.svg.png"
        }
        
        for country_name, ioc_code in additional_codes.items():
            if ioc_code in additional_flag_urls:
                flag_url = additional_flag_urls[ioc_code]
                success = self.download_flag_image(ioc_code, flag_url, country_name)
                if success:
                    self.downloaded_flags.append({
                        'country': country_name,
                        'ioc_code': ioc_code,
                        'flag_url': flag_url,
                        'type': 'additional'
                    })
                else:
                    self.failed_downloads.append({
                        'country': country_name,
                        'ioc_code': ioc_code,
                        'flag_url': flag_url,
                        'reason': 'Download failed',
                        'type': 'additional'
                    })
            else:
                self.missing_ioc_codes.append({
                    'country': country_name,
                    'ioc_code': ioc_code,
                    'reason': 'No flag URL available',
                    'type': 'additional'
                })

    def download_all_official_ioc_flags(self):
        """Main method to download all official IOC flags"""
        print("🚀 Starting Official IOC Flag Download")
        print("=" * 50)
        
        # Scrape countries from Wikipedia
        countries = self.scrape_sovereign_states()
        
        if not countries:
            print("❌ No countries found on Wikipedia page")
            return
        
        print(f"\n📊 Found {len(countries)} countries on Wikipedia")
        
        # Process each country
        for country_name, flag_url, ioc_code in countries:
            if ioc_code:
                if flag_url:
                    success = self.download_flag_image(ioc_code, flag_url, country_name)
                    if success:
                        self.downloaded_flags.append({
                            'country': country_name,
                            'ioc_code': ioc_code,
                            'flag_url': flag_url,
                            'type': 'sovereign_state'
                        })
                    else:
                        self.failed_downloads.append({
                            'country': country_name,
                            'ioc_code': ioc_code,
                            'flag_url': flag_url,
                            'reason': 'Download failed',
                            'type': 'sovereign_state'
                        })
                else:
                    self.missing_ioc_codes.append({
                        'country': country_name,
                        'ioc_code': ioc_code,
                        'reason': 'No flag URL found on Wikipedia'
                    })
            else:
                self.countries_without_ioc.append({
                    'country': country_name,
                    'flag_url': flag_url,
                    'reason': 'No IOC code found'
                })
        
        # Download additional IOC flags (not on sovereign states list)
        self.download_additional_ioc_flags()
        
        # Generate comprehensive report
        self.generate_detailed_report()

    def generate_detailed_report(self):
        """Generate comprehensive download report"""
        print("\n" + "=" * 50)
        print("📋 OFFICIAL IOC FLAG DOWNLOAD REPORT")
        print("=" * 50)
        
        # Summary statistics
        total_countries = len(self.downloaded_flags) + len(self.failed_downloads) + len(self.missing_ioc_codes)
        total_with_ioc = len(self.downloaded_flags) + len(self.failed_downloads) + len(self.missing_ioc_codes)
        
        print(f"\n📊 SUMMARY:")
        print(f"   Total countries found: {total_countries}")
        print(f"   Countries with IOC codes: {total_with_ioc}")
        print(f"   ✅ Successfully downloaded: {len(self.downloaded_flags)}")
        print(f"   ❌ Failed downloads: {len(self.failed_downloads)}")
        print(f"   🔍 Missing flag URLs: {len(self.missing_ioc_codes)}")
        print(f"   🌍 Countries without IOC codes: {len(self.countries_without_ioc)}")
        
        # Successfully downloaded flags
        if self.downloaded_flags:
            print(f"\n✅ SUCCESSFULLY DOWNLOADED ({len(self.downloaded_flags)} flags):")
            for flag in self.downloaded_flags:
                print(f"   {flag['ioc_code']} - {flag['country']}")
        
        # Failed downloads
        if self.failed_downloads:
            print(f"\n❌ FAILED DOWNLOADS ({len(self.failed_downloads)} flags):")
            for flag in self.failed_downloads:
                print(f"   {flag['ioc_code']} - {flag['country']} ({flag['reason']})")
        
        # Missing flag URLs (have IOC but no flag)
        if self.missing_ioc_codes:
            print(f"\n🔍 MISSING FLAG URLS ({len(self.missing_ioc_codes)} flags):")
            print("   These countries have IOC codes but no flag URLs were found on Wikipedia.")
            print("   You may need to find alternative sources for these flags:")
            for flag in self.missing_ioc_codes:
                print(f"   {flag['ioc_code']} - {flag['country']}")
        
        # Countries without IOC codes
        if self.countries_without_ioc:
            print(f"\n🌍 COUNTRIES WITHOUT IOC CODES ({len(self.countries_without_ioc)} countries):")
            print("   These countries are on the sovereign states list but don't have IOC codes:")
            for country in self.countries_without_ioc:
                print(f"   {country['country']}")
        
        # Save detailed report to file
        report_data = {
            'summary': {
                'total_countries': total_countries,
                'countries_with_ioc': total_with_ioc,
                'successfully_downloaded': len(self.downloaded_flags),
                'failed_downloads': len(self.failed_downloads),
                'missing_flag_urls': len(self.missing_ioc_codes),
                'countries_without_ioc': len(self.countries_without_ioc)
            },
            'downloaded_flags': self.downloaded_flags,
            'failed_downloads': self.failed_downloads,
            'missing_ioc_codes': self.missing_ioc_codes,
            'countries_without_ioc': self.countries_without_ioc
        }
        
        report_path = os.path.join(self.output_dir, "official_ioc_flags_report.json")
        with open(report_path, 'w', encoding='utf-8') as f:
            json.dump(report_data, f, indent=2, ensure_ascii=False)
        
        # Generate markdown report
        markdown_report = self.generate_markdown_report(report_data)
        markdown_path = os.path.join(self.output_dir, "OFFICIAL_IOC_FLAGS_DOWNLOAD_REPORT.md")
        with open(markdown_path, 'w', encoding='utf-8') as f:
            f.write(markdown_report)
        
        print(f"\n📄 Reports saved:")
        print(f"   JSON: {report_path}")
        print(f"   Markdown: {markdown_path}")
        
        # Special note for missing flags
        if self.missing_ioc_codes:
            print(f"\n🔗 NEXT STEPS:")
            print(f"   {len(self.missing_ioc_codes)} flags need alternative sources.")
            print(f"   Please provide the source URL for these missing flags.")

    def generate_markdown_report(self, report_data):
        """Generate markdown format report"""
        md = """# Official IOC Flags Download Report

## Summary

- **Total countries found**: {total_countries}
- **Countries with IOC codes**: {countries_with_ioc}
- **Successfully downloaded**: {successfully_downloaded}
- **Failed downloads**: {failed_downloads}
- **Missing flag URLs**: {missing_flag_urls}
- **Countries without IOC codes**: {countries_without_ioc}

## Successfully Downloaded Flags

""".format(**report_data['summary'])
        
        if report_data['downloaded_flags']:
            md += "| IOC Code | Country | Flag URL |\n"
            md += "|----------|---------|----------|\n"
            for flag in report_data['downloaded_flags']:
                md += f"| {flag['ioc_code']} | {flag['country']} | {flag['flag_url']} |\n"
        else:
            md += "No flags were successfully downloaded.\n"
        
        md += "\n## Failed Downloads\n\n"
        if report_data['failed_downloads']:
            md += "| IOC Code | Country | Reason |\n"
            md += "|----------|---------|--------|\n"
            for flag in report_data['failed_downloads']:
                md += f"| {flag['ioc_code']} | {flag['country']} | {flag['reason']} |\n"
        else:
            md += "No failed downloads.\n"
        
        md += "\n## Missing Flag URLs (Need Alternative Sources)\n\n"
        if report_data['missing_ioc_codes']:
            md += "These countries have IOC codes but no flag URLs were found on Wikipedia:\n\n"
            md += "| IOC Code | Country |\n"
            md += "|----------|---------|\n"
            for flag in report_data['missing_ioc_codes']:
                md += f"| {flag['ioc_code']} | {flag['country']} |\n"
        else:
            md += "No missing flag URLs.\n"
        
        md += "\n## Countries Without IOC Codes\n\n"
        if report_data['countries_without_ioc']:
            md += "These countries are on the sovereign states list but don't have IOC codes:\n\n"
            for country in report_data['countries_without_ioc']:
                md += f"- {country['country']}\n"
        else:
            md += "All countries have IOC codes.\n"
        
        return md

def main():
    """Main execution function"""
    downloader = OfficialIOCFlagDownloader()
    downloader.download_all_official_ioc_flags()

if __name__ == "__main__":
    main() 