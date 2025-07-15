#!/usr/bin/env python3
"""
Enhanced Flag Recognition Script for reStrike VTA

This script uses the comprehensive IOC flag database to recognize flags
that were not identified in the first pass. It includes:

- Comprehensive IOC database (Current NOCs, Historic NOCs, Special codes)
- Enhanced color analysis and pattern matching
- Multiple recognition algorithms
- Detailed reporting and confidence scoring
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
import pandas as pd
from PIL import Image
import requests

@dataclass
class RecognitionResult:
    """Result of flag recognition attempt"""
    original_filename: str
    new_filename: str
    country_code: str
    country_name: str
    confidence: float
    method: str
    category: str
    colors_detected: List[Tuple[int, int, int]]
    pattern_type: str

class EnhancedFlagRecognizer:
    def __init__(self, database_path: str = "scripts/media/ioc_flag_database.json"):
        self.database_path = database_path
        self.ioc_database = self._load_ioc_database()
        self.flag_patterns = self.ioc_database.get('flag_patterns', {})
        self.ioc_entries = self.ioc_database.get('ioc_entries', {})
        
    def _load_ioc_database(self) -> Dict:
        """Load the comprehensive IOC flag database"""
        try:
            with open(self.database_path, 'r') as f:
                return json.load(f)
        except FileNotFoundError:
            print(f"Warning: IOC database not found at {self.database_path}")
            print("Please run ioc-flag-database.py first to create the database.")
            return {'flag_patterns': {}, 'ioc_entries': {}}
    
    def analyze_image_colors(self, image_path: str) -> List[Tuple[int, int, int]]:
        """Analyze dominant colors in an image with enhanced algorithm"""
        try:
            img = cv2.imread(image_path)
            if img is None:
                return []
            
            # Convert to RGB
            img_rgb = cv2.cvtColor(img, cv2.COLOR_BGR2RGB)
            
            # Reshape to 2D array
            pixels = img_rgb.reshape(-1, 3)
            
            # Use k-means to find dominant colors
            from sklearn.cluster import KMeans
            kmeans = KMeans(n_clusters=8, random_state=42)  # Increased clusters for better analysis
            kmeans.fit(pixels)
            
            # Get dominant colors
            colors = kmeans.cluster_centers_.astype(int)
            
            # Sort by frequency
            labels = kmeans.labels_
            color_counts = np.bincount(labels)
            sorted_indices = np.argsort(color_counts)[::-1]
            
            # Return top 5 colors for better matching
            return [tuple(colors[i]) for i in sorted_indices[:5]]
            
        except Exception as e:
            print(f"Error analyzing colors in {image_path}: {e}")
            return []
    
    def calculate_color_similarity(self, color1: Tuple[int, int, int], 
                                 color2: Tuple[int, int, int]) -> float:
        """Calculate similarity between two colors using enhanced algorithm"""
        # Convert to HSV for better color comparison
        def rgb_to_hsv(rgb):
            r, g, b = rgb[0]/255.0, rgb[1]/255.0, rgb[2]/255.0
            max_val = max(r, g, b)
            min_val = min(r, g, b)
            diff = max_val - min_val
            
            if max_val == min_val:
                h = 0
            elif max_val == r:
                h = (60 * ((g-b)/diff) + 360) % 360
            elif max_val == g:
                h = (60 * ((b-r)/diff) + 120) % 360
            else:
                h = (60 * ((r-g)/diff) + 240) % 360
            
            s = 0 if max_val == 0 else diff / max_val
            v = max_val
            
            return (h, s, v)
        
        hsv1 = rgb_to_hsv(color1)
        hsv2 = rgb_to_hsv(color2)
        
        # Calculate HSV distance
        h_diff = min(abs(hsv1[0] - hsv2[0]), 360 - abs(hsv1[0] - hsv2[0])) / 360.0
        s_diff = abs(hsv1[1] - hsv2[1])
        v_diff = abs(hsv1[2] - hsv2[2])
        
        # Weighted combination (hue is most important for flags)
        similarity = 1.0 - (0.6 * h_diff + 0.2 * s_diff + 0.2 * v_diff)
        return max(0.0, similarity)
    
    def recognize_flag_enhanced(self, image_path: str) -> Optional[RecognitionResult]:
        """Enhanced flag recognition using comprehensive database"""
        # Analyze image colors
        image_colors = self.analyze_image_colors(image_path)
        if not image_colors:
            return None
        
        best_match = None
        best_score = 0.0
        best_method = "unknown"
        
        # Method 1: Direct color matching with comprehensive database
        for code, pattern in self.flag_patterns.items():
            score = 0.0
            color_matches = 0
            
            pattern_colors = pattern.get('colors', [])
            if not pattern_colors:
                continue
            
            # Check color matches
            for pattern_color in pattern_colors:
                for image_color in image_colors:
                    similarity = self.calculate_color_similarity(pattern_color, image_color)
                    if similarity > 0.6:  # Lowered threshold for better matching
                        color_matches += 1
                        score += similarity
            
            # Normalize score
            if color_matches > 0:
                score = (score / len(pattern_colors)) * pattern.get('confidence', 0.85)
                
                if score > best_score:
                    best_score = score
                    best_match = pattern
                    best_method = "enhanced_color_matching"
        
        # Method 2: Pattern-based recognition
        if best_score < 0.7:  # If color matching wasn't confident enough
            pattern_score = self._pattern_based_recognition(image_path, image_colors)
            if pattern_score and pattern_score['score'] > best_score:
                best_score = pattern_score['score']
                best_match = pattern_score['pattern']
                best_method = "pattern_analysis"
        
        if best_match and best_score > 0.5:  # Lowered threshold for better coverage
            return RecognitionResult(
                original_filename=Path(image_path).name,
                new_filename=f"{best_match['country_code']}.png",
                country_code=best_match['country_code'],
                country_name=best_match['country_name'],
                confidence=best_score,
                method=best_method,
                category=best_match.get('category', 'unknown'),
                colors_detected=image_colors,
                pattern_type=best_match.get('pattern_type', 'unknown')
            )
        
        return None
    
    def _pattern_based_recognition(self, image_path: str, colors: List[Tuple[int, int, int]]) -> Optional[Dict]:
        """Pattern-based flag recognition"""
        try:
            img = cv2.imread(image_path)
            if img is None:
                return None
            
            # Analyze flag structure
            height, width = img.shape[:2]
            aspect_ratio = width / height
            
            # Check for common flag patterns
            if aspect_ratio > 1.5:  # Wide flag
                # Analyze horizontal stripes
                stripe_colors = self._analyze_horizontal_stripes(img)
                if stripe_colors:
                    return self._match_stripe_pattern(stripe_colors)
            
            # Check for vertical stripes
            stripe_colors = self._analyze_vertical_stripes(img)
            if stripe_colors:
                return self._match_stripe_pattern(stripe_colors)
            
            # Check for cross patterns (like UK, Norway, etc.)
            cross_pattern = self._analyze_cross_pattern(img)
            if cross_pattern:
                return cross_pattern
            
        except Exception as e:
            print(f"Error in pattern analysis: {e}")
        
        return None
    
    def _analyze_horizontal_stripes(self, img) -> List[Tuple[int, int, int]]:
        """Analyze horizontal stripe patterns"""
        height, width = img.shape[:2]
        stripe_height = height // 3  # Assume 3 stripes
        
        colors = []
        for i in range(3):
            y_start = i * stripe_height
            y_end = (i + 1) * stripe_height
            stripe = img[y_start:y_end, :]
            
            # Get dominant color in this stripe
            stripe_rgb = cv2.cvtColor(stripe, cv2.COLOR_BGR2RGB)
            pixels = stripe_rgb.reshape(-1, 3)
            
            from sklearn.cluster import KMeans
            kmeans = KMeans(n_clusters=1, random_state=42)
            kmeans.fit(pixels)
            
            color = tuple(kmeans.cluster_centers_[0].astype(int))
            colors.append(color)
        
        return colors
    
    def _analyze_vertical_stripes(self, img) -> List[Tuple[int, int, int]]:
        """Analyze vertical stripe patterns"""
        height, width = img.shape[:2]
        stripe_width = width // 3  # Assume 3 stripes
        
        colors = []
        for i in range(3):
            x_start = i * stripe_width
            x_end = (i + 1) * stripe_width
            stripe = img[:, x_start:x_end]
            
            # Get dominant color in this stripe
            stripe_rgb = cv2.cvtColor(stripe, cv2.COLOR_BGR2RGB)
            pixels = stripe_rgb.reshape(-1, 3)
            
            from sklearn.cluster import KMeans
            kmeans = KMeans(n_clusters=1, random_state=42)
            kmeans.fit(pixels)
            
            color = tuple(kmeans.cluster_centers_[0].astype(int))
            colors.append(color)
        
        return colors
    
    def _match_stripe_pattern(self, stripe_colors: List[Tuple[int, int, int]]) -> Optional[Dict]:
        """Match stripe colors to known flag patterns"""
        best_match = None
        best_score = 0.0
        
        for code, pattern in self.flag_patterns.items():
            pattern_colors = pattern.get('colors', [])
            if len(pattern_colors) >= 3:  # Need at least 3 colors for stripe matching
                score = 0.0
                matches = 0
                
                for i, stripe_color in enumerate(stripe_colors[:3]):
                    if i < len(pattern_colors):
                        similarity = self.calculate_color_similarity(stripe_color, pattern_colors[i])
                        if similarity > 0.6:
                            matches += 1
                            score += similarity
                
                if matches >= 2:  # At least 2 stripes should match
                    score = score / len(stripe_colors)
                    if score > best_score:
                        best_score = score
                        best_match = pattern
        
        if best_match and best_score > 0.6:
            return {
                'pattern': best_match,
                'score': best_score
            }
        
        return None
    
    def _analyze_cross_pattern(self, img) -> Optional[Dict]:
        """Analyze cross patterns (like UK flag)"""
        # This is a simplified cross pattern analysis
        # In a real implementation, you'd use more sophisticated edge detection
        return None
    
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
    
    def process_unrecognized_flags(self, directory: str, interactive: bool = True) -> List[RecognitionResult]:
        """Process flags that were not recognized in the first pass"""
        results = []
        directory_path = Path(directory)
        
        # Find unrecognized flags (those still with "Asset" in filename)
        image_extensions = {'.png', '.jpg', '.jpeg', '.gif', '.bmp'}
        unrecognized_files = [f for f in directory_path.iterdir() 
                             if f.is_file() and f.suffix.lower() in image_extensions
                             and 'Asset' in f.name]
        
        print(f"Found {len(unrecognized_files)} unrecognized flag files to process")
        
        for image_file in unrecognized_files:
            print(f"\nProcessing: {image_file.name}")
            
            # Try enhanced recognition
            result = self.recognize_flag_enhanced(str(image_file))
            
            if result:
                print(f"  Enhanced recognition: {result.country_name} ({result.country_code}) [{result.method}, {result.confidence:.2f}]")
                
                if interactive:
                    response = input(f"  Rename to {result.country_code}.png? (y/n): ").lower()
                    if response == 'y':
                        new_path = self.rename_flag_file(str(image_file), result.country_code)
                        result.new_filename = Path(new_path).name
                        print(f"  Renamed to: {result.new_filename}")
                else:
                    new_path = self.rename_flag_file(str(image_file), result.country_code)
                    result.new_filename = Path(new_path).name
                    print(f"  Renamed to: {result.new_filename}")
            else:
                print(f"  Still could not recognize flag")
                if interactive:
                    manual_code = input("  Enter IOC country code manually (or press Enter to skip): ").strip().upper()
                    if manual_code and manual_code in self.ioc_entries:
                        result = RecognitionResult(
                            original_filename=image_file.name,
                            new_filename=f"{manual_code}.png",
                            country_code=manual_code,
                            country_name=self.ioc_entries[manual_code]['name'],
                            confidence=1.0,
                            method="manual",
                            category=self.ioc_entries[manual_code].get('category', 'unknown'),
                            colors_detected=[],
                            pattern_type="manual"
                        )
                        new_path = self.rename_flag_file(str(image_file), manual_code)
                        result.new_filename = Path(new_path).name
                        print(f"  Renamed to: {result.new_filename}")
            
            if result:
                results.append(result)
        
        return results
    
    def generate_enhanced_report(self, results: List[RecognitionResult], outdir: str):
        """Generate enhanced recognition report"""
        if not results:
            print("No new recognitions to report")
            return
        
        # Convert to DataFrame
        df_data = []
        for result in results:
            df_data.append({
                'original_filename': result.original_filename,
                'new_filename': result.new_filename,
                'country_code': result.country_code,
                'country_name': result.country_name,
                'confidence': result.confidence,
                'method': result.method,
                'category': result.category,
                'pattern_type': result.pattern_type
            })
        
        df = pd.DataFrame(df_data)
        
        # Save reports
        csv_path = os.path.join(outdir, 'enhanced_flag_recognition_results.csv')
        md_path = os.path.join(outdir, 'enhanced_flag_recognition_results.md')
        
        df.to_csv(csv_path, index=False)
        
        with open(md_path, 'w') as f:
            f.write('# Enhanced Flag Recognition Results\n\n')
            f.write(f'Generated on: {pd.Timestamp.now().strftime("%Y-%m-%d %H:%M:%S")}\n\n')
            f.write(f'Total new recognitions: {len(results)}\n\n')
            f.write('## Recognition Summary\n\n')
            f.write(df.to_markdown(index=False))
            
            # Method breakdown
            f.write('\n\n## Recognition Methods\n\n')
            method_counts = df['method'].value_counts()
            for method, count in method_counts.items():
                f.write(f'- {method}: {count} flags\n')
            
            # Category breakdown
            f.write('\n\n## Categories Recognized\n\n')
            category_counts = df['category'].value_counts()
            for category, count in category_counts.items():
                f.write(f'- {category}: {count} flags\n')
        
        print(f'Enhanced CSV report: {csv_path}')
        print(f'Enhanced Markdown report: {md_path}')

def main():
    parser = argparse.ArgumentParser(description='Enhanced Flag Recognition Tool')
    parser.add_argument('directory', help='Directory containing flag images')
    parser.add_argument('--batch', action='store_true', help='Batch mode (non-interactive)')
    parser.add_argument('--database', default='scripts/media/ioc_flag_database.json', 
                       help='Path to IOC flag database')
    
    args = parser.parse_args()
    
    if not os.path.exists(args.directory):
        print(f"Directory not found: {args.directory}")
        sys.exit(1)
    
    # Initialize enhanced recognizer
    recognizer = EnhancedFlagRecognizer(args.database)
    
    # Process unrecognized flags
    results = recognizer.process_unrecognized_flags(args.directory, not args.batch)
    
    # Generate enhanced report
    recognizer.generate_enhanced_report(results, args.directory)
    
    print(f"\nEnhanced recognition complete. {len(results)} new flags recognized.")

if __name__ == "__main__":
    main() 