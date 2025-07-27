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

// Function to fetch Wikipedia page content
function fetchWikipediaPage() {
  return new Promise((resolve, reject) => {
    const url = 'https://en.wikipedia.org/wiki/List_of_IOC_country_codes';
    
    https.get(url, (response) => {
      if (response.statusCode === 200) {
        let data = '';
        response.on('data', (chunk) => {
          data += chunk;
        });
        response.on('end', () => {
          resolve(data);
        });
      } else {
        reject(new Error(`HTTP ${response.statusCode}`));
      }
    }).on('error', (err) => {
      reject(err);
    });
  });
}

// Function to extract flag URLs from Wikipedia page
function extractFlagUrls(htmlContent) {
  const flagUrls = {};
  
  // Look for flag image patterns in the HTML
  // Pattern: <img src="//upload.wikimedia.org/wikipedia/commons/thumb/.../Flag_of_Country.svg/40px-Flag_of_Country.svg.png"
  const flagPattern = /<img[^>]*src="\/\/upload\.wikimedia\.org\/wikipedia\/commons\/thumb\/([^\/]+)\/([^\/]+)\/Flag_of_([^\.]+)\.svg\/[^"]*"[^>]*>/g;
  
  let match;
  while ((match = flagPattern.exec(htmlContent)) !== null) {
    const [, thumbPath, filename, countryName] = match;
    
    // Convert country name to code (this is a simplified mapping)
    const countryCode = getCountryCodeFromName(countryName);
    if (countryCode) {
      // Construct the full SVG URL
      const svgUrl = `https://upload.wikimedia.org/wikipedia/commons/thumb/${thumbPath}/${filename}/Flag_of_${countryName}.svg/1200px-Flag_of_${countryName}.svg`;
      flagUrls[countryCode] = svgUrl;
    }
  }
  
  return flagUrls;
}

