# Flag Management Module - Technical Specification

## Database Implementation

### SQLite Database Setup
```rust
// src-tauri/src/flag_manager/database.rs
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct FlagRecord {
    pub id: Option<i64>,
    pub filename: String,
    pub ioc_code: Option<String>,
    pub country_name: Option<String>,
    pub recognition_status: String,
    pub recognition_confidence: Option<f64>,
    pub upload_date: String,
    pub last_modified: String,
    pub file_size: i64,
    pub file_path: String,
    pub is_recognized: bool,
}

pub struct FlagDatabase {
    conn: Connection,
}

impl FlagDatabase {
    pub fn new(db_path: &PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = FlagDatabase { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        // Create flags table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS flags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL,
                ioc_code TEXT,
                country_name TEXT,
                recognition_status TEXT DEFAULT 'pending',
                recognition_confidence REAL,
                upload_date DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_modified DATETIME DEFAULT CURRENT_TIMESTAMP,
                file_size INTEGER,
                file_path TEXT NOT NULL,
                is_recognized BOOLEAN DEFAULT FALSE
            )",
            [],
        )?;

        // Create recognition_history table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS recognition_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                flag_id INTEGER,
                recognition_method TEXT,
                confidence REAL,
                recognized_as TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (flag_id) REFERENCES flags(id)
            )",
            [],
        )?;

        // Create settings table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        Ok(())
    }
}
```

## Frontend Component Specifications

### 1. FlagManager.tsx (Main Component)
```typescript
// ui/src/modules/FlagManager/components/FlagManager.tsx
import React, { useState, useEffect } from 'react';
import { FlagList } from './FlagList';
import { BulkUpload } from './BulkUpload';
import { RecognitionToggle } from './RecognitionToggle';
import { RecognitionReport } from './RecognitionReport';
import { useFlagDatabase } from '../hooks/useFlagDatabase';
import { useRecognition } from '../hooks/useRecognition';

interface FlagManagerProps {
  className?: string;
}

export const FlagManager: React.FC<FlagManagerProps> = ({ className }) => {
  const [activeTab, setActiveTab] = useState<'list' | 'upload' | 'report'>('list');
  const [autoRecognition, setAutoRecognition] = useState(false);
  
  const { flags, loading, error, refreshFlags } = useFlagDatabase();
  const { recognizeFlags, recognitionResults } = useRecognition();

  const stats = {
    total: flags.length,
    recognized: flags.filter(f => f.is_recognized).length,
    pending: flags.filter(f => !f.is_recognized).length,
  };

  return (
    <div className={`flag-manager ${className}`}>
      <div className="flag-manager-header">
        <h2>Flag Management</h2>
        <RecognitionToggle 
          enabled={autoRecognition}
          onChange={setAutoRecognition}
        />
      </div>

      <div className="flag-manager-stats">
        <div className="stat">
          <span className="stat-label">Total Flags</span>
          <span className="stat-value">{stats.total}</span>
        </div>
        <div className="stat">
          <span className="stat-label">Recognized</span>
          <span className="stat-value">{stats.recognized}</span>
        </div>
        <div className="stat">
          <span className="stat-label">Pending</span>
          <span className="stat-value">{stats.pending}</span>
        </div>
      </div>

      <div className="flag-manager-tabs">
        <button 
          className={`tab ${activeTab === 'list' ? 'active' : ''}`}
          onClick={() => setActiveTab('list')}
        >
          Flag List
        </button>
        <button 
          className={`tab ${activeTab === 'upload' ? 'active' : ''}`}
          onClick={() => setActiveTab('upload')}
        >
          Bulk Upload
        </button>
        <button 
          className={`tab ${activeTab === 'report' ? 'active' : ''}`}
          onClick={() => setActiveTab('report')}
        >
          Recognition Report
        </button>
      </div>

      <div className="flag-manager-content">
        {activeTab === 'list' && (
          <FlagList 
            flags={flags}
            loading={loading}
            onRefresh={refreshFlags}
            onRecognize={recognizeFlags}
          />
        )}
        {activeTab === 'upload' && (
          <BulkUpload 
            onUploadComplete={refreshFlags}
            autoRecognition={autoRecognition}
          />
        )}
        {activeTab === 'report' && (
          <RecognitionReport 
            results={recognitionResults}
          />
        )}
      </div>
    </div>
  );
};
```

