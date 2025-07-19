#!/usr/bin/env python3
"""
Test script to validate CPU monitoring PowerShell command and parsing
"""

import subprocess
import json
import sys

def test_powershell_command():
    """Test the PowerShell command used by CPU monitoring"""
    print("Testing PowerShell CPU monitoring command...")
    
    try:
        # Run the same PowerShell command used in the Rust code
        cmd = [
            "powershell", 
            "-Command", 
            "Get-Process | Select-Object Name, Id, CPU, WorkingSet | ConvertTo-Csv -NoTypeInformation"
        ]
        
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        
        print("✅ PowerShell command executed successfully")
        print(f"Output length: {len(result.stdout)} characters")
        
        # Parse the CSV output
        lines = result.stdout.strip().split('\n')
        print(f"Total lines: {len(lines)}")
        
        if len(lines) < 2:
            print("❌ No data lines found")
            return False
            
        # Parse header
        header = lines[0]
        print(f"Header: {header}")
        
        # Parse first few data lines
        data_lines = lines[1:6]  # First 5 data lines
        print("\nFirst 5 data lines:")
        for i, line in enumerate(data_lines, 1):
            print(f"  {i}: {line}")
            
        # Test parsing logic
        print("\nTesting parsing logic...")
        valid_processes = 0
        processes_with_cpu = 0
        processes_with_memory = 0
        
        for line in lines[1:]:  # Skip header
            parts = line.split(',')
            if len(parts) >= 4:
                process_name = parts[0].strip('"')
                pid_str = parts[1].strip('"')
                cpu_str = parts[2].strip('"')
                memory_str = parts[3].strip('"')
                
                valid_processes += 1
                
                # Try to parse CPU
                try:
                    cpu_seconds = float(cpu_str) if cpu_str else 0.0
                    if cpu_seconds > 0:
                        processes_with_cpu += 1
                except ValueError:
                    pass
                    
                # Try to parse memory
                try:
                    memory_bytes = int(memory_str) if memory_str else 0
                    if memory_bytes > 0:
                        processes_with_memory += 1
                except ValueError:
                    pass
        
        print(f"✅ Valid processes: {valid_processes}")
        print(f"✅ Processes with CPU data: {processes_with_cpu}")
        print(f"✅ Processes with memory data: {processes_with_memory}")
        
        # Test filtering logic (same as Rust code)
        print("\nTesting filtering logic...")
        filtered_processes = 0
        
        for line in lines[1:]:
            parts = line.split(',')
            if len(parts) >= 4:
                cpu_str = parts[2].strip('"')
                memory_str = parts[3].strip('"')
                
                try:
                    cpu_seconds = float(cpu_str) if cpu_str else 0.0
                    memory_bytes = int(memory_str) if memory_str else 0
                    
                    cpu_percent = cpu_seconds * 10.0  # Same conversion as Rust
                    memory_mb = memory_bytes / (1024.0 * 1024.0)
                    
                    # Same filtering logic as Rust
                    if cpu_percent > 0.1 or memory_bytes > 10 * 1024 * 1024:
                        filtered_processes += 1
                        
                except ValueError:
                    pass
        
        print(f"✅ Processes passing filter: {filtered_processes}")
        
        return True
        
    except subprocess.CalledProcessError as e:
        print(f"❌ PowerShell command failed: {e}")
        print(f"Error output: {e.stderr}")
        return False
    except Exception as e:
        print(f"❌ Unexpected error: {e}")
        return False

def test_system_cpu():
    """Test system CPU command"""
    print("\nTesting system CPU command...")
    
    try:
        cmd = ["wmic", "cpu", "get", "loadpercentage", "/value"]
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        
        print("✅ System CPU command executed successfully")
        print(f"Output: {result.stdout.strip()}")
        
        # Parse the output
        for line in result.stdout.strip().split('\n'):
            if line.startswith('LoadPercentage='):
                cpu_percent = line.split('=')[1]
                print(f"✅ System CPU: {cpu_percent}%")
                return True
                
        print("❌ Could not parse system CPU percentage")
        return False
        
    except subprocess.CalledProcessError as e:
        print(f"❌ System CPU command failed: {e}")
        return False

def main():
    """Main test function"""
    print("🧪 CPU Monitoring Test Suite")
    print("=" * 50)
    
    # Test PowerShell command
    powershell_ok = test_powershell_command()
    
    # Test system CPU
    system_cpu_ok = test_system_cpu()
    
    print("\n" + "=" * 50)
    print("📊 Test Results:")
    print(f"PowerShell Command: {'✅ PASS' if powershell_ok else '❌ FAIL'}")
    print(f"System CPU Command: {'✅ PASS' if system_cpu_ok else '❌ FAIL'}")
    
    if powershell_ok and system_cpu_ok:
        print("\n🎉 All tests passed! CPU monitoring should work correctly.")
        return 0
    else:
        print("\n⚠️  Some tests failed. Check the output above.")
        return 1

if __name__ == "__main__":
    sys.exit(main()) 