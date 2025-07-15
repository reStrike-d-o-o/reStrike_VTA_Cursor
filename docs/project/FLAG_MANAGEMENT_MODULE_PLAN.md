# Flag Management Module - Comprehensive Plan

## Overview
A modular flag management system for the reStrike VTA Windows desktop application, providing on-demand automatic recognition, bulk upload capabilities, and manual flag management tools.

## Architecture

### Database Technology
**SQLite** - Chosen for the following reasons:
- **Native Windows Support**: Perfect for desktop applications
- **Tauri Integration**: Built-in support via `rusqlite` crate
- **Zero Configuration**: No server setup required
- **File-based**: Database stored as a single file in the app directory
- **ACID Compliance**: Reliable for concurrent operations
- **Lightweight**: Minimal resource usage

### Module Structure
```
reStrike_VTA/
├── ui/src/modules/FlagManager/          # React frontend module
│   ├── components/
│   │   ├── FlagManager.tsx             # Main component
│   │   ├── FlagList.tsx                # Flag listing with previews
│   │   ├── BulkUpload.tsx              # File upload interface
│   │   ├── RenameDialog.tsx            # Manual rename tool
│   │   ├── RecognitionToggle.tsx       # Auto-recognition toggle
│   │   └── RecognitionReport.tsx       # Results display
│   ├── hooks/
│   │   ├── useFlagDatabase.ts          # Database operations
│   │   └── useRecognition.ts           # Recognition operations
│   ├── types/
│   │   └── flagTypes.ts                # TypeScript interfaces
│   └── utils/
│       └── flagUtils.tsx               # Utility functions
├── src-tauri/src/flag_manager/         # Rust backend module
│   ├── mod.rs                          # Module entry point
│   ├── database.rs                     # SQLite operations
│   ├── recognition.rs                  # Recognition engine
│   ├── file_manager.rs                 # File operations
│   └── commands.rs                     # Tauri command handlers
├── scripts/media/                      # Recognition scripts
│   ├── flag_recognition.py             # Python recognition script
│   └── flag_database.json              # Recognition database
└── ui/public/assets/flags/             # Flag image storage
```

## Database Schema

### Tables

#### 1. flags
```sql
CREATE TABLE flags (
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
);
```

#### 2. recognition_history
```sql
CREATE TABLE recognition_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    flag_id INTEGER,
    recognition_method TEXT,
    confidence REAL,
    recognized_as TEXT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (flag_id) REFERENCES flags(id)
);
```

#### 3. settings
```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

## Frontend Components

### 1. FlagManager.tsx (Main Component)
- **State Management**: Uses Zustand for global state
- **Layout**: Sidebar with navigation + main content area
- **Features**:
  - Toggle for automatic recognition (default: OFF)
  - Quick stats (total flags, recognized, pending)
  - Navigation to sub-components

### 2. FlagList.tsx
- **Grid Layout**: Responsive grid of flag previews
- **Actions**: 
  - Preview flag image
  - Show IOC code and country name
  - Manual rename option
  - Delete flag
  - Re-run recognition
- **Filtering**: By recognition status, country, upload date
- **Sorting**: By name, date, recognition status

### 3. BulkUpload.tsx
- **Drag & Drop**: Modern file upload interface
- **File Validation**: Image format, size limits
- **Progress Tracking**: Upload progress with status
- **Batch Processing**: Handle multiple files efficiently
- **Preview**: Show uploaded files before processing

### 4. RenameDialog.tsx
- **Modal Interface**: Clean rename dialog
- **Suggestions**: Show recognition suggestions
- **Manual Input**: Allow custom IOC codes
- **Validation**: Ensure valid IOC format
- **Confirmation**: Preview changes before applying

### 5. RecognitionToggle.tsx
- **Toggle Switch**: Modern toggle UI component
- **Status Display**: Show current recognition mode
- **Confirmation**: Warn about enabling auto-recognition
- **Settings Persistence**: Save preference to database

### 6. RecognitionReport.tsx
- **Results Display**: Show recognition results
- **Statistics**: Success rate, confidence levels
- **Error Handling**: Display failed recognitions
- **Export Options**: CSV, JSON reports

## Backend Implementation

### 1. Database Operations (database.rs)
```rust
pub struct FlagDatabase {
    conn: Connection,
}

impl FlagDatabase {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>>;
    pub fn init_tables(&self) -> Result<(), Box<dyn std::error::Error>>;
    pub fn add_flag(&self, flag: FlagRecord) -> Result<i64, Box<dyn std::error::Error>>;
    pub fn update_flag(&self, id: i64, updates: FlagUpdate) -> Result<(), Box<dyn std::error::Error>>;
    pub fn get_flags(&self, filters: FlagFilters) -> Result<Vec<FlagRecord>, Box<dyn std::error::Error>>;
    pub fn delete_flag(&self, id: i64) -> Result<(), Box<dyn std::error::Error>>;
}
```

### 2. Recognition Engine (recognition.rs)
```rust
pub struct RecognitionEngine {
    python_script_path: PathBuf,
    database_path: PathBuf,
}

impl RecognitionEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>>;
    pub fn recognize_single(&self, file_path: &Path) -> Result<RecognitionResult, Box<dyn std::error::Error>>;
    pub fn recognize_batch(&self, file_paths: Vec<PathBuf>) -> Result<Vec<RecognitionResult>, Box<dyn std::error::Error>>;
    pub fn is_auto_recognition_enabled(&self) -> Result<bool, Box<dyn std::error::Error>>;
    pub fn set_auto_recognition(&self, enabled: bool) -> Result<(), Box<dyn std::error::Error>>;
}
```

### 3. File Manager (file_manager.rs)
```rust
pub struct FlagFileManager {
    flags_directory: PathBuf,
    database: FlagDatabase,
}

