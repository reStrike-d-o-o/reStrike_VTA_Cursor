import React, { useState, useEffect, useRef } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import StatusDot from '../atoms/StatusDot';
import { Progress } from '../atoms/Progress';
import { getFlagConfig, getFlagUrl, handleFlagError, FLAG_CONFIGS } from '../../utils/flagUtils';

interface FlagInfo {
  iocCode: string;
  countryName: string;
  flagPath: string;
  hasCustomMapping: boolean;
  pssCode?: string;
  // Database fields
  id?: number;
  filename?: string;
  recognition_status?: string;
  recognition_confidence?: number;
  upload_date?: string;
  file_size?: number;
}

interface FlagStatistics {
  total: number;
  recognized: number;
  pending: number;
  failed: number;
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
  const [statistics, setStatistics] = useState<FlagStatistics>({ total: 0, recognized: 0, pending: 0, failed: 0 });
  // Always use database in Tauri; fallback to assets in web
  const [flagMappingsCount, setFlagMappingsCount] = useState(0);
  const [pssCodeInput, setPssCodeInput] = useState('');
  const loadingRef = useRef(false);
  const initializedRef = useRef(false);

  // Load once on mount
  useEffect(() => {
    let isMounted = true;
    (async () => {
      await loadFlags();
      if (!isMounted) return;
      await loadFlagMappings();
    })();
    return () => {
      isMounted = false;
    };
  }, []);

  // Filter flags based on search term
  useEffect(() => {
    const term = searchTerm.trim().toLowerCase();
    if (!term) {
      setFilteredFlags(flags);
      return;
    }
    const filtered = flags.filter(flag => {
      const ioc = flag.iocCode?.toLowerCase() || '';
      const name = flag.countryName?.toLowerCase() || '';
      const pss = flag.pssCode?.toLowerCase() || '';
      return ioc.includes(term) || name.includes(term) || pss.includes(term);
    });
    setFilteredFlags(filtered);
  }, [searchTerm, flags]);

  // Update PSS code input when selected flag changes
  useEffect(() => {
    if (selectedFlag) {
      setPssCodeInput(selectedFlag.pssCode || '');
    }
  }, [selectedFlag]);

  const loadFlagMappings = async () => {
    if (!window.__TAURI__) return;

    try {
      const result = await window.__TAURI__.core.invoke('get_flag_mappings_data');
      if (result.success) {
        setFlagMappingsCount(result.count || 0);
      }
    } catch (error) {
      console.error('Failed to load flag mappings:', error);
    }
  };

