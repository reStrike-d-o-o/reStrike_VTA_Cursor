#!/usr/bin/env python3
"""
Test script to verify the dramatic countdown works correctly for the final seconds (0:01 and 0:00).
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

def test_final_seconds_countdown():
    """Test the dramatic countdown for the final seconds."""
    print("🎭 Testing Final Seconds Dramatic Countdown...")
    print("📊 Expected behavior:")
    print("   - 00:03: Shows 3.00 first, then countdown from 2.99")
    print("   - 00:02: Shows 2.00 first, then countdown from 1.99")
    print("   - 00:01: Shows 1.00 first, then countdown from 0.99")
    print("   - 00:00: Shows 0.00 and stops (no negative countdown)")
    
    # Test the final seconds sequence
    test_times = [
        "00:03",  # Should show 3.00, then countdown from 2.99
        # Wait for countdown to complete
        "00:02",  # Should show 2.00, then countdown from 1.99
        # Wait for countdown to complete
        "00:01",  # Should show 1.00, then countdown from 0.99
        # Wait for countdown to complete
        "00:00"   # Should show 0.00 and stop
    ]
    
    for i, time_str in enumerate(test_times):
        print(f"\n⏰ Testing final second {i+1}: {time_str}")
        send_udp_message(f"clk;{time_str};")
        
        # Wait for dramatic countdown to complete
        if time_str in ["00:03", "00:02", "00:01"]:
            print("   ⏳ Waiting for dramatic countdown to complete...")
            time.sleep(12)  # Wait for ~11 seconds of dramatic countdown (1s show + 10s countdown)
        elif time_str == "00:00":
            print("   ⏳ Waiting for final display...")
            time.sleep(3)  # Wait a bit to see the final 0.00 display
        else:
            time.sleep(0.5)
    
    print("\n✅ Test completed!")
    print("🎯 Check the scoreboard overlay in your browser")
    print("🎭 You should see:")
    print("   - When PSS sends '00:01': Shows 1.00, then 0.99 → 0.98 → ... → 0.00")
    print("   - When PSS sends '00:00': Shows 0.00 and stops (no 0.99 going back)")
    print("   - No negative countdown or going back to 0.99!")

if __name__ == "__main__":
    test_final_seconds_countdown() 