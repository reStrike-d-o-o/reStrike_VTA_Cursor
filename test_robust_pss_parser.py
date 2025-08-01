#!/usr/bin/env python3
"""
Comprehensive Test Script for Robust PSS Parser
Tests all event types, edge cases, and malformed data handling
"""

import socket
import time
import json
import random
from typing import List, Dict, Any

class RobustPssTester:
    def __init__(self, host: str = "127.0.0.1", port: int = 8888):
        self.host = host
        self.port = port
        self.socket = None
        
    def connect(self):
        """Connect to UDP server"""
        try:
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            self.socket.settimeout(5.0)
            print(f"🔌 Connected to {self.host}:{self.port}")
            return True
        except Exception as e:
            print(f"❌ Failed to connect: {e}")
            return False
    
    def disconnect(self):
        """Disconnect from UDP server"""
        if self.socket:
            self.socket.close()
            print("🔌 Disconnected")
    
    def send_message(self, message: str) -> bool:
        """Send a message to the UDP server"""
        try:
            self.socket.sendto(message.encode('utf-8'), (self.host, self.port))
            print(f"📤 Sent: {message}")
            return True
        except Exception as e:
            print(f"❌ Failed to send '{message}': {e}")
            return False
    
    def test_valid_events(self):
        """Test all valid event types from PSS protocol"""
        print("\n" + "="*60)
        print("🧪 TESTING VALID PSS EVENTS")
        print("="*60)
        
        valid_events = [
            # Points events
            "pt1;1;", "pt1;2;", "pt1;3;", "pt1;4;", "pt1;5;",
            "pt2;1;", "pt2;2;", "pt2;3;", "pt2;4;", "pt2;5;",
            
            # Hit level events
            "hl1;1;", "hl1;50;", "hl1;100;",
            "hl2;1;", "hl2;50;", "hl2;100;",
            
            # Warnings/Gam-jeom events
            "wg1;0;wg2;0;", "wg1;1;wg2;2;", "wg1;4;wg2;3;",
            "wg2;1;", "wg2;4;",
            
            # Injury events
            "ij0;1:23;", "ij0;1:23;show;", "ij0;1:23;hide;", "ij0;1:23;reset;",
            "ij1;1:23;", "ij1;1:23;show;", "ij1;1:23;hide;", "ij1;1:23;reset;",
            "ij2;1:23;", "ij2;1:23;show;", "ij2;1:23;hide;", "ij2;1:23;reset;",
            "ij1;0:45;", "ij2;0:44;",
            
            # Challenge/IVR events
            "ch0;", "ch0;0;", "ch0;1;", "ch0;1;0;", "ch0;1;1;", "ch0;-1;",
            "ch1;", "ch1;0;", "ch1;1;", "ch1;1;0;", "ch1;1;1;", "ch1;-1;",
            "ch2;", "ch2;0;", "ch2;1;", "ch2;1;0;", "ch2;1;1;", "ch2;-1;",
            
            # Break events
            "brk;0:59;", "brk;58;", "brk;0:00;stopEnd;", "brk;0;stopEnd;",
            
            # Winner rounds events
            "wrd;rd1;0;rd2;0;rd3;0;", "wrd;rd1;2;rd2;1;rd3;0;",
            "wrd;rd1;1;rd2;2;rd3;1;",
            
            # Winner events
            "wmh;Nicolas DESMOND;", "wmh;Nicolas DESMOND;2-0 PTF;",
            "wmh;Marcel THIBAULT;1-2 PTF;",
            
            # Athletes events
            "at1;N. DESMOND;Nicolas DESMOND;MRN;at2;M. THIBAULT;Marcel THIBAULT;SUI;",
            "at1;J. SMITH;John Smith;USA;at2;A. JONES;Alice Jones;GBR;",
            
            # Match configuration
            "mch;101;Round of 16;M- 80 kg;1;#0000ff;#FFFFFF;#ff0000;#FFFFFF;a14ddd5c;Senior;3;120;cntDown;18;1;",
            "mch;202;Quarter Final;F- 67 kg;2;#00ff00;#000000;#ff00ff;#FFFFFF;b25eee6d;Junior;3;180;cntUp;20;1;",
            
            # Scores events
            "s11;0;", "s11;3;", "s11;5;",
            "s21;0;", "s21;3;", "s21;5;",
            "s12;0;", "s12;3;", "s12;5;",
            "s22;0;", "s22;3;", "s22;5;",
            "s13;0;", "s13;3;", "s13;5;",
            "s23;0;", "s23;3;", "s23;5;",
            
            # Current scores
            "sc1;0;", "sc1;3;", "sc1;5;",
            "sc2;0;", "sc2;3;", "sc2;5;",
            
            # Clock events
            "clk;2:00;", "clk;2:00;start;", "clk;1:59;", "clk;1:50;stop;",
            "clk;0:30;", "clk;0:01;",
            
            # Round events
            "rnd;1;", "rnd;2;", "rnd;3;",
            
            # System events
            "pre;FightLoaded;", "rdy;FightReady;",
            
            # Winner events
            "win;BLUE;", "win;RED;",
            
            # Athlete video time
            "avt;0;", "avt;30;", "avt;60;",
            
            # Additional events (handled as raw)
            "ref;some_data;", "sup;supervision_data;", "rst;reset_data;", "rsr;reset_data;",
        ]
        
        for event in valid_events:
            self.send_message(event)
            time.sleep(0.1)  # Small delay between messages
    
    def test_edge_cases(self):
        """Test edge cases and boundary conditions"""
        print("\n" + "="*60)
        print("🔍 TESTING EDGE CASES")
        print("="*60)
        
        edge_cases = [
            # Empty and whitespace messages
            "", "   ", ";;;",
            
            # Messages with extra semicolons
            "pt1;1;;;", "hl1;50;;", "wg1;1;wg2;2;;;",
            
            # Messages with missing parts
            "pt1;", "hl1;", "wg1;", "ij1;", "ch0;", "brk;",
            "wrd;", "wmh;", "at1;", "mch;", "s11;", "sc1;", "clk;", "rnd;",
            
            # Messages with invalid data types
            "pt1;abc;", "hl1;xyz;", "wg1;def;", "ij1;invalid;",
            "ch0;abc;", "brk;invalid;", "s11;xyz;", "sc1;def;",
            "clk;invalid;", "rnd;abc;", "avt;xyz;",
            
            # Messages with out-of-range values
            "pt1;0;", "pt1;6;", "pt1;255;",
            "hl1;0;", "hl1;101;", "hl1;255;",
            "wg1;11;", "wg1;255;",
            "s11;51;", "s11;255;",
            "sc1;51;", "sc1;255;",
            "rnd;0;", "rnd;11;", "rnd;255;",
            
            # Messages with invalid time formats
            "ij1;invalid_time;", "ij1;1:60;", "ij1;60:00;", "ij1;-1:30;",
            "brk;invalid_time;", "brk;1:60;", "brk;60:00;",
            "clk;invalid_time;", "clk;1:60;", "clk;60:00;",
            
            # Messages with invalid color formats
            "mch;101;Round of 16;M- 80 kg;1;invalid;#FFFFFF;#ff0000;#FFFFFF;a14ddd5c;Senior;3;120;cntDown;18;1;",
            "mch;101;Round of 16;M- 80 kg;1;#0000ff;invalid;#ff0000;#FFFFFF;a14ddd5c;Senior;3;120;cntDown;18;1;",
            "mch;101;Round of 16;M- 80 kg;1;#0000ff;#FFFFFF;invalid;#FFFFFF;a14ddd5c;Senior;3;120;cntDown;18;1;",
            "mch;101;Round of 16;M- 80 kg;1;#0000ff;#FFFFFF;#ff0000;invalid;a14ddd5c;Senior;3;120;cntDown;18;1;",
            
            # Messages with invalid actions
            "ij1;1:23;invalid_action;", "brk;0:59;invalid_action;", "clk;2:00;invalid_action;",
            
            # Messages with very long strings
            "wmh;" + "A" * 200 + ";",  # Very long name
            "at1;" + "A" * 100 + ";" + "B" * 150 + ";" + "C" * 20 + ";at2;" + "D" * 100 + ";" + "E" * 150 + ";" + "F" * 20 + ";",
            
            # Messages with special characters
            "wmh;Nicolas O'Connor;", "wmh;José María;", "wmh;李小龙;",
            "at1;N. O'Connor;Nicolas O'Connor;IRL;at2;M. O'Reilly;Mary O'Reilly;IRL;",
            
            # Messages with numbers in strings
            "wmh;John123;", "at1;A1;B2;C3;at2;D4;E5;F6;",
            
            # Messages with mixed case
            "win;blue;", "win;red;", "win;Blue;", "win;Red;",
            
            # Messages with extra whitespace
            " pt1;1; ", "  hl1;50;  ", "  wg1;1;wg2;2;  ",
            
            # Messages with tabs and newlines
            "pt1;1;\t", "hl1;50;\n", "wg1;1;wg2;2;\r\n",
        ]
        
        for case in edge_cases:
            self.send_message(case)
            time.sleep(0.1)
    
    def test_malformed_data(self):
        """Test malformed and corrupted data"""
        print("\n" + "="*60)
        print("🚨 TESTING MALFORMED DATA")
        print("="*60)
        
        malformed_data = [
            # Completely invalid messages
            "invalid_message", "random_text", "garbage_data",
            
            # Messages with wrong separators
            "pt1,1", "hl1.50", "wg1:1:wg2:2",
            
            # Messages with missing separators
            "pt11", "hl150", "wg11wg22",
            
            # Messages with extra separators
            "pt1;;1", "hl1;;50", "wg1;;1;;wg2;;2",
            
            # Messages with wrong event codes
            "pt3;1;", "hl3;50;", "wg3;1;", "ij3;1:23;", "ch3;1;",
            "s31;0;", "sc3;0;", "rnd4;1;", "avt3;0;",
            
            # Messages with negative numbers (as strings)
            "pt1;-1;", "hl1;-50;", "wg1;-1;", "s11;-1;", "sc1;-1;", "rnd;-1;",
            
            # Messages with floating point numbers
            "pt1;1.5;", "hl1;50.7;", "wg1;1.2;", "s11;3.8;", "sc1;2.1;", "rnd;1.9;",
            
            # Messages with hexadecimal numbers
            "pt1;0x01;", "hl1;0x32;", "wg1;0x01;", "s11;0x03;", "sc1;0x02;", "rnd;0x01;",
            
            # Messages with binary numbers
            "pt1;0b1;", "hl1;0b110010;", "wg1;0b1;", "s11;0b11;", "sc1;0b10;", "rnd;0b1;",
            
            # Messages with unicode characters
            "wmh;José María;", "at1;José;José María;ESP;at2;María;María José;ESP;",
            
            # Messages with control characters
            "pt1;1\x00;", "hl1;50\x01;", "wg1;1\x02;",
            
            # Messages with null bytes
            "pt1;1\0;", "hl1;50\0;", "wg1;1\0;",
            
            # Messages with backslashes
            "wmh;C:\\path\\to\\file;", "at1;A\\B;C\\D;USA;at2;E\\F;G\\H;GBR;",
            
            # Messages with quotes
            "wmh;'John Smith';", "wmh;\"Jane Doe\";",
            "at1;'A';'A B';USA;at2;'C';'C D';GBR;",
            
            # Messages with SQL injection attempts
            "wmh;'; DROP TABLE events; --;",
            "at1;'; DROP TABLE athletes; --;';'; DROP TABLE athletes; --;';USA;at2;'; DROP TABLE athletes; --;';'; DROP TABLE athletes; --;';GBR;",
            
            # Messages with XSS attempts
            "wmh;<script>alert('xss')</script>;",
            "at1;<script>alert('xss')</script>;<script>alert('xss')</script>;USA;at2;<script>alert('xss')</script>;<script>alert('xss')</script>;GBR;",
            
            # Messages with very large numbers
            "pt1;999999999;", "hl1;999999999;", "wg1;999999999;",
            "s11;999999999;", "sc1;999999999;", "rnd;999999999;",
            
            # Messages with negative large numbers
            "pt1;-999999999;", "hl1;-999999999;", "wg1;-999999999;",
            "s11;-999999999;", "sc1;-999999999;", "rnd;-999999999;",
        ]
        
        for data in malformed_data:
            self.send_message(data)
            time.sleep(0.1)
    
    def test_protocol_violations(self):
        """Test protocol violations and unexpected formats"""
        print("\n" + "="*60)
        print("⚠️ TESTING PROTOCOL VIOLATIONS")
        print("="*60)
        
        violations = [
            # Wrong number of arguments
            "pt1;1;2;", "hl1;50;100;", "wg1;1;2;3;",
            "ij1;1:23;show;extra;", "ch0;1;0;extra;", "brk;0:59;stop;extra;",
            "wrd;rd1;0;extra;", "wmh;John;Smith;extra;",
            "at1;A;B;C;D;at2;E;F;G;H;I;", "mch;101;A;B;C;D;E;F;G;H;I;J;K;L;M;N;O;P;",
            "s11;0;1;", "sc1;0;1;", "clk;2:00;start;extra;", "rnd;1;2;",
            
            # Missing required arguments
            "pt1;", "hl1;", "wg1;", "ij1;", "ch0;", "brk;",
            "wrd;", "wmh;", "at1;", "mch;", "s11;", "sc1;", "clk;", "rnd;",
            
            # Wrong argument types
            "pt1;abc;", "hl1;xyz;", "wg1;def;", "ij1;invalid;",
            "ch0;abc;", "brk;invalid;", "s11;xyz;", "sc1;def;",
            "clk;invalid;", "rnd;abc;", "avt;xyz;",
            
            # Invalid time formats
            "ij1;1:60;", "ij1;60:00;", "ij1;-1:30;", "ij1;1.5:30;",
            "brk;1:60;", "brk;60:00;", "brk;-1:30;", "brk;1.5:30;",
            "clk;1:60;", "clk;60:00;", "clk;-1:30;", "clk;1.5:30;",
            
            # Invalid color formats
            "mch;101;Round of 16;M- 80 kg;1;invalid;#FFFFFF;#ff0000;#FFFFFF;a14ddd5c;Senior;3;120;cntDown;18;1;",
            "mch;101;Round of 16;M- 80 kg;1;#0000ff;invalid;#ff0000;#FFFFFF;a14ddd5c;Senior;3;120;cntDown;18;1;",
            "mch;101;Round of 16;M- 80 kg;1;#0000ff;#FFFFFF;invalid;#FFFFFF;a14ddd5c;Senior;3;120;cntDown;18;1;",
            "mch;101;Round of 16;M- 80 kg;1;#0000ff;#FFFFFF;#ff0000;invalid;a14ddd5c;Senior;3;120;cntDown;18;1;",
            
            # Invalid actions
            "ij1;1:23;invalid_action;", "brk;0:59;invalid_action;", "clk;2:00;invalid_action;",
            
            # Invalid winner values
            "win;INVALID;", "win;GREEN;", "win;YELLOW;", "win;123;",
            
            # Invalid round winners
            "wrd;rd1;3;rd2;0;rd3;0;", "wrd;rd1;0;rd2;3;rd3;0;", "wrd;rd1;0;rd2;0;rd3;3;",
            
            # Invalid challenge values
            "ch0;2;", "ch0;3;", "ch0;255;", "ch1;2;", "ch1;3;", "ch1;255;",
            "ch2;2;", "ch2;3;", "ch2;255;",
            
            # Invalid point types
            "pt1;0;", "pt1;6;", "pt1;255;", "pt2;0;", "pt2;6;", "pt2;255;",
            
            # Invalid hit levels
            "hl1;0;", "hl1;101;", "hl1;255;", "hl2;0;", "hl2;101;", "hl2;255;",
            
            # Invalid warning counts
            "wg1;11;", "wg1;255;", "wg2;11;", "wg2;255;",
            
            # Invalid scores
            "s11;51;", "s11;255;", "s21;51;", "s21;255;",
            "s12;51;", "s12;255;", "s22;51;", "s22;255;",
            "s13;51;", "s13;255;", "s23;51;", "s23;255;",
            
            # Invalid current scores
            "sc1;51;", "sc1;255;", "sc2;51;", "sc2;255;",
            
            # Invalid rounds
            "rnd;0;", "rnd;11;", "rnd;255;",
            
            # Invalid video time
            "avt;256;", "avt;1000;", "avt;255;",
        ]
        
        for violation in violations:
            self.send_message(violation)
            time.sleep(0.1)
    
    def test_complete_match_flow(self):
        """Test a complete match flow with realistic data"""
        print("\n" + "="*60)
        print("🏆 TESTING COMPLETE MATCH FLOW")
        print("="*60)
        
        match_flow = [
            # Connection start
            "Udp Port 6000 connected;",
            
            # Load the match
            "pre;FightLoaded;",
            "at1;N. DESMOND;Nicolas DESMOND;MRN;at2;M. THIBAULT;Marcel THIBAULT;SUI;",
            "mch;101;Round of 16;M- 80 kg;1;#0000ff;#FFFFFF;#ff0000;#FFFFFF;a14ddd5c;Senior;3;120;cntDown;18;1;",
            "wg1;0;wg2;0;",
            "wrd;rd1;0;rd2;0;rd3;0;",
            "s11;0;s21;0;s12;0;s22;0;s13;0;s23;0;",
            "sc1;0;sc2;0;",
            "avt;0;",
            "clk;2:00;",
            "rnd;1;",
            "rdy;FightReady;",
            
            # Start the match
            "rnd;1;",
            "clk;2:00;start;",
            "clk;1:59;",
            "clk;1:58;",
            "clk;1:57;",
            
            # First point (Head kick - Blue)
            "hl1;85;",  # Hit level first
            "pt1;3;",   # Then point
            "s11;3;s21;0;s12;0;s22;0;s13;0;s23;0;",
            "sc1;3;sc2;0;",
            
            # Second point (Body kick - Red)
            "hl2;78;",  # Hit level first
            "pt2;2;",   # Then point
            "s11;3;s21;2;s12;0;s22;0;s13;0;s23;0;",
            "sc1;3;sc2;2;",
            
            # Third point (Punch - Blue)
            "hl1;65;",  # Hit level first
            "pt1;1;",   # Then point
            "s11;4;s21;2;s12;0;s22;0;s13;0;s23;0;",
            "sc1;4;sc2;2;",
            
            # Warning (Red athlete)
            "wg1;0;wg2;1;",
            
            # Injury time (Red athlete)
            "ij2;1:30;show;",
            "ij2;1:29;",
            "ij2;1:28;",
            "ij2;1:27;hide;",
            
            # Challenge (Red athlete)
            "ch2;1;",   # Challenge accepted
            "ch2;1;0;", # Challenge lost
            
            # Continue match
            "clk;1:30;",
            "clk;1:29;",
            
            # Fourth point (Technical head - Blue)
            "hl1;92;",  # Hit level first
            "pt1;5;",   # Then point
            "s11;9;s21;2;s12;0;s22;0;s13;0;s23;0;",
            "sc1;9;sc2;2;",
            
            # Round 1 ends
            "clk;0:00;stop;",
            "wrd;rd1;1;rd2;0;rd3;0;",  # Blue wins round 1
            
            # Round 2 starts
            "rnd;2;",
            "clk;2:00;start;",
            "clk;1:59;",
            
            # Fifth point (Body kick - Red)
            "hl2;88;",  # Hit level first
            "pt2;2;",   # Then point
            "s11;9;s21;4;s12;0;s22;0;s13;0;s23;0;",
            "sc1;9;sc2;4;",
            
            # Sixth point (Punch - Blue)
            "hl1;72;",  # Hit level first
            "pt1;1;",   # Then point
            "s11;10;s21;4;s12;0;s22;0;s13;0;s23;0;",
            "sc1;10;sc2;4;",
            
            # Round 2 ends
            "clk;0:00;stop;",
            "wrd;rd1;1;rd2;1;rd3;0;",  # Blue wins round 2
            
            # Round 3 starts
            "rnd;3;",
            "clk;2:00;start;",
            "clk;1:59;",
            
            # Seventh point (Head kick - Red)
            "hl2;95;",  # Hit level first
            "pt2;3;",   # Then point
            "s11;10;s21;7;s12;0;s22;0;s13;0;s23;0;",
            "sc1;10;sc2;7;",
            
            # Eighth point (Technical body - Blue)
            "hl1;89;",  # Hit level first
            "pt1;4;",   # Then point
            "s11;14;s21;7;s12;0;s22;0;s13;0;s23;0;",
            "sc1;14;sc2;7;",
            
            # Round 3 ends
            "clk;0:00;stop;",
            "wrd;rd1;1;rd2;1;rd3;1;",  # Blue wins round 3
            
            # Match ends
            "win;BLUE;",
            "wmh;N. DESMOND;2-1 PTF;",
            
            # Connection end
            "Udp Port 6000 disconnected;",
        ]
        
        for event in match_flow:
            self.send_message(event)
            time.sleep(0.2)  # Longer delay for realistic flow
    
    def test_stress_conditions(self):
        """Test stress conditions and high-frequency messages"""
        print("\n" + "="*60)
        print("💥 TESTING STRESS CONDITIONS")
        print("="*60)
        
        # Rapid fire messages
        print("🚀 Sending rapid fire messages...")
        for i in range(100):
            self.send_message(f"pt1;{i % 5 + 1};")
            time.sleep(0.01)  # Very fast
        
        # Large messages
        print("📏 Sending large messages...")
        large_name = "A" * 500
        self.send_message(f"wmh;{large_name};")
        self.send_message(f"at1;{large_name};{large_name};USA;at2;{large_name};{large_name};GBR;")
        
        # Mixed message types rapidly
        print("🔄 Sending mixed message types rapidly...")
        messages = [
            "pt1;1;", "hl1;50;", "wg1;1;wg2;0;", "clk;1:30;",
            "pt2;2;", "hl2;75;", "wg1;1;wg2;1;", "clk;1:29;",
            "pt1;3;", "hl1;85;", "wg1;2;wg2;1;", "clk;1:28;",
        ]
        for _ in range(10):  # Repeat 10 times
            for msg in messages:
                self.send_message(msg)
                time.sleep(0.01)
    
    def run_all_tests(self):
        """Run all test categories"""
        print("🧪 ROBUST PSS PARSER TEST SUITE")
        print("="*60)
        
        if not self.connect():
            return
        
        try:
            self.test_valid_events()
            self.test_edge_cases()
            self.test_malformed_data()
            self.test_protocol_violations()
            self.test_complete_match_flow()
            self.test_stress_conditions()
            
            print("\n" + "="*60)
            print("✅ ALL TESTS COMPLETED")
            print("="*60)
            print("📊 Test Summary:")
            print("  • Valid Events: Tested all PSS protocol event types")
            print("  • Edge Cases: Tested boundary conditions and edge cases")
            print("  • Malformed Data: Tested corrupted and invalid data")
            print("  • Protocol Violations: Tested protocol rule violations")
            print("  • Complete Match Flow: Tested realistic match scenario")
            print("  • Stress Conditions: Tested high-frequency and large messages")
            print("\n🔍 Check the application logs for parsing results and any errors.")
            
        finally:
            self.disconnect()

def main():
    """Main function to run the test suite"""
    tester = RobustPssTester()
    tester.run_all_tests()

if __name__ == "__main__":
    main() 