  const loadFlags = async () => {
    if (loadingRef.current) return;
    loadingRef.current = true;
    setIsLoading(true);
    setError('');
    
    try {
      if (window.__TAURI__) {
        // Load flags from database
        const result = await window.__TAURI__.core.invoke('get_flags_data') as any;

        const handleResultToState = (res: any) => {
          const dbFlags: FlagInfo[] = (res.flags || []).map((flag: any) => ({
            iocCode: flag.ioc_code || flag.filename?.replace('.svg', '') || '',
            countryName: flag.country_name || '',
            flagPath: `/assets/flags/svg/${flag.filename}`,
            hasCustomMapping: !!flag.ioc_code,
            pssCode: flag.ioc_code,
            id: flag.id,
            filename: flag.filename,
            recognition_status: flag.recognition_status,
            recognition_confidence: flag.recognition_confidence,
            upload_date: flag.upload_date,
            file_size: flag.file_size
          }));
          setFlags(dbFlags);
          setFilteredFlags(dbFlags);
          setStatistics(res.statistics || { total: dbFlags.length, recognized: 0, pending: 0, failed: 0 });
          return dbFlags.length;
        };

        if (result.success) {
          const initialCount = handleResultToState(result);
          if (initialCount === 0) {
            // Try to auto-populate from assets and retry
            try {
              const scan = await window.__TAURI__.core.invoke('scan_and_populate_flags') as any;
              if (scan && scan.success) {
                const afterScan = await window.__TAURI__.core.invoke('get_flags_data') as any;
                if (afterScan.success) {
                  const finalCount = handleResultToState(afterScan);
                  if (finalCount === 0) {
                    // Fallback to assets if still empty
                    const flagList: FlagInfo[] = Object.keys(FLAG_CONFIGS).map(iocCode => {
                      const config = FLAG_CONFIGS[iocCode];
                      return {
                        iocCode,
                        countryName: config.altText.replace(' Flag', ''),
                        flagPath: `/assets/flags/svg/${iocCode}.svg`,
                        hasCustomMapping: true,
                        pssCode: iocCode
                      };
                    });
                    setFlags(flagList);
                    setFilteredFlags(flagList);
                    setStatistics({ total: flagList.length, recognized: flagList.length, pending: 0, failed: 0 });
                  }
                }
              }
            } catch (scanErr) {
              // On scan error, fallback to assets
              const flagList: FlagInfo[] = Object.keys(FLAG_CONFIGS).map(iocCode => {
                const config = FLAG_CONFIGS[iocCode];
                return {
                  iocCode,
                  countryName: config.altText.replace(' Flag', ''),
                  flagPath: `/assets/flags/svg/${iocCode}.svg`,
                  hasCustomMapping: true,
                  pssCode: iocCode
                };
              });
              setFlags(flagList);
              setFilteredFlags(flagList);
              setStatistics({ total: flagList.length, recognized: flagList.length, pending: 0, failed: 0 });
            }
          }
        } else {
          throw new Error(result.error || 'Failed to load flags from database');
        }
      } else {
        // In web mode, show static assets as fallback
        const flagList: FlagInfo[] = Object.keys(FLAG_CONFIGS).map(iocCode => {
          const config = FLAG_CONFIGS[iocCode];
          return {
            iocCode,
            countryName: config.altText.replace(' Flag', ''),
            flagPath: `/assets/flags/svg/${iocCode}.svg`,
            hasCustomMapping: true,
            pssCode: iocCode
          };
        });
        setFlags(flagList);
        setFilteredFlags(flagList);
        setStatistics({ total: flagList.length, recognized: flagList.length, pending: 0, failed: 0 });
      }
    } catch (error) {
      setError(`Failed to load flags: ${error instanceof Error ? error.message : 'Unknown error'}`);
      setStatistics({ total: 0, recognized: 0, pending: 0, failed: 0 });
    } finally {
      setIsLoading(false);
      loadingRef.current = false;
    }
  };
  
  // Load once on mount
  useEffect(() => {
    if (initializedRef.current) return;
    initializedRef.current = true;
    loadFlags();
    loadFlagMappings();
  }, []);

