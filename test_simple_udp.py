#!/usr/bin/env python3
"""
Simple UDP test script to verify PSS event processing
"""

import socket
import time

def send_udp_message(message, host='127.0.0.1', port=8888):
    """Send a UDP message to the specified host and port"""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.sendto(message.encode('utf-8'), (host, port))
        print(f"📤 Sent: {message}")
        sock.close()
        return True
    except Exception as e:
        print(f"❌ Failed to send message: {e}")
        return False

def test_simple_events():
    """Test simple PSS events"""
    print("🚀 Starting simple PSS UDP test...")
    
    # Test 1: Athletes event
    print("\n📋 Test 1: Athletes")
    send_udp_message("at1;N. DESMOND;Nicolas DESMOND;MRN;at2;M. THIBAULT;Marcel THIBAULT;SUI;")
    time.sleep(2)
    
    # Test 2: Match Config event
    print("\n📋 Test 2: Match Config")
    send_udp_message("mch;101;Round of 16;M- 80 kg;1;#0000ff;#FFFFFF;#ff0000;#FFFFFF;a14ddd5c;Senior;3;120;cntDown;18;1;")
    time.sleep(2)
    
    # Test 3: Fight Loaded event
    print("\n📋 Test 3: Fight Loaded")
    send_udp_message("pre;FightLoaded;")
    time.sleep(2)
    
    print("\n✅ Simple PSS UDP test completed!")

if __name__ == "__main__":
    test_simple_events() 