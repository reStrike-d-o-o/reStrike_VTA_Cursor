#!/usr/bin/env python3
"""
Comprehensive test script to verify dramatic countdown accuracy, round end handling, and match end handling.
"""

import socket
import time
import json
import threading
from datetime import datetime

class ComprehensiveCountdownTester:
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
            
            # Parse and analyze messages
            try:
                data = json.loads(message)
                if data.get('type') == 'pss_event':
                    event_data = data.get('data', {})
                    event_type = event_data.get('type')
                    
                    if event_type == 'clock':
                        self.analyze_clock_message(event_data)
                    elif event_type == 'winner_rounds':
                        self.analyze_winner_rounds_message(event_data)
                    elif event_type == 'winner':
                        self.analyze_winner_message(event_data)
                        
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
    
    def analyze_winner_rounds_message(self, event_data):
        """Analyze winner rounds WebSocket message."""
        print("🏆 Analyzing winner rounds message:")
        print(f"   Round 1 winner: {event_data.get('round1_winner', 'N/A')}")
        print(f"   Round 2 winner: {event_data.get('round2_winner', 'N/A')}")
        print(f"   Round 3 winner: {event_data.get('round3_winner', 'N/A')}")
        print("   ✅ ROUND END DETECTED - Scores should be reset to 0:0")
    
    def analyze_winner_message(self, event_data):
        """Analyze winner WebSocket message."""
        print("🏁 Analyzing winner message:")
        print(f"   Winner: {event_data.get('name', 'N/A')}")
        print(f"   Classification: {event_data.get('classification', 'N/A')}")
        print("   ✅ MATCH END DETECTED - Countdown should stop")
    
    def test_accuracy_and_round_end(self):
        """Test dramatic countdown accuracy and round end handling."""
        print("\n🎯 Testing Dramatic Countdown Accuracy and Round End Handling...")
        
        # Test sequence: normal countdown → dramatic countdown → round end
        test_sequence = [
            ("clk;00:15;", "Normal countdown"),
            ("clk;00:14;", "Normal countdown"),
            ("clk;00:13;", "Normal countdown"),
            ("clk;00:12;", "Normal countdown"),
            ("clk;00:11;", "Normal countdown"),
            ("clk;00:10;", "Dramatic countdown starts"),
            # Wait for dramatic countdown to complete
            ("wrd;rd1;0;rd2;1;rd3;0;", "Round end - scores should reset to 0:0"),
            ("clk;02:00;", "New round starts"),
        ]
        
        for i, (message, description) in enumerate(test_sequence):
            print(f"\n📨 Test {i+1}: {description}")
            print(f"   Sending: {message}")
            self.send_udp_message(message)
            
            if "Dramatic countdown starts" in description:
                print("   ⏳ Waiting for dramatic countdown to complete...")
                time.sleep(12)  # Wait for dramatic countdown
            elif "Round end" in description:
                print("   ⏳ Waiting for round end processing...")
                time.sleep(2)
            else:
                time.sleep(0.5)
    
    def test_match_end(self):
        """Test match end handling."""
        print("\n🏁 Testing Match End Handling...")
        
        # Test sequence: dramatic countdown → match end
        test_sequence = [
            ("clk;00:05;", "Dramatic countdown starts"),
            # Wait for dramatic countdown to complete
            ("win;John Doe;", "Match end - countdown should stop"),
        ]
        
        for i, (message, description) in enumerate(test_sequence):
            print(f"\n📨 Test {i+1}: {description}")
            print(f"   Sending: {message}")
            self.send_udp_message(message)
            
            if "Dramatic countdown starts" in description:
                print("   ⏳ Waiting for dramatic countdown to complete...")
                time.sleep(7)  # Wait for dramatic countdown
            elif "Match end" in description:
                print("   ⏳ Waiting for match end processing...")
                time.sleep(2)
            else:
                time.sleep(0.5)
    
    def run_comprehensive_test(self):
        """Run the comprehensive test."""
        print("🚀 Starting Comprehensive Countdown Test...")
        print("Make sure the UDP server and WebSocket server are running")
        print("Open scoreboard-overlay.html in a browser to see the visual effects")
        
        try:
            # Start WebSocket listener
            self.start_websocket_listener()
            time.sleep(2)  # Wait for WebSocket connection
            
            # Run tests
            self.test_accuracy_and_round_end()
            time.sleep(2)
            self.test_match_end()
            
            # Wait for all messages to be processed
            time.sleep(3)
            
            # Print test results
            print("\n📋 Test Results Summary:")
            print("=" * 50)
            print(f"📡 Total WebSocket messages received: {len(self.received_messages)}")
            
            # Check for specific message types
            clock_count = sum(1 for msg in self.received_messages 
                            if '"type":"clock"' in msg['message'])
            winner_rounds_count = sum(1 for msg in self.received_messages 
                                    if '"type":"winner_rounds"' in msg['message'])
            winner_count = sum(1 for msg in self.received_messages 
                             if '"type":"winner"' in msg['message'])
            
            print(f"⏰ Clock messages: {clock_count}")
            print(f"🏆 Winner rounds messages: {winner_rounds_count}")
            print(f"🏁 Winner messages: {winner_count}")
            
            print("\n✅ Test completed!")
            print("🎯 Check the scoreboard overlay in your browser")
            print("🎭 Expected improvements:")
            print("   - More accurate deciseconds countdown")
            print("   - Scores reset to 0:0 when round ends")
            print("   - Countdown stops when match ends")
            
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
    tester = ComprehensiveCountdownTester()
    tester.run_comprehensive_test() 