# Flag Images Guide for reStrike VTA

## 📁 Directory Structure

Flag images are stored in: `ui/public/assets/flags/`

```
ui/public/assets/flags/
├── USA.png
├── JPN.png
├── KOR.png
├── CHN.png
├── GBR.png
└── ... (253 total flag files)
```

## 🏷️ Naming Convention

**Use IOC 3-letter country codes:**
- `USA.png` - United States
- `JPN.png` - Japan
- `KOR.png` - South Korea
- `CHN.png` - China
- `GBR.png` - Great Britain

## 🎨 Image Specifications

- **Format**: PNG (preferred) or JPG
- **Size**: 32x24px (4:3 aspect ratio)
- **Background**: Transparent (PNG) or white
- **Quality**: High resolution, crisp edges
- **Style**: Official flag designs

## ✅ **IOC Flag Download System - COMPLETED**

### **Current Status**
- **253 Flags Downloaded**: Complete IOC (International Olympic Committee) flag collection
- **Source**: Direct scraping from Wikipedia IOC codes page
- **Script**: `scripts/media/download_official_ioc_flags.py`
- **Coverage**: Current NOCs, Historic NOCs, Special Olympic/Paralympic codes

### **Flag Categories Downloaded:**
- **Current NOCs (Table 1)**: 206 flags - Main Olympic countries
- **Additional Territories (Table 2)**: 2 flags - Faroe Islands, Macau
- **Historic NOCs (Table 3)**: 12 flags - Soviet Union, Yugoslavia, East/West Germany
- **Historic Country Names (Table 4)**: 18 flags - Burma, Ceylon, Zaire, etc.
- **Special Olympic Codes (Table 5)**: 10 flags - Refugee Olympic Team, Independent Athletes
- **Special Paralympic Codes (Table 6)**: 5 flags - Refugee Paralympic Team, etc.

### **Technical Implementation:**
- **Download Script**: Python-based Wikipedia scraper with BeautifulSoup
- **Strategy**: Prioritized Current NOCs, then downloaded from other tables only if IOC code not already present
- **Reports**: JSON and Markdown reports generated automatically
- **React Integration**: `ui/src/utils/flagUtils.tsx` updated with all 253 IOC codes
- **Fallbacks**: Emoji flags for all codes with automatic error handling

## 🔧 Adding New Flags

### Option 1: Automated IOC Download (RECOMMENDED)
```bash
# Run the official IOC flag download script
python scripts/media/download_official_ioc_flags.py
```

### Option 2: Manual Download
1. Find official flag image online
2. Resize to 32x24px
3. Save as PNG with IOC country code
4. Place in `ui/public/assets/flags/`

### Option 3: Upload Your Images
1. Upload flag images to `ui/public/assets/flags/`
2. Rename files to match IOC country codes
3. Ensure proper sizing and format

## 🔗 Usage in Code

### Using the FlagImage Component
```typescript
import { FlagImage } from '../utils/flagUtils';

// In your component
<FlagImage countryCode="USA" />
<FlagImage countryCode="JPN" />
```

### Direct Image Usage
```typescript
<img src="/assets/flags/svg/USA.svg" alt="USA Flag" />
```

## 🛡️ Fallback System

The `FlagImage` component includes automatic fallback:
- If image fails to load → Shows emoji flag
- If country code not found → Shows white flag emoji
- Graceful degradation for missing images
- **Complete Coverage**: All 253 IOC codes have emoji fallbacks

## 📚 Reference Resources

- **IOC Country Codes**: [Wikipedia List](https://en.wikipedia.org/wiki/List_of_IOC_country_codes)
- **Flag Sources**: 
  - [FlagCDN](https://flagcdn.com/) - Free flag API
  - Government websites
  - Public domain sources
- **System Documentation**: `docs/FLAG_MANAGEMENT_SYSTEM.md`

## 🔄 Maintenance

- **Automated Updates**: Use the IOC download script for updates
- **Manual Updates**: Add new flags as needed for competitions
- **Quality Control**: Maintain consistent naming and quality standards
- **Testing**: Test fallback system with missing images
- **Documentation**: Keep system documentation updated

## 🎯 Common Taekwondo Countries

Priority countries for taekwondo competitions (all included in 253 flags):
- USA, KOR, CHN, JPN, GBR, FRA, GER, ITA, ESP
- THA, VIE, PHI, MAS, SGP, TUR, IRN, BRA, AUS, CAN

## 📊 **System Statistics**

- **Total Flags**: 253 IOC flags downloaded and integrated
- **Download Success Rate**: 100% (no failures)
- **React Integration**: Complete with utility functions
- **Fallback Coverage**: 100% emoji fallbacks for all codes
- **Documentation**: Complete system documentation available
- **Status**: ✅ **COMPLETED** - Production ready 