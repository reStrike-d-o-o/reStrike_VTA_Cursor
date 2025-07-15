#!/usr/bin/env python3
"""
Flag Recognition and Renaming Script for reStrike VTA

- Visual flag recognition using color analysis and pattern matching
- Optional deep learning classifier for world flags (if model is available)
- Support for all IOC-recognized countries
- Automatic file renaming with IOC codes
- Batch processing capabilities
- Confidence scoring for matches
- Fallback to manual identification
- Generates CSV and Markdown reports of results
"""

import os
import sys
import json
import cv2
import numpy as np
from pathlib import Path
from typing import Dict, List, Tuple, Optional
import argparse
from dataclasses import dataclass
import csv
import pandas as pd
from PIL import Image

# Optional: Deep learning model support
try:
    import torch
    import torchvision.transforms as transforms
    from torchvision import models
    DL_AVAILABLE = True
except ImportError:
    DL_AVAILABLE = False

@dataclass
class FlagPattern:
    country_code: str
    country_name: str
    colors: List[str]
    pattern_type: str
    dominant_colors: List[Tuple[int, int, int]]
    confidence: float

class FlagRecognizer:
    def __init__(self):
        self.ioc_codes = self._load_ioc_codes()
        self.flag_patterns = self._load_flag_patterns()
        self.color_names = self._load_color_names()
        self.dl_model = self._load_dl_model() if DL_AVAILABLE else None
        self.dl_labels = self._load_dl_labels() if DL_AVAILABLE else None

    def _load_ioc_codes(self) -> Dict[str, str]:
        """Load IOC country codes from Wikipedia data"""
        return {
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
    
    def _load_color_names(self) -> Dict[str, Tuple[int, int, int]]:
        """Load color name to RGB mapping"""
        return {
            'red': (255, 0, 0), 'blue': (0, 0, 255), 'white': (255, 255, 255),
            'green': (0, 255, 0), 'yellow': (255, 255, 0), 'black': (0, 0, 0),
            'orange': (255, 165, 0), 'purple': (128, 0, 128), 'pink': (255, 192, 203),
            'brown': (165, 42, 42), 'gray': (128, 128, 128), 'cyan': (0, 255, 255),
            'magenta': (255, 0, 255), 'lime': (0, 255, 0), 'navy': (0, 0, 128),
            'maroon': (128, 0, 0), 'olive': (128, 128, 0), 'teal': (0, 128, 128)
        }
    
    def _load_flag_patterns(self) -> Dict[str, FlagPattern]:
        """Load known flag patterns for recognition"""
        patterns = {}
        
        # Major countries with distinctive patterns
        patterns['USA'] = FlagPattern('USA', 'United States', 
                                    ['red', 'white', 'blue'], 'complex', 
                                    [(255, 0, 0), (255, 255, 255), (0, 0, 255)], 0.95)
        
        patterns['JPN'] = FlagPattern('JPN', 'Japan', 
                                    ['white', 'red'], 'circle', 
                                    [(255, 255, 255), (255, 0, 0)], 0.90)
        
        patterns['KOR'] = FlagPattern('KOR', 'South Korea', 
                                    ['white', 'red', 'blue'], 'complex', 
                                    [(255, 255, 255), (255, 0, 0), (0, 0, 255)], 0.85)
        
        patterns['CHN'] = FlagPattern('CHN', 'China', 
                                    ['red', 'yellow'], 'star', 
                                    [(255, 0, 0), (255, 255, 0)], 0.90)
        
        patterns['GBR'] = FlagPattern('GBR', 'Great Britain', 
                                    ['blue', 'white', 'red'], 'cross', 
                                    [(0, 0, 255), (255, 255, 255), (255, 0, 0)], 0.95)
        
        patterns['FRA'] = FlagPattern('FRA', 'France', 
                                    ['blue', 'white', 'red'], 'vertical', 
                                    [(0, 0, 255), (255, 255, 255), (255, 0, 0)], 0.95)
        
        patterns['GER'] = FlagPattern('GER', 'Germany', 
                                    ['black', 'red', 'yellow'], 'horizontal', 
                                    [(0, 0, 0), (255, 0, 0), (255, 255, 0)], 0.95)
        
        patterns['ITA'] = FlagPattern('ITA', 'Italy', 
                                    ['green', 'white', 'red'], 'vertical', 
                                    [(0, 255, 0), (255, 255, 255), (255, 0, 0)], 0.95)
        
        patterns['ESP'] = FlagPattern('ESP', 'Spain', 
                                    ['red', 'yellow', 'red'], 'horizontal', 
                                    [(255, 0, 0), (255, 255, 0), (255, 0, 0)], 0.90)
        
        patterns['CAN'] = FlagPattern('CAN', 'Canada', 
                                    ['red', 'white', 'red'], 'vertical', 
                                    [(255, 0, 0), (255, 255, 255), (255, 0, 0)], 0.85)
        
        return patterns

    def _load_dl_model(self):
        # Load a pre-trained model for flag classification if available
        # Placeholder: expects a model file 'flag_resnet.pth' and labels 'flag_labels.json'
        model_path = 'scripts/media/flag_resnet.pth'
        if not os.path.exists(model_path):
            return None
        model = models.resnet18()
        num_ftrs = model.fc.in_features
        model.fc = torch.nn.Linear(num_ftrs, 200)  # 200+ flags
        model.load_state_dict(torch.load(model_path, map_location='cpu'))
        model.eval()
        return model

    def _load_dl_labels(self):
        labels_path = 'scripts/media/flag_labels.json'
        if not os.path.exists(labels_path):
            return None
        with open(labels_path, 'r') as f:
            return json.load(f)

    def dl_predict(self, image_path: str) -> Optional[Tuple[str, float]]:
        if not self.dl_model or not self.dl_labels:
            return None
        img = Image.open(image_path).convert('RGB')
        transform = transforms.Compose([
            transforms.Resize((64, 64)),
            transforms.ToTensor(),
            transforms.Normalize([0.485, 0.456, 0.406], [0.229, 0.224, 0.225])
        ])
        input_tensor = transform(img).unsqueeze(0)
        with torch.no_grad():
            output = self.dl_model(input_tensor)
            prob = torch.nn.functional.softmax(output, dim=1)
            conf, pred = torch.max(prob, 1)
            label = self.dl_labels[str(pred.item())]
            return label, float(conf.item())

    def analyze_image_colors(self, image_path: str) -> List[Tuple[int, int, int]]:
        """Analyze dominant colors in an image"""
        try:
            # Read image
            img = cv2.imread(image_path)
            if img is None:
                return []
            
            # Convert to RGB
            img_rgb = cv2.cvtColor(img, cv2.COLOR_BGR2RGB)
            
            # Reshape to 2D array
            pixels = img_rgb.reshape(-1, 3)
            
            # Use k-means to find dominant colors
            from sklearn.cluster import KMeans
            kmeans = KMeans(n_clusters=5, random_state=42)
            kmeans.fit(pixels)
            
            # Get dominant colors
            colors = kmeans.cluster_centers_.astype(int)
            
            # Sort by frequency
            labels = kmeans.labels_
            color_counts = np.bincount(labels)
            sorted_indices = np.argsort(color_counts)[::-1]
            
            return [tuple(colors[i]) for i in sorted_indices[:3]]
            
        except Exception as e:
            print(f"Error analyzing colors in {image_path}: {e}")
            return []
    
    def calculate_color_similarity(self, color1: Tuple[int, int, int], 
                                 color2: Tuple[int, int, int]) -> float:
        """Calculate similarity between two colors using Euclidean distance"""
        return 1.0 - (np.sqrt(sum((a - b) ** 2 for a, b in zip(color1, color2))) / 441.67)
    
    def recognize_flag(self, image_path: str) -> Optional[FlagPattern]:
        """Recognize flag from image and return best match"""
        # Analyze image colors
        image_colors = self.analyze_image_colors(image_path)
        if not image_colors:
            return None
        
        best_match = None
        best_score = 0.0
        
        # Compare with known patterns
        for code, pattern in self.flag_patterns.items():
            score = 0.0
            color_matches = 0
            
            # Check color matches
            for pattern_color in pattern.dominant_colors:
                for image_color in image_colors:
                    similarity = self.calculate_color_similarity(pattern_color, image_color)
                    if similarity > 0.7:  # Threshold for color match
                        color_matches += 1
                        score += similarity
            
            # Normalize score
            if color_matches > 0:
                score = (score / len(pattern.dominant_colors)) * pattern.confidence
                
                if score > best_score:
                    best_score = score
                    best_match = pattern
        
        return best_match if best_score > 0.6 else None
    
    def rename_flag_file(self, old_path: str, country_code: str) -> str:
        """Rename flag file to IOC country code"""
        file_path = Path(old_path)
        new_name = f"{country_code}.png"
        new_path = file_path.parent / new_name
        
        # Handle conflicts
        counter = 1
        while new_path.exists():
            new_name = f"{country_code}_{counter}.png"
            new_path = file_path.parent / new_name
            counter += 1
        
        try:
            file_path.rename(new_path)
            return str(new_path)
        except Exception as e:
            print(f"Error renaming {old_path}: {e}")
            return old_path
    
    def generate_reports(self, results: List[Dict], outdir: str):
        csv_path = os.path.join(outdir, 'flag_recognition_results.csv')
        md_path = os.path.join(outdir, 'flag_recognition_results.md')
        df = pd.DataFrame(results)
        df.to_csv(csv_path, index=False)
        with open(md_path, 'w') as f:
            f.write('# Flag Recognition Results\n\n')
            f.write(df.to_markdown(index=False))
        print(f'CSV report: {csv_path}')
        print(f'Markdown report: {md_path}')

    def process_directory(self, directory: str, interactive: bool = True, report: bool = True) -> List[Dict]:
        """Process all flag images in a directory"""
        results = []
        directory_path = Path(directory)
        
        # Find all image files
        image_extensions = {'.png', '.jpg', '.jpeg', '.gif', '.bmp'}
        image_files = [f for f in directory_path.iterdir() 
                      if f.is_file() and f.suffix.lower() in image_extensions]
        
        print(f"Found {len(image_files)} image files to process")
        
        for image_file in image_files:
            print(f"\nProcessing: {image_file.name}")
            
            method = 'heuristic'
            confidence = 0.0
            country_code = ''
            country_name = ''
            new_path = ''

            # Try deep learning first
            if self.dl_model and self.dl_labels:
                dl_result = self.dl_predict(str(image_file))
                if dl_result:
                    country_code, confidence = dl_result
                    country_name = self.ioc_codes.get(country_code, '')
                    method = 'deep_learning'
            # If not confident, fallback to heuristic
            if not country_code or confidence < 0.7:
                pattern = self.recognize_flag(str(image_file))
                if pattern:
                    country_code = pattern.country_code
                    country_name = pattern.country_name
                    confidence = pattern.confidence
                    method = 'heuristic'
            if country_code:
                print(f"  Recognized as: {country_name} ({country_code}) [{method}, {confidence:.2f}]")
                if interactive:
                    response = input(f"  Rename to {country_code}.png? (y/n): ").lower()
                    if response == 'y':
                        new_path = self.rename_flag_file(str(image_file), country_code)
                        print(f"  Renamed to: {Path(new_path).name}")
                else:
                    new_path = self.rename_flag_file(str(image_file), country_code)
                    print(f"  Renamed to: {Path(new_path).name}")
            else:
                print(f"  Could not recognize flag")
                if interactive:
                    manual_code = input("  Enter IOC country code manually (or press Enter to skip): ").strip().upper()
                    if manual_code and manual_code in self.ioc_codes:
                        country_code = manual_code
                        country_name = self.ioc_codes[country_code]
                        new_path = self.rename_flag_file(str(image_file), country_code)
                        print(f"  Renamed to: {Path(new_path).name}")
            results.append({
                'original_filename': image_file.name,
                'new_filename': Path(new_path).name if new_path else '',
                'country_code': country_code,
                'country_name': country_name,
                'confidence': confidence,
                'method': method
            })
        if report:
            self.generate_reports(results, str(directory_path))
        return results

def main():
    parser = argparse.ArgumentParser(description='Flag Recognition and Renaming Tool')
    parser.add_argument('directory', help='Directory containing flag images')
    parser.add_argument('--batch', action='store_true', help='Batch mode (non-interactive)')
    parser.add_argument('--output', help='Output file for results JSON')
    
    args = parser.parse_args()
    
    if not os.path.exists(args.directory):
        print(f"Directory not found: {args.directory}")
        sys.exit(1)
    
    # Initialize recognizer
    recognizer = FlagRecognizer()
    
    # Process directory
    results = recognizer.process_directory(args.directory, not args.batch, report=True)
    
    # Save results
    if args.output:
        with open(args.output, 'w') as f:
            json.dump(results, f, indent=2)
        print(f"\nResults saved to: {args.output}")
    
    print(f"\nProcessing complete. {len(results)} files processed.")

if __name__ == "__main__":
    main() 