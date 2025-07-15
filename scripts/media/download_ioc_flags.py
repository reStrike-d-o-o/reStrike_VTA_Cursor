#!/usr/bin/env python3
"""
IOC Flag Downloader - Simplified Version
Downloads flag images from reliable sources using comprehensive IOC code list
"""

import requests
import json
import os
import time
from pathlib import Path
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class IOCFlagDownloader:
    def __init__(self, output_dir="ui/public/assets/flags"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)
        
        # Comprehensive IOC codes list (excluding obsolete codes)
        self.ioc_codes = {
            'AFG', 'ALB', 'ALG', 'AND', 'ANG', 'ANT', 'ARG', 'ARM', 'ARU', 'ASA', 'AUS', 'AUT', 'AZE',
            'BAH', 'BAN', 'BAR', 'BDI', 'BEL', 'BEN', 'BER', 'BHU', 'BIH', 'BIZ', 'BLR', 'BOL', 'BOT',
            'BRA', 'BRN', 'BRU', 'BUL', 'BUR', 'CAF', 'CAM', 'CAN', 'CHA', 'CHI', 'CHN', 'CIV', 'CMR',
            'COD', 'COG', 'COK', 'COL', 'COM', 'CPV', 'CRC', 'CRO', 'CUB', 'CYP', 'CZE', 'DEN', 'DJI',
            'DMA', 'DOM', 'EGY', 'ERI', 'ESA', 'ESP', 'EST', 'ETH', 'FIJ', 'FIN', 'FRA', 'FSM', 'GAB',
            'GAM', 'GBR', 'GBS', 'GEO', 'GEQ', 'GER', 'GHA', 'GIB', 'GRE', 'GRN', 'GUA', 'GUI', 'GUM',
            'GUY', 'HAI', 'HKG', 'HON', 'HUN', 'INA', 'IND', 'IRL', 'IRN', 'IRQ', 'ISL', 'ISR', 'ISV',
            'ITA', 'IVB', 'JAM', 'JOR', 'JPN', 'KAZ', 'KEN', 'KGZ', 'KIR', 'KOR', 'KOS', 'KSA', 'KUW',
            'LAO', 'LAT', 'LBN', 'LBR', 'LCA', 'LES', 'LIE', 'LTU', 'LUX', 'MAC', 'MAD', 'MAR', 'MAS',
            'MDA', 'MDV', 'MEX', 'MHL', 'MKD', 'MLI', 'MLT', 'MNG', 'MNE', 'MON', 'MOZ', 'MRI', 'MTN',
            'MYA', 'NAM', 'NCA', 'NED', 'NEP', 'NFK', 'NGR', 'NIG', 'NOR', 'NRU', 'NZL', 'OMA', 'PAK',
            'PAN', 'PAR', 'PER', 'PHI', 'PLE', 'PLW', 'PNG', 'POL', 'POR', 'PRK', 'PUR', 'QAT', 'ROU',
            'RSA', 'RUS', 'RWA', 'SAM', 'SEN', 'SEY', 'SGP', 'SKN', 'SLE', 'SLO', 'SMR', 'SOL', 'SOM',
            'SRB', 'SSD', 'STP', 'SUD', 'SUI', 'SUR', 'SVK', 'SWE', 'SWZ', 'SYR', 'TAN', 'TGA', 'THA',
            'TJK', 'TKM', 'TLS', 'TOG', 'TON', 'TPE', 'TTO', 'TUN', 'TUR', 'TUV', 'UAE', 'UGA', 'UKR',
            'URU', 'USA', 'UZB', 'VAN', 'VEN', 'VIN', 'VNM', 'YEM', 'ZAM', 'ZIM'
        }
        
        # Flag image sources (prioritized)
        self.flag_sources = [
            "https://flagcdn.com/w320/{code}.png",
            "https://flagcdn.com/w640/{code}.png",
            "https://www.flagpedia.net/data/flags/w1160/{code}.png"
        ]
        
        # Session for requests
        self.session = requests.Session()
        self.session.headers.update({
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
        })

    def download_flag_image(self, ioc_code):
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
                url = source_template.format(code=ioc_code.lower())
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

    def download_all_flags(self):
        """Download flags for all IOC codes"""
        success_count = 0
        
        logger.info(f"Starting download of {len(self.ioc_codes)} flags...")
        
        for code in sorted(self.ioc_codes):
            if self.download_flag_image(code):
                success_count += 1
            time.sleep(0.1)  # Be respectful to servers
            
        logger.info(f"Successfully downloaded {success_count} flags out of {len(self.ioc_codes)} codes")
        return success_count

    def generate_report(self):
        """Generate a report of downloaded flags"""
        flags = list(self.output_dir.glob("*.png"))
        
        report = {
            "total_flags": len(flags),
            "flags": [f.stem for f in flags],
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "source": "FlagCDN and Flagpedia"
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
    
    # Download all flags
    success_count = downloader.download_all_flags()
    
    # Generate report
    report = downloader.generate_report()
    
    logger.info(f"Download process completed!")
    logger.info(f"Total flags downloaded: {success_count}")
    logger.info(f"Report generated: {report['total_flags']} flags available")
    
    return success_count

if __name__ == "__main__":
    main() 