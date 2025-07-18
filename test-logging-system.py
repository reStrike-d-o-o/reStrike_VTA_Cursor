#!/usr/bin/env python3
"""
Test script for the reStrike VTA logging system
Generates test log files and tests archival functionality
"""

import os
import time
import json
from datetime import datetime, timedelta
import subprocess
import sys

def create_test_log_files():
    """Create test log files in the log directory"""
    log_dir = "src-tauri/log"
    
    # Create log directory if it doesn't exist
    os.makedirs(log_dir, exist_ok=True)
    
    # Create test log files for each subsystem
    subsystems = ["pss", "obs", "udp", "app"]
    
    for subsystem in subsystems:
        log_file = os.path.join(log_dir, f"{subsystem}.log")
        
        # Generate some test log entries
        with open(log_file, 'w') as f:
            for i in range(10):
                timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S.%f")[:-3]
                level = "INFO" if i % 2 == 0 else "DEBUG"
                message = f"Test log entry {i+1} for {subsystem} subsystem"
                f.write(f"[{timestamp}] [{level}] [{subsystem}] {message}\n")
        
        print(f"✅ Created test log file: {log_file}")

def create_old_test_files():
    """Create some old test files to trigger archival"""
    log_dir = "src-tauri/log"
    
    # Create some rotated log files with old timestamps
    old_date = datetime.now() - timedelta(days=35)  # 35 days old
    
    subsystems = ["pss", "obs", "udp"]
    
    for subsystem in subsystems:
        # Create a rotated log file with old timestamp
        timestamp = old_date.strftime("%Y%m%d_%H%M%S")
        rotated_file = os.path.join(log_dir, f"{subsystem}_{timestamp}_0.log")
        
        with open(rotated_file, 'w') as f:
            for i in range(5):
                log_timestamp = old_date.strftime("%Y-%m-%d %H:%M:%S.%f")[:-3]
                level = "INFO"
                message = f"Old test log entry {i+1} for {subsystem} (should be archived)"
                f.write(f"[{log_timestamp}] [{level}] [{subsystem}] {message}\n")
        
        print(f"✅ Created old test file: {rotated_file}")

def test_tauri_commands():
    """Test Tauri commands for logging functionality"""
    print("\n🧪 Testing Tauri logging commands...")
    
    # Test commands to run
    test_commands = [
        ("set_logging_enabled", {"subsystem": "pss", "enabled": True}),
        ("set_logging_enabled", {"subsystem": "obs", "enabled": True}),
        ("set_logging_enabled", {"subsystem": "udp", "enabled": True}),
        ("list_log_files", {}),
        ("list_archives", {}),
    ]
    
    for command, args in test_commands:
        print(f"\n📋 Testing command: {command}")
        print(f"   Args: {args}")
        
        # Note: In a real test, you would invoke these via Tauri
        # For now, we'll just print what would be tested
        print(f"   ✅ Would invoke: {command} with {args}")

def check_log_directory_structure():
    """Check the log directory structure"""
    print("\n📁 Checking log directory structure...")
    
    log_dir = "src-tauri/log"
    
    if os.path.exists(log_dir):
        print(f"✅ Log directory exists: {log_dir}")
        
        # List all files in log directory
        files = os.listdir(log_dir)
        print(f"📄 Files in log directory: {len(files)}")
        
        for file in files:
            file_path = os.path.join(log_dir, file)
            if os.path.isfile(file_path):
                size = os.path.getsize(file_path)
                modified = datetime.fromtimestamp(os.path.getmtime(file_path))
                print(f"   📄 {file} ({size} bytes, modified: {modified})")
            elif os.path.isdir(file_path):
                print(f"   📁 {file}/ (directory)")
    else:
        print(f"❌ Log directory does not exist: {log_dir}")

def main():
    """Main test function"""
    print("🚀 Starting reStrike VTA Logging System Test")
    print("=" * 50)
    
    # Step 1: Create test log files
    print("\n1️⃣ Creating test log files...")
    create_test_log_files()
    
    # Step 2: Create old test files for archival
    print("\n2️⃣ Creating old test files for archival...")
    create_old_test_files()
    
    # Step 3: Check directory structure
    print("\n3️⃣ Checking directory structure...")
    check_log_directory_structure()
    
    # Step 4: Test Tauri commands (simulated)
    print("\n4️⃣ Testing Tauri commands...")
    test_tauri_commands()
    
    print("\n" + "=" * 50)
    print("✅ Test setup complete!")
    print("\n📋 Next steps:")
    print("   1. Start the Tauri app")
    print("   2. Test the logging toggles (PSS, OBS, UDP)")
    print("   3. Check the Download Logs section")
    print("   4. Test the ARC option in the dropdown")
    print("   5. Verify log rotation (files >10MB)")
    print("   6. Test archival (files >30 days old)")
    print("\n🔧 Manual testing commands:")
    print("   - Toggle PSS logging: set_logging_enabled('pss', true/false)")
    print("   - List log files: list_log_files()")
    print("   - List archives: list_archives()")
    print("   - Extract archive: extract_archive('filename.zip')")

if __name__ == "__main__":
    main() 