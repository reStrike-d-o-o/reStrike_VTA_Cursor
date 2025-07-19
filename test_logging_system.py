#!/usr/bin/env python3
"""
Test script for the reStrike VTA logging system
"""

import asyncio
import json
import time
from pathlib import Path

# Simulate Tauri invoke function
async def invoke(command, args=None):
    """Simulate Tauri invoke for testing"""
    print(f"🔧 Invoking: {command} with args: {args}")
    
    # Simulate responses based on command
    if command == "list_log_files":
        # Check if log files exist
        log_dir = Path("src-tauri/logs")
        files = []
        
        if log_dir.exists():
            for log_file in log_dir.glob("*.log"):
                if args and "subsystem" in args:
                    subsystem = args["subsystem"]
                    if subsystem in log_file.name:
                        files.append({
                            "name": log_file.name,
                            "size": log_file.stat().st_size,
                            "modified": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.localtime(log_file.stat().st_mtime)),
                            "subsystem": subsystem
                        })
                else:
                    files.append({
                        "name": log_file.name,
                        "size": log_file.stat().st_size,
                        "modified": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.localtime(log_file.stat().st_mtime)),
                        "subsystem": "general"
                    })
        
        return {
            "success": True,
            "data": files
        }
    
    elif command == "set_logging_enabled":
        subsystem = args["subsystem"]
        enabled = args["enabled"]
        print(f"📝 Setting logging for {subsystem} to {enabled}")
        return {
            "success": True,
            "message": f"Logging {'enabled' if enabled else 'disabled'} for {subsystem}"
        }
    
    elif command == "download_log_file":
        filename = args["filename"]
        log_file = Path(f"src-tauri/logs/{filename}")
        
        if log_file.exists():
            content = log_file.read_bytes()
            return {
                "success": True,
                "data": list(content)
            }
        else:
            return {
                "success": False,
                "error": f"Log file {filename} not found"
            }
    
    return {"success": False, "error": f"Unknown command: {command}"}

async def test_logging_system():
    """Test the logging system functionality"""
    print("🧪 Testing reStrike VTA Logging System")
    print("=" * 50)
    
    # Test 1: Check current log files
    print("\n1️⃣ Checking current log files...")
    result = await invoke("list_log_files")
    if result["success"]:
        files = result["data"]
        print(f"   Found {len(files)} log files:")
        for file in files:
            print(f"   - {file['name']} ({file['size']} bytes)")
    else:
        print(f"   ❌ Error: {result['error']}")
    
    # Test 2: Enable PSS logging
    print("\n2️⃣ Enabling PSS logging...")
    result = await invoke("set_logging_enabled", {"subsystem": "pss", "enabled": True})
    print(f"   Result: {result['message']}")
    
    # Wait a moment for log file to be created
    await asyncio.sleep(2)
    
    # Test 3: Check PSS log files
    print("\n3️⃣ Checking PSS log files...")
    result = await invoke("list_log_files", {"subsystem": "pss"})
    if result["success"]:
        files = result["data"]
        print(f"   Found {len(files)} PSS log files:")
        for file in files:
            print(f"   - {file['name']} ({file['size']} bytes)")
    else:
        print(f"   ❌ Error: {result['error']}")
    
    # Test 4: Enable OBS logging
    print("\n4️⃣ Enabling OBS logging...")
    result = await invoke("set_logging_enabled", {"subsystem": "obs", "enabled": True})
    print(f"   Result: {result['message']}")
    
    # Wait a moment for log file to be created
    await asyncio.sleep(2)
    
    # Test 5: Check OBS log files
    print("\n5️⃣ Checking OBS log files...")
    result = await invoke("list_log_files", {"subsystem": "obs"})
    if result["success"]:
        files = result["data"]
        print(f"   Found {len(files)} OBS log files:")
        for file in files:
            print(f"   - {file['name']} ({file['size']} bytes)")
    else:
        print(f"   ❌ Error: {result['error']}")
    
    # Test 6: Enable UDP logging
    print("\n6️⃣ Enabling UDP logging...")
    result = await invoke("set_logging_enabled", {"subsystem": "udp", "enabled": True})
    print(f"   Result: {result['message']}")
    
    # Wait a moment for log file to be created
    await asyncio.sleep(2)
    
    # Test 7: Check all log files
    print("\n7️⃣ Checking all log files...")
    result = await invoke("list_log_files")
    if result["success"]:
        files = result["data"]
        print(f"   Found {len(files)} total log files:")
        for file in files:
            print(f"   - {file['name']} ({file['size']} bytes)")
    else:
        print(f"   ❌ Error: {result['error']}")
    
    # Test 8: Try to download a log file
    print("\n8️⃣ Testing log file download...")
    if result["success"] and result["data"]:
        first_file = result["data"][0]
        download_result = await invoke("download_log_file", {"filename": first_file["name"]})
        if download_result["success"]:
            print(f"   ✅ Successfully downloaded {first_file['name']} ({len(download_result['data'])} bytes)")
        else:
            print(f"   ❌ Download failed: {download_result['error']}")
    
    print("\n" + "=" * 50)
    print("🏁 Logging system test completed!")

if __name__ == "__main__":
    asyncio.run(test_logging_system()) 