### 2. FlagList.tsx (Flag Grid Component)
```typescript
// ui/src/modules/FlagManager/components/FlagList.tsx
import React, { useState } from 'react';
import { FlagRecord } from '../types/flagTypes';
import { RenameDialog } from './RenameDialog';

interface FlagListProps {
  flags: FlagRecord[];
  loading: boolean;
  onRefresh: () => void;
  onRecognize: (flagIds: number[]) => void;
}

export const FlagList: React.FC<FlagListProps> = ({
  flags,
  loading,
  onRefresh,
  onRecognize,
}) => {
  const [selectedFlags, setSelectedFlags] = useState<number[]>([]);
  const [renameDialog, setRenameDialog] = useState<{
    open: boolean;
    flag: FlagRecord | null;
  }>({ open: false, flag: null });

  const handleFlagSelect = (flagId: number) => {
    setSelectedFlags(prev => 
      prev.includes(flagId) 
        ? prev.filter(id => id !== flagId)
        : [...prev, flagId]
    );
  };

  const handleBulkRecognize = () => {
    if (selectedFlags.length > 0) {
      onRecognize(selectedFlags);
      setSelectedFlags([]);
    }
  };

  if (loading) {
    return <div className="loading">Loading flags...</div>;
  }

  return (
    <div className="flag-list">
      <div className="flag-list-header">
        <div className="flag-list-actions">
          <button 
            className="btn btn-primary"
            onClick={handleBulkRecognize}
            disabled={selectedFlags.length === 0}
          >
            Recognize Selected ({selectedFlags.length})
          </button>
          <button 
            className="btn btn-secondary"
            onClick={onRefresh}
          >
            Refresh
          </button>
        </div>
      </div>

      <div className="flag-grid">
        {flags.map(flag => (
          <div 
            key={flag.id}
            className={`flag-item ${selectedFlags.includes(flag.id!) ? 'selected' : ''}`}
            onClick={() => handleFlagSelect(flag.id!)}
          >
            <div className="flag-preview">
              <img 
                src={`/assets/flags/${flag.filename}`}
                alt={flag.country_name || flag.filename}
                onError={(e) => {
                  const target = e.target as HTMLImageElement;
                  target.style.display = 'none';
                  const fallback = document.createElement('span');
                  fallback.textContent = '🏳️';
                  fallback.className = 'flag-fallback';
                  target.parentNode?.appendChild(fallback);
                }}
              />
            </div>
            <div className="flag-info">
              <div className="flag-name">
                {flag.country_name || 'Unknown'}
              </div>
              <div className="flag-code">
                {flag.ioc_code || 'N/A'}
              </div>
              <div className="flag-status">
                <span className={`status ${flag.recognition_status}`}>
                  {flag.recognition_status}
                </span>
              </div>
            </div>
            <div className="flag-actions">
              <button 
                className="btn btn-sm"
                onClick={(e) => {
                  e.stopPropagation();
                  setRenameDialog({ open: true, flag });
                }}
              >
                Rename
              </button>
            </div>
          </div>
        ))}
      </div>

      {renameDialog.open && renameDialog.flag && (
        <RenameDialog
          flag={renameDialog.flag}
          onClose={() => setRenameDialog({ open: false, flag: null })}
          onSave={(newIocCode) => {
            // Handle rename
            setRenameDialog({ open: false, flag: null });
            onRefresh();
          }}
        />
      )}
    </div>
  );
};
```

