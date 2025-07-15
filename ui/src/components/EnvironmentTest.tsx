import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { useEnvironment } from '../hooks/useEnvironment';
import { EnvironmentWrapper, WindowsOnly, WebOnly, FeatureWrapper } from './EnvironmentWrapper';
import { useEnvironmentApi, useEnvironmentObs, useEnvironmentFileSystem } from '../hooks/useEnvironment';

const EnvironmentTest: React.FC = () => {
  const { environment, isWindows, isWeb, config, getEnvironmentClass } = useEnvironment();
  const { apiCall } = useEnvironmentApi();
  const { obsOperation } = useEnvironmentObs();
  const { fileOperation } = useEnvironmentFileSystem();
  
  const [testResults, setTestResults] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const addResult = (result: string) => {
    setTestResults(prev => [...prev, `${new Date().toLocaleTimeString()}: ${result}`]);
  };

  const runEnvironmentTests = async () => {
    setIsLoading(true);
    setTestResults([]);
    
    try {
      // Test 1: Environment Detection
      addResult(`Environment detected: ${environment}`);
      addResult(`Is Windows: ${isWindows}`);
      addResult(`Is Web: ${isWeb}`);
      
      // Test 2: Configuration
      addResult(`API Base URL: ${config.api.baseUrl}`);
      addResult(`OBS Use Tauri: ${config.obs.useTauriCommands}`);
      addResult(`OBS Use WebSocket: ${config.obs.useWebSocketDirect}`);
      
      // Test 3: Feature Flags
      addResult(`Tauri Commands Available: ${config.features.tauriCommands}`);
      addResult(`WebSocket Direct Available: ${config.features.webSocketDirect}`);
      addResult(`Native File System Available: ${config.features.nativeFileSystem}`);
      
      // Test 4: API Call Test
      try {
        addResult('Testing API call...');
        if (isWindows) {
          await apiCall('test');
          addResult('✅ API call successful (Windows/Tauri)');
        } else {
          await apiCall('test');
          addResult('✅ API call successful (Web)');
        }
      } catch (error) {
        addResult(`❌ API call failed (expected in web mode): ${error}`);
      }
      
      // Test 5: OBS Operation Test
      try {
        addResult('Testing OBS operation...');
        if (isWindows) {
          await obsOperation('test');
          addResult('✅ OBS operation successful (Windows/Tauri)');
        } else {
          throw new Error('OBS operation not available in web mode');
        }
      } catch (error) {
        addResult(`❌ OBS operation failed (expected in web mode): ${error}`);
      }
      
      // Test 6: File Operation Test (Web only)
      if (isWeb) {
        try {
          addResult('Testing file operation...');
          await fileOperation('test');
          addResult('✅ File operation successful');
        } catch (error) {
          addResult(`❌ File operation failed (expected): ${error}`);
        }
      }
      
    } catch (error) {
      addResult(`❌ Test failed: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="bg-gray-900 rounded-lg p-6"
      >
        <h2 className="text-2xl font-bold mb-6 flex items-center space-x-3">
          <span>🧪</span>
          <span>Environment System Test</span>
          <span className={`px-3 py-1 text-sm rounded-full font-medium ${
            isWindows ? 'bg-blue-600 text-white' : 'bg-green-600 text-white'
          }`}>
            {environment.toUpperCase()}
          </span>
        </h2>

        {/* Environment Information */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
          <div className="bg-gray-800 rounded-lg p-4">
            <h3 className="text-lg font-semibold mb-3">Environment Details</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span>Environment:</span>
                <span className="font-mono">{environment}</span>
              </div>
              <div className="flex justify-between">
                <span>Is Windows:</span>
                <span className={isWindows ? 'text-green-400' : 'text-red-400'}>
                  {isWindows ? '✅ Yes' : '❌ No'}
                </span>
              </div>
              <div className="flex justify-between">
                <span>Is Web:</span>
                <span className={isWeb ? 'text-green-400' : 'text-red-400'}>
                  {isWeb ? '✅ Yes' : '❌ No'}
                </span>
              </div>
              <div className="flex justify-between">
                <span>Production:</span>
                <span className={config.isProduction ? 'text-green-400' : 'text-yellow-400'}>
                  {config.isProduction ? '✅ Yes' : '❌ No'}
                </span>
              </div>
            </div>
          </div>

          <div className="bg-gray-800 rounded-lg p-4">
            <h3 className="text-lg font-semibold mb-3">Configuration</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span>API Base URL:</span>
                <span className="font-mono text-xs">{config.api.baseUrl}</span>
              </div>
              <div className="flex justify-between">
                <span>OBS Tauri Commands:</span>
                <span className={config.obs.useTauriCommands ? 'text-green-400' : 'text-red-400'}>
                  {config.obs.useTauriCommands ? '✅ Enabled' : '❌ Disabled'}
                </span>
              </div>
              <div className="flex justify-between">
                <span>OBS WebSocket Direct:</span>
                <span className={config.obs.useWebSocketDirect ? 'text-green-400' : 'text-red-400'}>
                  {config.obs.useWebSocketDirect ? '✅ Enabled' : '❌ Disabled'}
                </span>
              </div>
              <div className="flex justify-between">
                <span>Dev Port:</span>
                <span className="font-mono">{config.dev.port}</span>
              </div>
            </div>
          </div>
        </div>

        {/* Feature Flags */}
        <div className="bg-gray-800 rounded-lg p-4 mb-8">
          <h3 className="text-lg font-semibold mb-3">Feature Flags</h3>
          <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
            {Object.entries(config.features).map(([feature, enabled]) => (
              <div key={feature} className="flex items-center space-x-2">
                <span className={enabled ? 'text-green-400' : 'text-red-400'}>
                  {enabled ? '✅' : '❌'}
                </span>
                <span className="text-sm font-mono">{feature}</span>
              </div>
            ))}
          </div>
        </div>

        {/* Environment-Specific Components */}
        <div className="bg-gray-800 rounded-lg p-4 mb-8">
          <h3 className="text-lg font-semibold mb-3">Environment-Specific Components</h3>
          <div className="space-y-4">
            <EnvironmentWrapper>
              <div className="p-3 bg-blue-900 rounded border border-blue-700">
                <p className="text-blue-200">✅ This renders in ALL environments</p>
              </div>
            </EnvironmentWrapper>

            <WindowsOnly>
              <div className="p-3 bg-blue-900 rounded border border-blue-700">
                <p className="text-blue-200">🪟 This only renders in WINDOWS environment</p>
                <p className="text-blue-300 text-sm">You can see this because you're in Windows mode</p>
              </div>
            </WindowsOnly>

            <WebOnly>
              <div className="p-3 bg-green-900 rounded border border-green-700">
                <p className="text-green-200">🌐 This only renders in WEB environment</p>
                <p className="text-green-300 text-sm">You can see this because you're in Web mode</p>
              </div>
            </WebOnly>

            <WindowsOnly fallback={
              <div className="p-3 bg-yellow-900 rounded border border-yellow-700">
                <p className="text-yellow-200">⚠️ Windows feature not available (Web fallback)</p>
              </div>
            }>
              <div className="p-3 bg-blue-900 rounded border border-blue-700">
                <p className="text-blue-200">🪟 Windows-specific feature with fallback</p>
              </div>
            </WindowsOnly>
          </div>
        </div>

        {/* Feature Wrappers */}
        <div className="bg-gray-800 rounded-lg p-4 mb-8">
          <h3 className="text-lg font-semibold mb-3">Feature-Based Rendering</h3>
          <div className="space-y-4">
            <FeatureWrapper feature="tauriCommands">
              <div className="p-3 bg-purple-900 rounded border border-purple-700">
                <p className="text-purple-200">🔧 Tauri Commands Feature Available</p>
              </div>
            </FeatureWrapper>

            <FeatureWrapper feature="webSocketDirect">
              <div className="p-3 bg-orange-900 rounded border border-orange-700">
                <p className="text-orange-200">🔌 WebSocket Direct Feature Available</p>
              </div>
            </FeatureWrapper>

            <FeatureWrapper feature="nativeFileSystem">
              <div className="p-3 bg-indigo-900 rounded border border-indigo-700">
                <p className="text-indigo-200">📁 Native File System Feature Available</p>
              </div>
            </FeatureWrapper>
          </div>
        </div>

        {/* Test Controls */}
        <div className="bg-gray-800 rounded-lg p-4 mb-8">
          <h3 className="text-lg font-semibold mb-3">Test Controls</h3>
          <div className="space-y-4">
            <button
              onClick={runEnvironmentTests}
              disabled={isLoading}
              className={`px-4 py-2 rounded-lg font-medium transition-colors ${
                isLoading
                  ? 'bg-gray-600 text-gray-400 cursor-not-allowed'
                  : 'bg-blue-600 hover:bg-blue-700 text-white'
              }`}
            >
              {isLoading ? 'Running Tests...' : 'Run Environment Tests'}
            </button>

            <button
              onClick={() => setTestResults([])}
              className="px-4 py-2 rounded-lg font-medium bg-gray-600 hover:bg-gray-700 text-white transition-colors"
            >
              Clear Results
            </button>
          </div>
        </div>

        {/* Test Results */}
        {testResults.length > 0 && (
          <div className="bg-gray-800 rounded-lg p-4">
            <h3 className="text-lg font-semibold mb-3">Test Results</h3>
            <div className="bg-black rounded p-3 max-h-64 overflow-y-auto">
              {testResults.map((result, index) => (
                <div key={index} className="text-sm font-mono text-green-400 mb-1">
                  {result}
                </div>
              ))}
            </div>
          </div>
        )}
      </motion.div>
    </div>
  );
};

export default EnvironmentTest; 