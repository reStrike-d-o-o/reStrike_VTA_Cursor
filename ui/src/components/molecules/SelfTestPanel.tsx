import React, { useState, useEffect } from 'react';
import Button from '../atoms/Button';
import Label from '../atoms/Label';
import StatusDot from '../atoms/StatusDot';
import Icon from '../atoms/Icon';
import Toggle from '../atoms/Toggle';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';

// Use the proper Tauri v2 invoke function with fallback
const invoke = async (command: string, args?: any) => {
  try {
    return await tauriInvoke(command, args);
  } catch (error) {
    if (typeof window !== 'undefined' && (window as any).__TAURI__ && (window as any).__TAURI__.core) {
      return await (window as any).__TAURI__.core.invoke(command, args);
    }
    throw new Error('Tauri v2 core module not available');
  }
};

interface SelfTestResult {
  overall_status: string;
  summary: {
    total_tests: number;
    passed: number;
    failed: number;
    warnings: number;
    skipped: number;
    success_rate: number;
  };
  duration: number;
  start_time: string;
  end_time: string;
  categories: Record<string, any>;
}

interface SelfTestPanelProps {
  className?: string;
}

const SelfTestPanel: React.FC<SelfTestPanelProps> = ({ className = '' }) => {
  const [isRunning, setIsRunning] = useState(false);
  const [testResult, setTestResult] = useState<SelfTestResult | null>(null);
  const [reportContent, setReportContent] = useState<string>('');
  const [error, setError] = useState<string>('');
  const [success, setSuccess] = useState<string>('');
  const [isInstallingDeps, setIsInstallingDeps] = useState(false);
  
  // Selective testing state
  const [showSelective, setShowSelective] = useState(false);
  const [availableCategories, setAvailableCategories] = useState<string[]>([]);
  const [selectedCategories, setSelectedCategories] = useState<string[]>([]);
  const [isLoadingCategories, setIsLoadingCategories] = useState(false);

  // Load available categories
  const loadCategories = async () => {
    try {
      setIsLoadingCategories(true);
      const result = await invoke('simulation_get_self_test_categories');
      
      if (result.success && result.data.categories && result.data.categories.length > 0) {
        setAvailableCategories(result.data.categories);
        // Select all categories by default
        setSelectedCategories(result.data.categories);
      } else {
        // Fallback categories if backend fails
        const fallbackCategories = [
          'Backend Services',
          'Frontend Integration', 
          'Simulation System',
          'Data Flow',
          'UI Components',
          'Performance'
        ];
        setAvailableCategories(fallbackCategories);
        setSelectedCategories(fallbackCategories);
        setError('Using fallback categories - backend categories failed to load');
      }
    } catch (error) {
      // Fallback categories if backend fails
      const fallbackCategories = [
        'Backend Services',
        'Frontend Integration', 
        'Simulation System',
        'Data Flow',
        'UI Components',
        'Performance'
      ];
      setAvailableCategories(fallbackCategories);
      setSelectedCategories(fallbackCategories);
      setError(`Using fallback categories - failed to load: ${error}`);
    } finally {
      setIsLoadingCategories(false);
    }
  };

  // Run comprehensive self-test
  const runSelfTest = async () => {
    try {
      setIsRunning(true);
      setError('');
      setSuccess('');
      setTestResult(null);
      setReportContent('');

      let result;
      if (showSelective && selectedCategories.length > 0) {
        result = await invoke('simulation_run_selective_self_test', { selectedCategories });
      } else {
        result = await invoke('simulation_run_self_test');
      }

      if (result.success) {
        setSuccess(showSelective ? 'Selective self-test completed successfully!' : 'Self-test completed successfully!');
        
        // Get the detailed report
        const reportResult = await invoke('simulation_get_self_test_report');
        if (reportResult.success) {
          setReportContent(reportResult.data.report);
        }
        
        // Parse the output to extract test results
        const output = result.data.output;
        const lines = output.split('\n');
        
        // Extract summary information
        let overallStatus = 'UNKNOWN';
        let totalTests = 0;
        let passed = 0;
        let failed = 0;
        let warnings = 0;
        let successRate = 0;
        let duration = 0;

        for (const line of lines) {
          if (line.includes('Overall Status:')) {
            overallStatus = line.split('Overall Status:')[1]?.trim() || 'UNKNOWN';
          } else if (line.includes('Total Tests:')) {
            totalTests = parseInt(line.split('Total Tests:')[1]?.trim() || '0');
          } else if (line.includes('Passed:')) {
            passed = parseInt(line.split('Passed:')[1]?.split('✅')[0]?.trim() || '0');
          } else if (line.includes('Failed:')) {
            failed = parseInt(line.split('Failed:')[1]?.split('❌')[0]?.trim() || '0');
          } else if (line.includes('Warnings:')) {
            warnings = parseInt(line.split('Warnings:')[1]?.split('⚠️')[0]?.trim() || '0');
          } else if (line.includes('Success Rate:')) {
            successRate = parseFloat(line.split('Success Rate:')[1]?.split('%')[0]?.trim() || '0');
          } else if (line.includes('Duration:')) {
            duration = parseFloat(line.split('Duration:')[1]?.split('seconds')[0]?.trim() || '0');
          }
        }

        setTestResult({
          overall_status: overallStatus,
          summary: {
            total_tests: totalTests,
            passed,
            failed,
            warnings,
            skipped: 0,
            success_rate: successRate
          },
          duration,
          start_time: new Date().toISOString(),
          end_time: new Date().toISOString(),
          categories: {}
        });
      } else {
        setError(result.error || 'Self-test failed');
      }
    } catch (error) {
      setError(`Failed to run self-test: ${error}`);
    } finally {
      setIsRunning(false);
    }
  };

  // Toggle category selection
  const toggleCategory = (category: string) => {
    setSelectedCategories(prev => {
      if (prev.includes(category)) {
        return prev.filter(c => c !== category);
      } else {
        return [...prev, category];
      }
    });
  };

  // Select all categories
  const selectAllCategories = () => {
    setSelectedCategories([...availableCategories]);
  };

  // Deselect all categories
  const deselectAllCategories = () => {
    setSelectedCategories([]);
  };

  // Helper function to check if error is a simulation environment error
  const isSimulationEnvError = (errorMsg: string): boolean => {
    return errorMsg.includes('Simulation environment error') || 
           errorMsg.includes('PythonNotFound') ||
           errorMsg.includes('PythonVersionTooLow') ||
           errorMsg.includes('PipInstallFailed') ||
           errorMsg.includes('DependencyCheckFailed') ||
           errorMsg.includes('SimulationPathNotFound');
  };

  // Helper function to get user-friendly error message
  const getFriendlyErrorMessage = (errorMsg: string): string => {
    if (errorMsg.includes('PythonNotFound')) {
      return 'Python is not installed or not found in PATH. Please install Python 3.8 or higher.';
    }
    if (errorMsg.includes('PythonVersionTooLow')) {
      return 'Python version is too low. Please install Python 3.8 or higher.';
    }
    if (errorMsg.includes('PipInstallFailed')) {
      return 'Failed to install Python dependencies. Please check your internet connection and try again.';
    }
    if (errorMsg.includes('DependencyCheckFailed')) {
      return 'Required Python packages are missing. Click "Install Dependencies" to fix this.';
    }
    if (errorMsg.includes('SimulationPathNotFound')) {
      return 'Simulation files not found. Please reinstall the application.';
    }
    return errorMsg;
  };

  // Retry function that attempts to reload categories
  const retrySelfTest = async () => {
    setError('');
    setSuccess('');
    await loadCategories();
  };

  // Install dependencies function
  const installDependencies = async () => {
    try {
      setIsInstallingDeps(true);
      setError('');
      setSuccess('');
      
      // Try to trigger dependency installation by calling a simulation command
      const result = await invoke('simulation_get_self_test_categories');
      
      if (result.success) {
        setSuccess('Dependencies installed successfully!');
        await loadCategories();
      } else {
        setError(result.error || 'Failed to install dependencies');
      }
    } catch (error) {
      setError(`Failed to install dependencies: ${error}`);
    } finally {
      setIsInstallingDeps(false);
    }
  };

  // Load categories on component mount
  useEffect(() => {
    loadCategories();
  }, []);

  // Clear messages after 5 seconds
  useEffect(() => {
    if (success || error) {
      const timer = setTimeout(() => {
        setSuccess('');
        setError('');
      }, 5000);
      return () => clearTimeout(timer);
    }
  }, [success, error]);

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'PASSED':
        return 'text-green-400';
      case 'FAILED':
        return 'text-red-400';
      case 'WARNING':
        return 'text-yellow-400';
      default:
        return 'text-gray-400';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'PASSED':
        return '✅';
      case 'FAILED':
        return '❌';
      case 'WARNING':
        return '⚠️';
      default:
        return '❓';
    }
  };

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-3">
          <Icon name="🧪" className="w-6 h-6 text-purple-400" />
          <h3 className="text-lg font-semibold text-gray-200">System Self-Test</h3>
        </div>
        <div className="flex items-center space-x-2">
          <StatusDot color={isRunning ? 'bg-yellow-500' : testResult ? 'bg-green-500' : 'bg-gray-500'} size="w-3 h-3" />
          <span className="text-xs text-gray-400 leading-none">{isRunning ? 'Testing...' : testResult ? 'Ready' : 'Not Tested'}</span>
        </div>
      </div>

      {/* Status Messages */}
      {error && (
        <div className={`rounded-lg p-3 ${
          error.includes('fallback') 
            ? 'bg-yellow-900/20 border border-yellow-500/50' 
            : 'bg-red-900/20 border border-red-500/50'
        }`}>
          <p className={`text-sm mb-2 ${
            error.includes('fallback') ? 'text-yellow-400' : 'text-red-400'
          }`}>
            {isSimulationEnvError(error) ? getFriendlyErrorMessage(error) : error}
          </p>
          {isSimulationEnvError(error) && (
            <div className="flex space-x-2">
              <Button
                variant="outline"
                size="sm"
                onClick={retrySelfTest}
                disabled={isRunning || isInstallingDeps}
              >
                Retry
              </Button>
              {(error.includes('DependencyCheckFailed') || error.includes('PipInstallFailed')) && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={installDependencies}
                  disabled={isRunning || isInstallingDeps}
                >
                  {isInstallingDeps ? 'Installing...' : 'Install Dependencies'}
                </Button>
              )}
            </div>
          )}
        </div>
      )}
      {success && (
        <div className="bg-green-900/20 border border-green-500/50 rounded-lg p-3">
          <p className="text-green-400 text-sm">{success}</p>
        </div>
      )}

      {/* Selective Testing Toggle */}
      <div className="flex items-center justify-between">
        <Label>Selective Testing</Label>
        <Toggle
          checked={showSelective}
          onChange={(e) => setShowSelective(e.target.checked)}
          disabled={isRunning}
        />
      </div>

      {/* Category Selection */}
      {showSelective && (
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <Label>Test Categories</Label>
            <div className="flex space-x-2">
              <Button
                variant="outline"
                size="sm"
                onClick={selectAllCategories}
                disabled={isRunning || isLoadingCategories}
              >
                Select All
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={deselectAllCategories}
                disabled={isRunning || isLoadingCategories}
              >
                Deselect All
              </Button>
            </div>
          </div>
          
          {isLoadingCategories ? (
            <div className="flex items-center justify-center py-4">
              <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-500"></div>
              <span className="ml-2 text-sm text-gray-400">Loading categories...</span>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {availableCategories.map((category) => (
                <div key={category} className="flex items-center space-x-3">
                  <Toggle checked={selectedCategories.includes(category)} onChange={() => toggleCategory(category)} disabled={isRunning} />
                  <span className="text-sm text-gray-300 leading-none">{category}</span>
                </div>
              ))}
            </div>
          )}
          
          {selectedCategories.length === 0 && (
            <div className="text-center py-2 text-sm text-yellow-400">
              ⚠️ No categories selected. Please select at least one category to run tests.
            </div>
          )}
        </div>
      )}

      {/* Test Results Summary */}
      {testResult && (
        <div className="bg-gray-800/50 border border-gray-600 rounded-lg p-4">
          <div className="flex items-center justify-between mb-4">
            <h4 className="text-md font-semibold text-gray-200">Test Results</h4>
            <span className={`text-sm font-medium ${getStatusColor(testResult.overall_status)}`}>
              {getStatusIcon(testResult.overall_status)} {testResult.overall_status}
            </span>
          </div>
          
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center">
              <div className="text-2xl font-bold text-blue-400">{testResult.summary.total_tests}</div>
              <div className="text-xs text-gray-400">Total Tests</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-green-400">{testResult.summary.passed}</div>
              <div className="text-xs text-gray-400">Passed</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-red-400">{testResult.summary.failed}</div>
              <div className="text-xs text-gray-400">Failed</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-yellow-400">{testResult.summary.warnings}</div>
              <div className="text-xs text-gray-400">Warnings</div>
            </div>
          </div>
          
          <div className="mt-4 pt-4 border-t border-gray-600">
            <div className="flex justify-between items-center">
              <span className="text-sm text-gray-400">Success Rate</span>
              <span className="text-lg font-semibold text-green-400">
                {testResult.summary.success_rate.toFixed(1)}%
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm text-gray-400">Duration</span>
              <span className="text-sm text-gray-300">
                {testResult.duration.toFixed(2)}s
              </span>
            </div>
          </div>
        </div>
      )}

      {/* Control Button (no icon) */}
      <div className="flex">
        <Button variant="primary" size="sm" onClick={runSelfTest} disabled={isRunning || (showSelective && selectedCategories.length === 0)} className="flex-1">
          {isRunning ? 'Running Tests...' : (showSelective ? 'Run Selective Test' : 'Run Self-Test')}
        </Button>
      </div>

      {/* Detailed Report */}
      {reportContent && (
        <div className="space-y-3">
          <Label>Detailed Test Report</Label>
          <div className="bg-gray-900 border border-gray-600 rounded-lg p-4 max-h-96 overflow-y-auto">
            <pre className="text-sm text-gray-300 whitespace-pre-wrap font-mono">
              {reportContent}
            </pre>
          </div>
          <div className="flex justify-between items-center text-xs text-gray-400">
            <span>Markdown format report</span>
            <span>Scroll to view full report</span>
          </div>
        </div>
      )}

      {/* Test Categories Info */}
      <div className="bg-gray-800/30 border border-gray-600 rounded-lg p-4">
        <h4 className="text-md font-semibold text-gray-200 mb-3">Test Categories</h4>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-sm">
          <div className="flex items-center space-x-2">
            <Icon name="🖥️" className="w-4 h-4 text-blue-400" />
            <span className="text-gray-300">Backend Services</span>
          </div>
          <div className="flex items-center space-x-2">
            <Icon name="🖥️" className="w-4 h-4 text-green-400" />
            <span className="text-gray-300">Frontend Integration</span>
          </div>
          <div className="flex items-center space-x-2">
            <Icon name="🤖" className="w-4 h-4 text-purple-400" />
            <span className="text-gray-300">Simulation System</span>
          </div>
          <div className="flex items-center space-x-2">
            <Icon name="🗄️" className="w-4 h-4 text-yellow-400" />
            <span className="text-gray-300">Data Flow</span>
          </div>
          <div className="flex items-center space-x-2">
            <Icon name="📱" className="w-4 h-4 text-pink-400" />
            <span className="text-gray-300">UI Components</span>
          </div>
          <div className="flex items-center space-x-2">
            <Icon name="📊" className="w-4 h-4 text-orange-400" />
            <span className="text-gray-300">Performance</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SelfTestPanel; 