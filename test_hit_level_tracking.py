#!/usr/bin/env python3
"""
Hit Level Tracking Test Script
Tests the new hit level tracking feature that links hit levels with point events
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

def test_hit_level_tracking():
    """Test hit level tracking and linking with point events"""
    print("🚀 Starting Hit Level Tracking Test...")
    print("📋 This test will send hit level events followed by point events to verify linking")
    
    # Wait a moment for any existing events to clear
    time.sleep(1)
    
    # Test 1: Fight Loaded
    print("\n📋 Test 1: Fight Loaded")
    send_udp_message("pre;FightLoaded;")
    time.sleep(1)
    
    # Test 2: Athletes
    print("\n📋 Test 2: Athletes")
    send_udp_message("at1;N. DESMOND;Nicolas DESMOND;MRN;at2;M. THIBAULT;Marcel THIBAULT;SUI;")
    time.sleep(1)
    
    # Test 3: Match Config
    print("\n📋 Test 3: Match Config")
    send_udp_message("mch;101;Round of 16;M- 80 kg;1;#0000ff;#FFFFFF;#ff0000;#FFFFFF;a14ddd5c;Senior;3;120;cntDown;18;1;")
    time.sleep(1)
    
    # Test 4: Fight Ready
    print("\n📋 Test 4: Fight Ready")
    send_udp_message("rdy;FightReady;")
    time.sleep(2)
    
    # Test 5: Hit Level Events (should be tracked)
    print("\n📋 Test 5: Hit Level Events (tracking)")
    send_udp_message("hl1;75;")  # Athlete 1 hit level 75
    time.sleep(0.5)
    send_udp_message("hl1;85;")  # Athlete 1 hit level 85
    time.sleep(0.5)
    send_udp_message("hl2;60;")  # Athlete 2 hit level 60
    time.sleep(0.5)
    send_udp_message("hl1;90;")  # Athlete 1 hit level 90
    time.sleep(0.5)
    
    # Test 6: Point Event (should include recent hit levels)
    print("\n📋 Test 6: Point Event (should include recent hit levels)")
    send_udp_message("pt1;3;")  # Athlete 1 scores 3 points (head)
    time.sleep(1)
    
    # Test 7: More Hit Levels
    print("\n📋 Test 7: More Hit Levels")
    send_udp_message("hl2;80;")  # Athlete 2 hit level 80
    time.sleep(0.5)
    send_udp_message("hl2;95;")  # Athlete 2 hit level 95
    time.sleep(0.5)
    
    # Test 8: Another Point Event
    print("\n📋 Test 8: Another Point Event")
    send_udp_message("pt2;2;")  # Athlete 2 scores 2 points (body)
    time.sleep(1)
    
    # Test 9: Hit Level without Points (should still be stored)
    print("\n📋 Test 9: Hit Level without Points (should still be stored)")
    send_udp_message("hl1;50;")  # Athlete 1 hit level 50 (weak hit, no points)
    time.sleep(0.5)
    send_udp_message("hl2;40;")  # Athlete 2 hit level 40 (weak hit, no points)
    time.sleep(1)
    
    # Test 10: Point Event after weak hits
    print("\n📋 Test 10: Point Event after weak hits")
    send_udp_message("pt1;1;")  # Athlete 1 scores 1 point (punch)
    time.sleep(1)
    
    # Test 11: Multiple hit levels in quick succession
    print("\n📋 Test 11: Multiple hit levels in quick succession")
    send_udp_message("hl1;70;")  # Athlete 1 hit level 70
    time.sleep(0.2)
    send_udp_message("hl1;75;")  # Athlete 1 hit level 75
    time.sleep(0.2)
    send_udp_message("hl1;80;")  # Athlete 1 hit level 80
    time.sleep(0.2)
    send_udp_message("hl1;85;")  # Athlete 1 hit level 85
    time.sleep(0.2)
    
    # Test 12: Point Event after multiple hits
    print("\n📋 Test 12: Point Event after multiple hits")
    send_udp_message("pt1;4;")  # Athlete 1 scores 4 points (technical body)
    time.sleep(1)
    
    print("\n✅ Hit Level Tracking Test completed!")
    print("📋 Check the database to see if hit level events are stored")
    print("📋 Check if point events include recent_hit_levels, max_hit_level, and avg_hit_level details")
    print("📋 Verify that hit levels without points are still stored in the database")

if __name__ == "__main__":
    test_hit_level_tracking() 