#!/usr/bin/env python3
"""
Integration test script for tkStrikeHardwareSimulator with reStrikeVTA
Tests basic connectivity and event transmission
"""
import socket
import time
import threading
from tkstrike_hardware_simulator import tkStrikeHardwareSimulator, MatchScenario

class IntegrationTester:
    """Test integration between simulator and reStrikeVTA"""
    
    def __init__(self, host: str = "127.0.0.1", port: int = 8888):
        self.host = host
        self.port = port
        self.simulator = tkStrikeHardwareSimulator(host, port)
        self.test_results = {}
        
    def test_connection(self):
        """Test basic UDP connection"""
        print("🔌 Testing UDP connection...")
        try:
            # Test if port is open
            sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            sock.settimeout(1)
            sock.connect((self.host, self.port))
            sock.close()
            print("✅ UDP port 8888 is accessible")
            self.test_results['connection'] = True
            return True
        except Exception as e:
            print(f"❌ UDP connection failed: {e}")
            print("   Make sure reStrikeVTA is running and listening on port 8888")
            self.test_results['connection'] = False
            return False
    
    def test_simulator_connection(self):
        """Test simulator connection"""
        print("🔌 Testing simulator connection...")
        try:
            if self.simulator.connect():
                print("✅ Simulator connected successfully")
                self.test_results['simulator_connection'] = True
                return True
            else:
                print("❌ Simulator connection failed")
                self.test_results['simulator_connection'] = False
                return False
        except Exception as e:
            print(f"❌ Simulator connection error: {e}")
            self.test_results['simulator_connection'] = False
            return False
    
    def test_basic_events(self):
        """Test sending basic events"""
        print("📡 Testing basic event transmission...")
        try:
            # Load basic match
            if not self.simulator.load_match(MatchScenario.BASIC):
                print("❌ Failed to load match")
                return False
            
            # Send a few test events
            events_sent = 0
            
            # Add a point
            if self.simulator.add_point(1, 1):  # Blue athlete, punch
                events_sent += 1
                print("✅ Sent point event")
            
            time.sleep(0.5)
            
            # Add a warning
            if self.simulator.add_warning(2):  # Red athlete
                events_sent += 1
                print("✅ Sent warning event")
            
            time.sleep(0.5)
            
            # Add another point
            if self.simulator.add_point(2, 3):  # Red athlete, head kick
                events_sent += 1
                print("✅ Sent head kick event")
            
            print(f"✅ Successfully sent {events_sent} events")
            self.test_results['basic_events'] = events_sent
            return events_sent > 0
            
        except Exception as e:
            print(f"❌ Event transmission error: {e}")
            self.test_results['basic_events'] = 0
            return False
    
    def test_match_scenario(self):
        """Test complete match scenario"""
        print("🏆 Testing complete match scenario...")
        try:
            # Run a quick demo
            print("   Running 30-second demo...")
            self.simulator.run_demo(MatchScenario.BASIC)
            print("✅ Match scenario completed")
            self.test_results['match_scenario'] = True
            return True
        except Exception as e:
            print(f"❌ Match scenario error: {e}")
            self.test_results['match_scenario'] = False
            return False
    
    def cleanup(self):
        """Clean up connections"""
        try:
            self.simulator.disconnect()
            print("🧹 Cleanup completed")
        except:
            pass
    
    def run_all_tests(self):
        """Run all integration tests"""
        print("🧪 Starting Integration Tests")
        print("=" * 50)
        
        try:
            # Test 1: Connection
            if not self.test_connection():
                print("\n❌ Integration test failed: Cannot connect to reStrikeVTA")
                print("   Please ensure reStrikeVTA is running with UDP server on port 8888")
                return False
            
            # Test 2: Simulator connection
            if not self.test_simulator_connection():
                print("\n❌ Integration test failed: Simulator cannot connect")
                return False
            
            # Test 3: Basic events
            if not self.test_basic_events():
                print("\n❌ Integration test failed: Cannot send events")
                return False
            
            # Test 4: Match scenario
            if not self.test_match_scenario():
                print("\n❌ Integration test failed: Match scenario failed")
                return False
            
            print("\n" + "=" * 50)
            print("🎉 All Integration Tests Passed!")
            print("✅ reStrikeVTA is ready to receive events from the simulator")
            print("\n📋 Test Results:")
            for test, result in self.test_results.items():
                status = "✅ PASS" if result else "❌ FAIL"
                print(f"   {test}: {status}")
            
            return True
            
        except Exception as e:
            print(f"\n❌ Integration test error: {e}")
            return False
        finally:
            self.cleanup()

def main():
    """Main test function"""
    print("🔧 reStrikeVTA Integration Test")
    print("Testing tkStrikeHardwareSimulator with reStrikeVTA")
    print()
    
    # Check if reStrikeVTA is running
    print("📋 Prerequisites:")
    print("   1. reStrikeVTA should be running (cargo tauri dev)")
    print("   2. UDP server should be listening on port 8888")
    print("   3. WebSocket server should be running on port 3001")
    print()
    
    input("Press Enter to start integration tests...")
    
    # Run tests
    tester = IntegrationTester()
    success = tester.run_all_tests()
    
    if success:
        print("\n🚀 Integration successful! You can now:")
        print("   - Use interactive mode: python tkstrike_hardware_simulator.py --interactive")
        print("   - Run demos: python tkstrike_hardware_simulator.py --demo basic")
        print("   - Test random events: python tkstrike_hardware_simulator.py --random 60")
    else:
        print("\n🔧 Troubleshooting:")
        print("   1. Ensure reStrikeVTA is running: cd src-tauri && cargo tauri dev")
        print("   2. Check UDP port 8888 is open: netstat -an | findstr 8888")
        print("   3. Verify WebSocket port 3001 is open: netstat -an | findstr 3001")
        print("   4. Check reStrikeVTA logs for errors")
    
    return 0 if success else 1

if __name__ == "__main__":
    exit(main()) 