#!/usr/bin/env python3
"""
Test script to verify UDP parsing fixes for combined score and warning messages.
"""

import socket
import time
import json

def send_udp_message(message, host='127.0.0.1', port=8888):
    """Send a UDP message to the server."""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.sendto(message.encode('utf-8'), (host, port))
        sock.close()
        print(f"📤 Sent: {message}")
        return True
    except Exception as e:
        print(f"❌ Failed to send message: {e}")
        return False

def test_combined_score_messages():
    """Test the problematic combined score messages."""
    print("🧪 Testing combined score message parsing...")
    
    # Test 1: Combined current scores (this was the main issue)
    print("\n📊 Test 1: Combined current scores (sc1;0;sc2;1)")
    send_udp_message("sc1;0;sc2;1;")
    time.sleep(0.1)
    
    # Test 2: Combined round scores
    print("\n📊 Test 2: Combined round scores (s11;0;s21;1;s12;0;s22;0;s13;0;s23;0)")
    send_udp_message("s11;0;s21;1;s12;0;s22;0;s13;0;s23;0;")
    time.sleep(0.1)
    
    # Test 3: Combined warnings (this should already work)
    print("\n⚠️ Test 3: Combined warnings (wg1;1;wg2;0)")
    send_udp_message("wg1;1;wg2;0;")
    time.sleep(0.1)
    
    # Test 4: Individual score messages (should still work)
    print("\n📊 Test 4: Individual current score (sc1;2)")
    send_udp_message("sc1;2;")
    time.sleep(0.1)
    
    print("\n📊 Test 5: Individual round score (s21;3)")
    send_udp_message("s21;3;")
    time.sleep(0.1)

def test_sequence_from_user_messages():
    """Test the exact sequence from the user's PSS messages."""
    print("\n🎯 Testing exact sequence from user's PSS messages...")
    
    # Sequence that was causing issues
    messages = [
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
    
    for i, message in enumerate(messages, 1):
        print(f"\n📨 Message {i}: {message}")
        send_udp_message(message)
        time.sleep(0.1)

if __name__ == "__main__":
    print("🚀 Starting UDP parsing fix verification tests...")
    print("Make sure the UDP server is running on port 8888")
    
    try:
        test_combined_score_messages()
        test_sequence_from_user_messages()
        print("\n✅ All tests completed!")
        print("\n📋 Check the server logs to verify that:")
        print("   - Combined score messages are parsed correctly")
        print("   - Both athlete1 and athlete2 scores are updated")
        print("   - No score information is lost")
        
    except KeyboardInterrupt:
        print("\n⏹️ Tests interrupted by user")
    except Exception as e:
        print(f"\n❌ Test error: {e}") 