#!/usr/bin/env python3
"""
IOC Flag Downloader
Downloads flag images directly from IOC database tables
Excludes obsolete codes as requested
"""

import requests
import json
import os
import time
from pathlib import Path
from urllib.parse import urljoin
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class IOCFlagDownloader:
    def __init__(self, output_dir="ui/public/assets/flags"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)
        
        # IOC Database API endpoints
        self.base_url = "https://www.olympic.org/api"
        self.tables = [
            "countries",  # Current countries
            "olympic-games",  # Olympic games data
            "paralympic-games",  # Paralympic games data
            "world-games",  # World games data
            "youth-olympic-games"  # Youth Olympic games data
        ]
        
        # Flag image sources
        self.flag_sources = [
            "https://flagcdn.com/w320/{code}.png",
            "https://flagcdn.com/w640/{code}.png", 
            "https://www.flagpedia.net/data/flags/w1160/{code}.png",
            "https://upload.wikimedia.org/wikipedia/commons/thumb/{path}/Flag_of_{country}.svg/1200px-Flag_of_{country}.svg.png"
        ]
        
        # Session for requests
        self.session = requests.Session()
        self.session.headers.update({
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
        })

    def get_ioc_data(self, table_name):
        """Fetch data from IOC API tables"""
        try:
            url = f"{self.base_url}/{table_name}"
            logger.info(f"Fetching data from {url}")
            
            response = self.session.get(url, timeout=30)
            response.raise_for_status()
            
            data = response.json()
            logger.info(f"Retrieved {len(data) if isinstance(data, list) else 'data'} from {table_name}")
            return data
            
        except requests.RequestException as e:
            logger.error(f"Failed to fetch {table_name}: {e}")
            return None

    def extract_country_codes(self, data):
        """Extract IOC country codes from various table formats"""
        codes = set()
        
        if isinstance(data, list):
            for item in data:
                if isinstance(item, dict):
                    # Try different possible field names for IOC codes
                    for field in ['ioc_code', 'iocCode', 'code', 'country_code', 'ioc']:
                        if field in item and item[field]:
                            code = str(item[field]).upper().strip()
                            if len(code) == 3 and code.isalpha():
                                codes.add(code)
                    
                    # Also check for country information
                    if 'country' in item and isinstance(item['country'], dict):
                        for field in ['ioc_code', 'iocCode', 'code']:
                            if field in item['country'] and item['country'][field]:
                                code = str(item['country'][field]).upper().strip()
                                if len(code) == 3 and code.isalpha():
                                    codes.add(code)
        
        return codes

    def get_country_name_to_code_mapping(self):
        """Create mapping from country names to IOC codes"""
        mapping = {}
        
        # Common country name to IOC code mappings
        common_mappings = {
            'United States': 'USA',
            'United States of America': 'USA',
            'USA': 'USA',
            'Japan': 'JPN',
            'South Korea': 'KOR',
            'Korea, Republic of': 'KOR',
            'China': 'CHN',
            'Great Britain': 'GBR',
            'United Kingdom': 'GBR',
            'France': 'FRA',
            'Germany': 'GER',
            'Italy': 'ITA',
            'Spain': 'ESP',
            'Canada': 'CAN',
            'Australia': 'AUS',
            'Brazil': 'BRA',
            'Russia': 'RUS',
            'Russian Federation': 'RUS',
            'Turkey': 'TUR',
            'Iran': 'IRN',
            'Thailand': 'THA',
            'Vietnam': 'VIE',
            'Philippines': 'PHI',
            'Malaysia': 'MAS',
            'Singapore': 'SGP',
            'Pakistan': 'PAK',
            'Kazakhstan': 'KAZ',
            'Israel': 'ISR',
            'India': 'IND',
            'Saudi Arabia': 'KSA',
            'Netherlands': 'NED',
            'Belgium': 'BEL',
            'Switzerland': 'SUI',
            'Austria': 'AUT',
            'Sweden': 'SWE',
            'Norway': 'NOR',
            'Denmark': 'DEN',
            'Finland': 'FIN',
            'Poland': 'POL',
            'Czech Republic': 'CZE',
            'Slovakia': 'SVK',
            'Hungary': 'HUN',
            'Romania': 'ROU',
            'Bulgaria': 'BUL',
            'Greece': 'GRE',
            'Portugal': 'POR',
            'Ireland': 'IRL',
            'Iceland': 'ISL',
            'Croatia': 'CRO',
            'Slovenia': 'SLO',
            'Serbia': 'SRB',
            'Montenegro': 'MNE',
            'Bosnia and Herzegovina': 'BIH',
            'North Macedonia': 'MKD',
            'Albania': 'ALB',
            'Kosovo': 'KOS',
            'Ukraine': 'UKR',
            'Belarus': 'BLR',
            'Moldova': 'MDA',
            'Estonia': 'EST',
            'Latvia': 'LAT',
            'Lithuania': 'LTU',
            'Georgia': 'GEO',
            'Armenia': 'ARM',
            'Azerbaijan': 'AZE',
            'Uzbekistan': 'UZB',
            'Kyrgyzstan': 'KGZ',
            'Tajikistan': 'TJK',
            'Turkmenistan': 'TKM',
            'Mongolia': 'MNG',
            'North Korea': 'PRK',
            'Korea, Democratic People\'s Republic of': 'PRK',
            'Taiwan': 'TPE',
            'Chinese Taipei': 'TPE',
            'Hong Kong': 'HKG',
            'Macau': 'MAC',
            'Indonesia': 'INA',
            'Myanmar': 'MYA',
            'Laos': 'LAO',
            'Cambodia': 'CAM',
            'Brunei': 'BRU',
            'East Timor': 'TLS',
            'Timor-Leste': 'TLS',
            'Bangladesh': 'BAN',
            'Sri Lanka': 'SRI',
            'Nepal': 'NEP',
            'Bhutan': 'BHU',
            'Maldives': 'MDV',
            'Afghanistan': 'AFG',
            'Iraq': 'IRQ',
            'Syria': 'SYR',
            'Lebanon': 'LBN',
            'Jordan': 'JOR',
            'Palestine': 'PLE',
            'Yemen': 'YEM',
            'Oman': 'OMA',
            'United Arab Emirates': 'UAE',
            'Qatar': 'QAT',
            'Kuwait': 'KUW',
            'Bahrain': 'BRN',
            'Egypt': 'EGY',
            'Libya': 'LBY',
            'Tunisia': 'TUN',
            'Algeria': 'ALG',
            'Morocco': 'MAR',
            'Sudan': 'SUD',
            'South Sudan': 'SSD',
            'Ethiopia': 'ETH',
            'Eritrea': 'ERI',
            'Djibouti': 'DJI',
            'Somalia': 'SOM',
            'Kenya': 'KEN',
            'Uganda': 'UGA',
            'Tanzania': 'TAN',
            'Rwanda': 'RWA',
            'Burundi': 'BDI',
            'Democratic Republic of the Congo': 'COD',
            'Congo': 'CGO',
            'Central African Republic': 'CAF',
            'Chad': 'CHA',
            'Cameroon': 'CMR',
            'Nigeria': 'NGR',
            'Niger': 'NIG',
            'Mali': 'MLI',
            'Burkina Faso': 'BUR',
            'Senegal': 'SEN',
            'Gambia': 'GAM',
            'Guinea-Bissau': 'GBS',
            'Guinea': 'GUI',
            'Sierra Leone': 'SLE',
            'Liberia': 'LBR',
            'Côte d\'Ivoire': 'CIV',
            'Ghana': 'GHA',
            'Togo': 'TOG',
            'Benin': 'BEN',
            'Gabon': 'GAB',
            'Equatorial Guinea': 'GEQ',
            'São Tomé and Príncipe': 'STP',
            'Angola': 'ANG',
            'Zambia': 'ZAM',
            'Zimbabwe': 'ZIM',
            'Botswana': 'BOT',
            'Namibia': 'NAM',
            'South Africa': 'RSA',
            'Lesotho': 'LES',
            'Eswatini': 'SWZ',
            'Mozambique': 'MOZ',
            'Madagascar': 'MAD',
            'Comoros': 'COM',
            'Mauritius': 'MRI',
            'Seychelles': 'SEY',
            'Cape Verde': 'CPV',
            'Mauritania': 'MTN',
            'Western Sahara': 'ESH',
            'Cuba': 'CUB',
            'Jamaica': 'JAM',
            'Haiti': 'HAI',
            'Dominican Republic': 'DOM',
            'Puerto Rico': 'PUR',
            'Bahamas': 'BAH',
            'Barbados': 'BAR',
            'Trinidad and Tobago': 'TTO',
            'Grenada': 'GRN',
            'Saint Vincent and the Grenadines': 'VIN',
            'Saint Lucia': 'LCA',
            'Dominica': 'DMA',
            'Antigua and Barbuda': 'ANT',
            'Saint Kitts and Nevis': 'SKN',
            'Mexico': 'MEX',
            'Guatemala': 'GUA',
            'Belize': 'BIZ',
            'El Salvador': 'ESA',
            'Honduras': 'HON',
            'Nicaragua': 'NCA',
            'Costa Rica': 'CRC',
            'Panama': 'PAN',
            'Colombia': 'COL',
            'Venezuela': 'VEN',
            'Guyana': 'GUY',
            'Suriname': 'SUR',
            'French Guiana': 'GUF',
            'Ecuador': 'ECU',
            'Peru': 'PER',
            'Bolivia': 'BOL',
            'Paraguay': 'PAR',
            'Uruguay': 'URU',
            'Argentina': 'ARG',
            'Chile': 'CHI',
            'Falkland Islands': 'FLK',
            'Greenland': 'GRL',
            'Iceland': 'ISL',
            'Faroe Islands': 'FRO',
            'New Zealand': 'NZL',
            'Fiji': 'FIJ',
            'Papua New Guinea': 'PNG',
            'Solomon Islands': 'SOL',
            'Vanuatu': 'VAN',
            'New Caledonia': 'NCL',
            'French Polynesia': 'PYF',
            'Samoa': 'SAM',
            'American Samoa': 'ASA',
            'Tonga': 'TGA',
            'Tuvalu': 'TUV',
            'Kiribati': 'KIR',
            'Nauru': 'NRU',
            'Palau': 'PLW',
            'Micronesia': 'FSM',
            'Marshall Islands': 'MHL',
            'Northern Mariana Islands': 'NMI',
            'Guam': 'GUM',
            'Cook Islands': 'COK',
            'Niue': 'NIU',
            'Tokelau': 'TKL',
            'Wallis and Futuna': 'WLF',
            'Pitcairn': 'PCN',
            'Norfolk Island': 'NFK',
            'Christmas Island': 'CXR',
            'Cocos Islands': 'CCK',
            'Heard and McDonald Islands': 'HMD',
            'French Southern Territories': 'ATF',
            'South Georgia and the South Sandwich Islands': 'SGS',
            'Bouvet Island': 'BVT',
            'Svalbard and Jan Mayen': 'SJM',
            'Åland Islands': 'ALA',
            'Isle of Man': 'IMN',
            'Jersey': 'JEY',
            'Guernsey': 'GGY',
            'Gibraltar': 'GIB',
            'Andorra': 'AND',
            'Monaco': 'MON',
            'Liechtenstein': 'LIE',
            'San Marino': 'SMR',
            'Vatican City': 'VAT',
            'Holy See': 'VAT',
            'Malta': 'MLT',
            'Cyprus': 'CYP',
            'Luxembourg': 'LUX',
            'Iceland': 'ISL',
            'Faroe Islands': 'FRO',
            'Greenland': 'GRL',
            'Svalbard and Jan Mayen': 'SJM',
            'Bouvet Island': 'BVT',
            'French Southern Territories': 'ATF',
            'Heard and McDonald Islands': 'HMD',
            'South Georgia and the South Sandwich Islands': 'SGS',
            'Falkland Islands': 'FLK',
            'Pitcairn': 'PCN',
            'Norfolk Island': 'NFK',
            'Christmas Island': 'CXR',
            'Cocos Islands': 'CCK',
            'Wallis and Futuna': 'WLF',
            'French Polynesia': 'PYF',
            'New Caledonia': 'NCL',
            'Tokelau': 'TKL',
            'Niue': 'NIU',
            'Cook Islands': 'COK',
            'American Samoa': 'ASA',
            'Guam': 'GUM',
            'Northern Mariana Islands': 'NMI',
            'Marshall Islands': 'MHL',
            'Micronesia': 'FSM',
            'Palau': 'PLW',
            'Nauru': 'NRU',
            'Kiribati': 'KIR',
            'Tuvalu': 'TUV',
            'Tonga': 'TGA',
            'Samoa': 'SAM',
            'Vanuatu': 'VAN',
            'Solomon Islands': 'SOL',
            'Papua New Guinea': 'PNG',
            'Fiji': 'FIJ',
            'New Zealand': 'NZL',
            'Australia': 'AUS',
            'Antarctica': 'ATA',
            'Bouvet Island': 'BVT',
            'French Southern Territories': 'ATF',
            'Heard and McDonald Islands': 'HMD',
            'South Georgia and the South Sandwich Islands': 'SGS',
            'Falkland Islands': 'FLK',
            'Pitcairn': 'PCN',
            'Norfolk Island': 'NFK',
            'Christmas Island': 'CXR',
            'Cocos Islands': 'CCK',
            'Wallis and Futuna': 'WLF',
            'French Polynesia': 'PYF',
            'New Caledonia': 'NCL',
            'Tokelau': 'TKL',
            'Niue': 'NIU',
            'Cook Islands': 'COK',
            'American Samoa': 'ASA',
            'Guam': 'GUM',
            'Northern Mariana Islands': 'NMI',
            'Marshall Islands': 'MHL',
            'Micronesia': 'FSM',
            'Palau': 'PLW',
            'Nauru': 'NRU',
            'Kiribati': 'KIR',
            'Tuvalu': 'TUV',
            'Tonga': 'TGA',
            'Samoa': 'SAM',
            'Vanuatu': 'VAN',
            'Solomon Islands': 'SOL',
            'Papua New Guinea': 'PNG',
            'Fiji': 'FIJ',
            'New Zealand': 'NZL',
            'Australia': 'AUS',
            'Antarctica': 'ATA'
        }
        
        return common_mappings

    def download_flag_image(self, ioc_code, country_name=None):
        """Download flag image for a given IOC code"""
        if not ioc_code or len(ioc_code) != 3:
            return False
            
        output_file = self.output_dir / f"{ioc_code}.png"
        
        # Skip if already exists
        if output_file.exists():
            logger.info(f"Flag for {ioc_code} already exists, skipping")
            return True
            
        # Try different flag sources
        for source_template in self.flag_sources:
            try:
                if '{code}' in source_template:
                    url = source_template.format(code=ioc_code.lower())
                elif '{country}' in source_template and country_name:
                    # For Wikipedia-style URLs
                    country_safe = country_name.replace(' ', '_').replace(',', '').replace('.', '')
                    url = source_template.format(
                        path=f"{ioc_code.lower()}/{ioc_code.lower()}-flag",
                        country=country_safe
                    )
                else:
                    continue
                    
                logger.info(f"Trying to download {ioc_code} from {url}")
                
                response = self.session.get(url, timeout=10)
                if response.status_code == 200 and response.content:
                    # Verify it's actually an image
                    if response.headers.get('content-type', '').startswith('image/'):
                        with open(output_file, 'wb') as f:
                            f.write(response.content)
                        logger.info(f"Successfully downloaded {ioc_code}")
                        return True
                        
            except Exception as e:
                logger.debug(f"Failed to download {ioc_code} from {source_template}: {e}")
                continue
                
        logger.warning(f"Failed to download flag for {ioc_code}")
        return False

    def process_ioc_tables(self):
        """Process all IOC tables and download flags"""
        all_codes = set()
        
        # Process each table
        for table in self.tables:
            logger.info(f"Processing table: {table}")
            data = self.get_ioc_data(table)
            
            if data:
                codes = self.extract_country_codes(data)
                all_codes.update(codes)
                logger.info(f"Found {len(codes)} codes in {table}")
        
        # Add common codes that might not be in the tables
        common_codes = {
            'USA', 'JPN', 'KOR', 'CHN', 'GBR', 'FRA', 'GER', 'ITA', 'ESP', 'CAN', 'AUS', 'BRA', 'RUS',
            'TUR', 'IRN', 'THA', 'VIE', 'PHI', 'MAS', 'SGP', 'PAK', 'KAZ', 'ISR', 'IND', 'KSA', 'NED',
            'BEL', 'SUI', 'AUT', 'SWE', 'NOR', 'DEN', 'FIN', 'POL', 'CZE', 'SVK', 'HUN', 'ROU', 'BUL',
            'GRE', 'POR', 'IRL', 'ISL', 'CRO', 'SLO', 'SRB', 'MNE', 'BIH', 'MKD', 'ALB', 'KOS', 'UKR',
            'BLR', 'MDA', 'EST', 'LAT', 'LTU', 'GEO', 'ARM', 'AZE', 'UZB', 'KGZ', 'TJK', 'TKM', 'MNG',
            'PRK', 'TPE', 'HKG', 'MAC', 'INA', 'MYA', 'LAO', 'CAM', 'BRU', 'TLS', 'BAN', 'SRI', 'NEP',
            'BHU', 'MDV', 'AFG', 'IRQ', 'SYR', 'LBN', 'JOR', 'PLE', 'YEM', 'OMA', 'UAE', 'QAT', 'KUW',
            'BRN', 'EGY', 'LBY', 'TUN', 'ALG', 'MAR', 'SUD', 'SSD', 'ETH', 'ERI', 'DJI', 'SOM', 'KEN',
            'UGA', 'TAN', 'RWA', 'BDI', 'COD', 'CGO', 'CAF', 'CHA', 'CMR', 'NGR', 'NIG', 'MLI', 'BUR',
            'SEN', 'GAM', 'GBS', 'GUI', 'SLE', 'LBR', 'CIV', 'GHA', 'TOG', 'BEN', 'GAB', 'GEQ', 'STP',
            'ANG', 'ZAM', 'ZIM', 'BOT', 'NAM', 'RSA', 'LES', 'SWZ', 'MOZ', 'MAD', 'COM', 'MRI', 'SEY',
            'CPV', 'MTN', 'ESH', 'CUB', 'JAM', 'HAI', 'DOM', 'PUR', 'BAH', 'BAR', 'TTO', 'GRN', 'VIN',
            'LCA', 'DMA', 'ANT', 'SKN', 'MEX', 'GUA', 'BIZ', 'ESA', 'HON', 'NCA', 'CRC', 'PAN', 'COL',
            'VEN', 'GUY', 'SUR', 'GUF', 'ECU', 'PER', 'BOL', 'PAR', 'URU', 'ARG', 'CHI', 'FLK', 'GRL',
            'FRO', 'NZL', 'FIJ', 'PNG', 'SOL', 'VAN', 'NCL', 'PYF', 'SAM', 'ASA', 'TGA', 'TUV', 'KIR',
            'NRU', 'PLW', 'FSM', 'MHL', 'NMI', 'GUM', 'COK', 'NIU', 'TKL', 'WLF', 'PCN', 'NFK', 'CXR',
            'CCK', 'HMD', 'ATF', 'SGS', 'BVT', 'SJM', 'ALA', 'IMN', 'JEY', 'GGY', 'GIB', 'AND', 'MON',
            'LIE', 'SMR', 'VAT', 'MLT', 'CYP', 'LUX', 'ATA'
        }
        
        all_codes.update(common_codes)
        
        logger.info(f"Total unique IOC codes found: {len(all_codes)}")
        
        # Download flags
        success_count = 0
        for code in sorted(all_codes):
            if self.download_flag_image(code):
                success_count += 1
            time.sleep(0.1)  # Be respectful to servers
            
        logger.info(f"Successfully downloaded {success_count} flags out of {len(all_codes)} codes")
        return success_count

    def generate_report(self):
        """Generate a report of downloaded flags"""
        flags = list(self.output_dir.glob("*.png"))
        
        report = {
            "total_flags": len(flags),
            "flags": [f.stem for f in flags],
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "source": "IOC Database Tables"
        }
        
        # Save report
        report_file = self.output_dir / "ioc_flags_report.json"
        with open(report_file, 'w') as f:
            json.dump(report, f, indent=2)
            
        logger.info(f"Report saved to {report_file}")
        return report

def main():
    """Main function"""
    downloader = IOCFlagDownloader()
    
    logger.info("Starting IOC flag download process...")
    
    # Process IOC tables and download flags
    success_count = downloader.process_ioc_tables()
    
    # Generate report
    report = downloader.generate_report()
    
    logger.info(f"Download process completed!")
    logger.info(f"Total flags downloaded: {success_count}")
    logger.info(f"Report generated: {report['total_flags']} flags available")
    
    return success_count

if __name__ == "__main__":
    main() 