#!/usr/bin/env python3
"""
Test script for real-time PSS event system
Simulates PSS events to test the WebSocket connection and event parsing
"""

import socket
import time
import json
import websocket
import threading
from datetime import datetime

# PSS Event simulation
PSS_EVENTS = [
    # Match configuration
    "mch;101;Round of 16;M- 80 kg;1;#0000ff;#FFFFFF;#ff0000;#FFFFFF;a14ddd5c;Senior;3;120;cntDown;18;1;",
    
    # Athletes
    "at1;N. DESMOND;Nicolas DESMOND;MRN;at2;M. THIBAULT;Marcel THIBAULT;SUI;",
    
    # Round start
    "rnd;1;",
    "clk;2:00;start;",
    
    # Points events
    "pt1;3;",  # Blue head kick
    "pt2;1;",  # Red punch
    "pt1;2;",  # Blue body kick
    "pt2;3;",  # Red head kick
    
    # Warnings
    "wg1;1;wg2;0;",
    "wg1;2;wg2;1;",
    
    # Clock updates
    "clk;1:45;",
    "clk;1:30;",
    "clk;1:15;",
    
    # Hit levels
    "hl1;75;",
    "hl2;82;",
    
    # Challenge
    "ch1;",
    "ch0;1;",
    
    # Round end
    "clk;0:00;stop;",
    "rnd;2;",
    "clk;2:00;start;",
]

def send_udp_events():
    """Send PSS events via UDP to test the backend"""
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    server_address = ('localhost', 6000)
    
    print("📡 Sending PSS events via UDP...")
    
    for i, event in enumerate(PSS_EVENTS):
        print(f"  Event {i+1}: {event}")
        sock.sendto(event.encode(), server_address)
        time.sleep(1)  # Wait 1 second between events
    
    sock.close()
    print("✅ UDP events sent")

def test_websocket_connection():
    """Test WebSocket connection to receive real-time events"""
    events_received = []
    
    def on_message(ws, message):
        try:
            data = json.loads(message)
            events_received.append(data)
            print(f"📡 WebSocket received: {data}")
        except json.JSONDecodeError:
            print(f"📡 WebSocket received (raw): {message}")
    
    def on_error(ws, error):
        print(f"❌ WebSocket error: {error}")
    
    def on_close(ws, close_status_code, close_msg):
        print("🔌 WebSocket connection closed")
    
    def on_open(ws):
        print("🔗 WebSocket connected")
    
    # Connect to WebSocket
    ws = websocket.WebSocketApp(
        "ws://localhost:8080",
        on_open=on_open,
        on_message=on_message,
        on_error=on_error,
        on_close=on_close
    )
    
    # Run WebSocket in a separate thread
    ws_thread = threading.Thread(target=ws.run_forever)
    ws_thread.daemon = True
    ws_thread.start()
    
    # Wait for connection
    time.sleep(2)
    
    return ws, events_received

def main():
    print("🧪 Testing Real-time PSS Event System")
    print("=" * 50)
    
    # Test WebSocket connection first
    print("\n1. Testing WebSocket connection...")
    ws, events_received = test_websocket_connection()
    
    # Wait a bit for connection to establish
    time.sleep(3)
    
    # Send UDP events
    print("\n2. Sending PSS events via UDP...")
    send_udp_events()
    
    # Wait for events to be processed
    print("\n3. Waiting for events to be processed...")
    time.sleep(5)
    
    # Close WebSocket
    ws.close()
    
    # Summary
    print("\n" + "=" * 50)
    print("📊 Test Summary:")
    print(f"  - Events sent: {len(PSS_EVENTS)}")
    print(f"  - Events received via WebSocket: {len(events_received)}")
    print(f"  - Success rate: {len(events_received)/len(PSS_EVENTS)*100:.1f}%")
    
    if events_received:
        print("\n📋 Sample received events:")
        for i, event in enumerate(events_received[:5]):
            print(f"  {i+1}. {event}")
    
    print("\n✅ Test completed!")

if __name__ == "__main__":
    main() 