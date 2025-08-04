#!/usr/bin/env python3
"""
Test script to verify dramatic countdown effect in scoreboard overlay.
Tests both normal countdown (minutes:seconds) and dramatic countdown (seconds.deciseconds).
"""

import socket
import time
import json
import threading
from datetime import datetime

class DramaticCountdownTester:
    def __init__(self, udp_host='127.0.0.1', udp_port=8888, ws_host='127.0.0.1', ws_port=3001):
        self.udp_host = udp_host
        self.udp_port = udp_port
        self.ws_host = ws_host
        self.ws_port = ws_port
        self.received_messages = []
        
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
            
            # Parse and analyze clock messages
            try:
                data = json.loads(message)
                if data.get('type') == 'pss_event':
                    event_data = data.get('data', {})
                    event_type = event_data.get('type')
                    
                    if event_type == 'clock':
                        self.analyze_clock_message(event_data)
                        
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
    
    def analyze_clock_message(self, event_data):
        """Analyze clock WebSocket message."""
        print("⏰ Analyzing clock message:")
        print(f"   Raw data: {event_data.get('raw_data', 'N/A')}")
        print(f"   Time: {event_data.get('time', 'N/A')}")
        print(f"   Action: {event_data.get('action', 'N/A')}")
        
        # Check if it's dramatic format
        time_str = event_data.get('time', '')
        if '.' in time_str and ':' not in time_str:
            print("   🎭 DRAMATIC COUNTDOWN DETECTED!")
        else:
            print("   ⏰ Normal countdown format")
    
    def test_normal_countdown(self):
        """Test normal countdown from 2:00 to 0:11."""
        print("\n🧪 Testing Normal Countdown (2:00 → 0:11)...")
        
        normal_times = [
            "2:00", "1:59", "1:58", "1:57", "1:56", "1:55",
            "1:30", "1:00", "0:30", "0:20", "0:15", "0:12", "0:11"
        ]
        
        for time_str in normal_times:
            print(f"\n⏰ Testing normal time: {time_str}")
            self.send_udp_message(f"clk;{time_str};")
            time.sleep(0.3)
    
    def test_dramatic_countdown(self):
        """Test dramatic countdown from 10.0 to 0.0."""
        print("\n🎭 Testing Dramatic Countdown (10.0 → 0.0)...")
        
        # Test transition from normal to dramatic
        print("\n⏰ Testing transition: 0:11 → 10.0")
        self.send_udp_message("clk;0:11;")
        time.sleep(0.5)
        self.send_udp_message("clk;10.0;")
        time.sleep(0.5)
        
        # Test dramatic countdown with two-digit deciseconds
        dramatic_times = [
            "9.99", "9.98", "9.97", "9.96", "9.95",
            "9.00", "8.99", "8.98", "8.97", "8.96",
            "5.00", "4.99", "4.98", "4.97", "4.96",
            "1.00", "0.99", "0.98", "0.97", "0.96",
            "0.10", "0.09", "0.08", "0.07", "0.06",
            "0.05", "0.04", "0.03", "0.02", "0.01", "0.00"
        ]
        
        for time_str in dramatic_times:
            print(f"\n🎭 Testing dramatic time: {time_str}")
            self.send_udp_message(f"clk;{time_str};")
            time.sleep(0.2)  # Faster updates for dramatic effect
    
    def test_edge_cases(self):
        """Test edge cases and boundary conditions."""
        print("\n🔍 Testing Edge Cases...")
        
        edge_cases = [
            "0:10",  # Should trigger dramatic mode
            "10.0",  # Should be dramatic
            "9.99",  # Should be dramatic with two digits
            "0.00",  # Should be dramatic
            "0:09",  # Should be dramatic
            "9.0",   # Should be dramatic
            "0.1",   # Should be dramatic
            "0.01",  # Should be dramatic
        ]
        
        for time_str in edge_cases:
            print(f"\n🔍 Testing edge case: {time_str}")
            self.send_udp_message(f"clk;{time_str};")
            time.sleep(0.3)
    
    def run_comprehensive_test(self):
        """Run the complete dramatic countdown test."""
        print("🚀 Starting Dramatic Countdown Test...")
        print("Make sure the UDP server and WebSocket server are running")
        print("Open scoreboard-overlay.html in a browser to see the visual effects")
        
        try:
            # Start WebSocket listener
            self.start_websocket_listener()
            time.sleep(2)  # Wait for WebSocket connection
            
            # Run tests
            self.test_normal_countdown()
            time.sleep(1)
            self.test_dramatic_countdown()
            time.sleep(1)
            self.test_edge_cases()
            
            # Wait for all messages to be processed
            time.sleep(3)
            
            # Print test results
            print("\n📋 Test Results Summary:")
            print("=" * 50)
            print(f"📡 Total WebSocket messages received: {len(self.received_messages)}")
            
            # Check for clock messages
            clock_count = sum(1 for msg in self.received_messages 
                            if '"type":"clock"' in msg['message'])
            print(f"⏰ Clock messages: {clock_count}")
            
            # Check for dramatic format
            dramatic_count = sum(1 for msg in self.received_messages 
                               if '." in msg['message'] and ':" not in msg['message'])
            print(f"🎭 Dramatic format messages: {dramatic_count}")
            
            print("\n✅ Test completed!")
            print("🎯 Check the scoreboard overlay in your browser to see the visual effects")
            
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
    tester = DramaticCountdownTester()
    tester.run_comprehensive_test() 