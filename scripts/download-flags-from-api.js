const fs = require('fs');
const path = require('path');
const https = require('https');

// Function to download a file
function downloadFile(url, filepath) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(filepath);
    
    const options = {
      headers: {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36'
      }
    };
    
    https.get(url, options, (response) => {
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

// Function to get flag URL from flag API
function getFlagApiUrl(countryCode) {
  // Using flagcdn.com API which provides free SVG flags
  return `https://flagcdn.com/${countryCode.toLowerCase()}.svg`;
}

// Main function
async function downloadFlagsFromApi() {
  const flagsDir = path.join(__dirname, '..', 'ui', 'public', 'assets', 'flags');
  const svgFlagsDir = path.join(flagsDir, 'svg');
  
  // Create SVG flags directory if it doesn't exist
  if (!fs.existsSync(svgFlagsDir)) {
    fs.mkdirSync(svgFlagsDir, { recursive: true });
  }
  
  // Get list of PNG files
  const pngCodes = getPngFiles(flagsDir);
  console.log(`📁 Found ${pngCodes.length} PNG flag files`);
  
  console.log('🚀 Starting SVG flag download from flagcdn.com...');
  console.log(`📁 Output directory: ${svgFlagsDir}`);
  
  let successCount = 0;
  let failCount = 0;
  const failedDownloads = [];
  
  // Test with just a few countries first
  const testCodes = ['USA', 'GBR', 'FRA', 'GER', 'ITA', 'ESP', 'RUS', 'CHN', 'JPN', 'BRA'];
  
  for (const code of testCodes) {
    const svgFilename = `${code}.svg`;
    const svgPath = path.join(svgFlagsDir, svgFilename);
    
    // Skip if file already exists and has content
    if (fs.existsSync(svgPath) && fs.statSync(svgPath).size > 0) {
      console.log(`⏭️  Skipping ${svgFilename} (already exists)`);
      continue;
    }
    
    try {
      const url = getFlagApiUrl(code);
      console.log(`🔗 Trying URL: ${url}`);
      await downloadFile(url, svgPath);
      successCount++;
      console.log(`✅ Downloaded: ${svgFilename}`);
      
      // Add a small delay to be respectful to the server
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
downloadFlagsFromApi().catch(console.error); 