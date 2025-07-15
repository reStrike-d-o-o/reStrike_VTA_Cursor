#!/usr/bin/env python3
"""
Simple Enhanced Flag Recognition for reStrike VTA

This script attempts to recognize the remaining unrecognized flags
using an expanded database and enhanced color analysis.
"""

import os
import sys
import cv2
import numpy as np
from pathlib import Path
import argparse
from sklearn.cluster import KMeans

# Comprehensive IOC database (expanded)
IOC_DATABASE = {
    # Major countries with distinctive patterns
    'USA': {'name': 'United States', 'colors': [(255, 0, 0), (255, 255, 255), (0, 0, 255)], 'confidence': 0.95},
    'JPN': {'name': 'Japan', 'colors': [(255, 255, 255), (255, 0, 0)], 'confidence': 0.90},
    'KOR': {'name': 'South Korea', 'colors': [(255, 255, 255), (255, 0, 0), (0, 0, 255)], 'confidence': 0.85},
    'CHN': {'name': 'China', 'colors': [(255, 0, 0), (255, 255, 0)], 'confidence': 0.90},
    'GBR': {'name': 'Great Britain', 'colors': [(0, 0, 255), (255, 255, 255), (255, 0, 0)], 'confidence': 0.95},
    'FRA': {'name': 'France', 'colors': [(0, 0, 255), (255, 255, 255), (255, 0, 0)], 'confidence': 0.95},
    'GER': {'name': 'Germany', 'colors': [(0, 0, 0), (255, 0, 0), (255, 255, 0)], 'confidence': 0.95},
    'ITA': {'name': 'Italy', 'colors': [(0, 255, 0), (255, 255, 255), (255, 0, 0)], 'confidence': 0.95},
    'ESP': {'name': 'Spain', 'colors': [(255, 0, 0), (255, 255, 0), (255, 0, 0)], 'confidence': 0.90},
    'CAN': {'name': 'Canada', 'colors': [(255, 0, 0), (255, 255, 255), (255, 0, 0)], 'confidence': 0.85},
    'AUS': {'name': 'Australia', 'colors': [(0, 0, 255), (255, 255, 255), (255, 0, 0)], 'confidence': 0.85},
    'BRA': {'name': 'Brazil', 'colors': [(0, 255, 0), (255, 255, 0), (0, 0, 255)], 'confidence': 0.85},
    'RUS': {'name': 'Russia', 'colors': [(255, 255, 255), (0, 0, 255), (255, 0, 0)], 'confidence': 0.90},
    'TUR': {'name': 'Turkey', 'colors': [(255, 0, 0), (255, 255, 255)], 'confidence': 0.85},
    'IRN': {'name': 'Iran', 'colors': [(255, 255, 0), (0, 128, 0), (255, 0, 0)], 'confidence': 0.80},
    'THA': {'name': 'Thailand', 'colors': [(255, 0, 0), (255, 255, 255), (0, 0, 128)], 'confidence': 0.85},
    'VIE': {'name': 'Vietnam', 'colors': [(255, 0, 0), (255, 255, 0)], 'confidence': 0.85},
    'PHI': {'name': 'Philippines', 'colors': [(0, 0, 128), (255, 255, 255), (255, 0, 0)], 'confidence': 0.80},
    'MAS': {'name': 'Malaysia', 'colors': [(255, 0, 0), (255, 255, 255), (0, 0, 128)], 'confidence': 0.80},
    'SGP': {'name': 'Singapore', 'colors': [(255, 0, 0), (255, 255, 255)], 'confidence': 0.85},
    'IND': {'name': 'India', 'colors': [(255, 128, 0), (255, 255, 255), (0, 128, 0)], 'confidence': 0.80},
    'PAK': {'name': 'Pakistan', 'colors': [(0, 128, 0), (255, 255, 255)], 'confidence': 0.80},
    'BAN': {'name': 'Bangladesh', 'colors': [(0, 128, 0), (255, 0, 0)], 'confidence': 0.80},
    'NEP': {'name': 'Nepal', 'colors': [(255, 0, 0), (0, 0, 128)], 'confidence': 0.75},
    'SRI': {'name': 'Sri Lanka', 'colors': [(255, 128, 0), (0, 128, 0), (255, 255, 255)], 'confidence': 0.75},
    'MNG': {'name': 'Mongolia', 'colors': [(255, 0, 0), (255, 255, 255), (0, 0, 128)], 'confidence': 0.75},
    'KAZ': {'name': 'Kazakhstan', 'colors': [(0, 128, 255), (255, 255, 255)], 'confidence': 0.75},
    'UZB': {'name': 'Uzbekistan', 'colors': [(0, 128, 255), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'KGZ': {'name': 'Kyrgyzstan', 'colors': [(255, 0, 0), (255, 255, 255)], 'confidence': 0.70},
    'TJK': {'name': 'Tajikistan', 'colors': [(255, 0, 0), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'TKM': {'name': 'Turkmenistan', 'colors': [(0, 128, 0), (255, 255, 255), (255, 0, 0)], 'confidence': 0.70},
    'AFG': {'name': 'Afghanistan', 'colors': [(0, 0, 128), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'IRQ': {'name': 'Iraq', 'colors': [(255, 0, 0), (255, 255, 255), (0, 0, 128)], 'confidence': 0.70},
    'SYR': {'name': 'Syria', 'colors': [(255, 0, 0), (255, 255, 255), (0, 0, 128)], 'confidence': 0.70},
    'LBN': {'name': 'Lebanon', 'colors': [(255, 0, 0), (255, 255, 255)], 'confidence': 0.70},
    'JOR': {'name': 'Jordan', 'colors': [(255, 0, 0), (255, 255, 255), (0, 0, 128)], 'confidence': 0.70},
    'ISR': {'name': 'Israel', 'colors': [(255, 255, 255), (0, 0, 128)], 'confidence': 0.70},
    'PSE': {'name': 'Palestine', 'colors': [(0, 0, 128), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'EGY': {'name': 'Egypt', 'colors': [(255, 0, 0), (255, 255, 255), (0, 0, 128)], 'confidence': 0.70},
    'LBY': {'name': 'Libya', 'colors': [(0, 0, 128), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'TUN': {'name': 'Tunisia', 'colors': [(255, 0, 0), (255, 255, 255)], 'confidence': 0.70},
    'DZA': {'name': 'Algeria', 'colors': [(0, 128, 0), (255, 255, 255)], 'confidence': 0.70},
    'MAR': {'name': 'Morocco', 'colors': [(255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'MLI': {'name': 'Mali', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'NGA': {'name': 'Nigeria', 'colors': [(0, 128, 0), (255, 255, 255)], 'confidence': 0.70},
    'GHA': {'name': 'Ghana', 'colors': [(255, 255, 0), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'CIV': {'name': 'Ivory Coast', 'colors': [(255, 255, 0), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'SEN': {'name': 'Senegal', 'colors': [(255, 255, 0), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'GMB': {'name': 'Gambia', 'colors': [(255, 0, 0), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'GIN': {'name': 'Guinea', 'colors': [(255, 0, 0), (255, 255, 0), (0, 128, 0)], 'confidence': 0.70},
    'SLE': {'name': 'Sierra Leone', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'LBR': {'name': 'Liberia', 'colors': [(255, 0, 0), (255, 255, 255)], 'confidence': 0.70},
    'TGO': {'name': 'Togo', 'colors': [(255, 255, 0), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'BEN': {'name': 'Benin', 'colors': [(255, 255, 0), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'NER': {'name': 'Niger', 'colors': [(255, 255, 0), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'BFA': {'name': 'Burkina Faso', 'colors': [(255, 255, 0), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'CMR': {'name': 'Cameroon', 'colors': [(255, 255, 0), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'CAF': {'name': 'Central African Republic', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'TCD': {'name': 'Chad', 'colors': [(255, 0, 0), (255, 255, 255), (0, 0, 128)], 'confidence': 0.70},
    'SDN': {'name': 'Sudan', 'colors': [(255, 0, 0), (255, 255, 255), (0, 0, 128)], 'confidence': 0.70},
    'ETH': {'name': 'Ethiopia', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'ERI': {'name': 'Eritrea', 'colors': [(255, 0, 0), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'DJI': {'name': 'Djibouti', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'SOM': {'name': 'Somalia', 'colors': [(255, 255, 255), (0, 0, 128)], 'confidence': 0.70},
    'KEN': {'name': 'Kenya', 'colors': [(255, 0, 0), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'UGA': {'name': 'Uganda', 'colors': [(255, 0, 0), (255, 255, 255), (0, 0, 128)], 'confidence': 0.70},
    'RWA': {'name': 'Rwanda', 'colors': [(255, 0, 0), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'BDI': {'name': 'Burundi', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'TZA': {'name': 'Tanzania', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'MWI': {'name': 'Malawi', 'colors': [(255, 0, 0), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'ZMB': {'name': 'Zambia', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'ZWE': {'name': 'Zimbabwe', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'BWA': {'name': 'Botswana', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'NAM': {'name': 'Namibia', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'ZAF': {'name': 'South Africa', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'LSO': {'name': 'Lesotho', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'SWZ': {'name': 'Eswatini', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'MOZ': {'name': 'Mozambique', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'MDG': {'name': 'Madagascar', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'MUS': {'name': 'Mauritius', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'COM': {'name': 'Comoros', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'SEY': {'name': 'Seychelles', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'STP': {'name': 'São Tomé and Príncipe', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'CPV': {'name': 'Cape Verde', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'GMB': {'name': 'Gambia', 'colors': [(255, 0, 0), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'GIN': {'name': 'Guinea', 'colors': [(255, 0, 0), (255, 255, 0), (0, 128, 0)], 'confidence': 0.70},
    'GNB': {'name': 'Guinea-Bissau', 'colors': [(255, 255, 0), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'SLE': {'name': 'Sierra Leone', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'LBR': {'name': 'Liberia', 'colors': [(255, 0, 0), (255, 255, 255)], 'confidence': 0.70},
    'TGO': {'name': 'Togo', 'colors': [(255, 255, 0), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'BEN': {'name': 'Benin', 'colors': [(255, 255, 0), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'NER': {'name': 'Niger', 'colors': [(255, 255, 0), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'BFA': {'name': 'Burkina Faso', 'colors': [(255, 255, 0), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'CMR': {'name': 'Cameroon', 'colors': [(255, 255, 0), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'CAF': {'name': 'Central African Republic', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'TCD': {'name': 'Chad', 'colors': [(255, 0, 0), (255, 255, 255), (0, 0, 128)], 'confidence': 0.70},
    'SDN': {'name': 'Sudan', 'colors': [(255, 0, 0), (255, 255, 255), (0, 0, 128)], 'confidence': 0.70},
    'ETH': {'name': 'Ethiopia', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'ERI': {'name': 'Eritrea', 'colors': [(255, 0, 0), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'DJI': {'name': 'Djibouti', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'SOM': {'name': 'Somalia', 'colors': [(255, 255, 255), (0, 0, 128)], 'confidence': 0.70},
    'KEN': {'name': 'Kenya', 'colors': [(255, 0, 0), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'UGA': {'name': 'Uganda', 'colors': [(255, 0, 0), (255, 255, 255), (0, 0, 128)], 'confidence': 0.70},
    'RWA': {'name': 'Rwanda', 'colors': [(255, 0, 0), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'BDI': {'name': 'Burundi', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'TZA': {'name': 'Tanzania', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'MWI': {'name': 'Malawi', 'colors': [(255, 0, 0), (255, 255, 255), (0, 128, 0)], 'confidence': 0.70},
    'ZMB': {'name': 'Zambia', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'ZWE': {'name': 'Zimbabwe', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'BWA': {'name': 'Botswana', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'NAM': {'name': 'Namibia', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'ZAF': {'name': 'South Africa', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'LSO': {'name': 'Lesotho', 'colors': [(255, 255, 255), (0, 128, 0), (255, 0, 0)], 'confidence': 0.70},
    'SWZ': {'name': 'Eswatini', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'MOZ': {'name': 'Mozambique', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'MDG': {'name': 'Madagascar', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'MUS': {'name': 'Mauritius', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'COM': {'name': 'Comoros', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'SEY': {'name': 'Seychelles', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'STP': {'name': 'São Tomé and Príncipe', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
    'CPV': {'name': 'Cape Verde', 'colors': [(255, 255, 255), (255, 0, 0), (0, 128, 0)], 'confidence': 0.70},
}

def analyze_image_colors(image_path):
    """Analyze dominant colors in an image"""
    try:
        img = cv2.imread(image_path)
        if img is None:
            return []
        
        img_rgb = cv2.cvtColor(img, cv2.COLOR_BGR2RGB)
        pixels = img_rgb.reshape(-1, 3)
        
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

def calculate_color_similarity(color1, color2):
    """Calculate similarity between two colors"""
    return 1.0 - (np.sqrt(sum((a - b) ** 2 for a, b in zip(color1, color2))) / 441.67)

def recognize_flag(image_path):
    """Recognize flag from image using expanded database"""
    image_colors = analyze_image_colors(image_path)
    if not image_colors:
        return None
    
    best_match = None
    best_score = 0.0
    
    for code, data in IOC_DATABASE.items():
        score = 0.0
        color_matches = 0
        
        for pattern_color in data['colors']:
            for image_color in image_colors:
                similarity = calculate_color_similarity(pattern_color, image_color)
                if similarity > 0.6:
                    color_matches += 1
                    score += similarity
        
        if color_matches > 0:
            score = (score / len(data['colors'])) * data['confidence']
            
            if score > best_score:
                best_score = score
                best_match = {'code': code, 'name': data['name'], 'score': score}
    
    return best_match if best_score > 0.5 else None

def rename_flag_file(old_path, country_code):
    """Rename flag file to IOC country code"""
    file_path = Path(old_path)
    new_name = f"{country_code}.png"
    new_path = file_path.parent / new_name
    
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

def main():
    parser = argparse.ArgumentParser(description='Simple Enhanced Flag Recognition')
    parser.add_argument('directory', help='Directory containing flag images')
    parser.add_argument('--batch', action='store_true', help='Batch mode (non-interactive)')
    
    args = parser.parse_args()
    
    if not os.path.exists(args.directory):
        print(f"Directory not found: {args.directory}")
        sys.exit(1)
    
    directory_path = Path(args.directory)
    image_extensions = {'.png', '.jpg', '.jpeg', '.gif', '.bmp'}
    unrecognized_files = [f for f in directory_path.iterdir() 
                         if f.is_file() and f.suffix.lower() in image_extensions
                         and 'Asset' in f.name]
    
    print(f"Found {len(unrecognized_files)} unrecognized flag files to process")
    
    results = []
    for image_file in unrecognized_files:
        print(f"\nProcessing: {image_file.name}")
        
        match = recognize_flag(str(image_file))
        
        if match:
            print(f"  Enhanced recognition: {match['name']} ({match['code']}) [{match['score']:.2f}]")
            
            if args.batch:
                new_path = rename_flag_file(str(image_file), match['code'])
                print(f"  Renamed to: {Path(new_path).name}")
                results.append({
                    'original': image_file.name,
                    'new': Path(new_path).name,
                    'country': match['name'],
                    'code': match['code'],
                    'confidence': match['score']
                })
            else:
                response = input(f"  Rename to {match['code']}.png? (y/n): ").lower()
                if response == 'y':
                    new_path = rename_flag_file(str(image_file), match['code'])
                    print(f"  Renamed to: {Path(new_path).name}")
                    results.append({
                        'original': image_file.name,
                        'new': Path(new_path).name,
                        'country': match['name'],
                        'code': match['code'],
                        'confidence': match['score']
                    })
        else:
            print(f"  Still could not recognize flag")
    
    print(f"\nEnhanced recognition complete. {len(results)} new flags recognized.")
    
    if results:
        print("\nRecognized flags:")
        for result in results:
            print(f"  {result['original']} -> {result['new']} ({result['country']})")

if __name__ == "__main__":
    main() 