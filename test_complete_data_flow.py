#!/usr/bin/env python3
"""
Comprehensive test script to verify complete data flow from PSS to overlays.
Tests UDP parsing fixes and WebSocket message format compatibility.
"""

import socket
import time
import json
import requests
import threading
from datetime import datetime

class DataFlowTester:
    def __init__(self, udp_host='127.0.0.1', udp_port=8888, ws_host='127.0.0.1', ws_port=3001):
        self.udp_host = udp_host
        self.udp_port = udp_port
        self.ws_host = ws_host
        self.ws_port = ws_port
        self.received_messages = []
        self.test_results = {}
        
    def send_udp_message(self, message):
        """Send a UDP message to the server."""
        try:
            sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            sock.sendto(message.encode('utf-8'), (self.udp_host, self.udp_port))
            sock.close()
            print(f"📤 UDP Sent: {message}")
            return True
        except Exception as e:
            print(f"❌ Failed to send UDP message: {e}")
            return False
    
    def start_websocket_listener(self):
        """Start a WebSocket client to listen for messages."""
        import websocket
        
        def on_message(ws, message):
            timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
            print(f"📡 [{timestamp}] WebSocket received: {message}")
            self.received_messages.append({
                'timestamp': timestamp,
                'message': message
            })
            
            # Parse and analyze the message
            try:
                data = json.loads(message)
                if data.get('type') == 'pss_event':
                    event_data = data.get('data', {})
                    event_type = event_data.get('type')
                    
                    if event_type == 'current_scores':
                        self.analyze_current_scores_message(event_data)
                    elif event_type == 'scores':
                        self.analyze_scores_message(event_data)
                    elif event_type == 'warnings':
                        self.analyze_warnings_message(event_data)
                        
            except json.JSONDecodeError:
                print(f"⚠️ Non-JSON WebSocket message: {message}")
        
        def on_error(ws, error):
            print(f"❌ WebSocket error: {error}")
        
        def on_close(ws, close_status_code, close_msg):
            print("🔌 WebSocket connection closed")
        
        def on_open(ws):
            print("✅ WebSocket connected")
        
        # Connect to WebSocket server
        ws_url = f"ws://{self.ws_host}:{self.ws_port}"
        print(f"🔗 Connecting to WebSocket: {ws_url}")
        
        self.ws = websocket.WebSocketApp(
            ws_url,
            on_open=on_open,
            on_message=on_message,
            on_error=on_error,
            on_close=on_close
        )
        
        # Run WebSocket in a separate thread
        self.ws_thread = threading.Thread(target=self.ws.run_forever)
        self.ws_thread.daemon = True
        self.ws_thread.start()
        
        # Wait for connection
        time.sleep(1)
    
    def analyze_current_scores_message(self, event_data):
        """Analyze current_scores WebSocket message."""
        print("📊 Analyzing current_scores message:")
        print(f"   Raw data: {event_data.get('raw_data', 'N/A')}")
        print(f"   Structured data: {event_data.get('structured_data', 'N/A')}")
        print(f"   Direct fields: athlete1_score={event_data.get('athlete1_score', 'N/A')}, athlete2_score={event_data.get('athlete2_score', 'N/A')}")
        
        # Check if both raw and structured data are present
        has_raw = 'raw_data' in event_data and event_data['raw_data']
        has_structured = 'structured_data' in event_data and event_data['structured_data']
        has_direct = 'athlete1_score' in event_data and 'athlete2_score' in event_data
        
        print(f"   ✅ Raw data present: {has_raw}")
        print(f"   ✅ Structured data present: {has_structured}")
        print(f"   ✅ Direct fields present: {has_direct}")
        
        self.test_results['current_scores_format'] = {
            'has_raw': has_raw,
            'has_structured': has_structured,
            'has_direct': has_direct,
            'complete': has_raw and has_structured
        }
    
    def analyze_scores_message(self, event_data):
        """Analyze scores WebSocket message."""
        print("📊 Analyzing scores message:")
        print(f"   Raw data: {event_data.get('raw_data', 'N/A')}")
        print(f"   Structured data: {event_data.get('structured_data', 'N/A')}")
        
        has_raw = 'raw_data' in event_data and event_data['raw_data']
        has_structured = 'structured_data' in event_data and event_data['structured_data']
        
        print(f"   ✅ Raw data present: {has_raw}")
        print(f"   ✅ Structured data present: {has_structured}")
        
        self.test_results['scores_format'] = {
            'has_raw': has_raw,
            'has_structured': has_structured,
            'complete': has_raw and has_structured
        }
    
    def analyze_warnings_message(self, event_data):
        """Analyze warnings WebSocket message."""
        print("⚠️ Analyzing warnings message:")
        print(f"   Raw data: {event_data.get('raw_data', 'N/A')}")
        print(f"   Structured data: {event_data.get('structured_data', 'N/A')}")
        
        has_raw = 'raw_data' in event_data and event_data['raw_data']
        has_structured = 'structured_data' in event_data and event_data['structured_data']
        
        print(f"   ✅ Raw data present: {has_raw}")
        print(f"   ✅ Structured data present: {has_structured}")
        
        self.test_results['warnings_format'] = {
            'has_raw': has_raw,
            'has_structured': has_structured,
            'complete': has_raw and has_structured
        }
    
    def test_udp_parsing_fixes(self):
        """Test the UDP parsing fixes for combined messages."""
        print("\n🧪 Testing UDP Parsing Fixes...")
        
        # Test 1: Combined current scores (main issue)
        print("\n📊 Test 1: Combined current scores")
        self.send_udp_message("sc1;0;sc2;1;")
        time.sleep(0.5)
        
        # Test 2: Combined round scores
        print("\n📊 Test 2: Combined round scores")
        self.send_udp_message("s11;0;s21;1;s12;0;s22;0;s13;0;s23;0;")
        time.sleep(0.5)
        
        # Test 3: Combined warnings (should already work)
        print("\n⚠️ Test 3: Combined warnings")
        self.send_udp_message("wg1;1;wg2;0;")
        time.sleep(0.5)
        
        # Test 4: Individual messages (should still work)
        print("\n📊 Test 4: Individual current score")
        self.send_udp_message("sc1;2;")
        time.sleep(0.5)
        
        print("\n📊 Test 5: Individual round score")
        self.send_udp_message("s21;3;")
        time.sleep(0.5)
    
    def test_complete_sequence(self):
        """Test the complete sequence from user's PSS messages."""
        print("\n🎯 Testing Complete Sequence from User's PSS Messages...")
        
        sequence = [
            "sc1;0;sc2;1;",
            "s11;0;s21;1;s12;0;s22;0;s13;0;s23;0;",
            "wg1;1;wg2;0;",
            "sc1;0;sc2;2;",
            "s11;0;s21;2;s12;0;s22;0;s13;0;s23;0;",
            "wg1;2;wg2;0;",
            "sc1;0;sc2;3;",
            "s11;0;s21;3;s12;0;s22;0;s13;0;s23;0;",
            "wg1;3;wg2;0;",
            "sc1;1;sc2;3;",
            "s11;1;s21;3;s12;0;s22;0;s13;0;s23;0;",
            "wg1;3;wg2;1;",
            "sc1;2;sc2;3;",
            "s11;2;s21;3;s12;0;s22;0;s13;0;s23;0;",
            "wg1;3;wg2;2;",
            "sc1;3;sc2;3;",
            "s11;3;s21;3;s12;0;s22;0;s13;0;s23;0;",
            "wg1;3;wg2;3;",
        ]
        
        for i, message in enumerate(sequence, 1):
            print(f"\n📨 Message {i}: {message}")
            self.send_udp_message(message)
            time.sleep(0.3)  # Slightly longer delay for sequence
    
    def run_comprehensive_test(self):
        """Run the complete data flow test."""
        print("🚀 Starting Comprehensive Data Flow Test...")
        print("Make sure the UDP server and WebSocket server are running")
        
        try:
            # Start WebSocket listener
            self.start_websocket_listener()
            time.sleep(2)  # Wait for WebSocket connection
            
            # Run tests
            self.test_udp_parsing_fixes()
            time.sleep(2)
            self.test_complete_sequence()
            
            # Wait for all messages to be processed
            time.sleep(5)
            
            # Print test results
            print("\n📋 Test Results Summary:")
            print("=" * 50)
            
            for test_name, result in self.test_results.items():
                status = "✅ PASS" if result.get('complete', False) else "❌ FAIL"
                print(f"{test_name}: {status}")
                for key, value in result.items():
                    if key != 'complete':
                        print(f"  {key}: {value}")
            
            print(f"\n📡 Total WebSocket messages received: {len(self.received_messages)}")
            
            # Check for specific message types
            current_scores_count = sum(1 for msg in self.received_messages 
                                     if 'current_scores' in msg['message'])
            scores_count = sum(1 for msg in self.received_messages 
                             if '"type":"scores"' in msg['message'])
            warnings_count = sum(1 for msg in self.received_messages 
                               if '"type":"warnings"' in msg['message'])
            
            print(f"📊 Current scores messages: {current_scores_count}")
            print(f"📊 Round scores messages: {scores_count}")
            print(f"⚠️ Warnings messages: {warnings_count}")
            
        except KeyboardInterrupt:
            print("\n⏹️ Test interrupted by user")
        except Exception as e:
            print(f"\n❌ Test error: {e}")
        finally:
            # Clean up
            if hasattr(self, 'ws'):
                self.ws.close()
            print("\n✅ Test completed!")

if __name__ == "__main__":
    tester = DataFlowTester()
    tester.run_comprehensive_test() 