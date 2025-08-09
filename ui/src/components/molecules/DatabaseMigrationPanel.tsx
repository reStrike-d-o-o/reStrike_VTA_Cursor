import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import Button from '../atoms/Button';

interface MigrationStatus {
  database_enabled: boolean;
  migration_completed: boolean;
  backup_created: boolean;
  json_settings_count: number;
  database_settings_count: number;
}

interface TableData {
  table_name: string;
  columns: Array<{ name: string; type: string; not_null: boolean; primary_key: boolean }>;
  rows: Array<Record<string, any>>;
  row_count: number;
}

const DatabaseMigrationPanel: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'migration' | 'status' | 'preview'>('migration');
  const [migrationStatus, setMigrationStatus] = useState<MigrationStatus | null>(null);
  const [databaseTables, setDatabaseTables] = useState<string[]>([]);
  const [selectedTable, setSelectedTable] = useState<string>('');
  const [tableData, setTableData] = useState<TableData | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Refs for scroll synchronization
  const headerScrollRef = useRef<HTMLDivElement>(null);
  const bodyScrollRef = useRef<HTMLDivElement>(null);

  // Scroll synchronization functions
  const handleHeaderScroll = () => {
    if (headerScrollRef.current && bodyScrollRef.current) {
      bodyScrollRef.current.scrollLeft = headerScrollRef.current.scrollLeft;
    }
  };

  const handleBodyScroll = () => {
    if (headerScrollRef.current && bodyScrollRef.current) {
      headerScrollRef.current.scrollLeft = bodyScrollRef.current.scrollLeft;
    }
  };

  useEffect(() => {
    loadMigrationStatus();
    loadDatabaseTables();
  }, []);

  const loadMigrationStatus = async () => {
    try {
      const result = await invoke<{
        success: boolean;
        status?: {
          database_enabled: boolean;
          migration_completed: boolean;
          backup_created: boolean;
          json_settings_count: number;
          database_settings_count: number;
        };
        error?: string;
      }>('get_migration_status');
      
      if (result.success && result.status) {
        setMigrationStatus(result.status);
      } else {
        console.error('Failed to load migration status:', result.error);
        setError(result.error || 'Failed to load migration status');
        setMigrationStatus(null);
      }
    } catch (error) {
      console.error('Failed to load migration status:', error);
      setError('Failed to load migration status');
      setMigrationStatus(null);
    }
  };

  const loadDatabaseTables = async () => {
    try {
      const result = await invoke<{ success: boolean; tables?: string[]; error?: string }>('get_database_tables');
      if (result.success && result.tables) {
        setDatabaseTables(result.tables);
      } else {
        console.error('Failed to load database tables:', result.error);
        setError(result.error || 'Failed to load database tables');
        setDatabaseTables([]); // Ensure it's always an array
      }
    } catch (error) {
      console.error('Failed to load database tables:', error);
      setError('Failed to load database tables');
      setDatabaseTables([]); // Ensure it's always an array
    }
  };

  const loadTableData = async (tableName: string) => {
    try {
      const result = await invoke<{
        success: boolean;
        table_name?: string;
        columns?: Array<{ name: string; type: string; not_null: boolean; primary_key: boolean }>;
        rows?: Array<Record<string, any>>;
        row_count?: number;
        error?: string;
      }>('get_table_data', { tableName });
      
      if (result.success && result.table_name && result.columns && result.rows !== undefined) {
        setTableData({
          table_name: result.table_name,
          columns: result.columns,
          rows: result.rows,
          row_count: result.row_count || result.rows.length
        });
      } else {
        console.error('Failed to load table data:', result.error);
        setError(result.error || 'Failed to load table data');
        setTableData(null);
      }
    } catch (error) {
      console.error('Failed to load table data:', error);
      setError('Failed to load table data');
      setTableData(null);
    }
  };

  const handleTableSelect = (tableName: string) => {
    setSelectedTable(tableName);
    loadTableData(tableName);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-white">Database Migration</h2>
          <p className="text-sm text-gray-400">Manage database migration and view data</p>
        </div>
      </div>

      {/* Error/Success Messages */}
      {error && (
        <div className="bg-red-900/20 border border-red-500/50 rounded-lg p-3">
          <span className="text-red-400 font-medium">Error</span>
          <p className="text-red-300 mt-1 text-sm">{error}</p>
        </div>
      )}

      {success && (
        <div className="bg-green-900/20 border border-green-500/50 rounded-lg p-3">
          <span className="text-green-400 font-medium">Success</span>
          <p className="text-green-300 mt-1 text-sm">{success}</p>
        </div>
      )}

      {/* Tab Navigation */}
      <div className="flex border-b border-gray-200">
        <button
          onClick={() => setActiveTab('migration')}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors flex items-center gap-2 ${
            activeTab === 'migration'
              ? 'border-blue-500 text-blue-600'
              : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
          }`}
        >
          <span>🔄</span>
          Migration
        </button>
        <button
          onClick={() => setActiveTab('status')}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors flex items-center gap-2 ${
            activeTab === 'status'
              ? 'border-blue-500 text-blue-600'
              : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
          }`}
        >
          <span>📊</span>
          Status
        </button>
        <button
          onClick={() => setActiveTab('preview')}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors flex items-center gap-2 ${
            activeTab === 'preview'
              ? 'border-blue-500 text-blue-600'
              : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
          }`}
        >
          <span>👁️</span>
          Preview
        </button>
      </div>

      {/* Migration Tab */}
      {activeTab === 'migration' && (
        <div className="theme-card rounded-lg p-4 shadow-lg">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-lg font-semibold text-blue-300">Migration Status</h3>
            <Button
              onClick={loadMigrationStatus}
              variant="secondary"
              size="sm"
            >
              Refresh
            </Button>
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between p-3 bg-[#101820] rounded border border-gray-700">
              <div>
                <span className="text-gray-300 font-medium">Database Enabled</span>
                <p className="text-sm text-gray-400">Current database mode status</p>
              </div>
              <div className={`px-3 py-1 rounded-full text-sm font-medium ${
                migrationStatus?.database_enabled 
                  ? 'bg-green-900 text-green-300' 
                  : 'bg-yellow-900 text-yellow-300'
              }`}>
                {migrationStatus?.database_enabled ? 'Enabled' : 'Disabled'}
              </div>
            </div>

            <div className="flex items-center justify-between p-3 bg-[#101820] rounded border border-gray-700">
              <div>
                <span className="text-gray-300 font-medium">Migration Completed</span>
                <p className="text-sm text-gray-400">Status of data migration from JSON to database</p>
              </div>
              <div className={`px-3 py-1 rounded-full text-sm font-medium ${
                migrationStatus?.migration_completed 
                  ? 'bg-green-900 text-green-300' 
                  : 'bg-yellow-900 text-yellow-300'
              }`}>
                {migrationStatus?.migration_completed ? 'Completed' : 'Pending'}
              </div>
            </div>

            <div className="flex items-center justify-between p-3 bg-[#101820] rounded border border-gray-700">
              <div>
                <span className="text-gray-300 font-medium">Backup Created</span>
                <p className="text-sm text-gray-400">Status of JSON backup creation</p>
              </div>
              <div className={`px-3 py-1 rounded-full text-sm font-medium ${
                migrationStatus?.backup_created 
                  ? 'bg-green-900 text-green-300' 
                  : 'bg-yellow-900 text-yellow-300'
              }`}>
                {migrationStatus?.backup_created ? 'Created' : 'Not Created'}
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="p-3 bg-[#101820] rounded border border-gray-700">
                <span className="text-gray-300 font-medium">JSON Settings</span>
                <p className="text-2xl font-bold text-blue-400">{migrationStatus?.json_settings_count || 0}</p>
              </div>
              <div className="p-3 bg-[#101820] rounded border border-gray-700">
                <span className="text-gray-300 font-medium">Database Settings</span>
                <p className="text-2xl font-bold text-green-400">{migrationStatus?.database_settings_count || 0}</p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Status Tab */}
      {activeTab === 'status' && (
        <div className="theme-card rounded-lg p-4 shadow-lg">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-lg font-semibold text-blue-300">Database Status</h3>
            <Button
              onClick={loadMigrationStatus}
              variant="secondary"
              size="sm"
            >
              Refresh
            </Button>
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between p-3 bg-[#101820] rounded border border-gray-700">
              <div>
                <span className="text-gray-300 font-medium">Database Connection</span>
                <p className="text-sm text-gray-400">Connection status to SQLite database</p>
              </div>
              <div className="px-3 py-1 rounded-full text-sm font-medium bg-green-900 text-green-300">
                Connected
              </div>
            </div>

            <div className="flex items-center justify-between p-3 bg-[#101820] rounded border border-gray-700">
              <div>
                <span className="text-gray-300 font-medium">Tables Count</span>
                <p className="text-sm text-gray-400">Number of tables in database</p>
              </div>
              <div className="px-3 py-1 rounded-full text-sm font-medium bg-blue-900 text-blue-300">
                {databaseTables.length}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Preview Tab */}
      {activeTab === 'preview' && (
        <div className="theme-card rounded-lg p-4 shadow-lg">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-lg font-semibold text-blue-300">Data Preview</h3>
            <div className="flex gap-2">
              <select
                value={selectedTable}
                onChange={(e) => handleTableSelect(e.target.value)}
                className="px-3 py-1 bg-[#101820] border border-gray-700 rounded text-gray-300 text-sm"
                aria-label="Select database table to preview"
              >
                <option value="">Select a table</option>
                {(databaseTables || []).map(table => (
                  <option key={table} value={table}>{table}</option>
                ))}
              </select>
            </div>
          </div>

          {selectedTable && tableData && (
            <div className="space-y-4">
              <div className="p-3 bg-[#101820] rounded border border-gray-700">
                <span className="text-gray-300 font-medium">Table: {tableData.table_name}</span>
                <p className="text-sm text-gray-400">Rows: {tableData.row_count}</p>
              </div>

              <div className="w-full">
                <div className="w-[1360px] border border-gray-700 rounded overflow-hidden">
                  {/* Header Table - Fixed, only horizontal scroll */}
                  <div 
                    className="overflow-x-auto" 
                    ref={headerScrollRef}
                    onScroll={handleHeaderScroll}
                  >
                    <table className="min-w-full text-left text-sm text-gray-200">
                      <thead className="bg-[#101820]">
                        <tr>
                          {(tableData.columns || []).map(column => (
                            <th key={column.name} className="px-3 py-2 font-semibold whitespace-nowrap border-b border-gray-700">
                              {column.name}
                              {column.primary_key && <span className="text-blue-400 ml-1">🔑</span>}
                              {column.not_null && <span className="text-red-400 ml-1">*</span>}
                            </th>
                          ))}
                        </tr>
                      </thead>
                    </table>
                  </div>
                  
                  {/* Body Table - Vertical scroll with horizontal scroll sync */}
                  <div 
                    className="max-h-64 overflow-y-auto overflow-x-auto" 
                    ref={bodyScrollRef}
                    onScroll={handleBodyScroll}
                  >
                    <table className="min-w-full text-left text-sm text-gray-200">
                      <tbody>
                        {(tableData.rows || []).map((row, index) => (
                          <tr key={index} className="hover:bg-blue-900 transition-colors">
                            {(tableData.columns || []).map(column => (
                              <td key={column.name} className="px-3 py-2 whitespace-nowrap border-b border-gray-700/30">
                                {row[column.name] !== null && row[column.name] !== undefined 
                                  ? String(row[column.name]) 
                                  : <span className="text-gray-500">null</span>
                                }
                              </td>
                            ))}
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                  
                  {tableData.rows && tableData.rows.length > 0 && (
                    <p className="text-xs text-gray-400 mt-2 px-3">
                      Showing all {tableData.rows.length} rows
                    </p>
                  )}
                </div>
              </div>
            </div>
          )}

          {selectedTable && !tableData && (
            <div className="text-center py-8 text-gray-400">
              Loading table data...
            </div>
          )}

          {!selectedTable && (
            <div className="text-center py-8 text-gray-400">
              Select a table to preview its data
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default DatabaseMigrationPanel; 