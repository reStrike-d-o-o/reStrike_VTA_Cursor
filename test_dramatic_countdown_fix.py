#!/usr/bin/env python3
"""
Quick test script to verify the dramatic countdown fix works with the user's PSS time formats.
"""

import socket
import time

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

def test_problematic_sequence():
    """Test the exact sequence that was causing the error."""
    print("🧪 Testing the problematic sequence from user's PSS system...")
    
    # Test the exact sequence that was failing
    problematic_times = [
        "00:18", "00:17", "00:16", "00:15", "00:14", "00:13", "00:12", "00:11",
        "00:10", "00:09", "00:08", "00:07"
    ]
    
    for time_str in problematic_times:
        print(f"\n⏰ Testing time: {time_str}")
        send_udp_message(f"clk;{time_str};")
        time.sleep(0.5)
    
    print("\n✅ Test completed!")
    print("🎯 Check the scoreboard overlay in your browser")
    print("📊 Expected behavior:")
    print("   - 00:18 to 00:11: Normal countdown (MM:SS format)")
    print("   - 00:10 to 00:07: Dramatic countdown (SS.00 format)")

if __name__ == "__main__":
    test_problematic_sequence() 