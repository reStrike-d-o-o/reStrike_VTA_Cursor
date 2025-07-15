# Flag Assets for reStrike VTA

## 📁 Directory Purpose
This directory contains flag images for athlete country representation in the sidebar component.

## 🏷️ Naming Convention
Flags should be named using **IOC 3-letter country codes**:
- `USA.png` - United States
- `KOR.png` - South Korea  
- `CHN.png` - China
- `JPN.png` - Japan
- `GBR.png` - Great Britain
- `FRA.png` - France
- `GER.png` - Germany
- `ITA.png` - Italy
- `ESP.png` - Spain
- `CAN.png` - Canada
- `AUS.png` - Australia
- `BRA.png` - Brazil
- `RUS.png` - Russia
- `TUR.png` - Turkey
- `IRN.png` - Iran
- `THA.png` - Thailand
- `VIE.png` - Vietnam
- `PHI.png` - Philippines
- `MAS.png` - Malaysia
- `SGP.png` - Singapore

## 🎨 Image Specifications
- **Format**: PNG (preferred) or JPG
- **Size**: 32x24px (4:3 aspect ratio)
- **Background**: Transparent (PNG) or white
- **Quality**: High resolution, crisp edges
- **Style**: Official flag designs

## 🔗 Usage in Code
```typescript
// In React components
const flagUrl = `/assets/flags/${countryCode}.png`;

// Example
<img src="/assets/flags/USA.png" alt="USA Flag" />
```

## 📚 Reference
- **IOC Country Codes**: [Wikipedia - IOC Country Codes](https://en.wikipedia.org/wiki/List_of_IOC_country_codes)
- **Flag Sources**: Use official government flag images or public domain sources

## 🔄 Maintenance
- Add new flags as needed for new countries
- Update existing flags if official designs change
- Maintain consistent naming and quality standards 