impl FlagFileManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>>;
    pub fn upload_file(&self, source_path: &Path) -> Result<FlagRecord, Box<dyn std::error::Error>>;
    pub fn rename_file(&self, id: i64, new_ioc_code: &str) -> Result<(), Box<dyn std::error::Error>>;
    pub fn delete_file(&self, id: i64) -> Result<(), Box<dyn std::error::Error>>;
    pub fn get_file_info(&self, id: i64) -> Result<FileInfo, Box<dyn std::error::Error>>;
}
```

### 4. Tauri Commands (commands.rs)
```rust
#[tauri::command]
pub async fn list_flags(filters: FlagFilters) -> Result<Vec<FlagRecord>, String>;

#[tauri::command]
pub async fn upload_flags(file_paths: Vec<String>) -> Result<Vec<FlagRecord>, String>;

#[tauri::command]
pub async fn rename_flag(id: i64, new_ioc_code: String) -> Result<(), String>;

#[tauri::command]
pub async fn recognize_flags(flag_ids: Vec<i64>) -> Result<Vec<RecognitionResult>, String>;

#[tauri::command]
pub async fn set_auto_recognition(enabled: bool) -> Result<(), String>;

#[tauri::command]
pub async fn get_auto_recognition() -> Result<bool, String>;

#[tauri::command]
pub async fn delete_flag(id: i64) -> Result<(), String>;
```

## Integration with Existing Project

### 1. Sidebar Integration
- **Navigation**: Add Flag Manager to sidebar navigation
- **Quick Access**: Show flag count and recognition status
- **Context Menu**: Right-click on flags for quick actions

### 2. Existing Flag System
- **Backward Compatibility**: Support existing flag files
- **Migration**: Auto-import existing flags to database
- **Fallback**: Maintain emoji fallback system

### 3. Project Structure Alignment
- **Follows Existing Patterns**: Use same architecture as other modules
- **Consistent Styling**: Match existing UI components
- **Error Handling**: Use project-wide error handling patterns

## Implementation Phases

### Phase 1: Foundation (Week 1)
- [ ] Create module directory structure
- [ ] Implement SQLite database schema
- [ ] Create basic Rust backend with database operations
- [ ] Set up Tauri command handlers

### Phase 2: Core Features (Week 2)
- [ ] Implement FlagList component
- [ ] Create BulkUpload interface
- [ ] Add file management operations
- [ ] Integrate with existing recognition script

### Phase 3: Recognition System (Week 3)
- [ ] Implement recognition toggle
- [ ] Create on-demand recognition
- [ ] Add recognition reporting
- [ ] Test recognition accuracy

### Phase 4: Polish & Integration (Week 4)
- [ ] Add manual rename functionality
- [ ] Integrate with sidebar
- [ ] Add filtering and sorting
- [ ] Performance optimization

### Phase 5: Testing & Documentation (Week 5)
- [ ] Comprehensive testing
- [ ] User documentation
- [ ] Performance testing
- [ ] Security audit

## Technical Considerations

### Performance
- **Lazy Loading**: Load flag previews on demand
- **Batch Operations**: Process multiple files efficiently
- **Caching**: Cache recognition results
- **Database Indexing**: Optimize query performance

### Security
- **File Validation**: Validate uploaded files
- **Path Sanitization**: Prevent directory traversal
- **SQL Injection Prevention**: Use parameterized queries
- **File Size Limits**: Prevent large file uploads

### Error Handling
- **Graceful Degradation**: Handle recognition failures
- **User Feedback**: Clear error messages
- **Recovery**: Allow retry of failed operations
- **Logging**: Comprehensive error logging

### Windows Compatibility
- **File Paths**: Handle Windows path separators
- **File Permissions**: Handle Windows file permissions
- **Process Management**: Manage Python subprocess on Windows
- **Registry Integration**: Store settings in Windows registry if needed

## Success Metrics

### Functionality
- [ ] 100% flag recognition accuracy for known flags
- [ ] <2 second response time for flag operations
- [ ] Support for 200+ country flags
- [ ] Zero data loss during operations

### User Experience
- [ ] Intuitive interface requiring minimal training
- [ ] Bulk operations complete in <30 seconds
- [ ] Clear feedback for all user actions
- [ ] Consistent with existing application design

### Technical
- [ ] Zero memory leaks
- [ ] Database operations complete in <100ms
- [ ] File operations handle concurrent access
- [ ] Recognition script runs without blocking UI

## Future Enhancements

### Advanced Recognition
- **Machine Learning**: Train custom recognition models
- **Cloud Recognition**: Use cloud-based recognition services
- **Pattern Matching**: Improve recognition accuracy
- **Historical Data**: Learn from user corrections

### Integration Features
- **OBS Integration**: Direct flag selection in OBS
- **Competition Integration**: Auto-import competition flags
- **Backup/Restore**: Database backup functionality
- **Export/Import**: Share flag databases between installations

### User Experience
- **Drag & Drop**: Reorder flags in interface
- **Bulk Editing**: Edit multiple flags simultaneously
- **Search**: Full-text search across flag metadata
- **Statistics**: Usage analytics and reporting 