### 3. BulkUpload.tsx (File Upload Component)
```typescript
// ui/src/modules/FlagManager/components/BulkUpload.tsx
import React, { useState, useCallback } from 'react';
import { useDropzone } from 'react-dropzone';

interface BulkUploadProps {
  onUploadComplete: () => void;
  autoRecognition: boolean;
}

export const BulkUpload: React.FC<BulkUploadProps> = ({
  onUploadComplete,
  autoRecognition,
}) => {
  const [uploading, setUploading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState<{
    [key: string]: number;
  }>({});

  const onDrop = useCallback(async (acceptedFiles: File[]) => {
    setUploading(true);
    
    try {
      // Convert files to paths for Tauri
      const filePaths = acceptedFiles.map(file => file.path);
      
      // Call Tauri command to upload files
      const { invoke } = await import('@tauri-apps/api/tauri');
      const results = await invoke('upload_flags', { filePaths });
      
      // If auto-recognition is enabled, run recognition
      if (autoRecognition) {
        await invoke('recognize_flags', { flagIds: results.map((r: any) => r.id) });
      }
      
      onUploadComplete();
    } catch (error) {
      console.error('Upload failed:', error);
    } finally {
      setUploading(false);
      setUploadProgress({});
    }
  }, [autoRecognition, onUploadComplete]);

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      'image/*': ['.png', '.jpg', '.jpeg', '.gif', '.bmp']
    },
    multiple: true,
  });

  return (
    <div className="bulk-upload">
      <div 
        {...getRootProps()} 
        className={`upload-zone ${isDragActive ? 'drag-active' : ''}`}
      >
        <input {...getInputProps()} />
        {isDragActive ? (
          <p>Drop the flag files here...</p>
        ) : (
          <div className="upload-content">
            <div className="upload-icon">📁</div>
            <p>Drag & drop flag files here, or click to select</p>
            <p className="upload-hint">
              Supported formats: PNG, JPG, JPEG, GIF, BMP
            </p>
          </div>
        )}
      </div>

      {uploading && (
        <div className="upload-progress">
          <div className="progress-bar">
            <div 
              className="progress-fill"
              style={{ width: `${Object.values(uploadProgress).reduce((a, b) => a + b, 0) / Object.keys(uploadProgress).length}%` }}
            />
          </div>
          <p>Uploading files...</p>
        </div>
      )}

      <div className="upload-settings">
        <label className="setting-item">
          <input 
            type="checkbox" 
            checked={autoRecognition}
            disabled
          />
          <span>Auto-recognition on upload</span>
        </label>
      </div>
    </div>
  );
};
```

## Tauri Command Handlers

### 1. Command Implementation
```rust
// src-tauri/src/flag_manager/commands.rs
use tauri::command;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct FlagFilters {
    pub status: Option<String>,
    pub country: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct RecognitionResult {
    pub flag_id: i64,
    pub recognized_as: String,
    pub confidence: f64,
    pub method: String,
}

#[command]
pub async fn list_flags(filters: FlagFilters) -> Result<Vec<FlagRecord>, String> {
    let db = FlagDatabase::new(&get_db_path()?)?;
    db.get_flags(filters)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn upload_flags(file_paths: Vec<String>) -> Result<Vec<FlagRecord>, String> {
    let file_manager = FlagFileManager::new()?;
    let mut results = Vec::new();
    
    for path in file_paths {
        let flag = file_manager.upload_file(&PathBuf::from(path))?;
        results.push(flag);
    }
    
    Ok(results)
}

#[command]
pub async fn rename_flag(id: i64, new_ioc_code: String) -> Result<(), String> {
    let file_manager = FlagFileManager::new()?;
    file_manager.rename_file(id, &new_ioc_code)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn recognize_flags(flag_ids: Vec<i64>) -> Result<Vec<RecognitionResult>, String> {
    let recognition_engine = RecognitionEngine::new()?;
    let mut results = Vec::new();
    
    for flag_id in flag_ids {
        let flag = get_flag_by_id(flag_id)?;
        let result = recognition_engine.recognize_single(&PathBuf::from(&flag.file_path))?;
        
        results.push(RecognitionResult {
            flag_id,
            recognized_as: result.recognized_as,
            confidence: result.confidence,
            method: result.method,
        });
    }
    
    Ok(results)
}

#[command]
pub async fn set_auto_recognition(enabled: bool) -> Result<(), String> {
    let db = FlagDatabase::new(&get_db_path()?)?;
    db.set_setting("auto_recognition", &enabled.to_string())
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_auto_recognition() -> Result<bool, String> {
    let db = FlagDatabase::new(&get_db_path()?)?;
    let value = db.get_setting("auto_recognition")?;
    Ok(value == "true")
}

#[command]
pub async fn delete_flag(id: i64) -> Result<(), String> {
    let file_manager = FlagFileManager::new()?;
    file_manager.delete_file(id)
        .map_err(|e| e.to_string())
}
```

## Integration with Sidebar

