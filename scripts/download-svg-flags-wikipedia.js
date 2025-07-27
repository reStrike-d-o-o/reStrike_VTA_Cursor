const fs = require('fs');
const path = require('path');
const https = require('https');

// Function to download a file
function downloadFile(url, filepath) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(filepath);
    
    https.get(url, (response) => {
      if (response.statusCode === 200) {
        response.pipe(file);
        file.on('finish', () => {
          file.close();
          console.log(`✅ Downloaded: ${path.basename(filepath)}`);
          resolve();
        });
      } else {
        console.log(`❌ Failed to download ${url}: ${response.statusCode}`);
        reject(new Error(`HTTP ${response.statusCode}`));
      }
    }).on('error', (err) => {
      fs.unlink(filepath, () => {}); // Delete the file if download failed
      console.log(`❌ Error downloading ${url}: ${err.message}`);
      reject(err);
    });
  });
}

// Function to get the list of PNG files in the flags directory
function getPngFiles(flagsDir) {
  try {
    const files = fs.readdirSync(flagsDir);
    return files.filter(file => file.endsWith('.png')).map(file => file.replace('.png', ''));
  } catch (error) {
    console.error('Error reading flags directory:', error.message);
    return [];
  }
}

// Function to get Wikipedia SVG URL for a country code
function getWikipediaSvgUrl(countryCode) {
  // Based on the Wikipedia page structure, we can construct URLs like:
  // https://upload.wikimedia.org/wikipedia/commons/thumb/3/36/Flag_of_Albania.svg/40px-Flag_of_Albania.svg.png
  // But we need the actual country names, not codes
  
  const countryCodeToName = {
    'AFG': 'Afghanistan',
    'ALB': 'Albania',
    'ALG': 'Algeria',
    'AND': 'Andorra',
    'ANG': 'Angola',
    'ANT': 'Antigua_and_Barbuda',
    'ARG': 'Argentina',
    'ARM': 'Armenia',
    'ARU': 'Aruba',
    'ASA': 'American_Samoa',
    'AUS': 'Australia',
    'AUT': 'Austria',
    'AZE': 'Azerbaijan',
    'BAH': 'Bahamas',
    'BAN': 'Bangladesh',
    'BAR': 'Barbados',
    'BDI': 'Burundi',
    'BEL': 'Belgium',
    'BEN': 'Benin',
    'BER': 'Bermuda',
    'BHU': 'Bhutan',
    'BIH': 'Bosnia_and_Herzegovina',
    'BIZ': 'Belize',
    'BLR': 'Belarus',
    'BOL': 'Bolivia',
    'BOT': 'Botswana',
    'BRA': 'Brazil',
    'BRN': 'Brunei',
    'BRU': 'Brunei',
    'BUL': 'Bulgaria',
    'BUR': 'Burkina_Faso',
    'CAF': 'Central_African_Republic',
    'CAM': 'Cambodia',
    'CAN': 'Canada',
    'CAY': 'Cayman_Islands',
    'CGO': 'Republic_of_the_Congo',
    'CHA': 'Chad',
    'CHI': 'Chile',
    'CHN': 'China',
    'CIV': 'Ivory_Coast',
    'CMR': 'Cameroon',
    'COD': 'Democratic_Republic_of_the_Congo',
    'COK': 'Cook_Islands',
    'COL': 'Colombia',
    'COM': 'Comoros',
    'CPV': 'Cape_Verde',
    'CRC': 'Costa_Rica',
    'CRO': 'Croatia',
    'CUB': 'Cuba',
    'CYP': 'Cyprus',
    'CZE': 'Czech_Republic',
    'DEN': 'Denmark',
    'DJI': 'Djibouti',
    'DMA': 'Dominica',
    'DOM': 'Dominican_Republic',
    'ECU': 'Ecuador',
    'EGY': 'Egypt',
    'ERI': 'Eritrea',
    'ESA': 'El_Salvador',
    'ESP': 'Spain',
    'EST': 'Estonia',
    'ETH': 'Ethiopia',
    'FIJ': 'Fiji',
    'FIN': 'Finland',
    'FRA': 'France',
    'FSM': 'Micronesia',
    'GAB': 'Gabon',
    'GAM': 'Gambia',
    'GBR': 'United_Kingdom',
    'GBS': 'Guinea-Bissau',
    'GEO': 'Georgia',
    'GEQ': 'Equatorial_Guinea',
    'GER': 'Germany',
    'GHA': 'Ghana',
    'GRE': 'Greece',
    'GRN': 'Grenada',
    'GUA': 'Guatemala',
    'GUI': 'Guinea',
    'GUM': 'Guam',
    'GUY': 'Guyana',
    'HAI': 'Haiti',
    'HKG': 'Hong_Kong',
    'HON': 'Honduras',
    'HUN': 'Hungary',
    'INA': 'Indonesia',
    'IND': 'India',
    'IRI': 'Iran',
    'IRL': 'Ireland',
    'IRQ': 'Iraq',
    'ISL': 'Iceland',
    'ISR': 'Israel',
    'ISV': 'United_States_Virgin_Islands',
    'ITA': 'Italy',
    'IVB': 'British_Virgin_Islands',
    'JAM': 'Jamaica',
    'JOR': 'Jordan',
    'JPN': 'Japan',
    'KAZ': 'Kazakhstan',
    'KEN': 'Kenya',
    'KGZ': 'Kyrgyzstan',
    'KHM': 'Cambodia',
    'KIR': 'Kiribati',
    'KOR': 'South_Korea',
    'KOS': 'Kosovo',
    'KSA': 'Saudi_Arabia',
    'KUW': 'Kuwait',
    'LAO': 'Laos',
    'LAT': 'Latvia',
    'LBA': 'Libya',
    'LBR': 'Liberia',
    'LCA': 'Saint_Lucia',
    'LES': 'Lesotho',
    'LIE': 'Liechtenstein',
    'LTU': 'Lithuania',
    'LUX': 'Luxembourg',
    'MAD': 'Madagascar',
    'MAR': 'Morocco',
    'MAS': 'Malaysia',
    'MAW': 'Malawi',
    'MDA': 'Moldova',
    'MDV': 'Maldives',
    'MEX': 'Mexico',
    'MHL': 'Marshall_Islands',
    'MKD': 'North_Macedonia',
    'MLI': 'Mali',
    'MLT': 'Malta',
    'MNG': 'Mongolia',
    'MNE': 'Montenegro',
    'MOZ': 'Mozambique',
    'MRI': 'Mauritius',
    'MSR': 'Montserrat',
    'MTN': 'Mauritania',
    'MYA': 'Myanmar',
    'NAM': 'Namibia',
    'NCA': 'Nicaragua',
    'NED': 'Netherlands',
    'NEP': 'Nepal',
    'NGR': 'Nigeria',
    'NIG': 'Niger',
    'NOR': 'Norway',
    'NRU': 'Nauru',
    'NZL': 'New_Zealand',
    'OMA': 'Oman',
    'PAK': 'Pakistan',
    'PAN': 'Panama',
    'PAR': 'Paraguay',
    'PER': 'Peru',
    'PHI': 'Philippines',
    'PLE': 'Palestine',
    'PLW': 'Palau',
    'PNG': 'Papua_New_Guinea',
    'POL': 'Poland',
    'POR': 'Portugal',
    'PRK': 'North_Korea',
    'PUR': 'Puerto_Rico',
    'QAT': 'Qatar',
    'ROU': 'Romania',
    'RSA': 'South_Africa',
    'RUS': 'Russia',
    'RWA': 'Rwanda',
    'SAM': 'Samoa',
    'SEN': 'Senegal',
    'SEY': 'Seychelles',
    'SGP': 'Singapore',
    'SKN': 'Saint_Kitts_and_Nevis',
    'SLE': 'Sierra_Leone',
    'SLO': 'Slovenia',
    'SMR': 'San_Marino',
    'SOL': 'Solomon_Islands',
    'SOM': 'Somalia',
    'SRB': 'Serbia',
    'SRI': 'Sri_Lanka',
    'STP': 'São_Tomé_and_Príncipe',
    'SUD': 'Sudan',
    'SUI': 'Switzerland',
    'SUR': 'Suriname',
    'SVK': 'Slovakia',
    'SWE': 'Sweden',
    'SWZ': 'Eswatini',
    'SYR': 'Syria',
    'TAN': 'Tanzania',
    'TGA': 'Tonga',
    'THA': 'Thailand',
    'TJK': 'Tajikistan',
    'TKM': 'Turkmenistan',
    'TLS': 'East_Timor',
    'TOG': 'Togo',
    'TTO': 'Trinidad_and_Tobago',
    'TUN': 'Tunisia',
    'TUR': 'Turkey',
    'TUV': 'Tuvalu',
    'UAE': 'United_Arab_Emirates',
    'UGA': 'Uganda',
    'UKR': 'Ukraine',
    'URU': 'Uruguay',
    'USA': 'United_States',
    'UZB': 'Uzbekistan',
    'VAN': 'Vanuatu',
    'VEN': 'Venezuela',
    'VIN': 'Saint_Vincent_and_the_Grenadines',
    'VNM': 'Vietnam',
    'VUT': 'Vanuatu',
    'YEM': 'Yemen',
    'ZAM': 'Zambia',
    'ZIM': 'Zimbabwe'
  };

  const countryName = countryCodeToName[countryCode];
  if (!countryName) {
    throw new Error(`No mapping found for country code: ${countryCode}`);
  }

  // Try different URL patterns that might work
  const urlPatterns = [
    `https://upload.wikimedia.org/wikipedia/commons/thumb/0/0d/Flag_of_${countryName}.svg/1200px-Flag_of_${countryName}.svg`,
    `https://upload.wikimedia.org/wikipedia/commons/thumb/9/9f/Flag_of_${countryName}.svg/1200px-Flag_of_${countryName}.svg`,
    `https://upload.wikimedia.org/wikipedia/commons/thumb/3/36/Flag_of_${countryName}.svg/1200px-Flag_of_${countryName}.svg`,
    `https://upload.wikimedia.org/wikipedia/commons/thumb/1/1a/Flag_of_${countryName}.svg/1200px-Flag_of_${countryName}.svg`,
    `https://upload.wikimedia.org/wikipedia/commons/thumb/7/77/Flag_of_${countryName}.svg/1200px-Flag_of_${countryName}.svg`,
    `https://upload.wikimedia.org/wikipedia/commons/thumb/8/89/Flag_of_${countryName}.svg/1200px-Flag_of_${countryName}.svg`,
    `https://upload.wikimedia.org/wikipedia/commons/thumb/2/2f/Flag_of_${countryName}.svg/1200px-Flag_of_${countryName}.svg`,
    `https://upload.wikimedia.org/wikipedia/commons/thumb/f/f6/Flag_of_${countryName}.svg/1200px-Flag_of_${countryName}.svg`
  ];

  return urlPatterns;
}

