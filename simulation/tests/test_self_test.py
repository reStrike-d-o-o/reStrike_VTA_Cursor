#!/usr/bin/env python3
"""
Test script for self-test system functionality
"""
import sys
import os
import time
import json

sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'core'))

from self_test_system import SelfTestSystem

def test_self_test_system():
    """Test the self-test system"""
    print("🧪 Testing Self-Test System")
    print("=" * 50)
    
    # Create self-test system
    self_test = SelfTestSystem()
    
    # Set up callbacks
    def status_callback(message: str):
        print(f"  [STATUS] {message}")
    
    def progress_callback(current: int, total: int):
        print(f"  [PROGRESS] {current}/{total}")
    
    self_test.set_callbacks(status_callback, progress_callback)
    
    # Run comprehensive test
    print("Running comprehensive self-test...")
    result = self_test.run_comprehensive_test()
    
    print("\n📊 Test Results:")
    print(f"Overall Status: {result['overall_status']}")
    print(f"Total Tests: {result['summary']['total_tests']}")
    print(f"Passed: {result['summary']['passed']}")
    print(f"Failed: {result['summary']['failed']}")
    print(f"Warnings: {result['summary']['warnings']}")
    print(f"Success Rate: {result['summary']['success_rate']:.1f}%")
    print(f"Duration: {result['duration']:.2f} seconds")
    
    # Generate markdown report
    report = self_test.generate_markdown_report()
    print(f"\n📄 Report Length: {len(report)} characters")
    
    # Save report
    report_path = "test_self_test_report.md"
    with open(report_path, "w", encoding="utf-8") as f:
        f.write(report)
    
    print(f"📄 Report saved to: {report_path}")
    
    return result

def test_individual_tests():
    """Test individual test functions"""
    print("\n🔧 Testing Individual Test Functions")
    print("=" * 50)
    
    self_test = SelfTestSystem()
    
    # Test UDP server
    print("Testing UDP Server...")
    status, desc, details, error = self_test._test_udp_server()
    print(f"  Status: {status.value}")
    print(f"  Description: {desc}")
    print(f"  Details: {details}")
    if error:
        print(f"  Error: {error}")
    
    # Test WebSocket server
    print("\nTesting WebSocket Server...")
    status, desc, details, error = self_test._test_websocket_server()
    print(f"  Status: {status.value}")
    print(f"  Description: {desc}")
    print(f"  Details: {details}")
    if error:
        print(f"  Error: {error}")
    
    # Test Python simulator
    print("\nTesting Python Simulator...")
    status, desc, details, error = self_test._test_python_simulator()
    print(f"  Status: {status.value}")
    print(f"  Description: {desc}")
    print(f"  Details: {details}")
    if error:
        print(f"  Error: {error}")

def main():
    """Run all tests"""
    print("🧪 Self-Test System Test Suite")
    print("=" * 50)
    
    try:
        # Test comprehensive self-test
        result = test_self_test_system()
        
        # Test individual functions
        test_individual_tests()
        
        print("\n🎉 All tests completed successfully!")
        print(f"\nOverall System Status: {result['overall_status']}")
        print(f"Success Rate: {result['summary']['success_rate']:.1f}%")
        
        if result['summary']['failed'] > 0:
            print(f"\n⚠️  {result['summary']['failed']} tests failed - review the report")
        elif result['summary']['warnings'] > 0:
            print(f"\n⚠️  {result['summary']['warnings']} warnings detected - review the report")
        else:
            print("\n✅ All tests passed!")
        
    except Exception as e:
        print(f"❌ Test failed with error: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    return 0

if __name__ == "__main__":
    exit(main()) 