### 1. Sidebar Navigation Update
```typescript
// ui/src/components/Sidebar.tsx (update existing)
import { FlagManager } from '../modules/FlagManager/components/FlagManager';

// Add to navigation items
const navigationItems = [
  // ... existing items
  {
    id: 'flag-manager',
    label: 'Flag Manager',
    icon: '🏁',
    component: FlagManager,
    badge: flagCount, // Show count of flags
  },
];
```

### 2. Quick Access Widget
```typescript
// ui/src/components/Sidebar.tsx (add quick access)
const FlagQuickAccess: React.FC = () => {
  const { flags } = useFlagDatabase();
  
  const stats = {
    total: flags.length,
    pending: flags.filter(f => !f.is_recognized).length,
  };

  return (
    <div className="flag-quick-access">
      <div className="quick-stat">
        <span className="stat-number">{stats.total}</span>
        <span className="stat-label">Flags</span>
      </div>
      {stats.pending > 0 && (
        <div className="quick-stat warning">
          <span className="stat-number">{stats.pending}</span>
          <span className="stat-label">Pending</span>
        </div>
      )}
    </div>
  );
};
```

## CSS Styling

### 1. Flag Manager Styles
```css
/* ui/src/modules/FlagManager/FlagManager.css */
.flag-manager {
  @apply p-6 bg-white rounded-lg shadow-sm;
}

.flag-manager-header {
  @apply flex justify-between items-center mb-6;
}

.flag-manager-stats {
  @apply grid grid-cols-3 gap-4 mb-6;
}

.stat {
  @apply text-center p-4 bg-gray-50 rounded-lg;
}

.stat-label {
  @apply text-sm text-gray-600 block;
}

.stat-value {
  @apply text-2xl font-bold text-gray-900 block;
}

.flag-manager-tabs {
  @apply flex border-b border-gray-200 mb-6;
}

.tab {
  @apply px-4 py-2 text-sm font-medium text-gray-500 border-b-2 border-transparent hover:text-gray-700 hover:border-gray-300;
}

.tab.active {
  @apply text-blue-600 border-blue-600;
}

.flag-grid {
  @apply grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4;
}

.flag-item {
  @apply border border-gray-200 rounded-lg p-3 cursor-pointer hover:border-blue-300 transition-colors;
}

.flag-item.selected {
  @apply border-blue-500 bg-blue-50;
}

.flag-preview {
  @apply w-full h-20 mb-2 flex items-center justify-center bg-gray-100 rounded;
}

.flag-preview img {
  @apply max-w-full max-h-full object-contain;
}

.flag-info {
  @apply text-center;
}

.flag-name {
  @apply text-sm font-medium text-gray-900 truncate;
}

.flag-code {
  @apply text-xs text-gray-500;
}

.flag-status {
  @apply mt-1;
}

.status {
  @apply text-xs px-2 py-1 rounded-full;
}

.status.pending {
  @apply bg-yellow-100 text-yellow-800;
}

.status.recognized {
  @apply bg-green-100 text-green-800;
}

.status.failed {
  @apply bg-red-100 text-red-800;
}
```

## Testing Strategy

### 1. Unit Tests
```typescript
// ui/src/modules/FlagManager/__tests__/FlagManager.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { FlagManager } from '../components/FlagManager';

describe('FlagManager', () => {
  it('should render flag management interface', () => {
    render(<FlagManager />);
    expect(screen.getByText('Flag Management')).toBeInTheDocument();
  });

  it('should show flag statistics', () => {
    render(<FlagManager />);
    expect(screen.getByText('Total Flags')).toBeInTheDocument();
  });

  it('should handle tab navigation', () => {
    render(<FlagManager />);
    const uploadTab = screen.getByText('Bulk Upload');
    fireEvent.click(uploadTab);
    expect(uploadTab).toHaveClass('active');
  });
});
```

### 2. Integration Tests
```rust
// src-tauri/src/flag_manager/tests/mod.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = FlagDatabase::new(&PathBuf::from("test.db")).unwrap();
        assert!(db.conn.path().exists());
    }

    #[test]
    fn test_flag_upload() {
        let file_manager = FlagFileManager::new().unwrap();
        let flag = file_manager.upload_file(&PathBuf::from("test_flag.png")).unwrap();
        assert_eq!(flag.filename, "test_flag.png");
    }
}
```

This specification provides a complete technical foundation for implementing the Flag Management Module within the existing reStrike VTA project structure. 