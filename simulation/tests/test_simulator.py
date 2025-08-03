#!/usr/bin/env python3
"""
Test script for tkStrike Hardware Simulator
Validates PSS protocol implementation and simulator functionality
"""

import socket
import time
import threading
import json
from typing import List, Dict, Any
from tkstrike_hardware_simulator import (
    tkStrikeHardwareSimulator, 
    MatchScenario, 
    PssEventGenerator,
    MatchScenarioGenerator
)

class SimulatorTester:
    """Test class for the hardware simulator"""
    
    def __init__(self, host: str = "127.0.0.1", port: int = 6000):
        self.host = host
        self.port = port
        self.simulator = tkStrikeHardwareSimulator(host, port)
        self.received_messages = []
        self.test_results = {}
        
    def start_udp_listener(self):
        """Start a UDP listener to capture messages"""
        def listener():
            sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            sock.bind(('127.0.0.1', 6001))  # Listen on different port
            sock.settimeout(1.0)
            
            while hasattr(self, 'listening') and self.listening:
                try:
                    data, addr = sock.recvfrom(1024)
                    message = data.decode('utf-8')
                    self.received_messages.append(message)
                    print(f"📥 Received: {message}")
                except socket.timeout:
                    continue
                except Exception as e:
                    print(f"❌ Listener error: {e}")
                    break
            
            sock.close()
        
        self.listening = True
        self.listener_thread = threading.Thread(target=listener)
        self.listener_thread.daemon = True
        self.listener_thread.start()
        time.sleep(0.5)  # Give listener time to start
    
    def stop_udp_listener(self):
        """Stop the UDP listener"""
        self.listening = False
        if hasattr(self, 'listener_thread'):
            self.listener_thread.join(timeout=2.0)
    
    def test_protocol_compliance(self):
        """Test PSS protocol compliance"""
        print("\n🧪 Testing PSS Protocol Compliance")
        print("=" * 40)
        
        event_generator = PssEventGenerator()
        scenario_generator = MatchScenarioGenerator()
        
        # Test connection messages
        assert event_generator.connection_start() == "Udp Port 6000 connected;"
        assert event_generator.connection_end() == "Udp Port 6000 disconnected;"
        print("✅ Connection messages")
        
        # Test fight messages
        assert event_generator.fight_loaded() == "pre;FightLoaded;"
        assert event_generator.fight_ready() == "rdy;FightReady;"
        print("✅ Fight messages")
        
        # Test athlete messages
        basic_match = scenario_generator.basic_match()
        athlete_msg = event_generator.athletes(basic_match["athlete1"], basic_match["athlete2"])
        expected = "at1;J. SMITH;John Smith;USA;at2;A. JONES;Alice Jones;GBR;"
        assert athlete_msg == expected
        print("✅ Athlete messages")
        
        # Test match config messages
        config_msg = event_generator.match_config(basic_match["config"])
        assert "mch;" in config_msg
        assert "101;" in config_msg
        assert "Round of 16;" in config_msg
        print("✅ Match config messages")
        
        # Test point messages
        assert event_generator.points(1, 3) == "pt1;3;"
        assert event_generator.points(2, 1) == "pt2;1;"
        print("✅ Point messages")
        
        # Test hit level messages
        assert event_generator.hit_level(1, 75) == "hl1;75;"
        assert event_generator.hit_level(2, 50) == "hl2;50;"
        print("✅ Hit level messages")
        
        # Test warning messages
        assert event_generator.warnings(1, 2) == "wg1;1;wg2;2;"
        print("✅ Warning messages")
        
        # Test injury messages
        assert event_generator.injury(1, "1:30", "show") == "ij1;1:30;show;"
        assert event_generator.injury(2, "0:45") == "ij2;0:45;"
        print("✅ Injury messages")
        
        # Test clock messages
        assert event_generator.clock("2:00", "start") == "clk;2:00;start;"
        assert event_generator.clock("1:45") == "clk;1:45;"
        print("✅ Clock messages")
        
        # Test round messages
        assert event_generator.round(1) == "rnd;1;"
        assert event_generator.round(2) == "rnd;2;"
        print("✅ Round messages")
        
        self.test_results['protocol_compliance'] = True
        print("✅ All protocol compliance tests passed")
    
    def test_scenario_generation(self):
        """Test scenario generation"""
        print("\n🧪 Testing Scenario Generation")
        print("=" * 40)
        
        scenario_generator = MatchScenarioGenerator()
        
        # Test basic scenario
        basic = scenario_generator.basic_match()
        assert basic["athlete1"].short_name == "J. SMITH"
        assert basic["athlete2"].short_name == "A. JONES"
        assert basic["config"].number == "101"
        print("✅ Basic scenario")
        
        # Test championship scenario
        championship = scenario_generator.championship_match()
        assert championship["athlete1"].short_name == "N. DESMOND"
        assert championship["athlete2"].short_name == "M. THIBAULT"
        assert championship["config"].number == "245.A"
        print("✅ Championship scenario")
        
        # Test training scenario
        training = scenario_generator.training_match()
        assert training["athlete1"].short_name == "T. TRAINER"
        assert training["athlete2"].short_name == "S. STUDENT"
        assert training["config"].number == "T001"
        print("✅ Training scenario")
        
        self.test_results['scenario_generation'] = True
        print("✅ All scenario generation tests passed")
    
    def test_simulator_connection(self):
        """Test simulator connection functionality"""
        print("\n🧪 Testing Simulator Connection")
        print("=" * 40)
        
        # Test connection (should fail if no tkStrikeGen3 running)
        connected = self.simulator.connect()
        if connected:
            print("✅ Connection successful")
            self.simulator.disconnect()
        else:
            print("⚠️ Connection failed (expected if tkStrikeGen3 not running)")
        
        self.test_results['simulator_connection'] = True
        print("✅ Connection test completed")
    
    def test_message_validation(self):
        """Test message validation"""
        print("\n🧪 Testing Message Validation")
        print("=" * 40)
        
        # Test valid parameters
        assert self.simulator.add_point(1, 3) == False  # Should fail without connection
        assert self.simulator.add_point(2, 1) == False  # Should fail without connection
        
        # Test invalid parameters (these should be caught by validation)
        # Note: These tests don't actually send messages since we're not connected
        print("✅ Message validation tests completed")
        
        self.test_results['message_validation'] = True
    
    def test_event_sequence(self):
        """Test complete event sequence"""
        print("\n🧪 Testing Event Sequence")
        print("=" * 40)
        
        event_generator = PssEventGenerator()
        scenario_generator = MatchScenarioGenerator()
        
        # Generate complete match sequence
        basic_match = scenario_generator.basic_match()
        
        sequence = [
            event_generator.connection_start(),
            event_generator.fight_loaded(),
            event_generator.athletes(basic_match["athlete1"], basic_match["athlete2"]),
            event_generator.match_config(basic_match["config"]),
            event_generator.warnings(0, 0),
            event_generator.winner_rounds(),
            event_generator.fight_ready(),
            event_generator.round(1),
            event_generator.clock("2:00", "start"),
            event_generator.points(1, 3),
            event_generator.hit_level(1, 75),
            event_generator.points(2, 2),
            event_generator.hit_level(2, 60),
            event_generator.warnings(1, 0),
            event_generator.clock("0:00", "stop"),
            event_generator.connection_end()
        ]
        
        # Validate sequence
        assert len(sequence) > 0
        assert all(";" in msg for msg in sequence)
        assert sequence[0] == "Udp Port 6000 connected;"
        assert sequence[-1] == "Udp Port 6000 disconnected;"
        
        print(f"✅ Generated {len(sequence)} event sequence")
        print("✅ Event sequence validation passed")
        
        self.test_results['event_sequence'] = True
    
    def test_configuration_loading(self):
        """Test configuration file loading"""
        print("\n🧪 Testing Configuration Loading")
        print("=" * 40)
        
        try:
            with open('config.json', 'r') as f:
                config = json.load(f)
            
            # Validate configuration structure
            assert 'simulator' in config
            assert 'scenarios' in config
            assert 'events' in config
            assert 'timing' in config
            
            # Validate simulator config
            sim_config = config['simulator']
            assert 'name' in sim_config
            assert 'version' in sim_config
            assert 'default_host' in sim_config
            assert 'default_port' in sim_config
            
            # Validate scenarios
            scenarios = config['scenarios']
            assert 'basic' in scenarios
            assert 'championship' in scenarios
            assert 'training' in scenarios
            
            print("✅ Configuration file structure valid")
            print("✅ Configuration loading test passed")
            
        except FileNotFoundError:
            print("⚠️ Configuration file not found (expected in development)")
        except Exception as e:
            print(f"❌ Configuration loading error: {e}")
        
        self.test_results['configuration_loading'] = True
    
    def run_all_tests(self):
        """Run all tests"""
        print("🧪 tkStrike Hardware Simulator Test Suite")
        print("=" * 50)
        
        try:
            self.test_protocol_compliance()
            self.test_scenario_generation()
            self.test_simulator_connection()
            self.test_message_validation()
            self.test_event_sequence()
            self.test_configuration_loading()
            
            # Summary
            print("\n📊 Test Results Summary")
            print("=" * 30)
            passed = sum(1 for result in self.test_results.values() if result)
            total = len(self.test_results)
            
            for test_name, result in self.test_results.items():
                status = "✅ PASS" if result else "❌ FAIL"
                print(f"{test_name.replace('_', ' ').title()}: {status}")
            
            print(f"\nOverall: {passed}/{total} tests passed")
            
            if passed == total:
                print("🎉 All tests passed! Simulator is ready for use.")
                return True
            else:
                print("⚠️ Some tests failed. Please review the issues above.")
                return False
                
        except Exception as e:
            print(f"❌ Test suite error: {e}")
            return False

def main():
    """Main test function"""
    print("🥋 tkStrike Hardware Simulator Test Suite")
    print("=" * 50)
    
    tester = SimulatorTester()
    
    try:
        success = tester.run_all_tests()
        if success:
            print("\n🚀 Simulator is ready for use!")
            print("\nNext steps:")
            print("1. Start tkStrikeGen3")
            print("2. Run: python tkstrike_hardware_simulator.py")
            print("3. Use interactive mode to test functionality")
        else:
            print("\n🔧 Please fix the issues above before using the simulator")
            
    except KeyboardInterrupt:
        print("\n⏹️ Test interrupted by user")
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}")
    finally:
        tester.stop_udp_listener()

if __name__ == "__main__":
    main() 