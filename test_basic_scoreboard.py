#!/usr/bin/env python3
"""
Simple test script to verify basic scoreboard functionality without dramatic countdown.
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

def test_basic_scoreboard_functionality():
    """Test basic scoreboard functionality."""
    print("🎯 Testing Basic Scoreboard Functionality...")
    print("📊 Expected behavior:")
    print("   - Clock updates normally (2:00, 1:59, 1:58, etc.)")
    print("   - Round numbers update (1ST, 2ND, 3RD)")
    print("   - Scores reset to 0:0 when round ends")
    print("   - Rounds won update when round ends")
    
    # Test basic functionality
    test_sequence = [
        ("clk;02:00;", "Round 1 starts - 2 minutes"),
        ("clk;01:59;", "Countdown continues"),
        ("clk;01:58;", "Countdown continues"),
        ("clk;00:10;", "Last 10 seconds"),
        ("clk;00:09;", "Last 9 seconds"),
        ("clk;00:08;", "Last 8 seconds"),
        ("clk;00:07;", "Last 7 seconds"),
        ("clk;00:06;", "Last 6 seconds"),
        ("clk;00:05;", "Last 5 seconds"),
        ("clk;00:04;", "Last 4 seconds"),
        ("clk;00:03;", "Last 3 seconds"),
        ("clk;00:02;", "Last 2 seconds"),
        ("clk;00:01;", "Last 1 second"),
        ("clk;00:00;", "Round 1 ends"),
        ("wrd;rd1;1;rd2;0;rd3;0;", "Round 1 winner - Blue wins, scores should reset to 0:0"),
        ("rnd;2;", "Round 2 starts"),
        ("clk;02:00;", "Round 2 starts - 2 minutes again"),
        ("clk;01:59;", "Round 2 countdown"),
        ("wrd;rd1;1;rd2;2;rd3;0;", "Round 2 winner - Blue 1, Red 1, scores should reset to 0:0"),
        ("rnd;3;", "Round 3 starts"),
        ("clk;02:00;", "Round 3 starts - 2 minutes again"),
        ("wrd;rd1;1;rd2;2;rd3;1;", "Round 3 winner - Blue wins match (2-1)"),
        ("win;Blue Player;", "Match end - Blue player wins"),
    ]
    
    for i, (message, description) in enumerate(test_sequence):
        print(f"\n📨 Test {i+1}: {description}")
        send_udp_message(message)
        time.sleep(0.5)
    
    print("\n✅ Test completed!")
    print("🎯 Check the scoreboard overlay in your browser")
    print("📊 Expected results:")
    print("   - Clock should countdown normally from 2:00 to 0:00")
    print("   - Round numbers should update: 1ST → 2ND → 3RD")
    print("   - Scores should reset to 0:0 after each round end")
    print("   - Rounds won should update: Blue 1-0, then 1-1, then 2-1")
    print("   - No dramatic countdown, just normal MM:SS format")

if __name__ == "__main__":
    test_basic_scoreboard_functionality() 