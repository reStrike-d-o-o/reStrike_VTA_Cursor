import React, { useState, useEffect } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import StatusDot from '../atoms/StatusDot';
import { getFlagConfig, getFlagUrl, handleFlagError, FLAG_CONFIGS } from '../../utils/flagUtils';

interface FlagInfo {
  iocCode: string;
  countryName: string;
  flagPath: string;
  hasCustomMapping: boolean;
  pssCode?: string;
}

interface FlagManagementPanelProps {
  className?: string;
}

const FlagManagementPanel: React.FC<FlagManagementPanelProps> = ({ className = '' }) => {
  const [flags, setFlags] = useState<FlagInfo[]>([]);
  const [filteredFlags, setFilteredFlags] = useState<FlagInfo[]>([]);
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedFlag, setSelectedFlag] = useState<FlagInfo | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0);
  const [error, setError] = useState<string>('');
  const [success, setSuccess] = useState<string>('');

  // Load flags on component mount
  useEffect(() => {
    loadFlags();
  }, []);

  // Filter flags based on search term
  useEffect(() => {
    if (!searchTerm.trim()) {
      setFilteredFlags(flags);
    } else {
      const filtered = flags.filter(flag => 
        flag.iocCode.toLowerCase().includes(searchTerm.toLowerCase()) ||
        flag.countryName.toLowerCase().includes(searchTerm.toLowerCase()) ||
        (flag.pssCode && flag.pssCode.toLowerCase().includes(searchTerm.toLowerCase()))
      );
      setFilteredFlags(filtered);
    }
  }, [searchTerm, flags]);

  const loadFlags = async () => {
    setIsLoading(true);
    try {
        // Load flags from the available assets in /assets/flags/
  // This creates a comprehensive list based on the flagUtils configuration
  const flagList: FlagInfo[] = Object.keys(FLAG_CONFIGS).map(iocCode => {
    const config = FLAG_CONFIGS[iocCode];
    return {
      iocCode,
      countryName: config.altText.replace(' Flag', ''),
      flagPath: `/assets/flags/${iocCode}.png`,
      hasCustomMapping: true, // All flags have PSS mapping (same as IOC code)
      pssCode: iocCode // PSS code is the same as IOC code
    };
  });

  setFlags(flagList);
  setFilteredFlags(flagList);
    } catch (error) {
      setError('Failed to load flags');
    } finally {
      setIsLoading(false);
    }
  };

  const handleFlagUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    if (!file.type.startsWith('image/')) {
      setError('Please select an image file');
      return;
    }

    setIsLoading(true);
    setUploadProgress(0);
    setError('');

    try {
      // Simulate upload progress
      for (let i = 0; i <= 100; i += 10) {
        setUploadProgress(i);
        await new Promise(resolve => setTimeout(resolve, 50));
      }

      // This would typically call a Tauri command to upload the flag
      setSuccess(`Flag uploaded successfully: ${file.name}`);
      
      // Reload flags after upload
      await loadFlags();
    } catch (error) {
      setError('Failed to upload flag');
    } finally {
      setIsLoading(false);
      setUploadProgress(0);
    }
  };

  const handleMappingUpdate = async (iocCode: string, pssCode: string) => {
    try {
      // This would typically call a Tauri command to update mapping
      setFlags(prev => prev.map(flag => 
        flag.iocCode === iocCode 
          ? { ...flag, pssCode, hasCustomMapping: true }
          : flag
      ));
      setSuccess(`Mapping updated for ${iocCode}`);
    } catch (error) {
      setError('Failed to update mapping');
    }
  };

  const handleMappingRemove = async (iocCode: string) => {
    try {
      // Reset the PSS code back to the IOC code
      setFlags(prev => prev.map(flag => 
        flag.iocCode === iocCode 
          ? { ...flag, pssCode: iocCode, hasCustomMapping: true }
          : flag
      ));
      setSuccess(`PSS code reset to IOC code for ${iocCode}`);
    } catch (error) {
      setError('Failed to reset mapping');
    }
  };

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Error and Success Messages */}
      {error && (
        <div className="p-4 bg-red-900/20 border border-red-600/30 rounded-lg text-red-300">
          {error}
        </div>
      )}
      {success && (
        <div className="p-4 bg-green-900/20 border border-green-600/30 rounded-lg text-green-300">
          {success}
        </div>
      )}

      {/* Flag Upload Section */}
      <div className="p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
        <h3 className="text-lg font-semibold mb-4 text-gray-100">Upload Custom Flag</h3>
        <div className="space-y-4">
          <div className="flex items-center space-x-3">
            <input
              type="file"
              accept="image/*"
              onChange={handleFlagUpload}
              className="hidden"
              id="flag-upload"
              disabled={isLoading}
              aria-label="Choose flag image file"
              title="Choose flag image file"
            />
            <Button
              size="sm"
              variant="primary"
              onClick={() => document.getElementById('flag-upload')?.click()}
              disabled={isLoading}
            >
              Choose Flag Image
            </Button>
            <span className="text-xs text-gray-400">PNG, JPG, or GIF format</span>
          </div>
          
          {uploadProgress > 0 && (
            <div className="w-full bg-gray-700 rounded-full h-2">
              <div
                className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                style={{ width: `${uploadProgress}%` }}
              ></div>
            </div>
          )}
          
          <p className="text-xs text-gray-400">
            Upload a custom flag image to add it to the available flags. 
            The image should be in PNG, JPG, or GIF format.
          </p>
        </div>
      </div>

      {/* Flag List Section */}
      <div className="p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-100">Available Flags</h3>
          <div className="flex items-center space-x-2">
            <StatusDot color={isLoading ? 'yellow' : 'green'} />
            <span className="text-xs text-gray-400">
              {filteredFlags.length} of {flags.length} flags
            </span>
          </div>
        </div>

        {/* Search */}
        <div className="mb-4">
          <Label htmlFor="flag-search" className="text-xs text-gray-400">Search Flags</Label>
          <Input
            id="flag-search"
            type="text"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            placeholder="Search by IOC code, country name, or PSS code..."
            className="mt-1"
          />
        </div>

        {/* Flag Grid */}
        {isLoading ? (
          <div className="text-sm text-gray-400">Loading flags...</div>
        ) : filteredFlags.length === 0 ? (
          <div className="text-sm text-gray-400">No flags found</div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 max-h-96 overflow-y-auto">
            {filteredFlags.map((flag) => (
              <div
                key={flag.iocCode}
                className={`p-4 rounded-lg border transition-all duration-200 cursor-pointer ${
                  selectedFlag?.iocCode === flag.iocCode
                    ? 'border-blue-500 bg-blue-900/20'
                    : 'border-gray-600 bg-gray-700/30 hover:bg-gray-700/50'
                }`}
                onClick={() => setSelectedFlag(flag)}
              >
                                 <div className="flex items-center space-x-3 mb-2">
                   <img
                     src={getFlagUrl(flag.iocCode)}
                     alt={`${flag.countryName} flag`}
                     className="w-8 h-6 object-cover rounded border border-gray-600"
                     onError={(e) => handleFlagError(e, flag.iocCode)}
                   />
                   <span className="text-lg">{getFlagConfig(flag.iocCode).fallbackEmoji}</span>
                  <div className="flex-1">
                    <div className="text-sm font-medium text-gray-200">
                      {flag.iocCode}
                    </div>
                    <div className="text-xs text-gray-400">
                      {flag.countryName}
                    </div>
                  </div>
                  {flag.hasCustomMapping && (
                    <StatusDot color="blue" />
                  )}
                </div>
                
                {flag.hasCustomMapping && (
                  <div className="text-xs text-blue-300">
                    Mapped to: {flag.pssCode}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Flag Details Section */}
      {selectedFlag && (
        <div className="p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
          <h3 className="text-lg font-semibold mb-4 text-gray-100">Flag Details</h3>
          <div className="space-y-4">
                         <div className="flex items-center space-x-4">
               <img
                 src={getFlagUrl(selectedFlag.iocCode)}
                 alt={`${selectedFlag.countryName} flag`}
                 className="w-16 h-12 object-cover rounded border border-gray-600"
                 onError={(e) => handleFlagError(e, selectedFlag.iocCode)}
               />
               <span className="text-3xl">{getFlagConfig(selectedFlag.iocCode).fallbackEmoji}</span>
              <div>
                <div className="text-lg font-medium text-gray-200">
                  {selectedFlag.countryName}
                </div>
                <div className="text-sm text-gray-400">
                  IOC Code: {selectedFlag.iocCode}
                </div>
              </div>
            </div>

            {/* PSS Code Mapping */}
            <div className="border-t border-gray-600/30 pt-4">
              <h4 className="text-sm font-medium text-gray-300 mb-3">PSS Code Mapping</h4>
              <div className="space-y-3">
                <div className="flex items-center space-x-3">
                  <span className="text-sm text-gray-400">Current PSS Code:</span>
                  <span className="px-2 py-1 bg-blue-900/30 text-blue-300 text-sm rounded border border-blue-600/30">
                    {selectedFlag.pssCode}
                  </span>
                </div>
                <div className="text-xs text-gray-500 mb-2">
                  This flag is mapped to the PSS protocol code shown above. You can modify this mapping if needed.
                </div>
                <div className="flex space-x-2">
                  <Input
                    type="text"
                    placeholder="New PSS code..."
                    className="flex-1"
                    defaultValue={selectedFlag.pssCode}
                    onKeyPress={(e) => {
                      if (e.key === 'Enter') {
                        const target = e.target as HTMLInputElement;
                        handleMappingUpdate(selectedFlag.iocCode, target.value);
                      }
                    }}
                  />
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => {
                      const input = document.querySelector('input[placeholder="New PSS code..."]') as HTMLInputElement;
                      if (input) {
                        handleMappingUpdate(selectedFlag.iocCode, input.value);
                      }
                    }}
                  >
                    Update
                  </Button>
                  <Button
                    size="sm"
                    variant="danger"
                    onClick={() => handleMappingRemove(selectedFlag.iocCode)}
                  >
                    Reset to IOC
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default FlagManagementPanel; 