// Function to get country code from country name (simplified mapping)
function getCountryCodeFromName(countryName) {
  const nameToCode = {
    'Afghanistan': 'AFG',
    'Albania': 'ALB',
    'Algeria': 'ALG',
    'Andorra': 'AND',
    'Angola': 'ANG',
    'Antigua_and_Barbuda': 'ANT',
    'Argentina': 'ARG',
    'Armenia': 'ARM',
    'Aruba': 'ARU',
    'American_Samoa': 'ASA',
    'Australia': 'AUS',
    'Austria': 'AUT',
    'Azerbaijan': 'AZE',
    'Bahamas': 'BAH',
    'Bangladesh': 'BAN',
    'Barbados': 'BAR',
    'Burundi': 'BDI',
    'Belgium': 'BEL',
    'Benin': 'BEN',
    'Bermuda': 'BER',
    'Bhutan': 'BHU',
    'Bosnia_and_Herzegovina': 'BIH',
    'Belize': 'BIZ',
    'Belarus': 'BLR',
    'Bolivia': 'BOL',
    'Botswana': 'BOT',
    'Brazil': 'BRA',
    'Brunei': 'BRN',
    'Bulgaria': 'BUL',
    'Burkina_Faso': 'BUR',
    'Central_African_Republic': 'CAF',
    'Cambodia': 'CAM',
    'Canada': 'CAN',
    'Cayman_Islands': 'CAY',
    'Republic_of_the_Congo': 'CGO',
    'Chad': 'CHA',
    'Chile': 'CHI',
    'China': 'CHN',
    'Ivory_Coast': 'CIV',
    'Cameroon': 'CMR',
    'Democratic_Republic_of_the_Congo': 'COD',
    'Cook_Islands': 'COK',
    'Colombia': 'COL',
    'Comoros': 'COM',
    'Cape_Verde': 'CPV',
    'Costa_Rica': 'CRC',
    'Croatia': 'CRO',
    'Cuba': 'CUB',
    'Cyprus': 'CYP',
    'Czech_Republic': 'CZE',
    'Denmark': 'DEN',
    'Djibouti': 'DJI',
    'Dominica': 'DMA',
    'Dominican_Republic': 'DOM',
    'Ecuador': 'ECU',
    'Egypt': 'EGY',
    'Eritrea': 'ERI',
    'El_Salvador': 'ESA',
    'Spain': 'ESP',
    'Estonia': 'EST',
    'Ethiopia': 'ETH',
    'Fiji': 'FIJ',
    'Finland': 'FIN',
    'France': 'FRA',
    'Micronesia': 'FSM',
    'Gabon': 'GAB',
    'Gambia': 'GAM',
    'United_Kingdom': 'GBR',
    'Guinea-Bissau': 'GBS',
    'Georgia': 'GEO',
    'Equatorial_Guinea': 'GEQ',
    'Germany': 'GER',
    'Ghana': 'GHA',
    'Greece': 'GRE',
    'Grenada': 'GRN',
    'Guatemala': 'GUA',
    'Guinea': 'GUI',
    'Guam': 'GUM',
    'Guyana': 'GUY',
    'Haiti': 'HAI',
    'Hong_Kong': 'HKG',
    'Honduras': 'HON',
    'Hungary': 'HUN',
    'Indonesia': 'INA',
    'India': 'IND',
    'Iran': 'IRI',
    'Ireland': 'IRL',
    'Iraq': 'IRQ',
    'Iceland': 'ISL',
    'Israel': 'ISR',
    'United_States_Virgin_Islands': 'ISV',
    'Italy': 'ITA',
    'British_Virgin_Islands': 'IVB',
    'Jamaica': 'JAM',
    'Jordan': 'JOR',
    'Japan': 'JPN',
    'Kazakhstan': 'KAZ',
    'Kenya': 'KEN',
    'Kyrgyzstan': 'KGZ',
    'Kiribati': 'KIR',
    'South_Korea': 'KOR',
    'Kosovo': 'KOS',
    'Saudi_Arabia': 'KSA',
    'Kuwait': 'KUW',
    'Laos': 'LAO',
    'Latvia': 'LAT',
    'Libya': 'LBA',
    'Liberia': 'LBR',
    'Saint_Lucia': 'LCA',
    'Lesotho': 'LES',
    'Liechtenstein': 'LIE',
    'Lithuania': 'LTU',
    'Luxembourg': 'LUX',
    'Madagascar': 'MAD',
    'Morocco': 'MAR',
    'Malaysia': 'MAS',
    'Malawi': 'MAW',
    'Moldova': 'MDA',
    'Maldives': 'MDV',
    'Mexico': 'MEX',
    'Marshall_Islands': 'MHL',
    'North_Macedonia': 'MKD',
    'Mali': 'MLI',
    'Malta': 'MLT',
    'Mongolia': 'MNG',
    'Montenegro': 'MNE',
    'Mozambique': 'MOZ',
    'Mauritius': 'MRI',
    'Montserrat': 'MSR',
    'Mauritania': 'MTN',
    'Myanmar': 'MYA',
    'Namibia': 'NAM',
    'Nicaragua': 'NCA',
    'Netherlands': 'NED',
    'Nepal': 'NEP',
    'Nigeria': 'NGR',
    'Niger': 'NIG',
    'Norway': 'NOR',
    'Nauru': 'NRU',
    'New_Zealand': 'NZL',
    'Oman': 'OMA',
    'Pakistan': 'PAK',
    'Panama': 'PAN',
    'Paraguay': 'PAR',
    'Peru': 'PER',
    'Philippines': 'PHI',
    'Palestine': 'PLE',
    'Palau': 'PLW',
    'Papua_New_Guinea': 'PNG',
    'Poland': 'POL',
    'Portugal': 'POR',
    'North_Korea': 'PRK',
    'Puerto_Rico': 'PUR',
    'Qatar': 'QAT',
    'Romania': 'ROU',
    'South_Africa': 'RSA',
    'Russia': 'RUS',
    'Rwanda': 'RWA',
    'Samoa': 'SAM',
    'Senegal': 'SEN',
    'Seychelles': 'SEY',
    'Singapore': 'SGP',
    'Saint_Kitts_and_Nevis': 'SKN',
    'Sierra_Leone': 'SLE',
    'Slovenia': 'SLO',
    'San_Marino': 'SMR',
    'Solomon_Islands': 'SOL',
    'Somalia': 'SOM',
    'Serbia': 'SRB',
    'Sri_Lanka': 'SRI',
    'São_Tomé_and_Príncipe': 'STP',
    'Sudan': 'SUD',
    'Switzerland': 'SUI',
    'Suriname': 'SUR',
    'Slovakia': 'SVK',
    'Sweden': 'SWE',
    'Eswatini': 'SWZ',
    'Syria': 'SYR',
    'Tanzania': 'TAN',
    'Tonga': 'TGA',
    'Thailand': 'THA',
    'Tajikistan': 'TJK',
    'Turkmenistan': 'TKM',
    'East_Timor': 'TLS',
    'Togo': 'TOG',
    'Trinidad_and_Tobago': 'TTO',
    'Tunisia': 'TUN',
    'Turkey': 'TUR',
    'Tuvalu': 'TUV',
    'United_Arab_Emirates': 'UAE',
    'Uganda': 'UGA',
    'Ukraine': 'UKR',
    'Uruguay': 'URU',
    'United_States': 'USA',
    'Uzbekistan': 'UZB',
    'Vanuatu': 'VAN',
    'Venezuela': 'VEN',
    'Saint_Vincent_and_the_Grenadines': 'VIN',
    'Vietnam': 'VNM',
    'Yemen': 'YEM',
    'Zambia': 'ZAM',
    'Zimbabwe': 'ZIM'
  };
  
  return nameToCode[countryName] || null;
}

// Main function
async function scrapeAndDownloadFlags() {
  const flagsDir = path.join(__dirname, '..', 'ui', 'public', 'assets', 'flags');
  const svgFlagsDir = path.join(flagsDir, 'svg');
  
  // Create SVG flags directory if it doesn't exist
  if (!fs.existsSync(svgFlagsDir)) {
    fs.mkdirSync(svgFlagsDir, { recursive: true });
  }
  
  // Get list of PNG files
  const pngCodes = getPngFiles(flagsDir);
  console.log(`📁 Found ${pngCodes.length} PNG flag files`);
  
  console.log('🌐 Fetching Wikipedia IOC country codes page...');
  
  try {
    const htmlContent = await fetchWikipediaPage();
    console.log('✅ Successfully fetched Wikipedia page');
    
    const flagUrls = extractFlagUrls(htmlContent);
    console.log(`🔍 Found ${Object.keys(flagUrls).length} flag URLs on Wikipedia`);
    
    console.log('🚀 Starting SVG flag download...');
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
      
      const flagUrl = flagUrls[code];
      
      if (!flagUrl) {
        console.log(`⚠️  No flag URL found for ${code}`);
        failedDownloads.push({ code, error: 'No URL found' });
        failCount++;
        continue;
      }
      
      try {
        await downloadFile(flagUrl, svgPath);
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
    
  } catch (error) {
    console.error('❌ Error fetching Wikipedia page:', error.message);
  }
}

// Run the script
scrapeAndDownloadFlags().catch(console.error); 