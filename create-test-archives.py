#!/usr/bin/env python3
"""
Create test archives for the reStrike VTA logging system
This script manually creates ZIP archives to test the ARC dropdown functionality
"""

import os
import zipfile
from datetime import datetime
import shutil

def create_test_archives():
    """Create test ZIP archives in the log/archives directory"""
    log_dir = "src-tauri/log"
    archive_dir = "src-tauri/log/archives"
    
    # Create archive directory if it doesn't exist
    os.makedirs(archive_dir, exist_ok=True)
    
    # Get all the old log files that should be archived
    old_files = []
    for file in os.listdir(log_dir):
        if file.endswith('.log') and '20250613' in file:  # Old files from June 13th
            old_files.append(file)
    
    print(f"Found {len(old_files)} old log files to archive")
    
    # Group files by subsystem
    subsystems = {}
    for file in old_files:
        subsystem = file.split('_')[0]  # Extract subsystem name
        if subsystem not in subsystems:
            subsystems[subsystem] = []
        subsystems[subsystem].append(file)
    
    # Create archives for each subsystem
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    
    for subsystem, files in subsystems.items():
        if files:
            archive_name = f"{subsystem}_{timestamp}_archive.zip"
            archive_path = os.path.join(archive_dir, archive_name)
            
            print(f"Creating archive: {archive_name}")
            
            with zipfile.ZipFile(archive_path, 'w', zipfile.ZIP_DEFLATED, compresslevel=6) as zipf:
                for file in files:
                    file_path = os.path.join(log_dir, file)
                    if os.path.exists(file_path):
                        zipf.write(file_path, file)
                        print(f"  Added: {file}")
            
            # Get archive size
            archive_size = os.path.getsize(archive_path)
            print(f"  Archive size: {archive_size} bytes")
    
    print(f"\n✅ Created {len(subsystems)} archives in {archive_dir}")

def create_sample_archives():
    """Create sample archives with different content for testing"""
    archive_dir = "src-tauri/log/archives"
    os.makedirs(archive_dir, exist_ok=True)
    
    # Create sample archives with different timestamps
    timestamps = [
        "20241219_143022",
        "20241220_091545", 
        "20241221_162330"
    ]
    
    subsystems = ["pss", "obs", "udp"]
    
    for i, timestamp in enumerate(timestamps):
        for subsystem in subsystems:
            archive_name = f"{subsystem}_{timestamp}_archive.zip"
            archive_path = os.path.join(archive_dir, archive_name)
            
            print(f"Creating sample archive: {archive_name}")
            
            with zipfile.ZipFile(archive_path, 'w', zipfile.ZIP_DEFLATED, compresslevel=6) as zipf:
                # Create sample log content
                sample_log_content = f"""[2024-12-19 14:30:22.123] [INFO] [{subsystem}] Sample log entry 1
[2024-12-19 14:30:23.456] [DEBUG] [{subsystem}] Sample log entry 2
[2024-12-19 14:30:24.789] [INFO] [{subsystem}] Sample log entry 3
[2024-12-19 14:30:25.012] [WARN] [{subsystem}] Sample log entry 4
[2024-12-19 14:30:26.345] [INFO] [{subsystem}] Sample log entry 5
"""
                
                # Add multiple log files to each archive
                for j in range(3):
                    log_filename = f"{subsystem}_{timestamp}_{j}.log"
                    zipf.writestr(log_filename, sample_log_content)
                    print(f"  Added: {log_filename}")
            
            archive_size = os.path.getsize(archive_path)
            print(f"  Archive size: {archive_size} bytes")
    
    print(f"\n✅ Created {len(timestamps) * len(subsystems)} sample archives")

def list_archives():
    """List all archives in the archive directory"""
    archive_dir = "src-tauri/log/archives"
    
    if not os.path.exists(archive_dir):
        print(f"❌ Archive directory does not exist: {archive_dir}")
        return
    
    archives = []
    for file in os.listdir(archive_dir):
        if file.endswith('.zip'):
            file_path = os.path.join(archive_dir, file)
            size = os.path.getsize(file_path)
            modified = datetime.fromtimestamp(os.path.getmtime(file_path))
            archives.append((file, size, modified))
    
    if archives:
        print(f"\n📦 Found {len(archives)} archives:")
        for archive, size, modified in sorted(archives):
            print(f"  📄 {archive} ({size} bytes, {modified})")
    else:
        print(f"\n❌ No archives found in {archive_dir}")

def main():
    """Main function"""
    print("🚀 Creating test archives for reStrike VTA logging system")
    print("=" * 60)
    
    # Step 1: Create archives from existing old files
    print("\n1️⃣ Creating archives from existing old log files...")
    create_test_archives()
    
    # Step 2: Create sample archives for testing
    print("\n2️⃣ Creating sample archives for testing...")
    create_sample_archives()
    
    # Step 3: List all archives
    print("\n3️⃣ Listing all archives...")
    list_archives()
    
    print("\n" + "=" * 60)
    print("✅ Archive creation complete!")
    print("\n📋 Next steps:")
    print("   1. Start the Tauri app")
    print("   2. Go to Download Logs section")
    print("   3. Select ARC from the dropdown")
    print("   4. You should now see the created archives")
    print("   5. Double-click an archive to extract it")
    print("   6. Check log/archives/extracted/ for extracted files")

if __name__ == "__main__":
    main() 