#!/usr/bin/env python3
"""
Test script to verify the fixed CPU monitoring implementation
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
        print(f"\nFirst 5 data lines:")
        for i, line in enumerate(data_lines):
            print(f"  {i+1}: {line}")
        
        # Count processes with CPU data
        processes_with_cpu = 0
        processes_with_memory = 0
        significant_processes = 0
        
        for line in lines[1:]:  # Skip header
            parts = line.split(',')
            if len(parts) >= 4:
                process_name = parts[0].strip('"')
                cpu_str = parts[2].strip('"')
                memory_str = parts[3].strip('"')
                
                # Check CPU data
                if cpu_str and cpu_str != "":
                    processes_with_cpu += 1
                    # Handle European number format
                    normalized_cpu = cpu_str.replace(',', '.')
                    try:
                        cpu_seconds = float(normalized_cpu)
                        cpu_percent = cpu_seconds * 10.0  # Same conversion as Rust
                        
                        # Check memory data
                        memory_bytes = int(memory_str) if memory_str else 0
                        memory_mb = memory_bytes / (1024 * 1024)
                        
                        # Apply same filtering as Rust code
                        if cpu_percent > 0.1 or memory_bytes > 10 * 1024 * 1024:
                            significant_processes += 1
                            print(f"  Significant: {process_name} - CPU: {cpu_percent:.1f}%, Memory: {memory_mb:.1f}MB")
                    except ValueError:
                        pass
                
                # Check memory data
                if memory_str and memory_str != "":
                    processes_with_memory += 1
        
        print(f"\n📊 Statistics:")
        print(f"  Total processes: {len(lines) - 1}")
        print(f"  Processes with CPU data: {processes_with_cpu}")
        print(f"  Processes with memory data: {processes_with_memory}")
        print(f"  Significant processes (CPU > 0.1% or Memory > 10MB): {significant_processes}")
        
        return significant_processes > 0
        
    except subprocess.CalledProcessError as e:
        print(f"❌ PowerShell command failed: {e}")
        return False
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

def test_wmic_command():
    """Test the WMIC command for system CPU"""
    print("\nTesting WMIC system CPU command...")
    
    try:
        cmd = ["wmic", "cpu", "get", "loadpercentage", "/format:csv"]
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        
        print("✅ WMIC command executed successfully")
        print(f"Output: {result.stdout.strip()}")
        
        # Parse the output
        lines = result.stdout.strip().split('\n')
        if len(lines) >= 2:
            header = lines[0]
            data = lines[1]
            print(f"Header: {header}")
            print(f"Data: {data}")
            
            # Extract CPU percentage
            parts = data.split(',')
            if len(parts) >= 2:
                try:
                    cpu_percent = float(parts[1])
                    print(f"System CPU: {cpu_percent}%")
                    return True
                except ValueError:
                    print("❌ Could not parse CPU percentage")
                    return False
        
        return False
        
    except subprocess.CalledProcessError as e:
        print(f"❌ WMIC command failed: {e}")
        return False
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

def main():
    print("🔧 Testing Fixed CPU Monitoring Implementation")
    print("=" * 50)
    
    # Test PowerShell command
    powershell_ok = test_powershell_command()
    
    # Test WMIC command
    wmic_ok = test_wmic_command()
    
    print("\n" + "=" * 50)
    print("📋 Test Results:")
    print(f"  PowerShell process monitoring: {'✅ PASS' if powershell_ok else '❌ FAIL'}")
    print(f"  WMIC system CPU monitoring: {'✅ PASS' if wmic_ok else '❌ FAIL'}")
    
    if powershell_ok and wmic_ok:
        print("\n🎉 All tests passed! CPU monitoring should work correctly.")
        print("\nExpected behavior in the application:")
        print("  - System CPU percentage should be displayed")
        print("  - Process list should show real processes with CPU/memory data")
        print("  - Data should update every 2 seconds")
    else:
        print("\n⚠️ Some tests failed. Check the implementation.")
    
    return 0 if (powershell_ok and wmic_ok) else 1

if __name__ == "__main__":
    sys.exit(main()) 