// Function to try multiple URL patterns
async function tryDownloadWithPatterns(countryCode, svgPath) {
  const urlPatterns = getWikipediaSvgUrl(countryCode);
  
  for (let i = 0; i < urlPatterns.length; i++) {
    try {
      await downloadFile(urlPatterns[i], svgPath);
      return true; // Success
    } catch (error) {
      if (i === urlPatterns.length - 1) {
        // Last pattern failed
        throw error;
      }
      // Try next pattern
      continue;
    }
  }
}

// Main function
async function downloadSvgFlagsFromWikipedia() {
  const flagsDir = path.join(__dirname, '..', 'ui', 'public', 'assets', 'flags');
  const svgFlagsDir = path.join(flagsDir, 'svg');
  
  // Create SVG flags directory if it doesn't exist
  if (!fs.existsSync(svgFlagsDir)) {
    fs.mkdirSync(svgFlagsDir, { recursive: true });
  }
  
  // Get list of PNG files
  const pngCodes = getPngFiles(flagsDir);
  console.log(`📁 Found ${pngCodes.length} PNG flag files`);
  
  console.log('🚀 Starting SVG flag download from Wikipedia...');
  console.log(`📁 Output directory: ${svgFlagsDir}`);
  
  let successCount = 0;
  let failCount = 0;
  const failedDownloads = [];
  
  for (const code of pngCodes) {
    const svgFilename = `${code}.svg`;
    const svgPath = path.join(svgFlagsDir, svgFilename);
    
    // Skip if file already exists
    if (fs.existsSync(svgPath)) {
      console.log(`⏭️  Skipping ${svgFilename} (already exists)`);
      continue;
    }
    
    try {
      await tryDownloadWithPatterns(code, svgPath);
      successCount++;
      console.log(`✅ Downloaded: ${svgFilename}`);
      
      // Add a small delay to be respectful to Wikipedia's servers
      await new Promise(resolve => setTimeout(resolve, 200));
      
    } catch (error) {
      console.log(`❌ Failed to download ${svgFilename}: ${error.message}`);
      failedDownloads.push({ code, error: error.message });
      failCount++;
    }
  }
  
  console.log('\n📊 Download Summary:');
  console.log(`✅ Successfully downloaded: ${successCount} flags`);
  console.log(`❌ Failed downloads: ${failCount} flags`);
  
  if (failedDownloads.length > 0) {
    console.log('\n❌ Failed downloads:');
    failedDownloads.forEach(({ code, error }) => {
      console.log(`   ${code} - ${error}`);
    });
  }
  
  console.log(`\n🎉 SVG flags saved to: ${svgFlagsDir}`);
}

// Run the script
downloadSvgFlagsFromWikipedia().catch(console.error); 