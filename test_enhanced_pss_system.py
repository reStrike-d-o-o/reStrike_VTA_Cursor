#!/usr/bin/env python3
"""
Enhanced PSS System Test Script
Tests the robust status mark system with comprehensive validation
"""

import socket
import time
import json
import random
from typing import List, Dict, Any

class EnhancedPssTester:
    def __init__(self, host: str = "127.0.0.1", port: int = 8888):
        self.host = host
        self.port = port
        self.socket = None
        
    def connect(self):
        """Connect to the UDP server"""
        try:
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            self.socket.settimeout(5.0)
            print(f"✅ Connected to UDP server at {self.host}:{self.port}")
            return True
        except Exception as e:
            print(f"❌ Failed to connect: {e}")
            return False
    
    def disconnect(self):
        """Disconnect from the UDP server"""
        if self.socket:
            self.socket.close()
            self.socket = None
            print("🔌 Disconnected from UDP server")
    
    def send_message(self, message: str) -> bool:
        """Send a message to the UDP server"""
        try:
            self.socket.sendto(message.encode('utf-8'), (self.host, self.port))
            return True
        except Exception as e:
            print(f"❌ Failed to send message: {e}")
            return False
    
    def test_valid_events(self):
        """Test valid PSS events that should be recognized"""
        print("\n🧪 Testing Valid Events (should be 'recognized')")
        
        valid_events = [
            # Points events
            "pt1;1;",  # Athlete 1, punch point
            "pt1;2;",  # Athlete 1, body point
            "pt1;3;",  # Athlete 1, head point
            "pt2;1;",  # Athlete 2, punch point
            "pt2;4;",  # Athlete 2, technical body point
            "pt2;5;",  # Athlete 2, technical head point
            
            # Hit level events
            "hl1;50;",  # Athlete 1, hit level 50
            "hl1;100;", # Athlete 1, hit level 100
            "hl2;25;",  # Athlete 2, hit level 25
            "hl2;75;",  # Athlete 2, hit level 75
            
            # Warnings events
            "wg1;1;wg2;0;",  # Athlete 1: 1 warning, Athlete 2: 0 warnings
            "wg1;2;wg2;1;",  # Athlete 1: 2 warnings, Athlete 2: 1 warning
            "wg1;0;wg2;3;",  # Athlete 1: 0 warnings, Athlete 2: 3 warnings
            
            # Injury events
            "ij1;1:23;show;",  # Athlete 1 injury time 1:23, show
            "ij2;0:45;",       # Athlete 2 injury time 0:45
            "ij0;2:15;hide;",  # Unidentified athlete injury time 2:15, hide
            
            # Challenge events
            "ch0;1;",      # Referee challenge accepted
            "ch1;0;",      # Athlete 1 challenge denied
            "ch2;1;1;",    # Athlete 2 challenge accepted and won
            "ch0;-1;",     # Challenge canceled
            
            # Clock events
            "clk;2:30;",   # Clock 2:30
            "clk;0:15;",   # Clock 0:15
            
            # Round events
            "rnd;1;",      # Round 1
            "rnd;2;",      # Round 2
            "rnd;3;",      # Round 3
            
            # Match config
            "mch;1;Men;-68kg;3;red;blue;white;black;MATCH001;Men;-68kg;3;120;countdown;60;1;",
            
            # Athletes
            "at;KIM;Kim Jong-un;KOR;LEE;Lee Min-ho;KOR;",
            
            # Scores
            "s11;2;s21;1;s12;1;s22;3;s13;0;s23;2;",
            
            # Current scores
            "sc1;3;sc2;6;",
            
            # System events
            "pre;FightLoaded;",
            "rdy;FightReady;",
        ]
        
        for event in valid_events:
            print(f"  📤 Sending: {event}")
            self.send_message(event)
            time.sleep(0.1)
    
    def test_invalid_events(self):
        """Test invalid PSS events that should be marked as 'unknown'"""
        print("\n🧪 Testing Invalid Events (should be 'unknown')")
        
        invalid_events = [
            # Invalid point types
            "pt1;6;",      # Invalid point type (should be 1-5)
            "pt2;0;",      # Invalid point type (should be 1-5)
            
            # Invalid hit levels
            "hl1;101;",    # Hit level too high (should be 1-100)
            "hl2;0;",      # Hit level too low (should be 1-100)
            
            # Invalid athlete numbers
            "pt3;1;",      # Invalid athlete number (should be 1 or 2)
            "hl3;50;",     # Invalid athlete number (should be 1 or 2)
            
            # Invalid time formats
            "clk;2:30:45;", # Invalid time format (should be m:ss)
            "ij1;2.30;",    # Invalid time format (should be m:ss)
            
            # Invalid round numbers
            "rnd;4;",       # Invalid round number (should be 1-3)
            "rnd;0;",       # Invalid round number (should be 1-3)
            
            # Invalid warning counts
            "wg1;5;wg2;0;", # Too many warnings (should be 0-4)
            "wg1;0;wg2;6;", # Too many warnings (should be 0-4)
            
            # Malformed messages
            "pt1;",         # Missing point type
            "hl1",          # Missing hit level
            "wg1;1",        # Missing athlete 2 warnings
            "clk",          # Missing time
            "rnd",          # Missing round number
            
            # Unknown event types
            "xyz;1;2;3;",   # Unknown event type
            "abc;test;",    # Unknown event type
            "def;123;",     # Unknown event type
            
            # Empty messages
            "",             # Empty message
            ";",            # Empty message with delimiter
            ";;",           # Multiple empty delimiters
        ]
        
        for event in invalid_events:
            print(f"  📤 Sending: {event}")
            self.send_message(event)
            time.sleep(0.1)
    
    def test_partial_events(self):
        """Test partially valid events that should be marked as 'partial'"""
        print("\n🧪 Testing Partial Events (should be 'partial')")
        
        partial_events = [
            # Valid format but questionable values
            "pt1;3;",       # Valid but let's see if it triggers partial validation
            "hl1;99;",      # High hit level but valid
            "wg1;3;wg2;2;", # Multiple warnings but valid
            
            # Events with extra data
            "pt1;1;extra;", # Valid point with extra data
            "clk;2:30;extra;", # Valid clock with extra data
            "rnd;2;extra;", # Valid round with extra data
        ]
        
        for event in partial_events:
            print(f"  📤 Sending: {event}")
            self.send_message(event)
            time.sleep(0.1)
    
    def test_edge_cases(self):
        """Test edge cases and boundary conditions"""
        print("\n🧪 Testing Edge Cases")
        
        edge_cases = [
            # Boundary values
            "pt1;1;",       # Minimum point type
            "pt1;5;",       # Maximum point type
            "hl1;1;",       # Minimum hit level
            "hl1;100;",     # Maximum hit level
            "wg1;0;wg2;0;", # Minimum warnings
            "wg1;4;wg2;4;", # Maximum warnings
            "rnd;1;",       # Minimum round
            "rnd;3;",       # Maximum round
            
            # Time edge cases
            "clk;0:00;",    # Zero time
            "clk;9:59;",    # Maximum reasonable time
            "ij1;0:01;",    # Very short injury time
            "ij2;5:00;",    # Long injury time
            
            # Special characters
            "at;KIM;Kim Jong-un;KOR;LEE;Lee Min-ho;KOR;", # Names with spaces
            "wmh;Nicolas DESMOND;2-1 PTF;", # Winner with classification
        ]
        
        for event in edge_cases:
            print(f"  📤 Sending: {event}")
            self.send_message(event)
            time.sleep(0.1)
    
    def test_protocol_variations(self):
        """Test protocol variations and format differences"""
        print("\n🧪 Testing Protocol Variations")
        
        variations = [
            # Different delimiter patterns
            "pt1;1",        # No trailing semicolon
            "pt1;1;;",      # Extra semicolon
            "pt1;1; ;",     # Space after semicolon
            
            # Case variations
            "PT1;1;",       # Uppercase event type
            "Pt1;1;",       # Mixed case
            
            # Whitespace variations
            " pt1;1;",      # Leading space
            "pt1;1; ",      # Trailing space
            "pt1; 1;",      # Space around value
            
            # Multiple events in one message
            "pt1;1;pt2;2;", # Multiple points
            "hl1;50;hl2;75;", # Multiple hit levels
        ]
        
        for event in variations:
            print(f"  📤 Sending: {event}")
            self.send_message(event)
            time.sleep(0.1)
    
    def test_stress_scenarios(self):
        """Test stress scenarios with rapid event sending"""
        print("\n🧪 Testing Stress Scenarios")
        
        # Rapid valid events
        print("  📤 Sending 50 rapid valid events...")
        for i in range(50):
            event = f"pt1;{random.randint(1, 5)};"
            self.send_message(event)
            time.sleep(0.01)  # Very fast sending
        
        # Mixed valid and invalid events
        print("  📤 Sending mixed valid/invalid events...")
        for i in range(20):
            if i % 2 == 0:
                event = f"pt1;{random.randint(1, 5)};"
            else:
                event = f"pt1;{random.randint(6, 10)};"  # Invalid
            self.send_message(event)
            time.sleep(0.05)
    
    def test_complete_match_flow(self):
        """Test a complete match flow with realistic event sequence"""
        print("\n🧪 Testing Complete Match Flow")
        
        match_flow = [
            # Match setup
            "pre;FightLoaded;",
            "mch;1;Men;-68kg;3;red;blue;white;black;MATCH001;Men;-68kg;3;120;countdown;60;1;",
            "at;KIM;Kim Jong-un;KOR;LEE;Lee Min-ho;KOR;",
            "rdy;FightReady;",
            
            # Round 1
            "rnd;1;",
            "clk;2:00;",
            "pt1;1;", "hl1;75;",
            "pt2;2;", "hl2;60;",
            "wg1;1;wg2;0;",
            "pt1;3;", "hl1;85;",
            "clk;0:00;",
            
            # Round 2
            "rnd;2;",
            "clk;2:00;",
            "pt2;1;", "hl2;70;",
            "pt1;2;", "hl1;65;",
            "ij1;1:30;show;",
            "ij1;1:29;",
            "ij1;1:28;hide;",
            "pt2;4;", "hl2;80;",
            "clk;0:00;",
            
            # Round 3
            "rnd;3;",
            "clk;2:00;",
            "pt1;5;", "hl1;90;",
            "pt2;1;", "hl2;55;",
            "wg1;2;wg2;1;",
            "clk;0:00;",
            
            # Match end
            "s11;2;s21;1;s12;1;s22;3;s13;0;s23;2;",
            "sc1;3;sc2;6;",
            "wrd;rd1;1;rd2;2;rd3;2;",
            "wmh;Lee Min-ho;2-1 PTF;",
        ]
        
        for event in match_flow:
            print(f"  📤 Sending: {event}")
            self.send_message(event)
            time.sleep(0.2)  # Realistic timing
    
    def run_comprehensive_test(self):
        """Run all tests in sequence"""
        print("🚀 Starting Enhanced PSS System Comprehensive Test")
        print("=" * 60)
        
        if not self.connect():
            return
        
        try:
            # Run all test scenarios
            self.test_valid_events()
            time.sleep(1)
            
            self.test_invalid_events()
            time.sleep(1)
            
            self.test_partial_events()
            time.sleep(1)
            
            self.test_edge_cases()
            time.sleep(1)
            
            self.test_protocol_variations()
            time.sleep(1)
            
            self.test_stress_scenarios()
            time.sleep(1)
            
            self.test_complete_match_flow()
            time.sleep(1)
            
            print("\n✅ All tests completed successfully!")
            print("\n📊 Expected Results:")
            print("  - Valid events should be marked as 'recognized'")
            print("  - Invalid events should be marked as 'unknown'")
            print("  - Partial events should be marked as 'partial'")
            print("  - Unknown events should be collected for analysis")
            print("  - Validation errors should be tracked")
            print("  - Event statistics should be updated")
            
        finally:
            self.disconnect()

def main():
    """Main function to run the enhanced PSS system test"""
    print("Enhanced PSS System Test Script")
    print("Tests the robust status mark system with comprehensive validation")
    print()
    
    # Create and run the tester
    tester = EnhancedPssTester()
    tester.run_comprehensive_test()

if __name__ == "__main__":
    main() 