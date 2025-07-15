#!/usr/bin/env python3
"""
Comprehensive IOC Flag Database Generator

This script creates a complete database of IOC country codes and their flag patterns
covering all categories:
- Current NOCs (National Olympic Committees)
- Historic NOCs and teams
- Special codes for Olympics
- Special codes for Paralympics  
- Special codes for World Games

The script downloads flag images and creates a comprehensive pattern database
for enhanced flag recognition.
"""

import os
import sys
import json
import requests
import cv2
import numpy as np
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
from PIL import Image
import time
import re

@dataclass
class IOCEntry:
    """Represents an IOC country/territory entry"""
    code: str
    name: str
    category: str  # 'current', 'historic', 'olympics', 'paralympics', 'world_games'
    flag_url: Optional[str] = None
    dominant_colors: List[Tuple[int, int, int]] = None
    pattern_type: str = 'unknown'
    confidence: float = 0.0

class IOCFlagDatabase:
    def __init__(self):
        self.ioc_entries = {}
        self.flag_patterns = {}
        self.base_url = "https://en.wikipedia.org/wiki/List_of_IOC_country_codes"
        
    def load_comprehensive_ioc_data(self):
        """Load comprehensive IOC data from multiple sources"""
        
        # Current NOCs (National Olympic Committees)
        current_nocs = {
            'AFG': 'Afghanistan', 'ALB': 'Albania', 'ALG': 'Algeria', 'AND': 'Andorra',
            'ANG': 'Angola', 'ANT': 'Antigua and Barbuda', 'ARG': 'Argentina', 'ARM': 'Armenia',
            'ARU': 'Aruba', 'ASA': 'American Samoa', 'AUS': 'Australia', 'AUT': 'Austria',
            'AZE': 'Azerbaijan', 'BAH': 'Bahamas', 'BAN': 'Bangladesh', 'BAR': 'Barbados',
            'BDI': 'Burundi', 'BEL': 'Belgium', 'BEN': 'Benin', 'BER': 'Bermuda',
            'BHU': 'Bhutan', 'BIH': 'Bosnia and Herzegovina', 'BIZ': 'Belize', 'BLR': 'Belarus',
            'BOL': 'Bolivia', 'BOT': 'Botswana', 'BRA': 'Brazil', 'BRN': 'Brunei',
            'BRU': 'Brunei', 'BUL': 'Bulgaria', 'BUR': 'Burkina Faso', 'CAF': 'Central African Republic',
            'CAM': 'Cambodia', 'CAN': 'Canada', 'CAY': 'Cayman Islands', 'CGO': 'Republic of the Congo',
            'CHA': 'Chad', 'CHI': 'Chile', 'CHN': 'China', 'CIV': 'Ivory Coast',
            'CMR': 'Cameroon', 'COD': 'Democratic Republic of the Congo', 'COK': 'Cook Islands',
            'COL': 'Colombia', 'COM': 'Comoros', 'CPV': 'Cape Verde', 'CRC': 'Costa Rica',
            'CRO': 'Croatia', 'CUB': 'Cuba', 'CYP': 'Cyprus', 'CZE': 'Czech Republic',
            'DEN': 'Denmark', 'DJI': 'Djibouti', 'DMA': 'Dominica', 'DOM': 'Dominican Republic',
            'ECU': 'Ecuador', 'EGY': 'Egypt', 'ERI': 'Eritrea', 'ESA': 'El Salvador',
            'ESP': 'Spain', 'EST': 'Estonia', 'ETH': 'Ethiopia', 'FIJ': 'Fiji',
            'FIN': 'Finland', 'FRA': 'France', 'FSM': 'Micronesia', 'GAB': 'Gabon',
            'GAM': 'Gambia', 'GBR': 'Great Britain', 'GBS': 'Guinea-Bissau', 'GEO': 'Georgia',
            'GEQ': 'Equatorial Guinea', 'GER': 'Germany', 'GHA': 'Ghana', 'GRE': 'Greece',
            'GRN': 'Grenada', 'GUA': 'Guatemala', 'GUI': 'Guinea', 'GUM': 'Guam',
            'GUY': 'Guyana', 'HAI': 'Haiti', 'HKG': 'Hong Kong', 'HON': 'Honduras',
            'HUN': 'Hungary', 'INA': 'Indonesia', 'IND': 'India', 'IRI': 'Iran',
            'IRL': 'Ireland', 'IRQ': 'Iraq', 'ISL': 'Iceland', 'ISR': 'Israel',
            'ISV': 'Virgin Islands', 'ITA': 'Italy', 'IVB': 'British Virgin Islands',
            'JAM': 'Jamaica', 'JOR': 'Jordan', 'JPN': 'Japan', 'KAZ': 'Kazakhstan',
            'KEN': 'Kenya', 'KGZ': 'Kyrgyzstan', 'KIR': 'Kiribati', 'KOR': 'South Korea',
            'KSA': 'Saudi Arabia', 'KUW': 'Kuwait', 'LAO': 'Laos', 'LAT': 'Latvia',
            'LBA': 'Libya', 'LBR': 'Liberia', 'LCA': 'Saint Lucia', 'LES': 'Lesotho',
            'LIE': 'Liechtenstein', 'LTU': 'Lithuania', 'LUX': 'Luxembourg', 'MAD': 'Madagascar',
            'MAR': 'Morocco', 'MAS': 'Malaysia', 'MAW': 'Malawi', 'MDA': 'Moldova',
            'MDV': 'Maldives', 'MEX': 'Mexico', 'MHL': 'Marshall Islands', 'MKD': 'North Macedonia',
            'MLI': 'Mali', 'MLT': 'Malta', 'MNG': 'Mongolia', 'MOZ': 'Mozambique',
            'MRI': 'Mauritius', 'MTN': 'Mauritania', 'MYA': 'Myanmar', 'NAM': 'Namibia',
            'NCA': 'Nicaragua', 'NED': 'Netherlands', 'NEP': 'Nepal', 'NGR': 'Nigeria',
            'NIG': 'Niger', 'NOR': 'Norway', 'NRU': 'Nauru', 'NZL': 'New Zealand',
            'OMA': 'Oman', 'PAK': 'Pakistan', 'PAN': 'Panama', 'PAR': 'Paraguay',
            'PER': 'Peru', 'PHI': 'Philippines', 'PLE': 'Palestine', 'PLW': 'Palau',
            'PNG': 'Papua New Guinea', 'POL': 'Poland', 'POR': 'Portugal', 'PRK': 'North Korea',
            'QAT': 'Qatar', 'ROU': 'Romania', 'RSA': 'South Africa', 'RUS': 'Russia',
            'RWA': 'Rwanda', 'SAM': 'Samoa', 'SEN': 'Senegal', 'SEY': 'Seychelles',
            'SGP': 'Singapore', 'SKN': 'Saint Kitts and Nevis', 'SLE': 'Sierra Leone',
            'SLO': 'Slovenia', 'SMR': 'San Marino', 'SOL': 'Solomon Islands', 'SOM': 'Somalia',
            'SRB': 'Serbia', 'SRI': 'Sri Lanka', 'STP': 'São Tomé and Príncipe', 'SUD': 'Sudan',
            'SUI': 'Switzerland', 'SUR': 'Suriname', 'SVK': 'Slovakia', 'SWE': 'Sweden',
            'SWZ': 'Eswatini', 'SYR': 'Syria', 'TAN': 'Tanzania', 'TGA': 'Tonga',
            'THA': 'Thailand', 'TJK': 'Tajikistan', 'TKM': 'Turkmenistan', 'TLS': 'Timor-Leste',
            'TOG': 'Togo', 'TPE': 'Chinese Taipei', 'TTO': 'Trinidad and Tobago', 'TUN': 'Tunisia',
            'TUR': 'Turkey', 'TUV': 'Tuvalu', 'UAE': 'United Arab Emirates', 'UGA': 'Uganda',
            'UKR': 'Ukraine', 'URU': 'Uruguay', 'USA': 'United States', 'UZB': 'Uzbekistan',
            'VAN': 'Vanuatu', 'VEN': 'Venezuela', 'VIE': 'Vietnam', 'VIN': 'Saint Vincent and the Grenadines',
            'YEM': 'Yemen', 'ZAM': 'Zambia', 'ZIM': 'Zimbabwe'
        }
        
        # Historic NOCs and teams
        historic_nocs = {
            'ANZ': 'Australasia', 'BWI': 'British West Indies', 'EUA': 'United Team of Germany',
            'FRG': 'West Germany', 'GDR': 'East Germany', 'SCG': 'Serbia and Montenegro',
            'TCH': 'Czechoslovakia', 'URS': 'Soviet Union', 'YUG': 'Yugoslavia',
            'ZAI': 'Zaire', 'ZZX': 'Mixed Teams'
        }
        
        # Special codes for Olympics
        olympic_special = {
            'EUN': 'Unified Team', 'IOP': 'Independent Olympic Participants',
            'IOA': 'Independent Olympic Athletes', 'ROT': 'Refugee Olympic Team',
            'ROC': 'Russian Olympic Committee', 'OAR': 'Olympic Athletes from Russia'
        }
        
        # Special codes for Paralympics
        paralympic_special = {
            'IPA': 'Independent Paralympic Athletes', 'RPC': 'Russian Paralympic Committee',
            'RPT': 'Refugee Paralympic Team', 'NPA': 'Neutral Paralympic Athletes'
        }
        
        # Special codes for World Games
        world_games_special = {
            'WGA': 'World Games Athletes', 'WGT': 'World Games Team'
        }
        
        # Add all categories to the database
        for code, name in current_nocs.items():
            self.ioc_entries[code] = IOCEntry(code, name, 'current')
            
        for code, name in historic_nocs.items():
            self.ioc_entries[code] = IOCEntry(code, name, 'historic')
            
        for code, name in olympic_special.items():
            self.ioc_entries[code] = IOCEntry(code, name, 'olympics')
            
        for code, name in paralympic_special.items():
            self.ioc_entries[code] = IOCEntry(code, name, 'paralympics')
            
        for code, name in world_games_special.items():
            self.ioc_entries[code] = IOCEntry(code, name, 'world_games')
    
    def download_flag_image(self, country_code: str, country_name: str) -> Optional[str]:
        """Download flag image for a country"""
        try:
            # Try multiple flag image sources
            flag_urls = [
                f"https://flagcdn.com/w320/{country_code.lower()}.png",
                f"https://upload.wikimedia.org/wikipedia/commons/thumb/0/0d/Flag_of_{country_name.replace(' ', '_')}.svg/320px-Flag_of_{country_name.replace(' ', '_')}.svg.png",
                f"https://www.worldometers.info/img/flags/{country_code.lower()}-flag.gif"
            ]
            
            for url in flag_urls:
                try:
                    response = requests.get(url, timeout=10)
                    if response.status_code == 200:
                        # Save the image
                        flag_dir = Path("ui/public/assets/flags")
                        flag_dir.mkdir(parents=True, exist_ok=True)
                        
                        flag_path = flag_dir / f"{country_code}.png"
                        with open(flag_path, 'wb') as f:
                            f.write(response.content)
                        
                        print(f"Downloaded flag for {country_code} ({country_name})")
                        return str(flag_path)
                        
                except Exception as e:
                    continue
                    
        except Exception as e:
            print(f"Error downloading flag for {country_code}: {e}")
            
        return None
    
    def analyze_flag_colors(self, image_path: str) -> List[Tuple[int, int, int]]:
        """Analyze dominant colors in a flag image"""
        try:
            img = cv2.imread(image_path)
            if img is None:
                return []
            
            img_rgb = cv2.cvtColor(img, cv2.COLOR_BGR2RGB)
            pixels = img_rgb.reshape(-1, 3)
            
            from sklearn.cluster import KMeans
            kmeans = KMeans(n_clusters=5, random_state=42)
            kmeans.fit(pixels)
            
            colors = kmeans.cluster_centers_.astype(int)
            labels = kmeans.labels_
            color_counts = np.bincount(labels)
            sorted_indices = np.argsort(color_counts)[::-1]
            
            return [tuple(colors[i]) for i in sorted_indices[:3]]
            
        except Exception as e:
            print(f"Error analyzing colors in {image_path}: {e}")
            return []
    
    def determine_pattern_type(self, colors: List[Tuple[int, int, int]]) -> str:
        """Determine flag pattern type based on colors"""
        if len(colors) < 2:
            return 'solid'
        
        # Simple pattern detection based on color distribution
        # This is a basic heuristic - could be enhanced with more sophisticated analysis
        return 'horizontal'  # Default assumption
    
    def build_flag_patterns(self):
        """Build comprehensive flag pattern database"""
        print("Building comprehensive flag pattern database...")
        
        for code, entry in self.ioc_entries.items():
            # Try to download flag if not already present
            flag_path = f"ui/public/assets/flags/{code}.png"
            if not os.path.exists(flag_path):
                flag_path = self.download_flag_image(code, entry.name)
            
            if flag_path and os.path.exists(flag_path):
                # Analyze colors
                colors = self.analyze_flag_colors(flag_path)
                if colors:
                    pattern_type = self.determine_pattern_type(colors)
                    
                    self.flag_patterns[code] = {
                        'country_code': code,
                        'country_name': entry.name,
                        'category': entry.category,
                        'colors': colors,
                        'pattern_type': pattern_type,
                        'confidence': 0.85
                    }
                    
                    # Add some delay to be respectful to servers
                    time.sleep(0.1)
    
    def save_database(self, output_file: str = "scripts/media/ioc_flag_database.json"):
        """Save the comprehensive IOC flag database"""
        database = {
            'ioc_entries': {code: {
                'code': entry.code,
                'name': entry.name,
                'category': entry.category
            } for code, entry in self.ioc_entries.items()},
            'flag_patterns': self.flag_patterns,
            'metadata': {
                'total_entries': len(self.ioc_entries),
                'total_patterns': len(self.flag_patterns),
                'categories': {
                    'current': len([e for e in self.ioc_entries.values() if e.category == 'current']),
                    'historic': len([e for e in self.ioc_entries.values() if e.category == 'historic']),
                    'olympics': len([e for e in self.ioc_entries.values() if e.category == 'olympics']),
                    'paralympics': len([e for e in self.ioc_entries.values() if e.category == 'paralympics']),
                    'world_games': len([e for e in self.ioc_entries.values() if e.category == 'world_games'])
                }
            }
        }
        
        with open(output_file, 'w') as f:
            json.dump(database, f, indent=2)
        
        print(f"Database saved to {output_file}")
        print(f"Total IOC entries: {len(self.ioc_entries)}")
        print(f"Flag patterns created: {len(self.flag_patterns)}")
    
    def generate_summary_report(self):
        """Generate a summary report of the database"""
        report = []
        report.append("# Comprehensive IOC Flag Database Report\n")
        report.append(f"Generated on: {time.strftime('%Y-%m-%d %H:%M:%S')}\n")
        
        # Summary statistics
        report.append("## Summary Statistics\n")
        report.append(f"- Total IOC entries: {len(self.ioc_entries)}")
        report.append(f"- Flag patterns available: {len(self.flag_patterns)}")
        report.append(f"- Coverage: {len(self.flag_patterns)/len(self.ioc_entries)*100:.1f}%\n")
        
        # Category breakdown
        report.append("## Category Breakdown\n")
        categories = {}
        for entry in self.ioc_entries.values():
            categories[entry.category] = categories.get(entry.category, 0) + 1
        
        for category, count in categories.items():
            report.append(f"- {category.title()}: {count} entries")
        
        report.append("\n## Flag Patterns by Category\n")
        for category in ['current', 'historic', 'olympics', 'paralympics', 'world_games']:
            patterns = [p for p in self.flag_patterns.values() if p['category'] == category]
            report.append(f"- {category.title()}: {len(patterns)} patterns")
        
        # Save report
        with open("scripts/media/ioc_database_report.md", 'w') as f:
            f.write('\n'.join(report))
        
        print("Summary report generated: scripts/media/ioc_database_report.md")

def main():
    print("Creating comprehensive IOC flag database...")
    
    # Initialize database
    db = IOCFlagDatabase()
    
    # Load comprehensive IOC data
    print("Loading IOC data...")
    db.load_comprehensive_ioc_data()
    
    # Build flag patterns
    print("Building flag patterns...")
    db.build_flag_patterns()
    
    # Save database
    print("Saving database...")
    db.save_database()
    
    # Generate report
    print("Generating summary report...")
    db.generate_summary_report()
    
    print("\nComprehensive IOC flag database creation complete!")
    print("You can now use this database for enhanced flag recognition.")

if __name__ == "__main__":
    main() 