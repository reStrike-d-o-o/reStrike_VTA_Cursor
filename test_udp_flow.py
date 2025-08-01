#!/usr/bin/env python3
"""
Comprehensive UDP event flow test
Tests the complete flow from UDP server to frontend
"""

import socket
import time
import json

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

def test_complete_flow():
    """Test the complete UDP event flow"""
    print("🚀 Starting comprehensive UDP event flow test...")
    print("📋 This test will send events that should update the frontend MatchDetailsSection")
    
    # Wait a moment for any existing events to clear
    time.sleep(1)
    
    # Test 1: Fight Loaded event
    print("\n📋 Test 1: Fight Loaded")
    send_udp_message("pre;FightLoaded;")
    time.sleep(2)
    
    # Test 2: Athletes event (this should update the MatchDetailsSection)
    print("\n📋 Test 2: Athletes (should update frontend)")
    send_udp_message("at1;N. DESMOND;Nicolas DESMOND;MRN;at2;M. THIBAULT;Marcel THIBAULT;SUI;")
    time.sleep(3)
    
    # Test 3: Match Config event (this should also update the MatchDetailsSection)
    print("\n📋 Test 3: Match Config (should update frontend)")
    send_udp_message("mch;101;Round of 16;M- 80 kg;1;#0000ff;#FFFFFF;#ff0000;#FFFFFF;a14ddd5c;Senior;3;120;cntDown;18;1;")
    time.sleep(3)
    
    # Test 4: Fight Ready event
    print("\n📋 Test 4: Fight Ready")
    send_udp_message("rdy;FightReady;")
    time.sleep(2)
    
    # Test 5: Some scoring events
    print("\n📋 Test 5: Scoring events")
    send_udp_message("pt1;3;")  # Athlete 1 scores 3 points
    time.sleep(1)
    send_udp_message("s11;3;s21;0;s12;0;s22;0;s13;0;s23;0;")  # Update scores
    time.sleep(1)
    send_udp_message("sc1;3;sc2;0;")  # Update current scores
    time.sleep(2)
    
    print("\n✅ Comprehensive UDP event flow test completed!")
    print("📋 Check the frontend MatchDetailsSection to see if the data was updated")
    print("📋 Look for athlete names: Nicolas DESMOND vs Marcel THIBAULT")
    print("📋 Look for match details: Round of 16, M- 80 kg, Senior")

if __name__ == "__main__":
    test_complete_flow() 