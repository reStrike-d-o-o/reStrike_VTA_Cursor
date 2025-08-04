#!/usr/bin/env python3
"""
Test script to verify the dramatic countdown with deciseconds simulation.
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

def test_dramatic_countdown_with_deciseconds():
    """Test the dramatic countdown with deciseconds simulation."""
    print("🎭 Testing Dramatic Countdown with Deciseconds Simulation...")
    print("📊 Expected behavior:")
    print("   - 00:18 to 00:11: Normal countdown (MM:SS format)")
    print("   - 00:10: Shows 10.00 first, then starts countdown from 9.99")
    print("   - 00:09: Shows 9.00 first, then starts countdown from 8.99")
    print("   - And so on until 0.00")
    
    # Test the sequence that triggers dramatic countdown
    test_times = [
        "00:18", "00:17", "00:16", "00:15", "00:14", "00:13", "00:12", "00:11",
        "00:10",  # This should show 10.00 first, then countdown from 9.99
        # Wait for dramatic countdown to complete
        "00:09",  # This should show 9.00 first, then countdown from 8.99
        # Wait for dramatic countdown to complete
        "00:08",  # This should show 8.00 first, then countdown from 7.99
        # Wait for dramatic countdown to complete
        "00:07"   # This should show 7.00 first, then countdown from 6.99
    ]
    
    for i, time_str in enumerate(test_times):
        print(f"\n⏰ Testing time {i+1}: {time_str}")
        send_udp_message(f"clk;{time_str};")
        
        # Wait longer for dramatic countdown to complete
        if time_str in ["00:10", "00:09", "00:08", "00:07"]:
            print("   ⏳ Waiting for dramatic countdown to complete...")
            time.sleep(12)  # Wait for ~11 seconds of dramatic countdown (1s show + 10s countdown)
        else:
            time.sleep(0.5)
    
    print("\n✅ Test completed!")
    print("🎯 Check the scoreboard overlay in your browser")
    print("🎭 You should see:")
    print("   - When PSS sends '00:10': Shows 10.00, then 9.99 → 9.98 → ... → 9.00")
    print("   - When PSS sends '00:09': Shows 9.00, then 8.99 → 8.98 → ... → 8.00")
    print("   - And so on with deciseconds counting down from 99 to 00!")

if __name__ == "__main__":
    test_dramatic_countdown_with_deciseconds() 