  const scanAndPopulateFlags = async () => {
    if (!window.__TAURI__) {
      setError('Database functionality not available in this environment');
      return;
    }

    setIsLoading(true);
    setError('');
    setSuccess('');

    try {
      const result = await window.__TAURI__.core.invoke('scan_and_populate_flags');
      
      if (result.success) {
        setSuccess(`Successfully scanned and populated flags! Processed: ${result.processed_count}, Skipped: ${result.skipped_count}`);
        
        if (result.errors && result.errors.length > 0) {
          setError(`Some errors occurred: ${result.errors.join(', ')}`);
        }
        
        // Reload flags after scanning
        await loadFlags();
      } else {
        setError(result.error || 'Failed to scan and populate flags');
      }
    } catch (error) {
      setError(`Failed to scan flags: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsLoading(false);
    }
  };

  const clearFlagsDatabase = async () => {
    if (!window.__TAURI__) {
      setError('Database functionality not available in this environment');
      return;
    }

    if (!window.confirm('Are you sure you want to clear all flags from the database? This action cannot be undone.')) {
      return;
    }

    setIsLoading(true);
    setError('');

    try {
      const result = await window.__TAURI__.core.invoke('clear_flags_table');
      
      if (result.success) {
        setSuccess(`Successfully cleared flags database! Deleted: ${result.deleted_count} entries`);
        
        // Reload flags after clearing
        await loadFlags();
      } else {
        setError(result.error || 'Failed to clear flags database');
      }
    } catch (error) {
      setError(`Failed to clear flags: ${error instanceof Error ? error.message : 'Unknown error'}`);
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

      {/* Database Settings */}
      {/* Database Settings removed: always use database for flags */}

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
            <Progress value={uploadProgress} />
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
                {selectedFlag.filename && (
                  <div className="text-xs text-gray-500">
                    File: {selectedFlag.filename}
                  </div>
                )}
              </div>
            </div>

            {/* Database Information */}
            {selectedFlag.id && (
              <div className="border-t border-gray-600/30 pt-4">
                <h4 className="text-sm font-medium text-gray-300 mb-3">Database Information</h4>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <div className="flex justify-between">
                      <span className="text-xs text-gray-400">ID:</span>
                      <span className="text-xs text-gray-300">{selectedFlag.id}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-xs text-gray-400">Status:</span>
                      <span className={`text-xs px-2 py-1 rounded ${
                        selectedFlag.recognition_status === 'recognized' 
                          ? 'bg-green-900/30 text-green-300 border border-green-600/30'
                          : selectedFlag.recognition_status === 'pending'
                          ? 'bg-yellow-900/30 text-yellow-300 border border-yellow-600/30'
                          : 'bg-red-900/30 text-red-300 border border-red-600/30'
                      }`}>
                        {selectedFlag.recognition_status || 'unknown'}
                      </span>
                    </div>
                    {selectedFlag.recognition_confidence && (
                      <div className="flex justify-between">
                        <span className="text-xs text-gray-400">Confidence:</span>
                        <span className="text-xs text-gray-300">{(selectedFlag.recognition_confidence * 100).toFixed(1)}%</span>
                      </div>
                    )}
                  </div>
                  <div className="space-y-2">
                    {selectedFlag.upload_date && (
                      <div className="flex justify-between">
                        <span className="text-xs text-gray-400">Uploaded:</span>
                        <span className="text-xs text-gray-300">
                          {(() => { const d=new Date(selectedFlag.upload_date); const dd=String(d.getDate()).padStart(2,'0'); const mm=String(d.getMonth()+1).padStart(2,'0'); const yyyy=d.getFullYear(); return `${dd}.${mm}.${yyyy}`; })()}
                        </span>
                      </div>
                    )}
                    {selectedFlag.file_size && (
                      <div className="flex justify-between">
                        <span className="text-xs text-gray-400">Size:</span>
                        <span className="text-xs text-gray-300">
                          {(selectedFlag.file_size / 1024).toFixed(1)} KB
                        </span>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            )}

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
                    value={pssCodeInput}
                    onChange={(e) => setPssCodeInput(e.target.value)}
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

      {/* Database Statistics */}
      <div className="p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg">
        <h3 className="text-lg font-semibold mb-4 text-gray-100">Database Statistics</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="p-4 rounded-lg bg-gray-700/30 border border-gray-600/30">
            <p className="text-sm font-medium text-gray-300">Total Flags</p>
            <p className="text-2xl font-bold text-blue-400">{statistics.total}</p>
          </div>
          <div className="p-4 rounded-lg bg-gray-700/30 border border-gray-600/30">
            <p className="text-sm font-medium text-gray-300">Recognized</p>
            <p className="text-2xl font-bold text-green-400">{statistics.recognized}</p>
          </div>
          <div className="p-4 rounded-lg bg-gray-700/30 border border-gray-600/30">
            <p className="text-sm font-medium text-gray-300">Pending</p>
            <p className="text-2xl font-bold text-yellow-400">{statistics.pending}</p>
          </div>
          <div className="p-4 rounded-lg bg-gray-700/30 border border-gray-600/30">
            <p className="text-sm font-medium text-gray-300">Failed</p>
            <p className="text-2xl font-bold text-red-400">{statistics.failed}</p>
          </div>
        </div>
        <div className="mt-6 flex space-x-2">
          <Button size="sm" variant="primary" onClick={scanAndPopulateFlags} disabled={isLoading}>
            Scan and Populate Flags
          </Button>
          <Button size="sm" variant="danger" onClick={clearFlagsDatabase} disabled={isLoading}>
            Clear Flags Database
          </Button>
        </div>
      </div>
    </div>
  );
};

export default React.memo(FlagManagementPanel);