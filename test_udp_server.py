#!/usr/bin/env python3
"""
Test script for UDP PSS server
Sends mock PSS events to test the server functionality
"""

import socket
import time
import sys

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

def test_pss_events():
    """Test various PSS events"""
    print("🚀 Starting PSS UDP test...")
    
    # Test 1: Fight Loaded
    print("\n📋 Test 1: Fight Loaded")
    send_udp_message("pre;FightLoaded;")
    time.sleep(1)
    
    # Test 2: Athletes (at1; format from protocol)
    print("\n📋 Test 2: Athletes")
    send_udp_message("at1;N. DESMOND;Nicolas DESMOND;MRN;at2;M. THIBAULT;Marcel THIBAULT;SUI;")
    time.sleep(1)
    
    # Test 3: Match Config (mch; format from protocol)
    print("\n📋 Test 3: Match Config")
    send_udp_message("mch;101;Round of 16;M- 80 kg;1;#0000ff;#FFFFFF;#ff0000;#FFFFFF;a14ddd5c;Senior;3;120;cntDown;18;1;")
    time.sleep(1)
    
    # Test 4: Warnings
    print("\n📋 Test 4: Warnings")
    send_udp_message("wg1;0;wg2;0;")
    time.sleep(1)
    
    # Test 5: Winner Rounds
    print("\n📋 Test 5: Winner Rounds")
    send_udp_message("wrd;rd1;0;rd2;0;rd3;0;")
    time.sleep(1)
    
    # Test 6: Scores
    print("\n📋 Test 6: Scores")
    send_udp_message("s11;0;s21;0;s12;0;s22;0;s13;0;s23;0;")
    time.sleep(1)
    
    # Test 7: Current Scores
    print("\n📋 Test 7: Current Scores")
    send_udp_message("sc1;0;sc2;0;")
    time.sleep(1)
    
    # Test 8: Clock
    print("\n📋 Test 8: Clock")
    send_udp_message("clk;2:00;")
    time.sleep(1)
    
    # Test 9: Round
    print("\n📋 Test 9: Round")
    send_udp_message("rnd;1;")
    time.sleep(1)
    
    # Test 10: Fight Ready
    print("\n📋 Test 10: Fight Ready")
    send_udp_message("rdy;FightReady;")
    time.sleep(1)
    
    # Test 11: Points (athlete 1 scores)
    print("\n📋 Test 11: Points")
    send_udp_message("pt1;3;")  # Head kick for athlete 1
    time.sleep(1)
    
    # Test 12: Updated scores
    print("\n📋 Test 12: Updated Scores")
    send_udp_message("s11;3;s21;0;s12;0;s22;0;s13;0;s23;0;")
    send_udp_message("sc1;3;sc2;0;")
    time.sleep(1)
    
    print("\n✅ PSS UDP test completed!")

if __name__ == "__main__":
    # Allow custom host and port
    host = sys.argv[1] if len(sys.argv) > 1 else '127.0.0.1'
    port = int(sys.argv[2]) if len(sys.argv) > 2 else 8888
    
    print(f"🎯 Testing UDP server at {host}:{port}")
    test_pss_events() 